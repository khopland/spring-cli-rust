use std::{fs, io::Write};

use anyhow::{Context, Ok, Result};
use args::Args;
use clap::Parser;
use request::{get_zip, ResponseStep};
use resolve_path::PathResolveExt;
use std::io::Cursor;
use steps::Step;
use zip::ZipArchive;

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
    if path.extension().is_none() && ZipArchive::new(Cursor::new(&zip)).is_ok() {
        fs::create_dir_all(&path)?;
        println!("writing data to {}", path.display());
        let mut archive = ZipArchive::new(Cursor::new(&zip))?;
        archive.extract(&path)?;
    } else {
        let parent = &path.parent().context("dident find parent of file")?;
        fs::create_dir_all(parent)?;
        println!("writing data to {}", path.display());
        let mut file = fs::File::create(path)?;
        file.write_all(&zip)?;
    }
    Ok(())
}
