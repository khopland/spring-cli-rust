use crate::{
    args::Args,
    request::{get_zip, SpringResponse},
};
use anyhow::{Ok, Result};
use inquire::{MultiSelect, Select, Text};
use std::{fs, io::Write, path::Path};

pub struct Data {
    dependencies: Vec<String>,
    java_version: String,
    language: String,
    build_type: String,
    group_id: String,
    artifact_id: String,
    name: String,
    url: String,
}

impl Data {
    pub fn new(args: Args) -> Result<Self> {
        let url = args.get_url();

        println!("getting parameter from {}", &url);

        let options = SpringResponse::get_options(&url)?;

        if args.needs_to_get_user_input() {
            Ok(Data {
                dependencies: Self::get_deps_from_user(&options)?,
                build_type: Self::get_build_type_from_user(&options)?,
                language: Self::get_language_from_user(&options)?,
                java_version: Self::get_java_version_from_user(&options)?,
                artifact_id: Self::get_artifact_id_from_user(&options)?,
                group_id: Self::get_group_id_from_user(&options)?,
                name: Self::get_name_from_user(&options)?,
                url,
            })
        } else {
            Ok(Data {
                dependencies: args.dependencies.unwrap_or_default(),
                java_version: args.java_version.unwrap_or(options.java_version.default),
                language: args.language.unwrap_or(options.language.default),
                build_type: args.build_type.unwrap_or(options.build_type.default),
                group_id: args.group_id.unwrap_or(options.group_id.default),
                artifact_id: args.artifact_id.unwrap_or(options.artifact_id.default),
                name: args.name.unwrap_or(options.name.default),
                url,
            })
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn write_file_to_path<T: AsRef<Path>>(&self, path: T) -> Result<()> {
        let buf = self.get_zip_file(&self.url)?;
        let mut file = fs::File::create(path)?;
        file.write_all(&buf)?;
        Ok(())
    }

    fn get_zip_file(&self, url: &str) -> Result<Vec<u8>> {
        get_zip(
            url,
            &self.dependencies.join(","),
            &self.build_type,
            &self.java_version,
            &self.artifact_id,
            &self.group_id,
            &self.language,
            &self.name,
        )
    }

    fn get_deps_from_user(options: &SpringResponse) -> Result<Vec<String>> {
        Ok(MultiSelect::new(
            "Select the dependencies your want:",
            options
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
        .prompt()?
        .iter()
        .map(|x| x.id.to_owned())
        .collect::<Vec<String>>())
    }

    fn get_build_type_from_user(options: &SpringResponse) -> Result<String> {
        Ok(if options.build_type.values.len() > 1 {
            Select::new(
                "Select build type:",
                options
                    .build_type
                    .values
                    .iter()
                    .filter(|b| b.action == "/starter.zip")
                    .collect(),
            )
            .prompt()?
            .id
            .to_owned()
        } else {
            println!("> Selected build type:{}", options.build_type.default);
            options
                .build_type
                .values
                .iter()
                .find(|b| b.id == options.build_type.default)
                .expect("no build type found")
                .id
                .to_owned()
        })
    }

    fn get_java_version_from_user(options: &SpringResponse) -> Result<String> {
        Ok(if options.java_version.values.len() > 1 {
            Select::new(
                "Select java version:",
                options
                    .java_version
                    .values
                    .iter()
                    .map(|j| j.id.to_owned())
                    .collect(),
            )
            .with_starting_cursor(
                options
                    .java_version
                    .values
                    .iter()
                    .position(|x| x.id == options.java_version.default)
                    .unwrap_or(0),
            )
            .prompt()?
        } else {
            println!("> Selected java version: {}", options.java_version.default);
            options.java_version.default.to_owned()
        })
    }

    fn get_language_from_user(options: &SpringResponse) -> Result<String> {
        Ok(if options.language.values.len() > 1 {
            Select::new(
                "Select Language:",
                options
                    .language
                    .values
                    .iter()
                    .map(|l| l.id.to_owned())
                    .collect(),
            )
            .prompt()?
        } else {
            println!("> Selected Language: {}", options.language.default);
            options.language.default.to_owned()
        })
    }

    fn get_artifact_id_from_user(options: &SpringResponse) -> Result<String> {
        Ok(Text::new("artifactId")
            .with_default(&options.artifact_id.default)
            .prompt()?)
    }

    fn get_group_id_from_user(options: &SpringResponse) -> Result<String> {
        Ok(Text::new("groupId")
            .with_default(&options.group_id.default)
            .prompt()?)
    }

    fn get_name_from_user(options: &SpringResponse) -> Result<String> {
        Ok(Text::new("name")
            .with_default(&options.name.default)
            .prompt()?)
    }
}
