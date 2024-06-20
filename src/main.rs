use anyhow::Result;
use inquire::{MultiSelect, Select, Text};
use request::{get_deps, get_zip};
use std::fs;
mod request;
fn main() -> Result<()> {
    println!("getting parameter from spring.io");
    let response = get_deps()?;

    let deps = MultiSelect::new(
        "Select the dependencies your want:",
        response
            .dependencies
            .values
            .iter()
            .flat_map(|f| f.values.iter())
            .collect(),
    )
    .with_formatter(&|a| {
        a.iter()
            .map(|x| x.value.id.clone())
            .collect::<Vec<String>>()
            .join(", ")
    })
    .prompt()?;

    let build_type = Select::new(
        "Select build type",
        response
            .build_type
            .values
            .iter()
            .filter(|b| b.action == "/starter.zip")
            .collect(),
    )
    .prompt()?;

    let jvm = Select::new(
        "Select java version",
        response
            .java_version
            .values
            .iter()
            .map(|j| j.id.clone())
            .collect(),
    )
    .with_formatter(&|jvm| format!("Selected java version{}", jvm.value))
    .with_starting_cursor(
        response
            .java_version
            .values
            .into_iter()
            .position(|x| x.id == response.java_version.default)
            .unwrap_or(0),
    )
    .prompt()?;

    let artifact_id = Text::new("artifactId")
        .with_default(response.artifact_id.default.as_str())
        .prompt()?;

    let group_id = Text::new("groupId")
        .with_default(response.group_id.default.as_str())
        .prompt()?;

    let buf = get_zip(deps,build_type, jvm, artifact_id, group_id)?;

    fs::write("./test.zip", buf)?;
    Ok(())
}
