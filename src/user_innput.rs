use anyhow::Result;
use inquire::{MultiSelect, Select, Text};

use crate::steps::{DepGroup, Item};

pub fn get_multi_select(name: &str, values: &Vec<DepGroup>) -> Result<String> {
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

pub fn get_single_select(message: &str, values: &Vec<Item>, default: &str) -> Result<String> {
    Ok(Select::new(
        format!("Select the {} your want:", message).as_str(),
        values.to_vec(),
    )
    .with_starting_cursor(values.iter().position(|x| x.id == default).unwrap_or(0))
    .prompt()?
    .id
    .to_owned())
}

pub fn get_text(message: &str, default: &str) -> Result<String> {
    Ok(Text::new(&message).with_default(&default).prompt()?)
}
