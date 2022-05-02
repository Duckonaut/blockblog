use std::{fs::File, io::Read};

use structopt::StructOpt;

mod generator;
mod data;

#[derive(StructOpt, Debug)]
#[structopt(name = "blockblog", about = "YAML and Markdown based static HTML generator")]
enum Args {
    #[structopt(name = "generate", about = "Generate all static HTML pages")]
    Generate {
        #[structopt(short = "i", long = "input", parse(from_os_str), default_value = ".", help = "Input directory")]
        input: std::path::PathBuf,
        #[structopt(short = "o", long = "output", parse(from_os_str), default_value = "./output", help = "Output directory")]
        output: std::path::PathBuf,
        #[structopt(short = "s", long = "safe", help = "Do not remove output directory files already present")]
        safe: bool,
    }
}

fn main() {
    let args = Args::from_args();

    match args {
        Args::Generate { input, output, safe } => generator::generate(input, output, safe),
    }
}
