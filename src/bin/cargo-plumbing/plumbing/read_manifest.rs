use std::path::PathBuf;

use cargo::core::{find_workspace_root, EitherManifest, SourceId, WorkspaceConfig};
use cargo::util::toml::read_manifest;
use cargo::{CargoResult, GlobalContext};
use cargo_plumbing_schemas::read_manifest::ReadManifestMessage;
use cargo_util::paths;

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    /// Path to the manifest file
    #[clap(long)]
    manifest_path: PathBuf,
}

pub(crate) fn exec(gctx: &GlobalContext, args: Args) -> CargoResult<()> {
    let manifest_path = gctx.cwd().join(args.manifest_path);
    let source_id = SourceId::for_manifest_path(&manifest_path)?;

    let (pkg_id, ws_config, manifest) = match read_manifest(&manifest_path, source_id, gctx)? {
        EitherManifest::Real(r) => (
            Some(r.package_id().to_spec()),
            r.workspace_config().clone(),
            r.normalized_toml().clone(),
        ),
        EitherManifest::Virtual(v) => (
            None,
            v.workspace_config().clone(),
            v.normalized_toml().clone(),
        ),
    };

    let msg = ReadManifestMessage::Manifest {
        path: manifest_path.clone(),
        pkg_id,
        manifest,
    };
    gctx.shell().print_json(&msg)?;

    match ws_config {
        WorkspaceConfig::Root(..) => {
            // skip if the current manifest *is* the workspace manifest
        }
        WorkspaceConfig::Member {
            root: Some(path_to_root),
        } => {
            let path_to_root = PathBuf::from(path_to_root);
            print_workspace_root(gctx, manifest_path, path_to_root)?;
        }
        WorkspaceConfig::Member { root: None } => {
            if let Some(path_to_root) = find_workspace_root(&manifest_path, gctx)? {
                print_workspace_root(gctx, manifest_path, path_to_root)?;
            }
        }
    };

    Ok(())
}

fn print_workspace_root(
    gctx: &GlobalContext,
    manifest_path: PathBuf,
    path_to_root: PathBuf,
) -> CargoResult<()> {
    let workspace_path = paths::normalize_path(&gctx.cwd().join(path_to_root));
    let source_id = SourceId::for_manifest_path(&workspace_path)?;

    let (pkg_id, manifest) = match read_manifest(&manifest_path, source_id, gctx)? {
        EitherManifest::Real(r) => (Some(r.package_id().to_spec()), r.normalized_toml().clone()),
        EitherManifest::Virtual(v) => (None, v.normalized_toml().clone()),
    };

    let msg = ReadManifestMessage::Manifest {
        path: manifest_path.clone(),
        pkg_id,
        manifest,
    };
    gctx.shell().print_json(&msg)?;

    Ok(())
}
