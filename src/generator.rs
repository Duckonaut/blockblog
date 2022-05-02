use colored::*;
use std::{
    fs::{read_dir, File},
    io::{Read, Write},
    path::PathBuf,
};

pub fn generate(input: PathBuf, output: PathBuf, safe: bool) {
    let input_files = read_dir(input).expect("Failed to read input directory");

    if !output.exists() {
        std::fs::create_dir_all(&output).expect("Failed to create output directory");
    }

    let output_files = read_dir(output.clone()).expect("Failed to read output directory");

    if output_files.count() != 0 {
        if safe {
            panic!(
                "{}",
                "Output directory is not empty! Aborting because safe mode is on.".red()
            );
        } else {
            println!(
                "{}",
                "Output directory is not empty! Files will be overwritten...".yellow()
            );
        }
    }

    for file in input_files {
        let file = file.expect("Failed to read file");
        let file_name = file.file_name();
        let file_name = file_name
            .to_str()
            .expect("Failed to convert file name to string");

        if file_name.ends_with(".md") {
            let output_filename = file_name.replace(".md", ".html");
            let output_file = output.join(output_filename.clone());

            if output_file.exists() {
                if safe {
                    panic!(
                        "{}",
                        "Output file already exists! Aborting because safe mode is on.".red()
                    );
                } else {
                    println!(
                        "Output file {} already exists! File will be overwritten...",
                        output_filename
                    );
                }
            }

            let input_file = file.path();
            let mut input_file = File::open(input_file).expect("Failed to open input file");
            let mut input_file_content = String::new();

            input_file
                .read_to_string(&mut input_file_content)
                .expect("Failed to read input file");

            let mut file = File::create(output_file.clone()).expect("Failed to create output file");

            let contents =
                markdown::to_html(&input_file_content);

            file.write_all(contents.as_bytes())
                .expect("Failed to write to output file");
        }
    }
}
