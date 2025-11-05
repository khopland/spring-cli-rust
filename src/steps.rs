use anyhow::{Context, Result};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub kind: ItemKind,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemKind {
    Default,
    Dependency(String),
    Action(String),
}

impl Item {
    pub fn new_default(id: String, name: String) -> Self {
        Item {
            id,
            name,
            kind: ItemKind::Default,
        }
    }
    pub fn new_action(id: String, name: String, action: String) -> Self {
        Item {
            id,
            name,
            kind: ItemKind::Action(action),
        }
    }
    pub fn new_dependency(id: String, name: String, group: String) -> Self {
        Item {
            id,
            name,
            kind: ItemKind::Dependency(group),
        }
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ItemKind::Default => write!(f, "{}", self.name),
            ItemKind::Dependency(group) => write!(f, "{} - ({}) [{}]", self.name, self.id, group),
            ItemKind::Action(_) => write!(f, "{}", self.name),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]

pub struct Step {
    pub name: String,
    pub kind: StepKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepKind {
    Text { default: String },
    SingleSelect { default: String, values: Vec<Item> },
    Action { default: String, values: Vec<Item> },
    MultiSelect { values: Vec<Item> },
}

impl Step {
    pub fn from_json(json: serde_json::Value) -> Result<Vec<Step>> {
        let json = json.as_object().context("json")?;

        let mut list = Vec::with_capacity(json.len() - 1);

        for (key, body) in json {
            if key == "configuration" {
                continue;
            }

            let Some(t) = body["type"].as_str() else {
                continue;
            };

            let kind: StepKind = match t {
                "TEXT" => StepKind::Text {
                    default: body["content"]
                        .as_str()
                        .context("get default from TextStep")?
                        .to_string(),
                },
                "SINGLE_SELECT" => {
                    let content = body["content"]
                        .as_array()
                        .context("get content single select")?;

                    let default = content
                        .iter()
                        .find(|v| v["default"].as_bool().expect("to be a boolean"))
                        .expect("to have a default value")["id"]
                        .as_str()
                        .expect("to have id")
                        .to_owned();

                    let values = content
                        .iter()
                        .map(|v| {
                            Item::new_default(
                                v["id"].as_str().expect("to contain id").to_string(),
                                v["name"].as_str().expect("to contain name").to_string(),
                            )
                        })
                        .collect();

                    StepKind::SingleSelect { default, values }
                }
                "ACTION" => {
                    let content =
                        body["content"].as_array().context("get content action")?;

                    let default = content
                        .iter()
                        .find(|v| v["default"].as_bool().expect("to be a boolean"))
                        .expect("to have a default value")["id"]
                        .as_str()
                        .expect("to have id")
                        .to_owned();

                    let values = content
                        .iter()
                        .map(|v| {
                            Item::new_action(
                                v["id"].as_str().expect("to contain id").to_string(),
                                v["name"].as_str().expect("to contain name").to_string(),
                                v["action"].as_str().expect("to contain action").to_string(),
                            )
                        })
                        .collect();

                    StepKind::Action { default, values }
                }
                "HIERARCHICAL_MULTI_SELECT" => StepKind::MultiSelect {
                    values: body["content"]
                        .as_array()
                        .context("expectet content")?
                        .iter()
                        .flat_map(|v| {
                            let group = v["name"]
                                .as_str()
                                .expect("group to contain name")
                                .to_string();
                            v["content"]
                                .as_array()
                                .expect("to contain content")
                                .iter()
                                .map(move |v| {
                                    Item::new_dependency(
                                        v["id"].as_str().expect("to contain id").to_string(),
                                        v["name"].as_str().expect("to contain name").to_string(),
                                        group.clone(),
                                    )
                                })
                        })
                        .collect(),
                },
                _ => continue,
            };
            list.push(Step {
                name: body["id"].as_str().context("expect to have id")?.to_owned(),
                kind,
            });
        }
        Ok(list)
    }
}

#[cfg(test)]
mod test {
    use crate::steps;

    use super::*;
    use serde_json::json;

    #[test]
    fn test_text() {
        let json = json!({
            "groupeId":{
                "id":"groupeId",
                "type":"TEXT",
                "content":"test"
            },
        });
        let steps = Step::from_json(json);
        assert!(steps.is_ok());
        let steps = steps.unwrap();

        assert_eq!(steps.len(), 1);
        assert_eq!(
            steps[0],
            Step {
                name: "groupeId".to_owned(),
                kind: steps::StepKind::Text {
                    default: "test".to_owned()
                }
            }
        );
    }

