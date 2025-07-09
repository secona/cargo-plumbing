//! Messages used by `cargo plumbing locate-manifest` command

use std::{io::Read, marker::PhantomData};

use serde::{Deserialize, Serialize};

use crate::MessageIter;

/// Represents the messages outputted by the `cargo-plumbing locate-manifest` command.
///
/// This enum captures all possible JSON objects that the command can emit. The `reason`
/// field acts as a discriminant to distinguish between different message types.
#[derive(Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "kebab-case")]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
pub enum LocateManifestMessage {
    /// A message containing the location of a found `Cargo.toml` manifest.
    ManifestLocation {
        /// The absolute path to the manifest file.
        manifest_path: String,
    },
}

impl LocateManifestMessage {
    /// Creates an iterator to parse a stream of [`LocateManifestMessage`]s.
    pub fn parse_stream<R: Read>(input: R) -> MessageIter<R, Self> {
        MessageIter {
            input,
            _m: PhantomData::<Self>,
        }
    }
}

#[cfg(feature = "unstable-schema")]
#[test]
fn dump_project_location_schema() {
    let schema = schemars::schema_for!(LocateManifestMessage);
    let dump = serde_json::to_string_pretty(&schema).unwrap();
    snapbox::assert_data_eq!(dump, snapbox::file!("../locate-manifest.schema.json").raw());
}
