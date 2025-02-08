use anyhow::{Context, Result};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct ResponseStep {
    pub step: Step,
    pub response: String,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct DepGroup {
    pub name: String,
    pub values: Vec<Item>,
}
impl DepGroup {
    pub fn new(name: String, values: Vec<Item>) -> Self {
        DepGroup { name, values }
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Item {
    pub id: String,
    pub name: String,
}

impl Item {
    pub fn new(id: String, name: String) -> Self {
        Item { id, name }
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {}", self.id, self.name)
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ActionItem {
    pub id: String,
    pub name: String,
    pub action: String,
}

impl ActionItem {
    pub fn new(id: String, name: String, action: String) -> Self {
        ActionItem { id, name, action }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Step {
    Text {
        name: String,
        default: String,
    },
    SingleSelect {
        name: String,
        default: String,
        values: Vec<Item>,
    },
    Action {
        name: String,
        default: String,
        values: Vec<ActionItem>,
    },
    MultiSelect {
        name: String,
        values: Vec<DepGroup>,
    },
}

impl Step {
    pub fn get_name(&self) -> &str {
        match self {
            Step::Text { name, .. } => name,
            Step::SingleSelect { name, .. } => name,
            Step::Action { name, .. } => name,
            Step::MultiSelect { name, .. } => name,
        }
    }

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

            let step: Step = match t {
                "text" => Step::Text {
                    name: key.to_owned(),
                    default: body["default"]
                        .as_str()
                        .context("get default from TextStep")?
                        .to_string(),
                },
                "single-select" => Step::SingleSelect {
                    name: key.to_owned(),
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
                            Item::new(
                                b["id"].as_str().expect("to contain id feld").to_string(),
                                b["name"].as_str().expect("to contain id name").to_string(),
                            )
                        })
                        .collect(),
                },
                "action" => Step::Action {
                    name: key.to_owned(),
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
                            ActionItem::new(
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
                "hierarchical-multi-select" => Step::MultiSelect {
                    name: key.to_owned(),
                    values: body["values"]
                        .as_array()
                        .context("value")?
                        .iter()
                        .map(|v| {
                            let b = v.as_object().expect("not to be empty");
                            DepGroup::new(
                                b["name"]
                                    .as_str()
                                    .expect("to contain name feld")
                                    .to_string(),
                                b["values"]
                                    .as_array()
                                    .expect("to contain values")
                                    .iter()
                                    .map(|v| {
                                        Item::new(
                                            v["id"]
                                                .as_str()
                                                .expect("to contain id feld")
                                                .to_string(),
                                            v["name"]
                                                .as_str()
                                                .expect("to contain id name")
                                                .to_string(),
                                        )
                                    })
                                    .collect(),
                            )
                        })
                        .collect(),
                },
                _ => continue,
            };
            list.push(step);
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
            steps::Step::Text {
                name: "dep".to_owned(),
                default: "test".to_owned()
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
            steps::Step::SingleSelect {
                name: "language".to_string(),
                default: "java".to_string(),
                values: vec![
                    Item::new("java".to_owned(), "Java".to_owned()),
                    Item::new("kotlin".to_owned(), "Kotlin".to_owned()),
                    Item::new("groovy".to_owned(), "Groovy".to_owned())
                ]
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
            steps::Step::MultiSelect {
                name: "dependencies".to_string(),
                values: vec![DepGroup::new(
                    "Deps".to_string(),
                    vec![Item::new(
                        "native".to_string(),
                        "GraalVM Native Support".to_owned()
                    )]
                )]
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
            .map(|s| match s {
                Step::Text { name, default } => {
                    assert_eq!(name, &"dep".to_owned());
                    assert_eq!(default, &"test".to_owned());
                }
                Step::SingleSelect {
                    name,
                    default,
                    values,
                } => {
                    assert_eq!(name, &"language".to_owned());
                    assert_eq!(default, &"java".to_owned());
                    assert_eq!(
                        values,
                        &vec![
                            Item::new("java".to_owned(), "Java".to_owned()),
                            Item::new("kotlin".to_owned(), "Kotlin".to_owned()),
                            Item::new("groovy".to_owned(), "Groovy".to_owned())
                        ]
                    );
                }
                Step::Action { .. } => {
                    panic!("not in test data")
                }
                Step::MultiSelect { .. } => {
                    panic!("not in test data")
                }
            })
            .collect::<()>();
    }
}