    #[test]
    fn test_simple_parse() {
        let json = json!({
        "languages": {
            "id": "language",
            "type": "SINGLE_SELECT",
            "title": "Language",
            "description": "programming language",
            "content": [
                {
                    "name": "Java",
                    "id": "java",
                    "default": true
                },
                {
                    "name": "Kotlin",
                    "id": "kotlin",
                    "default": false
                },
                {
                    "name": "Groovy",
                    "id": "groovy",
                    "default": false
                }
            ]
        }});
        let steps = Step::from_json(json);
        assert!(steps.is_ok());
        let steps = steps.unwrap();

        assert_eq!(steps.len(), 1);
        assert_eq!(
            steps[0],
            Step {
                name: "language".to_string(),
                kind: steps::StepKind::SingleSelect {
                    default: "java".to_string(),
                    values: vec![
                        Item::new_default("java".to_owned(), "Java".to_owned()),
                        Item::new_default("kotlin".to_owned(), "Kotlin".to_owned()),
                        Item::new_default("groovy".to_owned(), "Groovy".to_owned())
                    ]
                }
            }
        );
    }

    #[test]
    fn test_multi_select_parse() {
        let json: serde_json::Value = json!({
        "dependencies": {
            "id": "dependencies",
            "type": "HIERARCHICAL_MULTI_SELECT",
            "content": [
                {
                    "name": "Deps",
                    "content": [
                        {
                            "name": "GraalVM Native Support",
                            "id": "native"
                        }
                    ]
                }
            ]
        }});
        let steps = Step::from_json(json);
        assert!(steps.is_ok());
        let steps = steps.unwrap();

        assert_eq!(steps.len(), 1);
        assert_eq!(
            steps[0],
            Step {
                name: "dependencies".to_string(),
                kind: steps::StepKind::MultiSelect {
                    values: vec![Item::new_dependency(
                        "native".to_string(),
                        "GraalVM Native Support".to_owned(),
                        "Deps".to_string(),
                    )]
                }
            }
        );
    }

    #[test]
    fn test_action_parse() {
        let json: serde_json::Value = json!({
        "types": {
           "id": "type",
           "type": "ACTION",
           "content": [
               {
                   "name": "Gradle - Groovy",
                   "id": "gradle-project",
                   "action": "/starter.zip",
                   "default": true
               }]
            }});
        let steps = Step::from_json(json);
        assert!(steps.is_ok());
        let steps = steps.unwrap();

        assert_eq!(steps.len(), 1);
        assert_eq!(
            steps[0],
            Step {
                name: "type".to_string(),
                kind: steps::StepKind::Action {
                    default: "gradle-project".to_owned(),
                    values: vec![Item::new_action(
                        "gradle-project".to_owned(),
                        "Gradle - Groovy".to_owned(),
                        "/starter.zip".to_owned()
                    )]
                }
            }
        );
    }

    #[test]
    fn test_multible_parse() {
        let json = json!({
            "languages": {
                "id":"language",
                "type": "SINGLE_SELECT",
                "content": [
                    {
                        "id": "java",
                        "name": "Java",
                        "default":true
                    },
                    {
                        "id": "kotlin",
                        "name": "Kotlin",
                        "default":false
                    },
                    {
                        "id": "groovy",
                        "name": "Groovy",
                        "default":false
                    }
                ]
            },
               "deps":{
                "id":"dep",
                "type":"TEXT",
                "content":"test"
            },
        });
        let steps = Step::from_json(json);
        assert!(steps.is_ok());
        let steps = steps.unwrap();

        assert_eq!(steps.len(), 2);

        let _ = steps
            .iter()
            .map(|s| match &s.kind {
                StepKind::Text { default } => {
                    assert_eq!(&s.name, &"dep".to_owned());
                    assert_eq!(default, &"test".to_owned());
                }
                StepKind::SingleSelect { default, values } => {
                    assert_eq!(&s.name, &"language".to_owned());
                    assert_eq!(default, &"java".to_owned());
                    assert_eq!(
                        values,
                        &vec![
                            Item::new_default("java".to_owned(), "Java".to_owned()),
                            Item::new_default("kotlin".to_owned(), "Kotlin".to_owned()),
                            Item::new_default("groovy".to_owned(), "Groovy".to_owned())
                        ]
                    );
                }
                StepKind::Action { .. } => {
                    panic!("not in test data")
                }
                StepKind::MultiSelect { .. } => {
                    panic!("not in test data")
                }
            })
            .collect::<()>();
    }

