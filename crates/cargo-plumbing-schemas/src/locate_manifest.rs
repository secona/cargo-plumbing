use std::{io::Read, marker::PhantomData};

use serde::{Deserialize, Serialize};

use crate::MessageIter;

#[derive(Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "kebab-case")]
#[cfg_attr(feature = "unstable-schema", derive(schemars::JsonSchema))]
pub enum LocateManifestMessage {
    ManifestLocation { manifest_path: String },
}

impl LocateManifestMessage {
    pub fn parse_stream<R: Read>(input: R) -> MessageIter<R, Self> {
        MessageIter {
            _m: PhantomData::<Self>,
            input,
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
