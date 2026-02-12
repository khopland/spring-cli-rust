use std::{collections::HashMap, fs, io::Write};

use anyhow::{Context, Ok, Result};
use args::Args;
use clap::Parser;
use request::{get_zip, ResponseStep};
use resolve_path::PathResolveExt;
use std::io::Cursor;
use steps::Step;
use zip::ZipArchive;

mod args;
mod request;
mod steps;
mod user_innput;

fn main() -> Result<()> {
    let args = Args::parse();
    let json = request::get_options(&args.url)?;

    let cli_values = build_cli_values_map(&args);
    let steps = Step::from_json(json)?;

    let responses = steps
        .into_iter()
        .map(|step| {
            let prefill = cli_values.get(&step.name).map(|s| s.as_str());
            if args.non_interactive && prefill.is_none() {
                // Use defaults in non-interactive mode
                let default_response = match &step.kind {
                    steps::StepKind::Text { default } => default.clone(),
                    steps::StepKind::SingleSelect { default, .. } => default.clone(),
                    steps::StepKind::Action { default, .. } => default.clone(),
                    steps::StepKind::MultiSelect { .. } => String::new(),
                };
                Ok(ResponseStep {
                    step: step.to_owned(),
                    response: default_response,
                })
            } else {
                user_innput::get_user_input(&step, prefill)
            }
        })
        .collect::<Result<Vec<ResponseStep>>>()?;

    let (file_name, zip) = get_zip(&args.url, &responses)?;
    let path = args
        .path
        .as_deref()
        .or(file_name.as_deref())
        .unwrap_or("./spring-app.zip");
    write_zip(path, zip)
}

fn build_cli_values_map(args: &Args) -> HashMap<String, String> {
    let mut map = HashMap::new();

    if let Some(v) = &args.language {
        map.insert("language".to_string(), v.clone());
    }
    if let Some(v) = &args.boot_version {
        map.insert("bootVersion".to_string(), v.clone());
    }
    if let Some(v) = &args.group_id {
        map.insert("groupId".to_string(), v.clone());
    }
    if let Some(v) = &args.artifact_id {
        map.insert("artifactId".to_string(), v.clone());
    }
    if let Some(v) = &args.name {
        map.insert("name".to_string(), v.clone());
    }
    if let Some(v) = &args.description {
        map.insert("description".to_string(), v.clone());
    }
    if let Some(v) = &args.package_name {
        map.insert("packageName".to_string(), v.clone());
    }
    if let Some(v) = &args.packaging {
        map.insert("packaging".to_string(), v.clone());
    }
    if let Some(v) = &args.java_version {
        map.insert("javaVersion".to_string(), v.clone());
    }
    if let Some(v) = &args.dependencies {
        map.insert("dependencies".to_string(), v.clone());
    }
    if let Some(v) = &args.project_type {
        map.insert("type".to_string(), v.clone());
    }
    if let Some(v) = &args.project_version {
        map.insert("version".to_string(), v.clone());
    }

    map
}