    #[test]
    fn test_with_start_spring_response() {
        let json = json!({
            "configuration": {
                "env": {
                    "artifactRepository": "https://repo.spring.io/release/",
                    "springBootMetadataUrl": "https://api.spring.io/projects/spring-boot/releases",
                    "googleAnalyticsTrackingCode": null,
                    "fallbackApplicationName": "Application",
                    "kotlin": {
                        "defaultVersion": null,
                        "mappings": []
                    },
                    "maven": {
                        "parent": {
                            "groupId": null,
                            "artifactId": null,
                            "version": null,
                            "relativePath": "",
                            "includeSpringBootBom": false
                        }
                    },
                    "platform": {
                        "compatibilityRange": "3.3.0",
                        "v1FormatCompatibilityRange": null,
                        "v2FormatCompatibilityRange": null
                    }
                }
            },
            "dependencies": {
                "id": "dependencies",
                "type": "HIERARCHICAL_MULTI_SELECT",
                "title": "Project dependencies",
                "description": "dependency identifiers (comma-separated)",
                "content": [
                    {
                        "name": "Developer Tools",
                        "content": [
                            {
                                "name": "GraalVM Native Support",
                                "id": "native",
                                "groupId": "org.springframework.boot",
                                "artifactId": "spring-boot",
                                "scope": "compile",
                                "description": "Support for compiling Spring applications to native executables using the GraalVM native-image compiler.",
                                "starter": false
                            },
                            {
                                "name": "GraphQL DGS Code Generation",
                                "id": "dgs-codegen",
                                "groupId": "com.netflix.graphql.dgs.codegen",
                                "artifactId": "graphql-dgs-codegen-gradle",
                                "version": "7.0.3",
                                "scope": "compile",
                                "description": "Generate data types and type-safe APIs for querying GraphQL APIs by parsing schema files.",
                                "starter": false
                            },
                            {
                                "name": "Spring Boot DevTools",
                                "id": "devtools",
                                "groupId": "org.springframework.boot",
                                "artifactId": "spring-boot-devtools",
                                "scope": "runtime",
                                "description": "Provides fast application restarts, LiveReload, and configurations for enhanced development experience.",
                                "starter": false
                            },
                            {
                                "name": "Lombok",
                                "id": "lombok",
                                "groupId": "org.projectlombok",
                                "artifactId": "lombok",
                                "scope": "annotationProcessor",
                                "description": "Java annotation library which helps to reduce boilerplate code.",
                                "starter": false
                            },
                            {
                                "name": "Spring Configuration Processor",
                                "id": "configuration-processor",
                                "groupId": "org.springframework.boot",
                                "artifactId": "spring-boot-configuration-processor",
                                "scope": "annotationProcessor",
                                "description": "Generate metadata for developers to offer contextual help and \"code completion\" when working with custom configuration keys (ex.application.properties/.yml files).",
                                "starter": false
                            },
                            {
                                "name": "Docker Compose Support",
                                "id": "docker-compose",
                                "groupId": "org.springframework.boot",
                                "artifactId": "spring-boot-docker-compose",
                                "scope": "runtime",
                                "description": "Provides docker compose support for enhanced development experience.",
                                "starter": false
                            },
                            {
                                "name": "Spring Modulith",
                                "id": "modulith",
                                "groupId": "org.springframework.modulith",
                                "artifactId": "spring-modulith-starter-core",
                                "scope": "compile",
                                "description": "Support for building modular monolithic applications.",
                                "compatibilityRange": "[3.3.0,3.5.0-M1)",
                                "bom": "spring-modulith",
                                "starter": true
                            }
                        ]
                    },
                    {
                        "name": "Web",
                        "content": [
                            {
                                "name": "Spring Web",
                                "id": "web",
                                "facets": [
                                    "web",
                                    "json"
                                ],
                                "groupId": "org.springframework.boot",
                                "artifactId": "spring-boot-starter-web",
                                "scope": "compile",
                                "description": "Build web, including RESTful, applications using Spring MVC. Uses Apache Tomcat as the default embedded container.",
                                "starter": true,
                                "links": [
                                    {
                                        "rel": "guide",
                                        "href": "https://spring.io/guides/gs/rest-service/",
                                        "description": "Building a RESTful Web Service"
                                    },
                                    {
                                        "rel": "reference",
                                        "href": "https://docs.spring.io/spring-boot/{bootVersion}/reference/web/servlet.html",
                                        "templated": true
                                    },
                                    {
                                        "rel": "guide",
                                        "href": "https://spring.io/guides/gs/serving-web-content/",
                                        "description": "Serving Web Content with Spring MVC"
                                    },
                                    {
                                        "rel": "guide",
                                        "href": "https://spring.io/guides/tutorials/rest/",
                                        "description": "Building REST services with Spring"
                                    }
                                ]
                            }
                        ]
                    }
                ]
            },
            "types": {
                "id": "type",
                "type": "ACTION",
                "title": "Type",
                "description": "project type",
                "content": [
                    {
                        "name": "Gradle - Groovy",
                        "id": "gradle-project",
                        "description": "Generate a Gradle based project archive using the Groovy DSL.",
                        "action": "/starter.zip",
                        "tags": {
                            "build": "gradle",
                            "dialect": "groovy",
                            "format": "project"
                        },
                        "default": true
                    },
                    {
                        "name": "Gradle - Kotlin",
                        "id": "gradle-project-kotlin",
                        "description": "Generate a Gradle based project archive using the Kotlin DSL.",
                        "action": "/starter.zip",
                        "tags": {
                            "build": "gradle",
                            "dialect": "kotlin",
                            "format": "project"
                        },
                        "default": false
                    },
                    {
                        "name": "Gradle Config",
                        "id": "gradle-build",
                        "description": "Generate a Gradle build file.",
                        "action": "/build.gradle",
                        "tags": {
                            "build": "gradle",
                            "format": "build"
                        },
                        "default": false
                    },
                    {
                        "name": "Maven",
                        "id": "maven-project",
                        "description": "Generate a Maven based project archive.",
                        "action": "/starter.zip",
                        "tags": {
                            "build": "maven",
                            "format": "project"
                        },
                        "default": false
                    },
                    {
                        "name": "Maven POM",
                        "id": "maven-build",
                        "description": "Generate a Maven pom.xml.",
                        "action": "/pom.xml",
                        "tags": {
                            "build": "maven",
                            "format": "build"
                        },
                        "default": false
                    }
                ]
            },
            "bootVersions": {
                "id": "bootVersion",
                "type": "SINGLE_SELECT",
                "title": "Spring Boot Version",
                "description": "spring boot version",
                "content": [
                    {
                        "name": "3.5.0 (SNAPSHOT)",
                        "id": "3.5.0-SNAPSHOT",
                        "default": false
                    },
                    {
                        "name": "3.5.0 (M1)",
                        "id": "3.5.0-M1",
                        "default": false
                    },
                    {
                        "name": "3.4.3 (SNAPSHOT)",
                        "id": "3.4.3-SNAPSHOT",
                        "default": false
                    },
                    {
                        "name": "3.4.2",
                        "id": "3.4.2",
                        "default": true
                    },
                    {
                        "name": "3.3.9 (SNAPSHOT)",
                        "id": "3.3.9-SNAPSHOT",
                        "default": false
                    },
                    {
                        "name": "3.3.8",
                        "id": "3.3.8",
                        "default": false
                    }
                ]
            },
            "packagings": {
                "id": "packaging",
                "type": "SINGLE_SELECT",
                "title": "Packaging",
                "description": "project packaging",
                "content": [
                    {
                        "name": "Jar",
                        "id": "jar",
                        "default": true
                    },
                    {
                        "name": "War",
                        "id": "war",
                        "default": false
                    }
                ]
            },
            "javaVersions": {
                "id": "javaVersion",
                "type": "SINGLE_SELECT",
                "title": "Java Version",
                "description": "language level",
                "content": [
                    {
                        "name": "23",
                        "id": "23",
                        "default": false
                    },
                    {
                        "name": "21",
                        "id": "21",
                        "default": false
                    },
                    {
                        "name": "17",
                        "id": "17",
                        "default": true
                    }
                ]
            },
            "languages": {
                "id": "language",
                "type": "SINGLE_SELECT",
                "title": "Language",
                "description": "programming language",
                "content": [
                    {
                        "name": "Java",
                        "id": "java",
                        "default": true
                    },
                    {
                        "name": "Kotlin",
                        "id": "kotlin",
                        "default": false
                    },
                    {
                        "name": "Groovy",
                        "id": "groovy",
                        "default": false
                    }
                ]
            },
            "name": {
                "id": "name",
                "type": "TEXT",
                "title": "Name",
                "description": "project name (infer application name)",
                "content": "demo"
            },
            "description": {
                "id": "description",
                "type": "TEXT",
                "title": "Description",
                "description": "project description",
                "content": "Demo project for Spring Boot"
            },
            "groupId": {
                "id": "groupId",
                "type": "TEXT",
                "title": "Group",
                "description": "project coordinates",
                "content": "com.example"
            },
            "artifactId": {
                "id": "artifactId",
                "type": "TEXT",
                "title": "Artifact",
                "description": "project coordinates (infer archive name)",
                "content": "demo"
            },
            "version": {
                "id": "version",
                "type": "TEXT",
                "title": "Version",
                "description": "project version",
                "content": "0.0.1-SNAPSHOT"
            },
            "packageName": {
                "id": "packageName",
                "type": "TEXT",
                "title": "Package Name",
                "description": "root package",
                "content": "com.example.demo"
            }
        });
        let steps = Step::from_json(json);
        assert!(steps.is_ok());
        let steps = steps.unwrap();

        assert_eq!(steps.len(), 12);
    }
}
