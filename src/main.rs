use std::{fs, io::Write};

use anyhow::{Ok, Result};
use args::Args;
use clap::Parser;
use request::get_zip;
use resolve_path::PathResolveExt;
use steps::Step;
use user_innput::ResponseStep;

mod args;
mod request;
mod steps;
mod user_innput;

fn main() -> Result<()> {
    let args = Args::parse();
    let json = request::get_options(&args.url)?;

    let responses = Step::from_json(json)?
        .into_iter()
        .map(|step| user_innput::get_user_input(&step.to_owned()))
        .collect::<Result<Vec<ResponseStep>>>()?;

    let (file_name, zip) = get_zip(&args.url, &responses)?;
    let path = args
        .path
        .as_deref()
        .or(file_name.as_deref())
        .unwrap_or("./spring-app.zip");
    write_zip(path, zip)
}

fn write_zip(file_name: &str, zip: Vec<u8>) -> Result<()> {
    let path = file_name.try_resolve()?;
    println!("writing data to {}", path.display());
    let mut file = fs::File::create(path)?;
    file.write_all(&zip)?;
    Ok(())
}
