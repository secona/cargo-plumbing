use std::path::PathBuf;

use cargo::core::{EitherManifest, SourceId};
use cargo::util::toml::read_manifest;
use cargo::{CargoResult, GlobalContext};
use cargo_plumbing_schemas::read_manifest::ReadManifestMessage;

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    /// Path to the manifest file
    #[clap(long)]
    manifest_path: PathBuf,
}

pub(crate) fn exec(gctx: &GlobalContext, args: Args) -> CargoResult<()> {
    let manifest_path = gctx.cwd().join(args.manifest_path);
    let source_id = SourceId::for_manifest_path(&manifest_path)?;

    let (pkg_id, manifest) = match read_manifest(&manifest_path, source_id, gctx)? {
        EitherManifest::Real(r) => (Some(r.package_id().to_spec()), r.normalized_toml().clone()),
        EitherManifest::Virtual(v) => (None, v.normalized_toml().clone()),
    };

    let msg = ReadManifestMessage::Manifest {
        path: manifest_path,
        pkg_id,
        manifest,
    };
    gctx.shell().print_json(&msg)?;

    Ok(())
}
