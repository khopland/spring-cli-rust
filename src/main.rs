use std::{fs, io::Write};

use anyhow::{Ok, Result};
use args::Args;
use clap::Parser;
use request::get_zip;
use resolve_path::PathResolveExt;
use steps::ResponseStep;

mod args;
mod request;
mod steps;
mod user_innput;

fn main() -> Result<()> {
    let args = Args::parse();
    let json = request::get_options(&args.get_url())?;

    let responses = steps::parse_options(json)?
        .into_iter()
        .map(|step| step.get_user_input(&args))
        .collect::<Result<Vec<ResponseStep>>>()?;

    let (file_name, zip) = get_zip(&args.get_url(), &responses)?;
    let p = args.get_path();
    let path = p.or(file_name.as_deref()).unwrap_or("./spring-app.zip");
    write_zip(&path, zip)
}

fn write_zip(file_name: &str, zip: Vec<u8>) -> Result<()> {
    let path = file_name.try_resolve()?;
    println!("writing data to {}", path.display());
    let mut file = fs::File::create(path)?;
    file.write_all(&zip)?;
    Ok(())
}
