use colored::*;
use std::{
    fs::{read_dir, DirEntry, File},
    io::{Read, Write},
    path::PathBuf,
};

use crate::data::templates::{get_block_definitions, construct_from_block};

pub fn generate(input: PathBuf, output: PathBuf, safe: bool) {
    let input_files = read_dir(input.clone()).expect("Failed to read input directory");

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

    let block_definitions = get_block_definitions(&input).unwrap();

    for file in input_files {
        let file = file.expect("Failed to read file");
        let file_name = file.file_name();
        let file_name = file_name
            .to_str()
            .expect("Failed to convert file name to string");

        if file_name.ends_with(".md") {
            generate_html_from_md(&file, file_name, &output, safe);
        } else if file_name.ends_with(".yml") {
            // we don't need to do anything with the block definitions
        } else {
            println!("Copying file {}", file_name);
            std::fs::copy(file.path(), output.join(file_name)).expect("Failed to copy file");
        }
    }

    dbg!(&block_definitions);

    for (block_name, block) in block_definitions.clone() {
        let mut block_file = File::create(output.join(block_name.to_owned() + ".md")).expect("Failed to create file");
        block_file
            .write_all(construct_from_block(block_name.as_str(), &block, &block_definitions).unwrap().as_bytes())
            .expect("Failed to write block content to file");
    }

    println!("{}", "Generation complete!".green());
}

pub fn generate_html_from_md(file: &DirEntry, file_name: &str, output: &PathBuf, safe: bool) {
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

    let contents = markdown::to_html(&input_file_content);

    file.write_all(contents.as_bytes())
        .expect("Failed to write to output file");
}
