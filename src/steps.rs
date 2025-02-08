use anyhow::{Context, Result};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct ResponseStep {
    pub step: Step,
    pub response: String,
}
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
            ItemKind::Action(_) => write!(f, "{} - {}", self.name, self.id),
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

        for (key, value) in json {
            if key == "_links" {
                continue;
            }

            let body = value.as_object().context(key.to_owned())?;

            let Some(t) = body["type"].as_str() else {
                continue;
            };

            let kind: StepKind = match t {
                "text" => StepKind::Text {
                    default: body["default"]
                        .as_str()
                        .context("get default from TextStep")?
                        .to_string(),
                },
                "single-select" => StepKind::SingleSelect {
                    default: body["default"]
                        .as_str()
                        .context("get default from SingleStep")?
                        .to_string(),
                    values: body["values"]
                        .as_array()
                        .context("value")?
                        .iter()
                        .map(|v| {
                            let b = v.as_object().expect("not to be empty");
                            Item::new_default(
                                b["id"].as_str().expect("to contain id feld").to_string(),
                                b["name"].as_str().expect("to contain id name").to_string(),
                            )
                        })
                        .collect(),
                },
                "action" => StepKind::Action {
                    default: body["default"]
                        .as_str()
                        .context("get default from SingleStep")?
                        .to_string(),
                    values: body["values"]
                        .as_array()
                        .context("value")?
                        .iter()
                        .map(|v| {
                            let b = v.as_object().expect("not to be empty");
                            Item::new_action(
                                b["id"].as_str().expect("to contain id feld").to_string(),
                                b["name"].as_str().expect("to contain id name").to_string(),
                                b["action"]
                                    .as_str()
                                    .expect("to contain id action")
                                    .to_string(),
                            )
                        })
                        .collect(),
                },
                "hierarchical-multi-select" => StepKind::MultiSelect {
                    values: body["values"]
                        .as_array()
                        .context("value")?
                        .iter()
                        .flat_map(|v| {
                            let b = v.as_object().expect("not to be empty");
                            let group = b["name"]
                                .as_str()
                                .expect("group to contain name feld")
                                .to_string();
                            b["values"]
                                .as_array()
                                .expect("to contain values")
                                .iter()
                                .map(move |v| {
                                    Item::new_dependency(
                                        v["id"].as_str().expect("to contain id feld").to_string(),
                                        v["name"].as_str().expect("to contain id name").to_string(),
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
            "language": {
                "type": "single-select",
                "default": "java",
                "values": [
                    {
                        "id": "java",
                        "name": "Java"
                    },
                    {
                        "id": "kotlin",
                        "name": "Kotlin"
                    },
                    {
                        "id": "groovy",
                        "name": "Groovy"
                    }
                ]
            }
        });
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
                "type": "hierarchical-multi-select",
                "values": [{ "name": "Deps","values": [{"id": "native","name": "GraalVM Native Support"}]}]},
        });
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
    fn test_multible_parse() {
        let json = json!({
            "language": {
                "type": "single-select",
                "default": "java",
                "values": [
                    {
                        "id": "java",
                        "name": "Java"
                    },
                    {
                        "id": "kotlin",
                        "name": "Kotlin"
                    },
                    {
                        "id": "groovy",
                        "name": "Groovy"
                    }
                ]
            },
               "dep":{
                "type":"text",
                "default":"test"
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
