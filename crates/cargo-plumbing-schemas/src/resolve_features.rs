use std::io::Read;
use std::marker::PhantomData;

use cargo_util_schemas::core::PackageIdSpec;
use serde::{Deserialize, Serialize};

use crate::lockfile::{NormalizedDependency, NormalizedPatch};
use crate::MessageIter;

/// Input messages for `cargo-plumbing resolve-features`.
#[derive(Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "kebab-case")]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
#[allow(clippy::large_enum_variant)]
pub enum ResolveFeaturesIn {
    Manifest {
        #[cfg_attr(feature = "unstable-schema", schemars(with = "Option<String>"))]
        id: PackageIdSpec,
    },
    Lockfile {
        version: Option<u32>,
    },
    LockedPackage {
        #[serde(flatten)]
        package: NormalizedDependency,
    },
    UnusedPatches {
        unused: NormalizedPatch,
    },
}

impl ResolveFeaturesIn {
    /// Creates an iterator to parse a stream of [`ResolveFeaturesIn`]s.
    pub fn parse_stream<R: Read>(input: R) -> MessageIter<R, Self> {
        MessageIter {
            input,
            _m: PhantomData::<Self>,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "kebab-case")]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
pub enum ResolveFeaturesOut {
    Activated {
        id: PackageIdSpec,
        #[serde(skip_serializing_if = "String::is_empty")]
        features_for: String,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        features: Vec<String>,
    },
}

#[cfg(feature = "unstable-schema")]
#[test]
fn dump_resolve_features_schema() {
    let schema = schemars::schema_for!(ResolveFeaturesIn);
    let dump = serde_json::to_string_pretty(&schema).unwrap();
    snapbox::assert_data_eq!(
        dump,
        snapbox::file!("../resolve-features.in.schema.json").raw()
    );

    let schema = schemars::schema_for!(ResolveFeaturesOut);
    let dump = serde_json::to_string_pretty(&schema).unwrap();
    snapbox::assert_data_eq!(
        dump,
        snapbox::file!("../resolve-features.out.schema.json").raw()
    );
}
