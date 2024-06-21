use anyhow::Result;
use args::Args;
use clap::Parser;
use inquire::{MultiSelect, Select, Text};
use request::{get_deps, get_zip, SpringResponse};
use resolve_path::PathResolveExt;
use std::{fs, io::Write};

mod args;
mod request;

fn main() -> Result<()> {
    let args = Args::parse();
    let url = args
        .url
        .to_owned()
        .unwrap_or("https://start.spring.io".to_owned());
    let url = url.trim();

    println!("getting parameter from {}", url);
    let response = get_deps(url)?;

    let (name, buf) = if needs_to_get_user_input(&args) {
        get_zip_based_on_user_input(url, response)?
    } else {
        get_zip_based_on_args(url, &args, response)?
    };

    let file_name = Text::new("where do you want to store the zip file?")
        .with_default(format!("./{}.zip", name).as_str())
        .prompt()?;

    let file_name = file_name.try_resolve()?;
    let mut file = fs::File::create(file_name)?;
    file.write_all(&buf)?;

    Ok(())
}

fn needs_to_get_user_input(args: &Args) -> bool {
    args.dependencies.is_none()
        && args.type_build.is_none()
        && args.java_version.is_none()
        && args.language.is_none()
        && args.artifact_id.is_none()
        && args.group_id.is_none()
        && args.name.is_none()
        && args.version_number.is_none()
}

fn get_zip_based_on_args(
    url: &str,
    args: &Args,
    response: SpringResponse,
) -> Result<(String, Vec<u8>)> {
    let name = args.name.to_owned().unwrap_or(response.name.default);
    let buf = get_zip(
        url,
        &args.dependencies.to_owned().unwrap_or_default().join(","),
        &args
            .type_build
            .to_owned()
            .unwrap_or(response.build_type.default),
        &args
            .java_version
            .to_owned()
            .unwrap_or(response.java_version.default),
        &args
            .artifact_id
            .to_owned()
            .unwrap_or(response.artifact_id.default),
        &args
            .group_id
            .to_owned()
            .unwrap_or(response.group_id.default),
        &args
            .language
            .to_owned()
            .unwrap_or(response.language.default),
        &name,
    )?;
    Ok((name, buf))
}

fn get_zip_based_on_user_input(url: &str, response: SpringResponse) -> Result<(String, Vec<u8>)> {
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
    Ok((name, buf))
}
