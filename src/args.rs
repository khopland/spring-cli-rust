use clap::{command, Parser};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub dependencies: Option<Vec<String>>,
    #[arg(short, long)]
    pub java_version: Option<String>,
    #[arg(short, long)]
    pub language: Option<String>,
    #[arg(short, long = "type")]
    pub build_type: Option<String>,
    #[arg(short, long)]
    pub group_id: Option<String>,
    #[arg(short, long)]
    pub artifact_id: Option<String>,
    #[arg(short, long)]
    pub version_number: Option<String>,
    #[arg(short, long)]
    pub name: Option<String>,
    #[arg(short, long)]
    url: Option<String>,
}
impl Args {
    pub fn get_url(&self) -> String {
        self.url
            .to_owned()
            .unwrap_or("https://start.spring.io".to_owned())
    }
    pub fn needs_to_get_user_input(&self) -> bool {
        self.dependencies.is_none()
            && self.build_type.is_none()
            && self.java_version.is_none()
            && self.language.is_none()
            && self.artifact_id.is_none()
            && self.group_id.is_none()
            && self.name.is_none()
            && self.version_number.is_none()
    }
}
