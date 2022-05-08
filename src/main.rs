use clap::Parser;
use color_eyre::Result;

mod data;
mod generator;

#[derive(Parser, Debug, Clone)]
#[clap(
    name = "blockblog",
    about = "YAML and Markdown based static HTML generator"
)]
enum Args {
    #[clap(name = "generate", about = "Generate all static HTML pages")]
    Generate {
        #[clap(
            short = 'i',
            long = "input",
            parse(from_os_str),
            default_value = ".",
            help = "Input directory"
        )]
        input: std::path::PathBuf,
        #[clap(
            short = 'o',
            long = "output",
            parse(from_os_str),
            default_value = "./output",
            help = "Output directory"
        )]
        output: std::path::PathBuf,
        #[clap(
            short = 's',
            long = "safe",
            help = "Do not remove output directory files already present"
        )]
        safe: bool,
        #[clap(
            short = 'd',
            long = "debug",
            help = "Insert debug information in the generated HTML"
        )]
        debug: bool,
    },
}

fn main() -> Result<()> {
    setup()?;

    let args = Args::parse();
    match args {
        Args::Generate {
            input,
            output,
            safe,
            debug,
        } => generator::generate(input, output, safe, debug)?,
    }

    Ok(())
}

fn setup() -> Result<()> {
    if std::env::var("RUST_BACKTRACE").is_err() {
        std::env::set_var("RUST_BACKTRACE", "1")
    }
    color_eyre::install()?;

    Ok(())
}
