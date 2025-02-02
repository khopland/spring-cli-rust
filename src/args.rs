use clap::{command, Parser};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    url: Option<String>,
    #[arg(short, long)]
    path: Option<String>,
}
impl Args {
    pub fn get_url(&self) -> String {
        self.url
            .to_owned()
            .unwrap_or("https://start.spring.io".to_owned())
    }
    pub fn get_path(&self) -> &Option<String> {
        &self.path
    }
}
