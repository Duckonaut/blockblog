use std::{
    collections::HashMap,
    io::{self, Error, Read},
    path::{Path, PathBuf},
};

use super::{
    blocks::{BlockItem, LinkStyle},
    error::ParseError,
};

pub struct BlockBuilderConfig<'a> {
    pub input_dir: PathBuf,
    pub output_dir: PathBuf,
    pub indent_string: &'a str,
}

pub struct BlockBuilder<'a> {
    pub block_items: HashMap<String, BlockItem>,
    pub config: BlockBuilderConfig<'a>,

    indent_level: usize,
    current_file: String,

    generated_styles: HashMap<String, HashMap<String, String>>,
}

impl<'a> BlockBuilder<'a> {
    pub fn new(config: BlockBuilderConfig<'a>) -> Self {
        Self {
            block_items: Self::get_block_definitions(&config.input_dir).unwrap(),
            config,
            indent_level: 0,
            generated_styles: HashMap::new(),
            current_file: String::new(),
        }
    }

    pub fn construct_by_name(&mut self, block_name: &str) -> Result<String, ParseError> {
        let block = {
            let block = self.block_items.get(block_name);
            let block = block.ok_or(ParseError {
                file: block_name.to_string(),
                message: format!("Block {} not found", block_name),
            })?;
            block.clone()
        };

        self.construct_block(&block)
    }

    fn construct_block(&mut self, block: &BlockItem) -> Result<String, ParseError> {
        let mut output = String::new();

        match block {
            BlockItem::Include(name) => {
                output.push_str(self.include(name)?.as_str());
            }
            BlockItem::Title(text) => {
                output.push_str(&self.get_indent());
                output.push_str(self.title(text)?.as_str());
            }
            BlockItem::Block {
                style,
                items,
                html_type,
            } => {
                output.push_str(self.block(style, html_type, items)?.as_str());
            }
            BlockItem::Markdown(md_file) => {
                output.push_str(&self.get_indent());
                output.push_str(self.markdown(md_file)?.as_str());
            }
            BlockItem::Code(code_file) => {
                output.push_str(&self.get_indent());
                output.push_str(self.code(code_file)?.as_str());
            }
            BlockItem::Image { path, alt } => {
                output.push_str(&self.get_indent());
                output.push_str(self.image(path, alt)?.as_str());
            }
            BlockItem::Text(raw_text) => {
                output.push_str(&self.get_indent());
                output.push_str(self.text(raw_text)?.as_str());
            }
            BlockItem::Link {
                text,
                url,
                link_style,
            } => {
                output.push_str(&self.get_indent());
                output.push_str(self.link(text, url, link_style)?.as_str());
            }
            BlockItem::Br => {
                output.push_str(&self.get_indent());
                output.push_str(self.br()?.as_str());
            }
        }

        output.push_str("\n");

        Ok(output)
    }

    fn get_block_definitions(input: &Path) -> Result<HashMap<String, BlockItem>, Error> {
        let mut definitions = HashMap::new();

        let dir = input.exists() && input.is_dir();

        if dir {
            for entry in std::fs::read_dir(input)? {
                let entry = entry?;
                let path = entry.path();
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

        Ok(definitions)
    }

    fn include(&mut self, included_block_name: &str) -> Result<String, ParseError> {
        if self.block_items.get(included_block_name).is_some() {
            Ok(self.construct_by_name(included_block_name).unwrap())
        } else {
            return Err(ParseError {
                file: self.current_file.to_owned(),
                message: format!(
                    "Could not find block definition for {}",
                    included_block_name
                ),
            });
        }
    }

    fn title(&self, title: &String) -> Result<String, ParseError> {
        Ok(format!("<h1>{}</h1>", title))
    }

    fn block(
        &mut self,
        style: &Option<String>,
        html_type: &Option<String>,
        items: &Vec<BlockItem>,
    ) -> Result<String, ParseError> {
        let mut output = String::new();
        let html_type = match html_type {
            Some(what) => what,
            None => "div",
        };

        output.push_str(&self.get_indent());
        match style {
            Some(style) => {
                output.push_str(&format!("<{} class=\"{}\">", html_type, style));
            }
            None => {
                output.push_str(&format!("<{}>", html_type));
            }
        }

        output.push_str("\n");
        self.indent_level += 1;

        for item in items {
            output.push_str(&self.construct_block(item)?);
        }

        self.indent_level -= 1;

        output.push_str(&self.get_indent());
        output.push_str(&format!("</{}>", html_type));

        Ok(output)
    }

    fn markdown(&self, markdown: &str) -> Result<String, ParseError> {
        Ok(markdown::to_html(markdown))
    }

    fn code(&self, code: &String) -> Result<String, ParseError> {
        Ok(format!("<pre><code>\n{}\n</code></pre>", code))
    }

    fn image(&self, image: &String, alt: &Option<String>) -> Result<String, ParseError> {
        let alt = match alt {
            Some(what) => what,
            None => "",
        };

        Ok(format!("<img src=\"{}\" alt=\"{}\" />", image, alt))
    }

    fn text(&self, text: &String) -> Result<String, ParseError> {
        Ok(format!("{}\n", text))
    }

    fn link(
        &self,
        text: &String,
        url: &String,
        link_style: &LinkStyle,
    ) -> Result<String, ParseError> {
        match link_style {
            LinkStyle::Explicit {
                color: _color,
                underline: _underline,
                visited_color: _visited_color,
            } => Ok(format!("<a href=\"{}\">{}</a>", url, text)),
            LinkStyle::Style(style) => Ok(format!(
                "<a href=\"{}\" class=\"{}\">{}</a>",
                url, style, text
            )),
        }
    }

    fn br(&self) -> Result<String, ParseError> {
        Ok("<br />".into())
    }

    fn get_indent(&self) -> String {
        let mut indent = String::new();
        for _ in 0..self.indent_level {
            indent.push_str(self.config.indent_string);
        }
        indent
    }
}
