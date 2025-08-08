//! Messages used by `cargo plumbing lock-dependencies` command

use std::{io::Read, marker::PhantomData};

use serde::{Deserialize, Serialize};

use crate::lockfile::{NormalizedDependency, NormalizedMetadata, NormalizedPatch};
use crate::MessageIter;

/// Output messages for `cargo-plumbing lock-dependencies`.
#[derive(Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "kebab-case")]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
#[allow(clippy::large_enum_variant)]
pub enum LockDependenciesOut {
    Lockfile {
        version: Option<u32>,
    },
    LockedPackage {
        #[serde(flatten)]
        package: NormalizedDependency,
    },
    Metadata {
        #[serde(flatten)]
        metadata: NormalizedMetadata,
    },
    UnusedPatches {
        unused: NormalizedPatch,
    },
}

impl LockDependenciesOut {
    /// Creates an iterator to parse a stream of [`LockDependenciesOut`]s.
    pub fn parse_stream<R: Read>(input: R) -> MessageIter<R, Self> {
        MessageIter {
            input,
            _m: PhantomData::<Self>,
        }
    }
}

#[cfg(feature = "unstable-schema")]
#[test]
fn dump_lock_dependencies_schema() {
    let schema = schemars::schema_for!(LockDependenciesOut);
    let dump = serde_json::to_string_pretty(&schema).unwrap();
    snapbox::assert_data_eq!(
        dump,
        snapbox::file!("../lock-dependencies.out.schema.json").raw()
    );
}
