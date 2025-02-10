use anyhow::{Context, Result};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub kind: ItemKind,
}
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]

pub struct Step {
    pub name: String,
    pub kind: StepKind,
}

#[derive(Debug, Clone, PartialEq)]
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
                    let content: &Vec<serde_json::Value> =
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
                name: key.to_owned(),
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
            "dep":{
                "type":"text",
                "default":"test"
            },
        });
        let steps = Step::from_json(json);
        assert!(steps.is_ok());
        let steps = steps.unwrap();

        assert_eq!(steps.len(), 1);
        assert_eq!(
            steps[0],
            Step {
                name: "dep".to_owned(),
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
                name: "types".to_string(),
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
            "language": {
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
               "dep":{
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
}
