use cargo_test_support::prelude::*;
use cargo_test_support::*;

use crate::ProjectExt;

#[cargo_test]
fn workspace() {
    let p = project()
        .file("crate2/src/lib.rs", "")
        .file(
            "crate2/Cargo.toml",
            r#"
                [package]
                name = "crate2"
                version = "0.1.0"
                authors = []
                edition = "2024"
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
        .file("src/main.rs", &main_file("Hello", &[]))
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                resolver = "3"
                members = ["crate1", "crate2"]

                [package]
                name = "read-manifest-test"
                version = "0.1.0"
                authors = []
                edition = "2024"
            "#,
        )
        .build();

    p.cargo_global("generate-lockfile").run();

    p.cargo_plumbing_example("metadata")
        .args(&["--format-version", "1", "--no-deps"])
        .with_status(0)
        .with_stdout_data(
            str![[r#"
[
  {
    "build_directory": "",
    "metadata": null,
    "packages": [
      {
        "authors": [],
        "categories": [],
        "default_run": null,
        "dependencies": [],
        "description": null,
        "documentation": null,
        "edition": "2024",
        "features": {},
        "homepage": null,
        "id": "path+[ROOTURL]/foo#read-manifest-test@0.1.0",
        "keywords": [],
        "license": null,
        "license_file": null,
        "links": null,
        "manifest_path": "[ROOT]/foo/Cargo.toml",
        "metadata": null,
        "name": "read-manifest-test",
        "publish": null,
        "readme": null,
        "repository": null,
        "rust_version": null,
        "source": null,
        "targets": [],
        "version": "0.1.0"
      },
      {
        "authors": [],
        "categories": [],
        "default_run": null,
        "dependencies": [],
        "description": null,
        "documentation": null,
        "edition": "2024",
        "features": {},
        "homepage": null,
        "id": "path+[ROOTURL]/foo/crate1#0.1.0",
        "keywords": [],
        "license": null,
        "license_file": null,
        "links": null,
        "manifest_path": "[ROOT]/foo/crate1/Cargo.toml",
        "metadata": null,
        "name": "crate1",
        "publish": null,
        "readme": null,
        "repository": null,
        "rust_version": null,
        "source": null,
        "targets": [],
        "version": "0.1.0"
      },
      {
        "authors": [],
        "categories": [],
        "default_run": null,
        "dependencies": [],
        "description": null,
        "documentation": null,
        "edition": "2024",
        "features": {},
        "homepage": null,
        "id": "path+[ROOTURL]/foo/crate2#0.1.0",
        "keywords": [],
        "license": null,
        "license_file": null,
        "links": null,
        "manifest_path": "[ROOT]/foo/crate2/Cargo.toml",
        "metadata": null,
        "name": "crate2",
        "publish": null,
        "readme": null,
        "repository": null,
        "rust_version": null,
        "source": null,
        "targets": [],
        "version": "0.1.0"
      }
    ],
    "resolve": null,
    "target_directory": "",
    "version": 1,
    "workspace_default_members": [
      "path+[ROOTURL]/foo#read-manifest-test@0.1.0"
    ],
    "workspace_members": [
      "path+[ROOTURL]/foo#read-manifest-test@0.1.0",
      "path+[ROOTURL]/foo/crate1#0.1.0",
      "path+[ROOTURL]/foo/crate2#0.1.0"
    ],
    "workspace_root": "[ROOT]/foo/Cargo.toml"
  }
]
"#]]
            .is_json()
            .against_jsonlines(),
        )
        .with_stderr_data(str![])
        .run();
}
