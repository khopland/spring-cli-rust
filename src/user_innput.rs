use anyhow::Result;
use inquire::{MultiSelect, Select, Text};

use crate::steps::{DepGroup, Item, ResponseStep, Step};

fn get_multi_select(name: &str, values: &[DepGroup]) -> Result<String> {
    Ok(MultiSelect::new(
        format!("Select the {} you want:", name).as_str(),
        values.iter().flat_map(|d| d.values.iter()).collect(),
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
    .collect::<Vec<String>>()
    .join(","))
}

fn get_single_select(name: &str, values: &[Item], default: &str) -> Result<String> {
    Ok(Select::new(
        format!("Select the {} your want:", name).as_str(),
        values.to_vec(),
    )
    .with_starting_cursor(values.iter().position(|x| x.id == default).unwrap_or(0))
    .prompt()?
    .id
    .to_owned())
}

fn get_text(name: &str, default: &str) -> Result<String> {
    Ok(Text::new(format!("What {} do you want:", name).as_str()).with_default(default).prompt()?)
}

pub(crate) fn get_user_input(step: &Step) -> Result<ResponseStep> {
    match step {
        Step::Text { name, default } => Ok(ResponseStep {
            step: step.to_owned(),
            response: get_text(name, default)?,
        }),
        Step::SingleSelect {
            name,
            default,
            values,
        } => Ok(ResponseStep {
            step: step.to_owned(),
            response: get_single_select(name, values, default)?,
        }),
        Step::Action {
            name,
            default,
            values,
        } => Ok(ResponseStep {
            step: step.to_owned(),
            response: get_single_select(
                name,
                &values
                    .iter()
                    .map(|v| Item::new(v.id.clone(), v.name.clone()))
                    .collect::<Vec<Item>>(),
                default,
            )?,
        }),
        Step::MultiSelect { name, values } => Ok(ResponseStep {
            step: step.to_owned(),
            response: get_multi_select(name, values)?,
        }),
    }
}
