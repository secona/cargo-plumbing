use std::io::{self, BufReader, IsTerminal};

use cargo::core::{PackageId, PackageIdSpec, SourceId, SourceKind};
use cargo::{CargoResult, GlobalContext};
use cargo_plumbing_schemas::read_lockfile::ReadLockfileMessage;

#[derive(Debug, clap::Args)]
pub(crate) struct Args {}

pub(crate) fn exec(_gctx: &mut GlobalContext, _args: Args) -> CargoResult<()> {
    let stdin = io::stdin();
    if stdin.is_terminal() {
        anyhow::bail!("input must be piped from a file or another command");
    }

    let messages = ReadLockfileMessage::parse_stream(BufReader::new(stdin));

    let mut lock_version = None;

    for message in messages {
        match message? {
            ReadLockfileMessage::Lockfile { version } => lock_version = Some(version),
            ReadLockfileMessage::LockedPackage { package } => {
                let Ok(Some(..)) = spec_to_id(package.id) else {
                    continue;
                };
            }
            _ => {}
        }
    }

    let _ = lock_version;

    Ok(())
}

fn spec_to_id(spec: PackageIdSpec) -> CargoResult<Option<PackageId>> {
    if let Some(kind) = spec.kind() {
        if let Some(url) = spec.url() {
            if let Some(version) = spec.version() {
                let name = spec.name();
                let source_id = match kind {
                    SourceKind::Git(git_ref) => SourceId::for_git(url, git_ref.clone()),
                    SourceKind::Registry | SourceKind::SparseRegistry => {
                        SourceId::for_registry(url)
                    }
                    _ => anyhow::bail!("unsupported source"),
                }?;

                return Ok(Some(PackageId::new(name.into(), version, source_id)));
            }
        }
    }

    Ok(None)
}
