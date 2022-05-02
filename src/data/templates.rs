use std::{
    collections::HashMap,
    io::{self, Error, Read},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

use super::{colors::LinkColor, error::ParseError};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LinkStyle {
    #[serde(rename = "explicit")]
    Explicit {
        underline: bool,
        color: LinkColor,
        visited_color: Option<LinkColor>,
    },
    #[serde(rename = "style")]
    Style(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BlockItem {
    #[serde(rename = "include")]
    Include(String),
    #[serde(rename = "title")]
    Title(String),
    #[serde(rename = "block")]
    Block {
        #[serde(rename = "style")]
        style: String,
        #[serde(rename = "items")]
        items: Vec<BlockItem>,
    },
    #[serde(rename = "markdown")]
    Markdown(String),
    #[serde(rename = "code")]
    Code(String),
    #[serde(rename = "image")]
    Image(String),
    #[serde(rename = "text")]
    Text(String),
    #[serde(rename = "link")]
    Link {
        #[serde(rename = "text")]
        text: String,
        #[serde(rename = "url")]
        url: String,
        #[serde(rename = "link_style")]
        link_style: LinkStyle,
    },
    #[serde(rename = "br")]
    Br,
}

pub fn get_block_definitions(input: &PathBuf) -> Result<HashMap<String, BlockItem>, Error> {
    let mut definitions = HashMap::new();

    let dir = input.exists() && input.is_dir();

    if dir {
        for entry in std::fs::read_dir(input)? {
            let entry = entry?;
            let path = entry.path();
            let name = path.file_name().unwrap().to_str().unwrap();
            let ext = path.extension().unwrap().to_str().unwrap();

            if ext == "yml" {
                let mut file = std::fs::File::open(path.clone())?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;

                let item: BlockItem = match serde_yaml::from_str(&contents) {
                    Ok(what) => what,
                    Err(why) => return Err(Error::new(io::ErrorKind::Other, why)),
                };
                definitions.insert(path.file_stem().unwrap().to_str().unwrap().into(), item);
            }
        }
    }

    return Ok(definitions);
}

pub fn construct_from_block(
    block_name: &str,
    root_template: &BlockItem,
    block_items: &HashMap<String, BlockItem>,
) -> Result<String, ParseError> {
    let mut output = String::new();

    match root_template {
        BlockItem::Include(name) => {
            if let Some(item) = block_items.get(name) {
                output.push_str(&construct_from_block(block_name, item, block_items)?);
            } else {
                return Err(ParseError {
                    file: block_name.to_owned(),
                    message: format!("Could not find block definition for {}", name),
                });
            }
        }
        BlockItem::Title(title) => {
            output.push_str(&format!("# {}\n", title));
        }
        BlockItem::Block { style, items } => {
            for item in items {
                output.push_str(&construct_from_block(block_name, item, block_items)?);
            }
        }
        BlockItem::Markdown(markdown) => {
            output.push_str(&markdown);
        }
        BlockItem::Code(code) => {
            output.push_str(&format!("```\n{}\n```\n", code));
        }
        BlockItem::Image(image) => {
            output.push_str(&format!("![]({})\n", image));
        }
        BlockItem::Text(text) => {
            output.push_str(&text);
        }
        BlockItem::Link {
            text,
            url,
            link_style,
        } => {
            output.push_str(&format!(
                "[{}]({}){}",
                text,
                url,
                match link_style {
                    LinkStyle::Explicit {
                        color,
                        underline,
                        visited_color,
                    } => {
                        let underline = if underline.to_owned() {
                            "underline"
                        } else {
                            ""
                        };
                        format!(
                            "\n\n[{}]: <{}, {}> {}",
                            text, color.normal, color.hover, underline
                        )
                    }
                    LinkStyle::Style(style) => format!("\n\n[{}]: {}", text, style),
                }
            ));
        }
        BlockItem::Br => {
            output.push_str("\n");
        }
    }

    return Ok(output);
}
