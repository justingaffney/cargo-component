use std::fs;

use anyhow::{Context, Result};
use assert_cmd::prelude::*;
use predicates::str::contains;

use crate::support::*;

mod support;

#[test]
fn help() {
    for arg in ["help build", "build -h", "build --help"] {
        wit(arg.split_whitespace())
            .assert()
            .stdout(contains("Build a binary WIT package"))
            .success();
    }
}

#[test]
fn it_fails_with_missing_toml_file() -> Result<()> {
    wit(["build"])
        .assert()
        .stderr(contains(
            "error: failed to find configuration file `wit.toml`",
        ))
        .failure();
    Ok(())
}

#[test]
fn it_builds() -> Result<()> {
    let project = Project::new("foo")?;
    project.file(
        "bar.wit",
        r#"package foo:bar;
interface bar {}
world bar-world {}
"#,
    )?;
    project.file(
        "baz.wit",
        r#"package foo:bar;
interface baz {}
world baz-world {}
"#,
    )?;

    project
        .wit(["build"])
        .assert()
        .stderr(contains("Created package `bar.wasm`"))
        .success();

    validate_component(&project.root().join("bar.wasm"))?;

    let path = project.root().join("wit.lock");
    let contents = fs::read_to_string(&path)
        .with_context(|| format!("failed to read lock file `{path}`", path = path.display()))?;

    let contents = contents.replace("\r\n", "\n");

    assert_eq!(
        contents,
        "# This file is automatically generated by wit.\n# It is not intended for manual editing.\nversion = 1\n",
        "unexpected lock file contents"
    );

    Ok(())
}

#[test]
fn it_adds_a_producers_field() -> Result<()> {
    let project = Project::new("foo")?;
    project.file("producers.wit", "package test:producers;")?;

    project
        .wit(["build"])
        .assert()
        .stderr(contains("Created package `producers.wasm`"))
        .success();

    let path = project.root().join("producers.wasm");
    validate_component(&path)?;

    let wasm = fs::read(&path)
        .with_context(|| format!("failed to read wasm file `{path}`", path = path.display()))?;
    let section = wasm_metadata::Producers::from_wasm(&wasm)?.expect("missing producers section");

    assert_eq!(
        section
            .get("processed-by")
            .expect("missing processed-by field")
            .get(env!("CARGO_PKG_NAME"))
            .expect("missing wit field"),
        option_env!("WIT_VERSION_INFO").unwrap_or(env!("CARGO_PKG_VERSION"))
    );

    Ok(())
}
