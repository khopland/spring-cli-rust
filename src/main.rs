use anyhow::Result;
use args::Args;
use clap::Parser;
use inquire::{MultiSelect, Select, Text};
use request::{get_deps, get_zip};
use resolve_path::PathResolveExt;
use std::{fs, io::Write};
mod args;
mod request;
fn main() -> Result<()> {
    let args = Args::parse();
    let url = args.url.unwrap_or("https://start.spring.io".to_owned());
    let url = url.trim();

    println!("getting parameter from {}", url);
    let response = get_deps(url)?;

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
            .map(|x| x.value.id.to_owned())
            .collect::<Vec<String>>()
            .join(", ")
    })
    .prompt()?;

    let build_type = if response.build_type.values.len() > 1 {
        Select::new(
            "Select build type:",
            response
                .build_type
                .values
                .iter()
                .filter(|b| b.action == "/starter.zip")
                .collect(),
        )
        .prompt()?
    } else {
        println!("> Select build type:{}", response.build_type.default);
        response
            .build_type
            .values
            .iter()
            .find(|b| b.id == response.build_type.default)
            .expect("shoud never happen")
    };

    let language = if response.language.values.len() > 1 {
        Select::new(
            "Select Language:",
            response
                .language
                .values
                .iter()
                .map(|l| l.id.to_owned())
                .collect(),
        )
        .prompt()?
    } else {
        println!("> Selected Language: {}", response.language.default);
        response.language.default
    };

    let jvm = if response.java_version.values.len() > 1 {
        Select::new(
            "Select java version:",
            response
                .java_version
                .values
                .iter()
                .map(|j| j.id.to_owned())
                .collect(),
        )
        .with_starting_cursor(
            response
                .java_version
                .values
                .into_iter()
                .position(|x| x.id == response.java_version.default)
                .unwrap_or(0),
        )
        .prompt()?
    } else {
        println!("> Selected java version: {}", response.java_version.default);
        response.java_version.default
    };

    let artifact_id = Text::new("artifactId")
        .with_default(response.artifact_id.default.as_str())
        .prompt()?;

    let group_id = Text::new("groupId")
        .with_default(response.group_id.default.as_str())
        .prompt()?;

    let name = Text::new("name")
        .with_default(response.name.default.as_str())
        .prompt()?;

    let buf = get_zip(
        url,
        &deps
            .iter()
            .map(|d| d.id.clone())
            .collect::<Vec<String>>()
            .join(","),
        &build_type.id,
        &jvm,
        &artifact_id,
        &group_id,
        &language,
        &name,
    )?;

    let file_name = Text::new("where do you want to store the zip file?")
        .with_default(format!("./{}.zip", name).as_str())
        .prompt()?;

    let file_name = file_name.try_resolve()?;
    let mut file = fs::File::create(file_name)?;
    file.write_all(&buf)?;

    Ok(())
}
