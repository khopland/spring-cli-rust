use anyhow::Result;
use inquire::{MultiSelect, Select, Text};

use crate::{
    request::ResponseStep,
    steps::{Item, Step, StepKind},
};

fn get_multi_select(name: &str, values: &[Item]) -> Result<String> {
    Ok(MultiSelect::new(
        format!("Select the {} you want:", name).as_str(),
        values.to_vec(),
    )
    .with_page_size(11)
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
    Ok(Text::new(format!("What {} do you want:", name).as_str())
        .with_default(default)
        .prompt()?)
}

pub(crate) fn get_user_input(step: &Step) -> Result<ResponseStep> {
    Ok(ResponseStep {
        step: step.to_owned(),
        response: match &step.kind {
            StepKind::Text { default } => get_text(&step.name, default)?,
            StepKind::SingleSelect { default, values } => {
                get_single_select(&step.name, values, default)?
            }
            StepKind::Action { default, values } => get_single_select(&step.name, values, default)?,
            StepKind::MultiSelect { values } => get_multi_select(&step.name, values)?,
        },
    })
}
