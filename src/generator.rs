use colored::*;
use std::{
    fs::{read_dir, DirEntry, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use crate::data::block_builder::{BlockBuilder, BlockBuilderConfig};

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

    let mut block_builder = BlockBuilder::new(BlockBuilderConfig {
        input_dir: input,
        output_dir: output.to_owned(),
        indent_string: "    ",
    });


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

    for (block_name, _) in block_builder.block_items.clone() {
        let block_name = block_name.to_string();

        let block_file = output.join(format!("{}.html", block_name));

        if block_file.exists() {
            if safe {
                panic!(
                    "{}",
                    format!(
                        "Block file {} already exists! Aborting because safe mode is on.",
                        block_name.normal()
                    )
                    .red()
                );
            } else {
                println!(
                    "{}",
                    format!(
                        "Block file {} already exists! File will be overwritten...",
                        block_name.normal()
                    )
                    .yellow()
                );
            }
        }

        let mut block_file = File::create(output.join(block_name.to_owned() + ".html"))
            .expect("Failed to create file");

        block_file
            .write_all(
                block_builder.construct_by_name(block_name.as_str())
                    .unwrap()
                    .as_bytes(),
            )
            .expect("Failed to write block content to file");
    }

    println!("{}", "Generation complete!".green());
}

pub fn generate_html_from_md(file: &DirEntry, file_name: &str, output: &Path, safe: bool) {
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

    let mut file = File::create(output_file).expect("Failed to create output file");

    let contents = markdown::to_html(&input_file_content);

    file.write_all(contents.as_bytes())
        .expect("Failed to write to output file");
}
