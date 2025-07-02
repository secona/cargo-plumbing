use std::io::BufReader;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use cargo::CargoResult;
use cargo_plumbing_schemas::locate_manifest::LocateManifestMessage;
use cargo_plumbing_schemas::read_manifest::{ReadManifestMessage, TomlManifest};
use clap::Parser;

#[derive(Debug, Parser)]
struct Args {
    #[arg(long)]
    manifest_path: Option<PathBuf>,
}

fn run(args: &Args) -> CargoResult<()> {
    let manifest_path = {
        let mut cmd = Command::new("cargo");
        cmd.args(["run", "plumbing", "locate-manifest"])
            .stdout(Stdio::piped());

        if let Some(manifest_path) = &args.manifest_path {
            cmd.arg("--manifest-path");
            cmd.arg(manifest_path);
        }

        let mut child = cmd.spawn().expect("failed to run locate-manifest");
        let stdout = child.stdout.take().expect("failed to get stdout");
        let messages = LocateManifestMessage::parse_stream(BufReader::new(stdout));

        let mut manifest_path: Option<String> = None;

        #[allow(clippy::never_loop)]
        for message in messages {
            match message.expect("failed to parse message") {
                LocateManifestMessage::ManifestLocation { manifest_path: m } => {
                    manifest_path = Some(m);
                    break;
                }
            }
        }

        child.wait().expect("failed to wait for locate-manifest");
        manifest_path.expect("failed to get manifest_path")
    };

    let _manifest = {
        let mut cmd = Command::new("cargo");
        cmd.args(["run", "plumbing", "read-manifest"])
            .args(["--manifest-path", &manifest_path])
            .stdout(Stdio::piped());

        let mut child = cmd.spawn().expect("failed to run read-manifest");
        let stdout = child.stdout.take().expect("failed to get stdout");
        let messages = ReadManifestMessage::parse_stream(BufReader::new(stdout));

        let mut manifest: Option<TomlManifest> = None;

        #[allow(clippy::never_loop)]
        for message in messages {
            match message.expect("failed to parse message") {
                ReadManifestMessage::Manifest { manifest: m, .. } => {
                    manifest = Some(m);
                    break;
                }
            }
        }

        child.wait().expect("failed to wait for read-manifest");
        manifest.expect("failed to get manifest")
    };

    anyhow::bail!("check for {manifest_path} is not implemented!");
}

fn main() {
    let args = Args::parse();
    match run(&args) {
        Ok(()) => {}
        Err(e) => println!("error: {e}"),
    }
}
