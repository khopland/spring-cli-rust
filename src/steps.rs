use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::{
    args::Args,
    user_innput::{get_multi_select, get_single_select, get_text},
};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepGroup {
    pub name: String,
    pub values: Vec<Item>,
}
impl DepGroup {
    pub fn new(name: String, values: Vec<Item>) -> Self {
        DepGroup { name, values }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextStep {
    pub name: String,
    pub default: String,
}

impl TextStep {
    pub fn new(name: String, default: String) -> Self {
        TextStep { name, default }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SingleStep {
    pub name: String,
    pub default: String,
    pub values: Vec<Item>,
}
impl SingleStep {
    pub fn new(name: String, default: String, values: Vec<Item>) -> Self {
        SingleStep {
            name,
            default,
            values,
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionStep {
    pub name: String,
    pub default: String,
    pub values: Vec<ActionItem>,
}
impl ActionStep {
    pub fn new(name: String, default: String, values: Vec<ActionItem>) -> Self {
        ActionStep {
            name,
            default,
            values,
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiSelectStep {
    pub name: String,
    pub values: Vec<DepGroup>,
}

impl MultiSelectStep {
    pub fn new(name: String, values: Vec<DepGroup>) -> Self {
        MultiSelectStep { name, values }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Step {
    Text(TextStep),
    SingleSelect(SingleStep),
    Action(ActionStep),
    MultiSelect(MultiSelectStep),
}

impl Step {
    pub fn get_name(&self) -> &str {
        match self {
            Step::Text(text_step) => &text_step.name,
            Step::SingleSelect(single_step) => &single_step.name,
            Step::Action(action_step) => &action_step.name,
            Step::MultiSelect(multi_select_step) => &multi_select_step.name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponseStep {
    pub step: Step,
    pub response: String,
}

impl Step {
    pub fn get_user_input(&self, _args: &Args) -> Result<ResponseStep> {
        match self {
            Step::Text(text_step) => Ok(ResponseStep {
                step: self.to_owned(),
                response: get_text(&text_step.name, &text_step.default)?,
            }),
            Step::SingleSelect(single_step) => Ok(ResponseStep {
                step: self.to_owned(),
                response: get_single_select(
                    &single_step.name,
                    &single_step.values,
                    &single_step.default,
                )?,
            }),
            Step::Action(action_step) => Ok(ResponseStep {
                step: self.to_owned(),
                response: get_single_select(
                    &action_step.name,
                    &action_step
                        .values
                        .iter()
                        .map(|v| Item::new(v.id.clone(), v.name.clone()))
                        .collect(),
                    &action_step.default,
                )?,
            }),
            Step::MultiSelect(multi_select_step) => Ok(ResponseStep {
                step: self.to_owned(),
                response: get_multi_select(&multi_select_step.name, &multi_select_step.values)?,
            }),
        }
    }
}

pub fn parse_options(json: serde_json::Value) -> Result<Vec<Step>> {
    let json = json.as_object().context("json")?;

    let mut list: Vec<Step> = Vec::with_capacity(json.len() - 1);

    for (key, value) in json {
        if key == "_links" {
            continue;
        }

        let body: &serde_json::Map<String, serde_json::Value> =
            value.as_object().context(key.clone())?;

        let Some(t) = body["type"].as_str() else {
            continue;
        };

        let step: Step = match t {
            "text" => Step::Text(TextStep::new(
                key.to_owned(),
                body["default"]
                    .as_str()
                    .context("get default from TextStep")?
                    .to_string(),
            )),
            "single-select" => Step::SingleSelect(SingleStep::new(
                key.to_owned(),
                body["default"]
                    .as_str()
                    .context("get default from SingleStep")?
                    .to_string(),
                body["values"]
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
            )),
            "action" => Step::Action(ActionStep::new(
                key.to_owned(),
                body["default"]
                    .as_str()
                    .context("get default from SingleStep")?
                    .to_string(),
                body["values"]
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
            )),
            "hierarchical-multi-select" => Step::MultiSelect(MultiSelectStep::new(
                key.to_owned(),
                body["values"]
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
                                        v["id"].as_str().expect("to contain id feld").to_string(),
                                        v["name"].as_str().expect("to contain id name").to_string(),
                                    )
                                })
                                .collect(),
                        )
                    })
                    .collect(),
            )),
            _ => continue,
        };
        list.push(step);
    }
    Ok(list)
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
        let steps = parse_options(json);
        assert!(steps.is_ok());
        let steps = steps.unwrap();

        assert_eq!(steps.len(), 1);
        assert_eq!(
            steps[0],
            steps::Step::Text(TextStep::new("dep".to_string(), "test".to_string()))
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
        let steps = parse_options(json);
        assert!(steps.is_ok());
        let steps = steps.unwrap();

        assert_eq!(steps.len(), 1);
        assert_eq!(
            steps[0],
            steps::Step::SingleSelect(SingleStep::new(
                "language".to_string(),
                "java".to_string(),
                vec![
                    Item::new("java".to_owned(), "Java".to_owned()),
                    Item::new("kotlin".to_owned(), "Kotlin".to_owned()),
                    Item::new("groovy".to_owned(), "Groovy".to_owned())
                ]
            ))
        );
    }

    #[test]
    fn test_multi_select_parse() {
        let json: serde_json::Value = json!({
            "dependencies": {
                "type": "hierarchical-multi-select",
                "values": [{ "name": "Deps","values": [{"id": "native","name": "GraalVM Native Support"}]}]},
        });
        let steps = parse_options(json);
        assert!(steps.is_ok());
        let steps = steps.unwrap();

        assert_eq!(steps.len(), 1);
        assert_eq!(
            steps[0],
            steps::Step::MultiSelect(MultiSelectStep::new(
                "dependencies".to_string(),
                vec![DepGroup::new(
                    "Deps".to_string(),
                    vec![Item::new(
                        "native".to_string(),
                        "GraalVM Native Support".to_owned()
                    )]
                )]
            ))
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
        let steps = parse_options(json);
        assert!(steps.is_ok());
        let steps = steps.unwrap();

        assert_eq!(steps.len(), 2);

        let _ = steps
            .iter()
            .map(|s| match s {
                Step::Text(text_step) => {
                    assert_eq!(
                        *text_step,
                        TextStep::new("dep".to_string(), "test".to_string(),)
                    );
                }
                Step::SingleSelect(single_step) => {
                    assert_eq!(
                        *single_step,
                        SingleStep::new(
                            "language".to_string(),
                            "java".to_string(),
                            vec![
                                Item::new("java".to_owned(), "Java".to_owned()),
                                Item::new("kotlin".to_owned(), "Kotlin".to_owned()),
                                Item::new("groovy".to_owned(), "Groovy".to_owned())
                            ]
                        )
                    );
                }
                Step::Action(_) => {
                    panic!("not in test data")
                }
                Step::MultiSelect(_) => {
                    panic!("not in test data")
                }
            })
            .collect::<()>();
    }
}