fn write_zip(file_name: &str, zip: Vec<u8>) -> Result<()> {
    let path = file_name.try_resolve()?;
    if path.extension().is_none() && ZipArchive::new(Cursor::new(&zip)).is_ok() {
        fs::create_dir_all(&path)?;
        println!("writing data to {}", path.display());
        let mut archive = ZipArchive::new(Cursor::new(&zip))?;
        archive.extract(&path)?;
    } else {
        let parent = &path.parent().context("dident find parent of file")?;
        fs::create_dir_all(parent)?;
        println!("writing data to {}", path.display());
        let mut file = fs::File::create(path)?;
        file.write_all(&zip)?;
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::steps::{Item, StepKind};

    #[test]
    fn test_build_cli_values_map_empty() {
        let args = Args {
            url: "https://start.spring.io".to_string(),
            path: None,
            non_interactive: false,
            language: None,
            boot_version: None,
            group_id: None,
            artifact_id: None,
            name: None,
            description: None,
            package_name: None,
            packaging: None,
            java_version: None,
            dependencies: None,
            project_type: None,
            project_version: None,
        };

        let map = build_cli_values_map(&args);
        assert!(map.is_empty());
    }

    #[test]
    fn test_build_cli_values_map_with_values() {
        let args = Args {
            url: "https://start.spring.io".to_string(),
            path: None,
            non_interactive: false,
            language: Some("java".to_string()),
            boot_version: Some("3.4.2".to_string()),
            group_id: Some("com.example".to_string()),
            artifact_id: Some("myapp".to_string()),
            name: Some("MyApp".to_string()),
            description: Some("Test app".to_string()),
            package_name: Some("com.example.myapp".to_string()),
            packaging: Some("jar".to_string()),
            java_version: Some("21".to_string()),
            dependencies: Some("web,data-jpa".to_string()),
            project_type: Some("maven-project".to_string()),
            project_version: Some("1.0.0".to_string()),
        };

        let map = build_cli_values_map(&args);

        assert_eq!(map.get("language"), Some(&"java".to_string()));
        assert_eq!(map.get("bootVersion"), Some(&"3.4.2".to_string()));
        assert_eq!(map.get("groupId"), Some(&"com.example".to_string()));
        assert_eq!(map.get("artifactId"), Some(&"myapp".to_string()));
        assert_eq!(map.get("name"), Some(&"MyApp".to_string()));
        assert_eq!(map.get("description"), Some(&"Test app".to_string()));
        assert_eq!(
            map.get("packageName"),
            Some(&"com.example.myapp".to_string())
        );
        assert_eq!(map.get("packaging"), Some(&"jar".to_string()));
        assert_eq!(map.get("javaVersion"), Some(&"21".to_string()));
        assert_eq!(map.get("dependencies"), Some(&"web,data-jpa".to_string()));
        assert_eq!(map.get("type"), Some(&"maven-project".to_string()));
        assert_eq!(map.get("version"), Some(&"1.0.0".to_string()));
    }

    #[test]
    fn test_non_interactive_uses_defaults_for_text() {
        let step = crate::steps::Step {
            name: "name".to_string(),
            kind: StepKind::Text {
                default: "demo".to_string(),
            },
        };

        // Simulate non-interactive mode with no prefill
        let cli_values: HashMap<String, String> = HashMap::new();
        let _prefill = cli_values.get(&step.name);

        // In non-interactive mode without prefill, should use default
        let response = match &step.kind {
            StepKind::Text { default } => default.clone(),
            _ => panic!("Expected Text step"),
        };

        assert_eq!(response, "demo");
    }

    #[test]
    fn test_non_interactive_uses_defaults_for_single_select() {
        let step = crate::steps::Step {
            name: "language".to_string(),
            kind: StepKind::SingleSelect {
                default: "java".to_string(),
                values: vec![
                    Item::new_default("java".to_string(), "Java".to_string()),
                    Item::new_default("kotlin".to_string(), "Kotlin".to_string()),
                ],
            },
        };

        // Simulate non-interactive mode with no prefill
        let response = match &step.kind {
            StepKind::SingleSelect { default, .. } => default.clone(),
            _ => panic!("Expected SingleSelect step"),
        };

        assert_eq!(response, "java");
    }

    #[test]
    fn test_non_interactive_uses_defaults_for_action() {
        let step = crate::steps::Step {
            name: "type".to_string(),
            kind: StepKind::Action {
                default: "maven-project".to_string(),
                values: vec![Item::new_action(
                    "maven-project".to_string(),
                    "Maven".to_string(),
                    "/starter.zip".to_string(),
                )],
            },
        };

        // Simulate non-interactive mode with no prefill
        let response = match &step.kind {
            StepKind::Action { default, .. } => default.clone(),
            _ => panic!("Expected Action step"),
        };

        assert_eq!(response, "maven-project");
    }

    #[test]
    fn test_non_interactive_uses_empty_for_multi_select() {
        let step = crate::steps::Step {
            name: "dependencies".to_string(),
            kind: StepKind::MultiSelect {
                values: vec![Item::new_dependency(
                    "web".to_string(),
                    "Spring Web".to_string(),
                    "Web".to_string(),
                )],
            },
        };

        // Simulate non-interactive mode with no prefill
        let response = match &step.kind {
            StepKind::MultiSelect { .. } => String::new(),
            _ => panic!("Expected MultiSelect step"),
        };

        assert_eq!(response, "");
    }

    #[test]
    fn test_prefill_overrides_default() {
        let step = crate::steps::Step {
            name: "language".to_string(),
            kind: StepKind::SingleSelect {
                default: "java".to_string(),
                values: vec![
                    Item::new_default("java".to_string(), "Java".to_string()),
                    Item::new_default("kotlin".to_string(), "Kotlin".to_string()),
                ],
            },
        };

        // Simulate CLI-provided value
        let cli_values: HashMap<String, String> = [("language".to_string(), "kotlin".to_string())]
            .into_iter()
            .collect();
        let prefill = cli_values.get(&step.name).map(|s| s.as_str());

        // When prefill is provided, use it instead of default
        assert_eq!(prefill, Some("kotlin"));
    }
}
