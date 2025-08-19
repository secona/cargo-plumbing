use std::fmt::Debug;
use std::io::{BufReader, IsTerminal};
use std::path::PathBuf;
use std::{env, io};

use cargo::core::registry::PackageRegistry;
use cargo::core::resolver::{CliFeatures, HasDevUnits};
use cargo::core::{ResolveVersion, Workspace};
use cargo::ops::resolve_with_previous;
use cargo::sources::SourceConfigMap;
use cargo::{CargoResult, GlobalContext};
use cargo_plumbing::cargo::core::resolver::encode::{
    encodable_resolve_node, encodable_source_id, EncodableDependency, EncodeState,
};
use cargo_plumbing::ops::resolve::{into_resolve, normalize_dependency, normalize_packages};
use cargo_plumbing_schemas::lock_dependencies::{LockDependenciesIn, LockDependenciesOut};
use cargo_plumbing_schemas::lockfile::NormalizedPatch;

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    /// Path to the manifest file
    // HACK: We are reading manifests from disk and not purely from stdin because of cargo API
    // limitations.
    //
    // See: https://github.com/crate-ci/cargo-plumbing/issues/82
    #[arg(long)]
    manifest_path: Option<PathBuf>,
}

pub(crate) fn exec(gctx: &GlobalContext, args: Args) -> CargoResult<()> {
    // HACK: We are reading manifests from disk and not purely from stdin because of cargo API
    // limitations.
    //
    // See: https://github.com/crate-ci/cargo-plumbing/issues/82
    let manifest_path = args
        .manifest_path
        .unwrap_or(env::current_dir()?.join("Cargo.toml"));
    let path = gctx.cwd().join(manifest_path);
    let ws = Workspace::new(&path, gctx)?;

    let stdin = io::stdin();
    if stdin.is_terminal() {
        anyhow::bail!("input must be piped from a file or another command");
    }

    let messages = LockDependenciesIn::parse_stream(BufReader::new(stdin));

    let mut lockfile_version: Option<u32> = None;
    let mut locked_packages = Vec::new();
    let mut unused_patches = None;

    for message in messages {
        match message? {
            LockDependenciesIn::Lockfile { version } => lockfile_version = version,
            LockDependenciesIn::LockedPackage { package } => locked_packages.push(package),
            LockDependenciesIn::UnusedPatches { unused } => unused_patches = Some(unused),
        }
    }

    let previous_resolve = if !locked_packages.is_empty() {
        Some(into_resolve(
            &ws,
            lockfile_version,
            locked_packages,
            unused_patches.unwrap_or_default(),
        )?)
    } else {
        None
    };
    let source_config = SourceConfigMap::new(gctx)?;
    let mut registry = PackageRegistry::new_with_source_config(gctx, source_config)?;

    let resolve = resolve_with_previous(
        &mut registry,
        &ws,
        &CliFeatures::new_all(true),
        HasDevUnits::Yes,
        previous_resolve.as_ref(),
        None,
        &[],
        true,
    )?;

    let mut ids: Vec<_> = resolve.iter().collect();
    ids.sort();
    let state = EncodeState::new(&resolve);
    let packages = ids
        .iter()
        .map(|&id| encodable_resolve_node(id, &resolve, &state))
        .collect::<Vec<_>>();
    let metadata = resolve.metadata().clone();

    let version = match resolve.version() {
        ResolveVersion::V5 => Some(5),
        ResolveVersion::V4 => Some(4),
        ResolveVersion::V3 => Some(3),
        ResolveVersion::V2 | ResolveVersion::V1 => None,
    };
    gctx.shell()
        .print_json(&LockDependenciesOut::Lockfile { version })?;

    for package in normalize_packages(None, Some(packages), Some(metadata))? {
        gctx.shell()
            .print_json(&LockDependenciesOut::LockedPackage { package })?;
    }

    let unused: Vec<_> = resolve
        .unused_patches()
        .iter()
        .map(|id| {
            normalize_dependency(EncodableDependency {
                name: id.name().to_string(),
                version: id.version().to_string(),
                source: encodable_source_id(id.source_id(), resolve.version()),
                dependencies: None,
                replace: None,
                checksum: if resolve.version() >= ResolveVersion::V2 {
                    resolve.checksums().get(id).and_then(|x| x.clone())
                } else {
                    None
                },
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    if !unused.is_empty() {
        let unused = NormalizedPatch { unused };
        let msg = LockDependenciesOut::UnusedPatches { unused };
        gctx.shell().print_json(&msg)?;
    }

    Ok(())
}
