use std::path::PathBuf;

use cargo::core::{
    find_workspace_root, EitherManifest, MaybePackage, SourceId, Workspace, WorkspaceConfig,
};
use cargo::util::toml::read_manifest;
use cargo::{CargoResult, GlobalContext};
use cargo_plumbing_schemas::read_manifest::ReadManifestMessage;
use cargo_util::paths;

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    /// Path to the manifest file
    #[clap(long)]
    manifest_path: PathBuf,
    /// Read all manifests in the workspace
    #[clap(long, default_value_t = false)]
    workspace: bool,
}

pub(crate) fn exec(gctx: &GlobalContext, args: Args) -> CargoResult<()> {
    let manifest_path = gctx.cwd().join(args.manifest_path);

    if args.workspace {
        let workspace = Workspace::new(&manifest_path, gctx)?;

        // Here, we print the root Package or the Virtual Manifest of the workspace.
        let msg = match workspace.root_maybe() {
            MaybePackage::Package(pkg) => ReadManifestMessage::Manifest {
                path: gctx.cwd().join(pkg.manifest_path()),
                pkg_id: Some(pkg.package_id().to_spec()),
                manifest: pkg.manifest().normalized_toml().clone(),
            },
            MaybePackage::Virtual(v) => ReadManifestMessage::Manifest {
                path: gctx.cwd().join(workspace.root_manifest()),
                pkg_id: None,
                manifest: v.normalized_toml().clone(),
            },
        };
        gctx.shell().print_json(&msg)?;

        // We don't want to print the same manifest twice. This is why we're filtering the
        // workspace members to make sure only those that are not root manifest are printed in this
        // stage.
        for member in workspace
            .members()
            .filter(|p| p.manifest_path() != workspace.root_manifest())
        {
            let msg = ReadManifestMessage::Manifest {
                path: gctx.cwd().join(member.manifest_path()),
                pkg_id: Some(member.package_id().to_spec()),
                manifest: member.manifest().normalized_toml().clone(),
            };
            gctx.shell().print_json(&msg)?;
        }
    } else {
        // As this is the branch without `--workspace`, we want to only read one manifest.
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

        // We also want to print the root workspace even if no `--workspace` flag is provided.
        match ws_config {
            WorkspaceConfig::Root(..) => {
                // Skip if the current manifest *is* the workspace manifest.
            }
            WorkspaceConfig::Member {
                root: Some(path_to_root),
            } => {
                // This case is when the workspace members is defined through the package workspace
                // key. Hence, we make it into a `PathBuf` first.
                let path_to_root = PathBuf::from(path_to_root);
                print_workspace_root(gctx, manifest_path, path_to_root)?;
            }
            WorkspaceConfig::Member { root: None } => {
                // This case is the common case for workspace members where the members are defined
                // from the workspace manifest
                if let Some(path_to_root) = find_workspace_root(&manifest_path, gctx)? {
                    print_workspace_root(gctx, manifest_path, path_to_root)?;
                }
            }
        };
    }

    Ok(())
}

/// Given a manifest path the the path to workspace root, we print the manifest there.
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
