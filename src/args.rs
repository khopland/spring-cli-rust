use clap::{command, Parser};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub(crate) struct Args {
    #[arg(short, long)]
    pub dependencies: Option<Vec<String>>,
    #[arg(short, long)]
    pub java_version: Option<String>,
    #[arg(short, long)]
    pub language: Option<String>,
    #[arg(short, long = "type")]
    pub type_build: Option<String>,
    #[arg(short, long)]
    pub group_id: Option<String>,
    #[arg(short, long)]
    pub artifact_id: Option<String>,
    #[arg(short, long)]
    pub version_number: Option<String>,
    #[arg(short, long)]
    pub name: Option<String>,
    #[arg(short, long)]
    pub url: Option<String>,
}
