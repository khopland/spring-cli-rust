use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub(crate) struct Args {
    #[arg(short, long)]
    #[clap(default_value = "https://start.spring.io")]
    pub(crate) url: String,

    #[arg(short, long)]
    pub(crate) path: Option<String>,

    #[arg(long)]
    pub(crate) non_interactive: bool,

    #[arg(short = 'l', long)]
    pub(crate) language: Option<String>,

    #[arg(short = 'b', long)]
    pub(crate) boot_version: Option<String>,

    #[arg(short = 'g', long)]
    pub(crate) group_id: Option<String>,

    #[arg(short = 'a', long)]
    pub(crate) artifact_id: Option<String>,

    #[arg(short = 'n', long)]
    pub(crate) name: Option<String>,

    #[arg(short = 'd', long)]
    pub(crate) description: Option<String>,

    #[arg(long)]
    pub(crate) package_name: Option<String>,

    #[arg(long)]
    pub(crate) packaging: Option<String>,

    #[arg(short = 'j', long)]
    pub(crate) java_version: Option<String>,

    #[arg(short = 'D', long)]
    pub(crate) dependencies: Option<String>,

    #[arg(short = 't', long)]
    pub(crate) project_type: Option<String>,

    #[arg(long)]
    pub(crate) project_version: Option<String>,
}
