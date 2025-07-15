//! `Cargo.lock` / Lock file schema definition

use std::{collections::BTreeMap, fmt, str::FromStr};

use cargo_util_schemas::core::{GitReference, SourceKind};
use serde::{de, ser, Deserialize, Serialize};
use url::Url;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct UrlParseError(#[from] UrlParseErrorKind);

#[derive(Debug, thiserror::Error)]
enum UrlParseErrorKind {
    #[error("invalid url `{0}`: {1}; try using `{2}` instead")]
    UrlSuggest(String, url::ParseError, String),

    #[error("invalid url `{0}`: {1}")]
    Url(String, url::ParseError),
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum EncodeError {
    Encode(#[from] EncodeErrorKind),
    UrlParse(#[from] UrlParseError),
}

#[derive(Debug, thiserror::Error)]
pub enum EncodeErrorKind {
    #[error("invalid serialized PackageId")]
    InvalidSerializedPackageId,

    #[error("invalid source `{0}`")]
    InvalidSource(String),

    #[error("unsupported source protocol: {0}")]
    UnsupportedSourceProtocol(String),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct EncodableResolve {
    pub version: Option<u32>,
    pub package: Option<Vec<EncodableDependency>>,
    pub root: Option<EncodableDependency>,
    pub metadata: Option<Metadata>,
    #[serde(default, skip_serializing_if = "EncodablePatch::is_empty")]
    pub patch: EncodablePatch,
}

pub type Metadata = BTreeMap<String, String>;

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "kebab-case")]
pub struct EncodablePatch {
    pub unused: Vec<EncodableDependency>,
}

impl EncodablePatch {
    fn is_empty(&self) -> bool {
        self.unused.is_empty()
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct EncodableDependency {
    pub name: String,
    pub version: String,
    pub source: Option<EncodableSourceId>,
    pub checksum: Option<String>,
    pub dependencies: Option<Vec<EncodablePackageId>>,
    pub replace: Option<EncodablePackageId>,
}

#[derive(Debug)]
pub struct EncodablePackageId {
    pub name: String,
    pub version: Option<String>,
    pub source: Option<EncodableSourceId>,
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
    type Err = EncodeError;

    fn from_str(s: &str) -> Result<EncodablePackageId, Self::Err> {
        let mut s = s.splitn(3, ' ');
        let name = s.next().unwrap();
        let version = s.next();
        let source_id = match s.next() {
            Some(s) => {
                if let Some(s) = s.strip_prefix('(').and_then(|s| s.strip_suffix(')')) {
                    Some(EncodableSourceId::from_url(s)?)
                } else {
                    return Err(EncodeErrorKind::InvalidSerializedPackageId.into());
                }
            }
            None => None,
        };

        Ok(EncodablePackageId {
            name: name.to_owned(),
            version: version.map(|v| v.to_owned()),
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

    pub fn from_url(string: &str) -> Result<Self, EncodeError> {
        let (kind, url) = string
            .split_once('+')
            .ok_or_else(|| EncodeErrorKind::InvalidSource(string.to_owned()))?;

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
            kind => Err(EncodeErrorKind::UnsupportedSourceProtocol(kind.to_string()).into()),
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

fn str_to_url(string: &str) -> Result<Url, UrlParseError> {
    Url::parse(string).map_err(|s| {
        if string.starts_with("git@") {
            let suggestion = format!("ssh://{}", string.replacen(':', "/", 1));
            UrlParseErrorKind::UrlSuggest(string.to_string(), s, suggestion).into()
        } else {
            UrlParseErrorKind::Url(string.to_string(), s).into()
        }
    })
}
