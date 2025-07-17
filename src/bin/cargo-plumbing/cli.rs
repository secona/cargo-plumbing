use cargo::{CargoResult, GlobalContext};
use clap::{ArgAction, Parser};

#[derive(Debug, Parser)]
#[command(bin_name = "cargo")]
#[command(styles = clap_cargo::style::CLAP_STYLING)]
pub(crate) enum Command {
    #[command()]
    Plumbing(PlumbingCommand),
}

#[derive(Debug, Parser)]
pub(crate) struct PlumbingCommand {
    #[arg(global = true, long)]
    pub(crate) locked: bool,

    #[arg(global = true, long)]
    pub(crate) offline: bool,

    #[arg(global = true, long)]
    pub(crate) frozen: bool,

    #[arg(global = true, long, value_name = "KEY=VALUE", action = ArgAction::Append)]
    pub(crate) config: Vec<String>,

    #[command(subcommand)]
    pub(crate) subcommand: crate::plumbing::Plumbing,
}

impl Command {
    pub(crate) fn exec(self, gctx: &mut GlobalContext) -> CargoResult<()> {
        match self {
            Self::Plumbing(cmd) => {
                let verbose = 0;
                let quiet = false;
                let color = None;
                let target_dir = None;
                let unstable_flags = &[];

                gctx.configure(
                    verbose,
                    quiet,
                    color,
                    cmd.frozen,
                    cmd.locked,
                    cmd.offline,
                    &target_dir,
                    unstable_flags,
                    &cmd.config,
                )?;

                cmd.subcommand.exec(gctx)
            }
        }
    }
}

#[test]
fn verify_app() {
    use clap::CommandFactory;
    Command::command().debug_assert();
}
