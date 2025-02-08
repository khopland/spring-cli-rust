use clap::{command, Parser};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub(crate) struct Args {
    #[arg(short, long)]
    #[clap(default_value = "https://start.spring.io")]
    pub(crate) url: String,
    #[arg(short, long)]
   pub(crate)  path: Option<String>,
}
