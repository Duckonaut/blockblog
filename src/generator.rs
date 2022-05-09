use color_eyre::Result;
use colored::*;
use std::{
    fs::{read_dir, DirEntry, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use log::{error, info, warn};

use crate::data::block_builder::{BlockBuilder, BlockBuilderConfig};

pub fn generate(input: PathBuf, output: PathBuf, safe: bool, debug: bool) -> Result<()> {
    build_asset_files(&input, &output, safe)?;

    let mut block_builder = BlockBuilder::new(BlockBuilderConfig {
        input_dir: input,
        output_dir: output.to_owned(),
        indent_string: "    ",
        debug,
    })?;

    for (block_name, _) in block_builder.block_items.clone() {
        info!("Building block: {}", block_name.cyan().bold());

        let block_name = block_name.to_string();

        let block_file = output.join(format!("{}.html", block_name));

        if block_file.exists() {
            if safe {
                error!(
                    "{}",
                    format!(
                        "Block file {} already exists! Ignoring it because safe mode is on.",
                        block_name.cyan().bold()
                    )
                    .red()
                );
                continue;
            } else {
                warn!(
                    "{}",
                    format!(
                        "Block file {} already exists! File will be overwritten...",
                        block_name.yellow().bold()
                    )
                );
            }
        }

        let mut block_file = File::create(output.join(block_name.to_owned() + ".html"))?;

        block_file.write_all(
            block_builder
                .construct_by_name(block_name.as_str())?
                .as_bytes(),
        )?;
    }

    let generated_style_file = output.join("generated_style.css");

    if safe && generated_style_file.exists() {
        error!(
            "Generated style file {} already exists! Ignoring it because safe mode is on.",
            generated_style_file.to_string_lossy().red().bold()
        );
    } else {
        warn!(
            "Generated style file {} already exists! File will be overwritten...",
            generated_style_file.to_string_lossy().cyan().bold()
        );

        let mut generated_style_file = File::create(generated_style_file)?;

        generated_style_file.write_all(block_builder.get_generated_styles().as_bytes())?;
    }

    info!("{}", "Done!".green().bold());
    Ok(())
}

fn build_asset_files(input: &Path, output: &Path, safe: bool) -> Result<()> {
    let input_files = read_dir(input)?;

    if !output.exists() {
        std::fs::create_dir_all(&output)?;
    }

    let output_files = read_dir(output)?;

    if output_files.count() != 0 {
        if safe {
            return Err(color_eyre::eyre::eyre!(
                "{}",
                "Output directory is not empty! Aborting because safe mode is on.".red()
            ));
        } else {
            warn!(
                "Output directory {} is not empty! Files will be overwritten...",
                output.to_string_lossy().yellow().bold()
            );
        }
    }
    for file in input_files {
        let file = file.expect("Failed to read file");
        let file_name = file.file_name();
        let file_name = file_name.to_str().unwrap();

        if file_name.ends_with(".md") {
            generate_html_from_md(&file, file_name, output, safe);
        } else if file_name.ends_with(".yml") {
            // we don't need to do anything with the block definitions
        } else if file.path().is_dir() {
            let mut new_input = input.to_owned();
            new_input.push(file_name);
            let mut new_output = output.to_owned();
            new_output.push(file_name);
            build_asset_files(&new_input, &new_output, safe)?;
        } else {
            println!("Copying file {}", file_name);
            std::fs::copy(file.path(), output.join(file_name))?;
        }
    }

    Ok(())
}

pub fn generate_html_from_md(file: &DirEntry, file_name: &str, output: &Path, safe: bool) {
    let output_filename = file_name.replace(".md", ".html");
    let output_file = output.join(output_filename.clone());

    if output_file.exists() {
        if safe {
            panic!(
                "Output file {} already exists! Aborting because safe mode is on.",
                output_file.to_string_lossy().red()
            );
        } else {
            info!(
                "Output file {} already exists! File will be overwritten...",
                output_filename.yellow().bold()
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
