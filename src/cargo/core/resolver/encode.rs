//! Definition of how to encode a `Resolve` into a TOML `Cargo.lock` file
//!
//! This module is a temporary copy from the cargo codebase.

use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

use cargo::{
    core::{GitReference, SourceKind},
    CargoResult,
};
use serde::{de, ser, Deserialize, Serialize};
use url::Url;

/// The `Cargo.lock` structure.
#[derive(Serialize, Deserialize, Debug)]
pub struct EncodableResolve {
    version: Option<u32>,
    package: Option<Vec<EncodableDependency>>,
    /// `root` is optional to allow backward compatibility.
    root: Option<EncodableDependency>,
    metadata: Option<Metadata>,
    #[serde(default, skip_serializing_if = "Patch::is_empty")]
    patch: Patch,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Patch {
    unused: Vec<EncodableDependency>,
}

pub type Metadata = BTreeMap<String, String>;

impl Patch {
    fn is_empty(&self) -> bool {
        self.unused.is_empty()
    }
}

#[derive(Eq, PartialEq, Clone, Debug, Hash)]
pub enum Precise {
    Locked,
    GitUrlFragment(String),
}

impl fmt::Display for Precise {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Precise::Locked => "locked".fmt(f),
            Precise::GitUrlFragment(s) => s.fmt(f),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EncodableDependency {
    name: String,
    version: String,
    source: Option<EncodableSourceId>,
    checksum: Option<String>,
    dependencies: Option<Vec<EncodablePackageId>>,
    replace: Option<EncodablePackageId>,
}

#[derive(Debug)]
pub struct EncodableSourceId {
    pub url: Url,
    pub kind: SourceKind,
    pub precise: Option<Precise>,
    pub encoded: bool,
}

impl EncodableSourceId {
    pub fn new(url: Url, kind: SourceKind) -> Self {
        Self {
            url,
            kind,
            encoded: true,
            precise: None,
        }
    }

    pub fn from_url(string: &str) -> CargoResult<Self> {
        let (kind, url) = string
            .split_once('+')
            .ok_or_else(|| anyhow::format_err!("invalid source `{}`", string))?;

        match kind {
            "git" => {
                let mut url = str_to_url(url)?;
                let reference = GitReference::from_query(url.query_pairs());
                let precise = url.fragment().map(|s| s.to_owned());
                url.set_fragment(None);
                url.set_query(None);
                Ok(Self {
                    url,
                    kind: SourceKind::Git(reference),
                    encoded: false,
                    precise: precise.map(Precise::GitUrlFragment),
                })
            }
            "registry" => {
                let url = str_to_url(url)?;
                Ok(Self {
                    url,
                    kind: SourceKind::Registry,
                    encoded: false,
                    precise: Some(Precise::Locked),
                })
            }
            "sparse" => {
                let url = str_to_url(string)?;
                Ok(Self {
                    url,
                    kind: SourceKind::SparseRegistry,
                    encoded: false,
                    precise: Some(Precise::Locked),
                })
            }
            "path" => {
                let url = str_to_url(url)?;
                Ok(Self {
                    url,
                    kind: SourceKind::Path,
                    encoded: false,
                    precise: None,
                })
            }
            kind => Err(anyhow::format_err!("unsupported source protocol: {}", kind)),
        }
    }

    fn is_path(&self) -> bool {
        self.kind == SourceKind::Path
    }
}

impl fmt::Display for EncodableSourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(protocol) = self.kind.protocol() {
            write!(f, "{protocol}+")?;
        }
        write!(f, "{}", self.url)?;
        if let Self {
            kind: SourceKind::Git(ref reference),
            ref precise,
            ..
        } = self
        {
            if let Some(pretty) = reference.pretty_ref(true) {
                write!(f, "?{pretty}")?;
            }
            if let Some(precise) = precise.as_ref() {
                write!(f, "#{precise}")?;
            }
        }
        Ok(())
    }
}

impl Serialize for EncodableSourceId {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        if self.is_path() {
            None::<String>.serialize(s)
        } else {
            s.collect_str(self)
        }
    }
}

impl<'de> Deserialize<'de> for EncodableSourceId {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let string = String::deserialize(d)?;
        Self::from_url(&string).map_err(de::Error::custom)
    }
}

#[derive(Debug)]
pub struct EncodablePackageId {
    name: String,
    version: Option<String>,
    source: Option<EncodableSourceId>,
}

impl fmt::Display for EncodablePackageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if let Some(s) = &self.version {
            write!(f, " {s}")?;
        }
        if let Some(s) = &self.source {
            write!(f, " ({s})")?;
        }
        Ok(())
    }
}

impl FromStr for EncodablePackageId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> CargoResult<EncodablePackageId> {
        let mut s = s.splitn(3, ' ');
        let name = s.next().unwrap();
        let version = s.next();
        let source_id = match s.next() {
            Some(s) => {
                if let Some(s) = s.strip_prefix('(').and_then(|s| s.strip_suffix(')')) {
                    Some(EncodableSourceId::from_url(s)?)
                } else {
                    anyhow::bail!("invalid serialized PackageId")
                }
            }
            None => None,
        };

        Ok(EncodablePackageId {
            name: name.to_owned(),
            version: version.map(|v| v.to_owned()),
            // Default to url encoded.
            source: source_id,
        })
    }
}

impl Serialize for EncodablePackageId {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        s.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for EncodablePackageId {
    fn deserialize<D>(d: D) -> Result<EncodablePackageId, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        String::deserialize(d).and_then(|string| {
            string
                .parse::<EncodablePackageId>()
                .map_err(de::Error::custom)
        })
    }
}

fn str_to_url(string: &str) -> CargoResult<Url> {
    Url::parse(string).map_err(|s| {
        if string.starts_with("git@") {
            anyhow::format_err!(
                "invalid url `{}`: {}; try using `{}` instead",
                string,
                s,
                format_args!("ssh://{}", string.replacen(':', "/", 1))
            )
        } else {
            anyhow::format_err!("invalid url `{}`: {}", string, s)
        }
    })
}
