use std::collections::{BTreeMap, HashMap, HashSet};

use cargo::core::{
    PackageId, PackageIdSpec, Resolve, ResolveVersion, SourceId, SourceKind, Workspace,
};
use cargo::util::Graph;
use cargo::CargoResult;
use cargo_plumbing_schemas::lockfile::{NormalizedDependency, NormalizedPatch};

use crate::cargo::core::resolver::encode::build_path_deps;

/// Converts plumbing messages into an incomplete [`Resolve`]
///
/// The `features` and `summaries` fields of the returned struct is empty.
pub fn into_resolve(
    ws: &Workspace<'_>,
    version: Option<u32>,
    packages: Vec<NormalizedDependency>,
    patch: NormalizedPatch,
) -> CargoResult<Resolve> {
    let path_deps = build_path_deps(ws)?;

    let mut checksums = HashMap::new();

    let live_pkgs = {
        let mut all_pkgs = HashSet::new();
        let mut live_pkgs = HashMap::new();
        for pkg in packages.iter() {
            if !all_pkgs.insert(pkg.id.clone()) {
                anyhow::bail!("package `{}` is specified twice", pkg.id.name());
            }

            let source_id = get_path_deps_source_id(&path_deps, pkg.id.name(), pkg.id.version());
            let Ok(Some(id)) = spec_to_id(pkg.id.clone(), source_id) else {
                continue;
            };

            if let Some(cksum) = &pkg.checksum {
                checksums.insert(id, Some(cksum.clone()));
            }

            live_pkgs.insert(pkg.id.clone(), (id, pkg));
        }
        live_pkgs
    };

    // When decoding a V2 version the edges in `dependencies` aren't
    // guaranteed to have either version or source information. This `map`
    // is used to find package ids even if dependencies have missing
    // information. This map is from name to version to source to actual
    // package ID. (various levels to drill down step by step)
    let mut map = HashMap::new();
    for (id, _) in live_pkgs.values() {
        map.entry(id.name().as_str())
            .or_insert_with(HashMap::new)
            .entry(id.version())
            .or_insert_with(HashMap::new)
            .insert(id.source_id(), *id);
    }

    let lookup_id = |pkg_id: &PackageIdSpec, source_id: Option<&SourceId>| -> Option<PackageId> {
        let by_version = map.get(pkg_id.name())?;

        let by_source = match &pkg_id.version() {
            Some(version) => by_version.get(version)?,
            None => {
                if by_version.len() == 1 {
                    by_version.values().next().unwrap()
                } else {
                    return None;
                }
            }
        };

        match &source_id {
            Some(source) => by_source.get(source).cloned(),
            None => {
                let mut path_packages = by_source.values().filter(|p| p.source_id().is_path());
                if let Some(path) = path_packages.next() {
                    if path_packages.next().is_some() {
                        None
                    } else {
                        Some(*path)
                    }
                } else if by_source.len() == 1 {
                    Some(*by_source.values().next().unwrap())
                } else {
                    None
                }
            }
        }
    };

    let graph = {
        let mut g = Graph::new();

        for (id, _) in live_pkgs.values() {
            g.add(*id);
        }

        for &(ref id, pkg) in live_pkgs.values() {
            let Some(ref deps) = pkg.dependencies else {
                continue;
            };

            for edge in deps.iter() {
                let package_id = spec_to_id(edge.clone(), None)?;
                let source_id = package_id.map(|p| p.source_id());
                if let Some(to_depend_on) = lookup_id(edge, source_id.as_ref()) {
                    g.link(*id, to_depend_on);
                }
            }
        }
        g
    };

    let replacements = {
        let mut replacements = HashMap::new();
        for &(ref id, pkg) in live_pkgs.values() {
            if let Some(ref replace) = pkg.replace {
                assert!(pkg.dependencies.is_none());
                let source_id = id.source_id();
                if let Some(replace_id) = lookup_id(replace, Some(&source_id)) {
                    replacements.insert(*id, replace_id);
                }
            }
        }
        replacements
    };

    let unused_patches = {
        let mut unused_patches = Vec::new();
        for pkg in patch.unused {
            let source_id = get_path_deps_source_id(&path_deps, pkg.id.name(), pkg.id.version());
            let Ok(Some(id)) = spec_to_id(pkg.id.clone(), source_id) else {
                continue;
            };
            unused_patches.push(id);
        }
        unused_patches
    };

    let metadata = BTreeMap::new();
    let features = HashMap::new();
    let summaries = HashMap::new();

    let version = match version {
        Some(4) => ResolveVersion::V4,
        Some(3) => ResolveVersion::V3,
        Some(2) => ResolveVersion::V2,
        Some(1) => ResolveVersion::V1,
        None => ResolveVersion::V2,
        Some(_) => anyhow::bail!("invalid lockfile version"),
    };

    Ok(Resolve::new(
        graph,
        replacements,
        features,
        checksums,
        metadata,
        unused_patches,
        version,
        summaries,
    ))
}

pub fn get_path_deps_source_id<'a>(
    path_deps: &'a HashMap<String, HashMap<semver::Version, SourceId>>,
    package_name: &str,
    package_version: Option<semver::Version>,
) -> Option<&'a SourceId> {
    path_deps.iter().find_map(|(name, version_source)| {
        if name != package_name || version_source.is_empty() {
            return None;
        }

        if version_source.len() == 1 {
            return Some(version_source.values().next().unwrap());
        }

        if let Some(pkg_version) = &package_version {
            if let Some(source_id) = version_source.get(pkg_version) {
                return Some(source_id);
            }
        }

        None
    })
}

pub fn spec_to_id(
    spec: PackageIdSpec,
    source_id: Option<&SourceId>,
) -> CargoResult<Option<PackageId>> {
    if let Some(kind) = spec.kind() {
        if let Some(url) = spec.url() {
            if let Some(version) = spec.version() {
                let name = spec.name();
                let source_id = match kind {
                    SourceKind::Git(git_ref) => SourceId::for_git(url, git_ref.clone()),
                    SourceKind::Registry | SourceKind::SparseRegistry => {
                        SourceId::for_registry(url)
                    }
                    _ => anyhow::bail!("unsupported source"),
                }?;

                return Ok(Some(PackageId::new(name.into(), version, source_id)));
            }
        }
    }

    if let Some(source_id) = source_id {
        if let Some(version) = spec.version() {
            let name = spec.name();
            return Ok(Some(PackageId::new(name.into(), version, *source_id)));
        }
    }

    Ok(None)
}
