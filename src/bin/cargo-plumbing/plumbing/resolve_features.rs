use std::env;
use std::io::{self, BufReader, IsTerminal};
use std::path::PathBuf;

use cargo::core::compiler::{CompileKind, RustcTargetData};
use cargo::core::resolver::features::{FeatureOpts, FeatureResolver};
use cargo::core::resolver::{CliFeatures, HasDevUnits};
use cargo::core::Workspace;
use cargo::ops::{get_resolved_packages, resolve_with_previous};
use cargo::{CargoResult, GlobalContext};
use cargo_plumbing::ops::resolve::into_resolve;
use cargo_plumbing_schemas::resolve_features::{ResolveFeaturesIn, ResolveFeaturesOut};

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    /// Path to the manifest file
    // HACK: We are reading manifests from disk and not purely from stdin because of cargo API
    // limitations.
    //
    // See: https://github.com/crate-ci/cargo-plumbing/issues/82
    #[arg(long)]
    manifest_path: Option<PathBuf>,
    /// List of features to activate
    #[arg(long, short = 'F')]
    features: Vec<String>,
    /// Activate all available features
    #[arg(long)]
    all_features: bool,
    /// Do not activate the `default` feature
    #[arg(long)]
    no_default_features: bool,
    /// Include dev units
    //
    // HACK: We're asking the users if they want to include dev units or not. Ideally this should
    // be done through stdin messages. However, due to cargo API limitations, this workaround is necessary
    //
    // See: https://github.com/crate-ci/cargo-plumbing/pull/68#discussion_r2277484208
    #[arg(long, default_value_t = false)]
    dev_units: bool,
    /// Target triple
    #[arg(long)]
    target: Vec<String>,
}

pub(crate) fn exec(gctx: &mut GlobalContext, args: Args) -> CargoResult<()> {
    let stdin = io::stdin();
    if stdin.is_terminal() {
        anyhow::bail!("input must be piped from a file or another command");
    }

    // HACK: We are reading manifests from disk and not purely from stdin because of cargo API
    // limitations.
    //
    // See: https://github.com/crate-ci/cargo-plumbing/issues/82
    let manifest_path = args
        .manifest_path
        .unwrap_or(env::current_dir()?.join("Cargo.toml"));
    let path = gctx.cwd().join(manifest_path);
    let ws = Workspace::new(&path, gctx)?;

    let messages = ResolveFeaturesIn::parse_stream(BufReader::new(stdin));

    let mut lock_version = None;
    let mut locked_packages = Vec::new();
    let mut unused_patches = None;
    let mut specs = Vec::new();

    for message in messages {
        match message? {
            ResolveFeaturesIn::Lockfile { version } => lock_version = version,
            ResolveFeaturesIn::LockedPackage { package } => locked_packages.push(package),
            ResolveFeaturesIn::UnusedPatches { unused } => unused_patches = Some(unused),
            ResolveFeaturesIn::Manifest { id } => specs.push(id),
        }
    }

    if locked_packages.is_empty() {
        anyhow::bail!("incomplete input. no packages found.");
    }

    let resolve = into_resolve(
        &ws,
        lock_version,
        locked_packages,
        unused_patches.unwrap_or_default(),
    )?;
    let cli_features = CliFeatures::from_command_line(
        &args.features,
        args.all_features,
        !args.no_default_features,
    )?;

    // HACK: We're asking the users if they want to include dev units or not. Ideally this should
    // be done through stdin messages. However, due to cargo API limitations, this workaround is necessary
    //
    // See: https://github.com/crate-ci/cargo-plumbing/pull/68#discussion_r2277484208
    let has_dev_units = if args.dev_units {
        HasDevUnits::Yes
    } else {
        HasDevUnits::No
    };

    let mut registry = ws.package_registry()?;
    let add_patches = true;
    let resolve_with_overrides = resolve_with_previous(
        &mut registry,
        &ws,
        &cli_features,
        has_dev_units,
        Some(&resolve),
        None,
        &specs,
        add_patches,
    )?;

    let requested_kinds = CompileKind::from_requested_targets(gctx, &args.target)?;
    let mut target_data = RustcTargetData::new(&ws, &requested_kinds)?;
    let members_with_features = ws.members_with_features(&specs, &cli_features)?;
    let member_ids = members_with_features
        .iter()
        .map(|(p, _fts)| p.package_id())
        .collect::<Vec<_>>();
    let force_all_targets = cargo::core::resolver::ForceAllTargets::No;

    // HACK: The resolver must download packages before it can resolve features. This is a
    // workaround for a known limitation of the feature resolver.
    //
    // See: https://github.com/rust-lang/cargo/issues/15834
    let pkg_set = get_resolved_packages(&resolve_with_overrides, registry)?;
    pkg_set.download_accessible(
        &resolve_with_overrides,
        &member_ids,
        has_dev_units,
        &requested_kinds,
        &target_data,
        force_all_targets,
    )?;

    let feature_opts = FeatureOpts::new(&ws, has_dev_units, force_all_targets)?;
    let resolved_features = FeatureResolver::resolve(
        &ws,
        &mut target_data,
        &resolve_with_overrides,
        &pkg_set,
        &cli_features,
        &specs,
        &requested_kinds,
        feature_opts,
    )?;

    for ((id, feat_for), feats) in resolved_features.activated_features {
        let id = id.to_spec();
        let features_for = feat_for.to_string();
        let features = feats.iter().map(|feat| feat.to_string()).collect();
        gctx.shell().print_json(&ResolveFeaturesOut::Activated {
            id,
            features_for,
            features,
        })?;
    }

    Ok(())
}
