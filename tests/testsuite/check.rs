use cargo_test_support::prelude::*;
use cargo_test_support::project;
use cargo_test_support::str;

use crate::ProjectExt;

#[cargo_test]
fn package() {
    let p = project()
        .file("src/lib.rs", "")
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.1.0"
                authors = []
                edition = "2024"
            "#,
        )
        .build();

    p.cargo_global("generate-lockfile").run();

    let manifest_path = p.root().join("Cargo.toml");
    let lockfile_path = p.root().join("Cargo.lock");

    p.cargo_plumbing_example("check")
        .arg("--manifest-path")
        .arg(manifest_path)
        .arg("--lockfile-path")
        .arg(lockfile_path)
        .with_status(0)
        .with_stdout_data(str![[r#"
[ERROR] check for [ROOT]/foo/Cargo.toml is not implemented!

"#]])
        .with_stderr_data(str![])
        .run();
}
