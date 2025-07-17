use cargo_test_support::basic_manifest;
use cargo_test_support::git;
use cargo_test_support::prelude::*;
use cargo_test_support::project;
use cargo_test_support::registry::Package;
use cargo_test_support::registry::RegistryBuilder;
use cargo_test_support::str;

use crate::ProjectExt;

#[cargo_test]
fn package_with_no_deps() {
    let p = project()
        .file("src/lib.rs", "")
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "read-manifest-test"
                version = "0.1.0"
                authors = []
                edition = "2024"
            "#,
        )
        .build();

    let manifest_path = p.root().join("Cargo.toml");
    p.cargo_plumbing("plumbing lock-dependencies")
        .args(&["--manifest-path", manifest_path.to_str().unwrap()])
        .with_status(0)
        .with_stderr_data("")
        .with_stdout_data(
            str![[r#"
[
  {
    "metadata": null,
    "package": [
      {
        "checksum": null,
        "dependencies": [],
        "name": "read-manifest-test",
        "replace": null,
        "source": null,
        "version": "0.1.0"
      }
    ],
    "root": null,
    "version": 4
  }
]
"#]]
            .is_json()
            .against_jsonlines(),
        )
        .run();
}

#[cargo_test]
fn package_with_deps() {
    Package::new("a", "1.0.0")
        .file("src/lib.rs", r#"pub fn f() -> i32 { 12 }"#)
        .publish();

    let p = project()
        .file("src/lib.rs", "")
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "read-manifest-test"
                version = "0.1.0"
                authors = []
                edition = "2024"

                [dependencies]
                a = "1.0.0"
            "#,
        )
        .build();

    let manifest_path = p.root().join("Cargo.toml");
    p.cargo_plumbing("plumbing lock-dependencies")
        .args(&["--manifest-path", manifest_path.to_str().unwrap()])
        .with_status(0)
        .with_stderr_data(str![[r#"
[UPDATING] `dummy-registry` index

"#]])
        .with_stdout_data(
            str![[r#"
[
  {
    "metadata": null,
    "package": [
      {
        "checksum": "8ade940ad37d44e5895dd7dd5d1cd392bd0007a59a067b0ee1f4f114eec7ff41",
        "dependencies": [],
        "name": "a",
        "replace": null,
        "source": "registry+https://github.com/rust-lang/crates.io-index",
        "version": "1.0.0"
      },
      {
        "checksum": null,
        "dependencies": [
          "a"
        ],
        "name": "read-manifest-test",
        "replace": null,
        "source": null,
        "version": "0.1.0"
      }
    ],
    "root": null,
    "version": 4
  }
]
"#]]
            .is_json()
            .against_jsonlines(),
        )
        .run();
}

#[cargo_test]
fn package_with_path_deps() {
    let p = project()
        .file("b/src/lib.rs", "")
        .file(
            "b/Cargo.toml",
            r#"
                [package]
                name = "b"
                version = "1.0.0"
                authors = []
                edition = "2024"
            "#,
        )
        .file("src/lib.rs", "")
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "read-manifest-test"
                version = "0.1.0"
                authors = []
                edition = "2024"

                [dependencies]
                b = { path = "./b" }
            "#,
        )
        .build();

    let manifest_path = p.root().join("Cargo.toml");
    p.cargo_plumbing("plumbing lock-dependencies")
        .args(&["--manifest-path", manifest_path.to_str().unwrap()])
        .with_status(0)
        .with_stderr_data("")
        .with_stdout_data(
            str![[r#"
[
  {
    "metadata": null,
    "package": [
      {
        "checksum": null,
        "dependencies": [],
        "name": "b",
        "replace": null,
        "source": null,
        "version": "1.0.0"
      },
      {
        "checksum": null,
        "dependencies": [
          "b"
        ],
        "name": "read-manifest-test",
        "replace": null,
        "source": null,
        "version": "0.1.0"
      }
    ],
    "root": null,
    "version": 4
  }
]
"#]]
            .is_json()
            .against_jsonlines(),
        )
        .run();
}

#[cargo_test]
fn package_with_git_deps() {
    let git_dep = git::new("b", |p| {
        p.file("src/lib.rs", "").file(
            "Cargo.toml",
            r#"
                [package]
                name = "b"
                version = "1.0.0"
                authors = []
                edition = "2024"
            "#,
        )
    });

    let p = project()
        .file("src/lib.rs", "")
        .file(
            "Cargo.toml",
            &format!(
                r#"
                    [package]
                    name = "read-manifest-test"
                    version = "0.1.0"
                    authors = []
                    edition = "2024"

                    [dependencies]
                    b = {{ git = '{}' }}
                "#,
                git_dep.url()
            ),
        )
        .build();

    let manifest_path = p.root().join("Cargo.toml");
    p.cargo_plumbing("plumbing lock-dependencies")
        .args(&["--manifest-path", manifest_path.to_str().unwrap()])
        .with_status(0)
        .with_stderr_data(str![[r#"
[UPDATING] git repository `[ROOTURL]/b`

"#]])
        .with_stdout_data(
            str![[r#"
[
  {
    "metadata": null,
    "package": [
      {
        "checksum": null,
        "dependencies": [],
        "name": "b",
        "replace": null,
        "source": "git+[ROOTURL]/b#[..]",
        "version": "1.0.0"
      },
      {
        "checksum": null,
        "dependencies": [
          "b"
        ],
        "name": "read-manifest-test",
        "replace": null,
        "source": null,
        "version": "0.1.0"
      }
    ],
    "root": null,
    "version": 4
  }
]
"#]]
            .is_json()
            .against_jsonlines(),
        )
        .run();
}

#[cargo_test]
fn package_with_varying_deps_sources() {
    Package::new("a", "1.0.0").publish();
    Package::new("a", "2.0.0").publish();

    let _registry = RegistryBuilder::new().http_index().alternative().build();
    Package::new("a", "1.0.0").alternative(true).publish();

    let ref_1 = "v1.0.0";
    let ref_2 = "v2.0.0";
    let url = {
        let (p, r) = git::new_repo("my-git-repo", |p| {
            p.file("Cargo.toml", &basic_manifest("a", "1.0.0"))
                .file("src/lib.rs", "")
        });

        git::tag(&r, ref_1);

        p.change_file("Cargo.toml", &basic_manifest("a", "2.0.0"));
        git::add(&r);
        git::commit(&r);
        git::tag(&r, ref_2);

        p.url()
    };

    let p = project()
        .file("a/src/lib.rs", "")
        .file(
            "a/Cargo.toml",
            r#"
                [package]
                name = "a"
                version = "1.0.0"
                authors = []
                edition = "2024"
            "#,
        )
        .file("src/lib.rs", "")
        .file(
            "Cargo.toml",
            &format!(
                r#"
                    [package]
                    name = "lock-dependencies-test"
                    version = "0.1.0"
                    authors = []
                    edition = "2024"

                    [dependencies]
                    a  = {{ path = "./a" }}
                    a1 = {{ package = "a", version = "1.0.0" }}
                    a2 = {{ package = "a", version = "2.0.0" }}
                    a3 = {{ package = "a", git = "{url}", rev = "{ref_1}" }}
                    a4 = {{ package = "a", git = "{url}", rev = "{ref_2}" }}
                    a5 = {{ package = "a", git = "{url}" }}
                    a6 = {{ package = "a", version = "1.0.0", registry = "alternative" }}
                "#,
            ),
        )
        .build();

    p.cargo_plumbing("plumbing lock-dependencies")
        .with_status(0)
        .with_stdout_data(
            str![[r#"
[
  {
    "version": 4,
    "package": [
      {
        "name": "a",
        "version": "1.0.0",
        "source": null,
        "checksum": null,
        "dependencies": [],
        "replace": null
      },
      {
        "name": "a",
        "version": "1.0.0",
        "source": "registry+https://github.com/rust-lang/crates.io-index",
        "checksum": "3a351dafbc8a3a9cba7c06dfe8caa11a3a45f800a336bb5b913a8f1e2652d454",
        "dependencies": [],
        "replace": null
      },
      {
        "name": "a",
        "version": "1.0.0",
        "source": "sparse+http://127.0.0.1:[..]/index/",
        "checksum": "3a351dafbc8a3a9cba7c06dfe8caa11a3a45f800a336bb5b913a8f1e2652d454",
        "dependencies": [],
        "replace": null
      },
      {
        "name": "a",
        "version": "1.0.0",
        "source": "git+[ROOTURL]/my-git-repo?rev=v1.0.0#[..]",
        "checksum": null,
        "dependencies": [],
        "replace": null
      },
      {
        "name": "a",
        "version": "2.0.0",
        "source": "registry+https://github.com/rust-lang/crates.io-index",
        "checksum": "50bc2065af6476063cea5b8d28dc20df4c7ad146759b4712b5e86a6d25d74ddc",
        "dependencies": [],
        "replace": null
      },
      {
        "name": "a",
        "version": "2.0.0",
        "source": "git+[ROOTURL]/my-git-repo?rev=v2.0.0#[..]",
        "checksum": null,
        "dependencies": [],
        "replace": null
      },
      {
        "name": "a",
        "version": "2.0.0",
        "source": "git+[ROOTURL]/my-git-repo#[..]",
        "checksum": null,
        "dependencies": [],
        "replace": null
      },
      {
        "name": "lock-dependencies-test",
        "version": "0.1.0",
        "source": null,
        "checksum": null,
        "dependencies": [
          "a 1.0.0",
          "a 1.0.0 (registry+https://github.com/rust-lang/crates.io-index)",
          "a 1.0.0 (sparse+http://127.0.0.1:[..]/index/)",
          "a 1.0.0 (git+[ROOTURL]/my-git-repo?rev=v1.0.0)",
          "a 2.0.0 (registry+https://github.com/rust-lang/crates.io-index)",
          "a 2.0.0 (git+[ROOTURL]/my-git-repo?rev=v2.0.0)",
          "a 2.0.0 (git+[ROOTURL]/my-git-repo)"
        ],
        "replace": null
      }
    ],
    "root": null,
    "metadata": null
  }
]
"#]]
            .is_json()
            .against_jsonlines(),
        )
        .run();
}

#[cargo_test]
fn package_with_unused_patches() {
    Package::new("a", "1.0.0").publish();

    let p = project()
        .file("a/src/lib.rs", "")
        .file(
            "a/Cargo.toml",
            r#"
                [package]
                name = "a"
                version = "2.0.0"
                authors = []
                edition = "2024"
            "#,
        )
        .file("src/lib.rs", "")
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "lock-dependencies-test"
                version = "0.1.0"
                authors = []
                edition = "2024"

                [dependencies]
                a = "1.0.0"

                [patch.crates-io]
                a = { path = "a" }
            "#,
        )
        .build();

    p.cargo_plumbing("plumbing lock-dependencies")
        .with_status(0)
        .with_stdout_data(
            str![[r#"
[
  {
    "version": 4,
    "package": [
      {
        "name": "a",
        "version": "1.0.0",
        "source": "registry+https://github.com/rust-lang/crates.io-index",
        "checksum": "3a351dafbc8a3a9cba7c06dfe8caa11a3a45f800a336bb5b913a8f1e2652d454",
        "dependencies": [],
        "replace": null
      },
      {
        "name": "lock-dependencies-test",
        "version": "0.1.0",
        "source": null,
        "checksum": null,
        "dependencies": [
          "a"
        ],
        "replace": null
      }
    ],
    "root": null,
    "metadata": null,
    "patch": {
      "unused": [
        {
          "name": "a",
          "version": "2.0.0",
          "source": null,
          "checksum": null,
          "dependencies": null,
          "replace": null
        }
      ]
    }
  }
]
"#]]
            .is_json()
            .against_jsonlines(),
        )
        .run();
}

#[cargo_test]
fn workspace_member_with_inherited_deps() {
    Package::new("a", "1.0.0").publish();
    Package::new("b", "1.0.0").publish();

    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                resolver = "3"
                members = ["crate1"]

                [workspace.dependencies]
                a = "1.0.0"
                b = "1.0.0"
            "#,
        )
        .file("crate1/src/lib.rs", "")
        .file(
            "crate1/Cargo.toml",
            r#"
                [package]
                name = "crate1"
                version = "0.1.0"
                authors = []
                edition = "2024"

                [dependencies]
                a.workspace = true
                b.workspace = true
            "#,
        )
        .build();

    p.cargo_plumbing("plumbing lock-dependencies")
        .with_status(0)
        .with_stdout_data(
            str![[r#"
[
  {
    "version": 4,
    "package": [
      {
        "name": "a",
        "version": "1.0.0",
        "source": "registry+https://github.com/rust-lang/crates.io-index",
        "checksum": "3a351dafbc8a3a9cba7c06dfe8caa11a3a45f800a336bb5b913a8f1e2652d454",
        "dependencies": [],
        "replace": null
      },
      {
        "name": "b",
        "version": "1.0.0",
        "source": "registry+https://github.com/rust-lang/crates.io-index",
        "checksum": "909035bb08757fa6f58bf655da5337acb736003f7301533602d348a329097837",
        "dependencies": [],
        "replace": null
      },
      {
        "name": "crate1",
        "version": "0.1.0",
        "source": null,
        "checksum": null,
        "dependencies": [
          "a",
          "b"
        ],
        "replace": null
      }
    ],
    "root": null,
    "metadata": null
  }
]
"#]]
            .is_json()
            .against_jsonlines(),
        )
        .run();
}

#[cargo_test]
fn workspace_package_depend_on_workspace_member() {
    Package::new("a", "1.0.0").publish();

    let p = project()
        .file("src/lib.rs", "")
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                resolver = "3"
                members = ["crate1"]

                [package]
                name = "lock-dependencies-test"
                version = "0.1.0"
                authors = []
                edition = "2024"

                [dependencies]
                crate1 = { path = "./crate1" }
                a = "1.0.0"
            "#,
        )
        .file("crate1/src/lib.rs", "")
        .file(
            "crate1/Cargo.toml",
            r#"
                [package]
                name = "crate1"
                version = "0.1.0"
                authors = []
                edition = "2024"
            "#,
        )
        .build();

    p.cargo_plumbing("plumbing lock-dependencies")
        .with_status(0)
        .with_stdout_data(
            str![[r#"
[
  {
    "version": 4,
    "package": [
      {
        "name": "a",
        "version": "1.0.0",
        "source": "registry+https://github.com/rust-lang/crates.io-index",
        "checksum": "3a351dafbc8a3a9cba7c06dfe8caa11a3a45f800a336bb5b913a8f1e2652d454",
        "dependencies": [],
        "replace": null
      },
      {
        "name": "crate1",
        "version": "0.1.0",
        "source": null,
        "checksum": null,
        "dependencies": [],
        "replace": null
      },
      {
        "name": "lock-dependencies-test",
        "version": "0.1.0",
        "source": null,
        "checksum": null,
        "dependencies": [
          "a",
          "crate1"
        ],
        "replace": null
      }
    ],
    "root": null,
    "metadata": null
  }
]
"#]]
            .is_json()
            .against_jsonlines(),
        )
        .run();
}
