use cargo::GlobalContext;
use cargo_plumbing::CargoResult;

#[derive(Debug, clap::Subcommand)]
#[command(styles = clap_cargo::style::CLAP_STYLING)]
pub(crate) enum Plumbing {
    /// Temporary dummy command to make the enum inhabited
    Dummy,
}

impl Plumbing {
    pub(crate) fn exec(self, _gctx: &GlobalContext) -> CargoResult<()> {
        match self {
            Self::Dummy => anyhow::bail!("not implemented"),
        }
    }
}
