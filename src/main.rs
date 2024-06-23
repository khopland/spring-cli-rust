use anyhow::Result;
use args::Args;
use clap::Parser;
use inquire::Text;
use resolve_path::PathResolveExt;
use state::Data;

mod args;
mod request;
mod state;

fn main() -> Result<()> {
    let args = Args::parse();

    let state = Data::new(args)?;

    let file_name = Text::new("where do you want to store the zip file?")
        .with_default(format!("./{}.zip", state.get_name()).as_str())
        .prompt()?;
    let path = file_name.try_resolve()?;

    state.write_file_to_path(path)?;
    Ok(())
}
