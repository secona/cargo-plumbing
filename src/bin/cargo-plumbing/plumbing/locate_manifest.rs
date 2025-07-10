use std::env;
use std::fmt::Debug;
use std::path::PathBuf;

use camino::Utf8PathBuf;
use cargo::util::important_paths::find_root_manifest_for_wd;
use cargo::{CargoResult, GlobalContext};
use cargo_plumbing_schemas::locate_manifest::LocateManifestMessage;

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    /// Path to the manifest file
    #[arg(long)]
    manifest_path: Option<PathBuf>,
}

pub(crate) fn exec(gctx: &GlobalContext, args: Args) -> CargoResult<()> {
    let path = args.manifest_path.unwrap_or(env::current_dir()?);
    let manifest_path = find_root_manifest_for_wd(&path)?;

    let location = LocateManifestMessage::ManifestLocation {
        manifest_path: Utf8PathBuf::try_from(manifest_path)?,
    };
    gctx.shell().print_json(&location)?;

    Ok(())
}
