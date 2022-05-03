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
    current_loop_value: String,

    generated_styles: HashMap<String, HashMap<String, String>>,
}

impl<'a> BlockBuilder<'a> {
    pub fn new(config: BlockBuilderConfig<'a>) -> Self {
        Self {
            block_items: Self::get_block_definitions(&config.input_dir, &config.input_dir).unwrap(),
            config,
            indent_level: 0,
            generated_styles: HashMap::new(),
            current_file: String::new(),
            current_loop_value: String::new(),
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
            BlockItem::IncludeVerbose { path, params } => {
                output.push_str(&self.get_indent());
                output.push_str(self.include_verbose(path, params)?.as_str());
            }
            BlockItem::ForEach { values, pattern, items } => {
                output.push_str(&self.get_indent());

                if values.is_some() && pattern.is_some() {
                    return Err(ParseError {
                        file: self.current_file.clone(),
                        message: "ForEach block cannot have both values and pattern".to_string(),
                    });
                }

                match values {
                    Some(what) => {
                        output.push_str(self.for_each(what, items)?.as_str());
                    },
                    None => (),
                }

                match pattern {
                    Some(what) => { 
                        output.push_str(self.for_each_file(what, items)?.as_str());
                    },
                    None => (),
                }
            }
            BlockItem::LoopValue => output.push_str(&self.loop_value()?.as_str()),
            BlockItem::LoopValueFileName => output.push_str(&self.loop_value_filename()?.as_str()),
        }

        output.push('\n');

        Ok(output)
    }

    fn get_block_definitions(input: &Path, base_input: &Path) -> Result<HashMap<String, BlockItem>, Error> {
        let mut definitions = HashMap::new();

        let dir = input.exists() && input.is_dir();

        if dir {
            for entry in std::fs::read_dir(input)? {
                let entry = entry?;
                let path = entry.path();
                let path_relative_to_input = path.clone();
                let path_relative_to_input = path_relative_to_input.strip_prefix(base_input).unwrap();
                let path_str = path.to_str().unwrap();

                if path.is_dir() {
                    let mut block_items = Self::get_block_definitions(&path, &base_input)?;

                    for (name, item) in block_items.drain() {
                        let block_name = format!("{}/{}", path_relative_to_input.to_str().unwrap(), name);
                        definitions.insert(block_name, item);
                    }
                } else if path.is_file() {
                    let ext = path.extension().unwrap().to_str().unwrap();

                    if ext == "yml" {
                        let mut file = std::fs::File::open(path.clone())?;
                        let mut contents = String::new();
                        file.read_to_string(&mut contents)?;

                        let item: BlockItem = match serde_yaml::from_str(&contents) {
                            Ok(what) => what,
                            Err(why) => return Err(Error::new(io::ErrorKind::Other, why)),
                        };
                        definitions
                            .insert(path.file_stem().unwrap().to_str().unwrap().into(), item);
                    }
                }
            }
        }

        Ok(definitions)
    }

    fn include(&mut self, included_block_name: &str) -> Result<String, ParseError> {
        if self.block_items.get(included_block_name).is_some() {
            let old_file = self.current_file.clone();
            self.current_file = included_block_name.to_string();
            let output = self.construct_by_name(included_block_name).unwrap();
            self.current_file = old_file.to_string();
            Ok(output)
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

    fn include_verbose(
        &mut self,
        included_block_name: &str,
        params: &Option<Vec<String>>,
    ) -> Result<String, ParseError> {
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
        items: &[BlockItem],
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

        output.push('\n');
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
        Ok(format!("{}", text))
    }

    fn link(
        &mut self,
        text: &String,
        url: &String,
        link_style: &LinkStyle,
    ) -> Result<String, ParseError> {
        match link_style {
            LinkStyle::Explicit {
                color,
                underline,
                visited_color,
            } => {
                let mut normal_style: HashMap<String, String> = HashMap::new();
                let mut hover_style: HashMap<String, String> = HashMap::new();

                normal_style.insert("color".to_string(), color.normal.to_string());
                hover_style.insert("color".to_string(), color.hover.to_string());

                if underline.to_owned() {
                    normal_style.insert("text-decoration".to_string(), "underline".to_string());
                }

                hover_style.insert("text-decoration".to_string(), "underline".to_string());

                let mut visited_style = normal_style.clone();

                match visited_color {
                    Some(what) => {
                        visited_style.insert("color".to_string(), what.normal.to_string());
                    }
                    None => {}
                };

                let class = format!("link-{}-{}", color.normal, underline);

                self.generated_styles
                    .insert(format!("{}:link", class.clone()), normal_style);
                self.generated_styles
                    .insert(format!("{}:visited", class.clone()), visited_style);
                self.generated_styles
                    .insert(format!("{}:hover", class.clone()), hover_style.clone());
                self.generated_styles
                    .insert(format!("{}:active", class.clone()), hover_style);

                Ok(format!(
                    "<a href=\"{}\" class=\"{}\">{}</a>",
                    url, class, text
                ))
            }
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

    fn for_each(&mut self, values: &[String], items: &[BlockItem]) -> Result<String, ParseError> {
        let mut output = String::new();

        for value in values {
            self.current_loop_value = value.clone();
            output.push_str(&self.get_indent());

            for item in items {
                output.push_str(&self.construct_block(item)?);
            }

            output.push_str(&self.get_indent());
        }

        Ok(output)
    }

    fn for_each_file(&mut self, pattern: &str, items: &[BlockItem]) -> Result<String, ParseError> {
        let mut output = String::new();

        let options = glob::MatchOptions {
            case_sensitive: false,
            require_literal_separator: false,
            require_literal_leading_dot: false,
        };

        let pattern = self.config.input_dir.to_str().unwrap().to_string() + "/" + &pattern.to_string();

        let files = glob::glob_with(&pattern, options).unwrap();

        for entry in files {
            let entry = entry.unwrap();
            let file_name = entry.file_name().unwrap().to_str().unwrap();

            self.current_file = file_name.to_owned();
            self.current_loop_value = file_name.to_owned();

            output.push_str(&self.get_indent());

            for item in items {
                output.push_str(&self.construct_block(item)?);
            }

            output.push_str(&self.get_indent());
        }

        Ok(output)
    }   

    fn loop_value(&self) -> Result<String, ParseError> {
        Ok(self.current_loop_value.clone())
    }

    fn loop_value_filename(&self) -> Result<String, ParseError> {
        let output = self.current_loop_value.clone();

        let output = Path::new(&output).file_stem().unwrap().to_str().unwrap().to_string();

        Ok(output)
    }
}
