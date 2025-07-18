//! Messages used by `cargo plumbing read-lockfile` command

use std::io::Read;
use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::lockfile::{Metadata, NormalizedDependency, NormalizedPatch};
use crate::MessageIter;

/// Represents the messages outputted by the `cargo-plumbing read-lockfile` command.
///
/// This enum captures all possible JSON objects that the command can emit. The `reason`
/// field acts as a discriminant to distinguish between different message types.
#[derive(Deserialize, Serialize)]
#[serde(tag = "reason", rename_all = "kebab-case")]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
#[allow(clippy::large_enum_variant)]
pub enum ReadLockfileMessage {
    Lockfile {
        version: Option<u32>,
    },
    LockedPackage {
        #[serde(flatten)]
        package: NormalizedDependency,
    },
    Metadata {
        #[serde(flatten)]
        metadata: Metadata,
    },
    UnusedPatches {
        unused: NormalizedPatch,
    },
}

impl ReadLockfileMessage {
    /// Creates an iterator to parse a stream of [`ReadLockfileMessage`]s.
    pub fn parse_stream<R: Read>(input: R) -> MessageIter<R, Self> {
        MessageIter {
            input,
            _m: PhantomData::<Self>,
        }
    }
}

#[cfg(feature = "unstable-schema")]
#[test]
fn dump_read_lockfile_schema() {
    let schema = schemars::schema_for!(ReadLockfileMessage);
    let dump = serde_json::to_string_pretty(&schema).unwrap();
    snapbox::assert_data_eq!(dump, snapbox::file!("../read-lockfile.schema.json").raw());
}
