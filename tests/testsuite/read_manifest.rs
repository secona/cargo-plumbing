use cargo_test_macro::cargo_test;
use cargo_test_support::registry::Package;
use cargo_test_support::{main_file, project, str};

use crate::ProjectExt;

#[cargo_test]
fn simple_with_deps() {
    Package::new("a", "1.0.0")
        .file("src/lib.rs", r#"pub fn f() -> i32 { 12 }"#)
        .publish();

    let p = project()
        .file("src/main.rs", &main_file("Hello", &[]))
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

    p.cargo_plumbing("plumbing read-manifest")
        .arg("--manifest-path")
        .arg(p.root().join("Cargo.toml"))
        .with_stderr_data(
            str![[r#"
[ERROR] unrecognized subcommand 'read-manifest'

  tip: a similar subcommand exists: 'locate-manifest'

Usage: cargo plumbing <COMMAND>

For more information, try '--help'.

"#]]
        )
        .with_status(2)
        .run();

    p.cargo_plumbing("plumbing read-manifest")
        .arg("--manifest-path")
        .arg(p.root().join("Cargo.toml"))
        .arg("--workspace")
        .with_stderr_data(
            str![[r#"
[ERROR] unrecognized subcommand 'read-manifest'

  tip: a similar subcommand exists: 'locate-manifest'

Usage: cargo plumbing <COMMAND>

For more information, try '--help'.

"#]]
        )
        .with_status(2)
        .run();
}

#[cargo_test]
fn workspace_real_manifest_with_deps() {
    Package::new("a", "1.0.0")
        .file("src/lib.rs", r#"pub fn f() -> i32 { 12 }"#)
        .publish();

    let p = project()
        .file("src/main.rs", &main_file("Hello", &[]))
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                resolver = "3"

                [workspace.dependencies]
                a = "1.0.0"

                [package]
                name = "read-manifest-test"
                version = "0.1.0"
                authors = []
                edition = "2024"

                [dependencies]
                a.workspace = true
            "#,
        )
        .build();

    p.cargo_plumbing("plumbing read-manifest")
        .arg("--manifest-path")
        .arg(p.root().join("Cargo.toml"))
        .with_stderr_data(
            str![[r#"
[ERROR] unrecognized subcommand 'read-manifest'

  tip: a similar subcommand exists: 'locate-manifest'

Usage: cargo plumbing <COMMAND>

For more information, try '--help'.

"#]]
        )
        .with_status(2)
        .run();

    p.cargo_plumbing("plumbing read-manifest")
        .arg("--manifest-path")
        .arg(p.root().join("Cargo.toml"))
        .arg("--workspace")
        .with_stderr_data(
            str![[r#"
[ERROR] unrecognized subcommand 'read-manifest'

  tip: a similar subcommand exists: 'locate-manifest'

Usage: cargo plumbing <COMMAND>

For more information, try '--help'.

"#]]
        )
        .with_status(2)
        .run();
}

#[cargo_test]
fn workspace_real_manifest_with_multiple_members() {
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

    p.cargo_plumbing("plumbing read-manifest")
        .arg("--manifest-path")
        .arg(p.root().join("Cargo.toml"))
        .with_stderr_data(
            str![[r#"
[ERROR] unrecognized subcommand 'read-manifest'

  tip: a similar subcommand exists: 'locate-manifest'

Usage: cargo plumbing <COMMAND>

For more information, try '--help'.

"#]]
        )
        .with_status(2)
        .run();

    p.cargo_plumbing("plumbing read-manifest")
        .arg("--manifest-path")
        .arg(p.root().join("Cargo.toml"))
        .arg("--workspace")
        .with_stderr_data(
            str![[r#"
[ERROR] unrecognized subcommand 'read-manifest'

  tip: a similar subcommand exists: 'locate-manifest'

Usage: cargo plumbing <COMMAND>

For more information, try '--help'.

"#]]
        )
        .with_status(2)
        .run();
}

#[cargo_test]
fn workspace_virtual_manifest_with_multiple_members() {
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
            "#,
        )
        .build();

    p.cargo_plumbing("plumbing read-manifest")
        .arg("--manifest-path")
        .arg(p.root().join("Cargo.toml"))
        .with_stderr_data(
            str![[r#"
[ERROR] unrecognized subcommand 'read-manifest'

  tip: a similar subcommand exists: 'locate-manifest'

Usage: cargo plumbing <COMMAND>

For more information, try '--help'.

"#]]
        )
        .with_status(2)
        .run();

    p.cargo_plumbing("plumbing read-manifest")
        .arg("--manifest-path")
        .arg(p.root().join("Cargo.toml"))
        .arg("--workspace")
        .with_stderr_data(
            str![[r#"
[ERROR] unrecognized subcommand 'read-manifest'

  tip: a similar subcommand exists: 'locate-manifest'

Usage: cargo plumbing <COMMAND>

For more information, try '--help'.

"#]]
        )
        .with_status(2)
        .run();
}

#[cargo_test]
fn workspace_member_with_inherited_deps() {
    Package::new("a", "1.0.0")
        .file("src/lib.rs", r#"pub fn f() -> i32 { 12 }"#)
        .publish();

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

                [dependencies]
                a.workspace = true
            "#,
        )
        .file("src/main.rs", &main_file("Hello", &[]))
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                resolver = "3"
                members = ["crate1", "crate2"]

                [workspace.dependencies]
                a = "1.0.0"

                [package]
                name = "read-manifest-test"
                version = "0.1.0"
                authors = []
                edition = "2024"
            "#,
        )
        .build();

    p.cargo_plumbing("plumbing read-manifest")
        .arg("--manifest-path")
        .arg(p.root().join("crate1/Cargo.toml"))
        .with_stderr_data(
            str![[r#"
[ERROR] unrecognized subcommand 'read-manifest'

  tip: a similar subcommand exists: 'locate-manifest'

Usage: cargo plumbing <COMMAND>

For more information, try '--help'.

"#]]
        )
        .with_status(2)
        .run();

    p.cargo_plumbing("plumbing read-manifest")
        .arg("--manifest-path")
        .arg(p.root().join("crate1/Cargo.toml"))
        .arg("--workspace")
        .with_stderr_data(
            str![[r#"
[ERROR] unrecognized subcommand 'read-manifest'

  tip: a similar subcommand exists: 'locate-manifest'

Usage: cargo plumbing <COMMAND>

For more information, try '--help'.

"#]]
        )
        .with_status(2)
        .run();
}

#[cargo_test]
fn workspace_member_via_workspace_field() {
    Package::new("a", "1.0.0")
        .file("src/lib.rs", r#"pub fn f() -> i32 { 12 }"#)
        .publish();

    let p = project()
        .file(
            "workspace-root/Cargo.toml",
            r#"
                [workspace]
                resolver = "3"
                members = ["../crate1", "../crate2"]

                [workspace.dependencies]
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
                workspace = "../workspace-root"

                [dependencies]
                a.workspace = true
            "#,
        )
        .file("crate2/src/lib.rs", "")
        .file(
            "crate2/Cargo.toml",
            r#"
                [package]
                name = "crate2"
                version = "0.1.0"
                authors = []
                edition = "2024"
                workspace = "../workspace-root"

                [dependencies]
                a.workspace = true
            "#,
        )
        .build();

    p.cargo_plumbing("plumbing read-manifest")
        .arg("--manifest-path")
        .arg(p.root().join("crate1/Cargo.toml"))
        .with_stderr_data(
            str![[r#"
[ERROR] unrecognized subcommand 'read-manifest'

  tip: a similar subcommand exists: 'locate-manifest'

Usage: cargo plumbing <COMMAND>

For more information, try '--help'.

"#]]
        )
        .with_status(2)
        .run();

    p.cargo_plumbing("plumbing read-manifest")
        .arg("--manifest-path")
        .arg(p.root().join("crate1/Cargo.toml"))
        .arg("--workspace")
        .with_stderr_data(
            str![[r#"
[ERROR] unrecognized subcommand 'read-manifest'

  tip: a similar subcommand exists: 'locate-manifest'

Usage: cargo plumbing <COMMAND>

For more information, try '--help'.

"#]]
        )
        .with_status(2)
        .run();
}

