//! Messages used by `cargo plumbing locate-manifest` command

use std::{io::Read, marker::PhantomData};

use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};

use crate::MessageIter;

/// Output messages for `cargo-plumbing locate-manifest`.
#[derive(Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "kebab-case")]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
pub enum LocateManifestOut {
    /// A message containing the location of a found `Cargo.toml` manifest.
    ManifestLocation {
        /// The absolute path to the manifest file.
        #[cfg_attr(feature = "unstable-schema", schemars(with = "String"))]
        manifest_path: Utf8PathBuf,
    },
}

impl LocateManifestOut {
    /// Creates an iterator to parse a stream of [`LocateManifestOut`]s.
    pub fn parse_stream<R: Read>(input: R) -> MessageIter<R, Self> {
        MessageIter {
            input,
            _m: PhantomData::<Self>,
        }
    }
}

#[cfg(feature = "unstable-schema")]
#[test]
fn dump_locate_manifest_schema() {
    let schema = schemars::schema_for!(LocateManifestOut);
    let dump = serde_json::to_string_pretty(&schema).unwrap();
    snapbox::assert_data_eq!(
        dump,
        snapbox::file!("../locate-manifest.out.schema.json").raw()
    );
}
