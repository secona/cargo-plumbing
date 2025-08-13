use cargo_test_support::prelude::*;
use cargo_test_support::project;
use cargo_test_support::str;

use crate::ProjectExt;

#[cargo_test]
fn finds_package_manifest() {
    let p = project().build();

    p.cargo_plumbing("plumbing locate-manifest")
        .with_stdout_data(
            str![[r#"
[
  {
    "manifest_path": "[ROOT]/foo/Cargo.toml",
    "reason": "manifest-location"
  }
]
"#]]
            .is_json()
            .against_jsonlines(),
        )
        .with_stderr_data("")
        .with_status(0)
        .run();
}

#[cargo_test]
fn finds_package_manifest_via_manifest_path_flag() {
    let p = project().file("src/lib.rs", "").build();

    let wd = p.root().join("src");
    let wd = wd.to_str().unwrap();

    p.cargo_plumbing("plumbing locate-manifest")
        .args(&["--manifest-path", wd])
        .with_stdout_data(
            str![[r#"
[
  {
    "manifest_path": "[ROOT]/foo/Cargo.toml",
    "reason": "manifest-location"
  }
]
"#]]
            .is_json()
            .against_jsonlines(),
        )
        .with_stderr_data("")
        .with_status(0)
        .run();
}

#[cargo_test]
fn finds_virtual_manifest() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                resolver = "3"
                members = ["crate1", "crate2"]
            "#,
        )
        .file("crate1/src/lib.rs", "")
        .file(
            "crate1/Cargo.toml",
            r#"
                [package]
                name = "crate1"
                version = "0.1.0"
                edition = "2024"
            "#,
        )
        .file("crate2/src/lib.rs", "")
        .file(
            "crate2/Cargo.toml",
            r#"
                [package]
                name = "crate2"
                version = "0.1.0"
                edition = "2024"
            "#,
        )
        .build();

    p.cargo_plumbing("plumbing locate-manifest")
        .with_stdout_data(
            str![[r#"
[
  {
    "manifest_path": "[ROOT]/foo/Cargo.toml",
    "reason": "manifest-location"
  }
]
"#]]
            .is_json()
            .against_jsonlines(),
        )
        .with_stderr_data("")
        .with_status(0)
        .run();
}

#[cargo_test]
fn finds_member_manifest() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                resolver = "3"
                members = ["crate1", "crate2"]
            "#,
        )
        .file("crate1/src/lib.rs", "")
        .file(
            "crate1/Cargo.toml",
            r#"
                [package]
                name = "crate1"
                version = "0.1.0"
                edition = "2024"
            "#,
        )
        .file("crate2/src/lib.rs", "")
        .file(
            "crate2/Cargo.toml",
            r#"
                [package]
                name = "crate2"
                version = "0.1.0"
                edition = "2024"
            "#,
        )
        .build();

    let manifest_path = p.root().join("crate1/Cargo.toml");

    p.cargo_plumbing("plumbing locate-manifest")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .with_stdout_data(
            str![[r#"
[
  {
    "manifest_path": "[ROOT]/foo/crate1/Cargo.toml",
    "reason": "manifest-location"
  }
]
"#]]
            .is_json()
            .against_jsonlines(),
        )
        .with_stderr_data("")
        .with_status(0)
        .run();
}

#[cargo_test]
fn errors_when_no_manifest() {
    let p = project().no_manifest().build();

    p.cargo_plumbing("plumbing locate-manifest")
        .with_stderr_data(str![[r#"
[ERROR] could not find `Cargo.toml` in `[ROOT]/foo` or any parent directory

"#]])
        .with_stdout_data("")
        .with_status(101)
        .run();
}
