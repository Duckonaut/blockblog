use std::{
    collections::HashMap,
    ffi::OsStr,
    io::{self, Error, Read},
    path::{Path, PathBuf},
};

use color_eyre::Result;

use super::blocks::{BlockItem, Head, LinkStyle};

use regex::{Captures, Regex};

pub struct BlockBuilderConfig<'a> {
    pub input_dir: PathBuf,
    pub output_dir: PathBuf,
    pub indent_string: &'a str,
    pub debug: bool,
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

    pub fn construct_by_name(&mut self, block_name: &str) -> Result<String> {
        let block = {
            let block = self.block_items.get(block_name);
            let block = block.ok_or_else(|| {
                Error::new(
                    io::ErrorKind::NotFound,
                    format!("Block {} not found", block_name),
                )
            })?;
            block.clone()
        };

        self.construct_block(&block)
    }

    fn construct_block(&mut self, block: &BlockItem) -> Result<String> {
        let mut output = String::new();

        match block {
            BlockItem::Include(name) => {
                let name = self.process_special_values(name)?;
                output.push_str(self.include(&name)?.as_str());
            }
            BlockItem::Title(text) => {
                let text = self.process_special_values(text)?;
                output.push_str(&self.get_indent());
                output.push_str(self.title(&text)?.as_str());
            }
            BlockItem::Block {
                style,
                items,
                html_type,
            } => {
                output.push_str(self.block(style, html_type, items)?.as_str());
            }
            BlockItem::Markdown(md_file) => {
                let md_file = self.process_special_values(md_file)?;
                output.push_str(&self.get_indent());
                output.push_str(self.markdown(&md_file)?.as_str());
            }
            BlockItem::Code(code_file) => {
                let code_file = self.process_special_values(code_file)?;
                output.push_str(&self.get_indent());
                output.push_str(self.code(&code_file)?.as_str());
            }
            BlockItem::Image { path, alt } => {
                let path = self.process_special_values(path)?;
                output.push_str(&self.get_indent());
                output.push_str(self.image(&path, alt)?.as_str());
            }
            BlockItem::Text(raw_text) => {
                let raw_text = self.process_special_values(raw_text)?;
                output.push_str(&self.get_indent());
                output.push_str(self.text(&raw_text)?.as_str());
            }
            BlockItem::Link {
                text,
                url,
                link_style,
            } => {
                let text = self.process_special_values(text)?;
                output.push_str(&self.get_indent());
                output.push_str(self.link(&text, url, link_style)?.as_str());
            }
            BlockItem::Br => {
                output.push_str(&self.get_indent());
                output.push_str(self.br()?.as_str());
            }
            BlockItem::IncludeVerbose { path, params } => {
                let path = self.process_special_values(path)?;
                output.push_str(&self.get_indent());
                output.push_str(self.include_verbose(&path, params)?.as_str());
            }
            BlockItem::ForEach {
                values,
                pattern,
                items,
            } => {
                if values.is_some() && pattern.is_some() {
                    return Err(Error::new(
                        io::ErrorKind::InvalidInput,
                        "ForEach: values and pattern are both set",
                    )
                    .into());
                }

                match values {
                    Some(what) => {
                        output.push_str(self.for_each(what, items)?.as_str());
                    }
                    None => (),
                }

                match pattern {
                    Some(what) => {
                        output.push_str(self.for_each_file(what, items)?.as_str());
                    }
                    None => (),
                }
            }
            BlockItem::LoopValue => {
                output.push_str(&self.get_indent());
                output.push_str(self.loop_value()?.as_str())
            }
            BlockItem::LoopValueFileName => {
                output.push_str(&self.get_indent());
                output.push_str(self.loop_value_filename()?.as_str())
            }
            BlockItem::Html { head, body } => output.push_str(self.html(head, body)?.as_str()),
        }

        output.push('\n');

        Ok(output)
    }

    fn get_block_definitions(
        input: &Path,
        base_input: &Path,
    ) -> Result<HashMap<String, BlockItem>> {
        let mut definitions = HashMap::new();

        let dir = input.exists() && input.is_dir();

        if dir {
            for entry in std::fs::read_dir(input)? {
                let entry = entry?;
                let path = entry.path();
                let path_relative_to_input = path.clone();
                let path_relative_to_input = path_relative_to_input.strip_prefix(base_input)?;

                if path.is_dir() {
                    let mut block_items = Self::get_block_definitions(&path, base_input)?;

                    for (name, item) in block_items.drain() {
                        let block_name =
                            format!("{}/{}", path_relative_to_input.to_str().unwrap(), name);
                        definitions.insert(block_name, item);
                    }
                } else if path.is_file() {
                    let ext = match path.extension() {
                        Some(e) => e,
                        None => {
                            return Err(Error::new(
                                io::ErrorKind::InvalidInput,
                                "File has no extension",
                            )
                            .into())
                        }
                    }
                    .to_str()
                    .unwrap();

                    if ext == "yml" {
                        let mut file = std::fs::File::open(path.clone())?;
                        let mut contents = String::new();
                        file.read_to_string(&mut contents)?;

                        let item: BlockItem = match serde_yaml::from_str(&contents) {
                            Ok(what) => what,
                            Err(why) => return Err(Error::new(io::ErrorKind::Other, why).into()),
                        };
                        definitions
                            .insert(path.file_stem().unwrap().to_str().unwrap().into(), item);
                    }
                }
            }
        }

        Ok(definitions)
    }

    fn html(&mut self, head: &Option<Head>, body: &Option<Vec<BlockItem>>) -> Result<String> {
        let mut output = String::new();

        output.push_str("<!DOCTYPE html>\n");
        output.push_str("<html>\n");

        self.indent_level += 1;
        output.push_str(&self.get_indent());

        output.push_str("<head>\n");
        self.indent_level += 1;
        output.push_str(&self.get_indent());
        output.push_str("<meta charset=\"utf-8\">\n");

        if let Some(head) = head {
            if let Some(title) = &head.title {
                output.push_str(&self.get_indent());
                output.push_str(&format!("<title>{}</title>\n", title));
            }

            if let Some(icon) = &head.icon {
                output.push_str(&self.get_indent());
                output.push_str(&format!(
                    "<link rel=\"icon\" href=\"{}\" type=\"image/x-icon\" />\n",
                    icon
                ));
            }

            if let Some(styles) = &head.styles {
                for style in styles {
                    output.push_str(&self.get_indent());
                    output.push_str(&format!("<link rel=\"stylesheet\" href=\"{}\" />\n", style));
                }
            }

            if let Some(scripts) = &head.scripts {
                for script in scripts {
                    output.push_str(&self.get_indent());
                    output.push_str(&format!("<script src=\"{}\" />\n", script));
                }
            }
        }

        self.indent_level -= 1;
        output.push_str(&self.get_indent());
        output.push_str("</head>\n");
        output.push_str(&self.get_indent());
        output.push_str("<body>\n");

        self.indent_level += 1;
        if let Some(body) = body {
            for item in body {
                output.push_str(self.construct_block(item)?.as_str());
            }
        }
        self.indent_level -= 1;

        output.push_str(&self.get_indent());
        output.push_str("</body>\n");
        self.indent_level -= 1;

        output.push_str("</html>\n");
        debug_assert_eq!(self.indent_level, 0);
        Ok(output)
    }

    fn include(&mut self, included_block_name: &str) -> Result<String> {
        if self.block_items.get(included_block_name).is_some() {
            let mut output = String::new();

            if self.config.debug {
                output.push_str(&self.get_indent());
                output.push_str(
                    format!("<!-- Including block {} -->\n", included_block_name).as_str(),
                );
            }

            let old_file = self.current_file.clone();
            self.current_file = included_block_name.to_string();

            output.push_str(self.construct_by_name(included_block_name)?.as_str());

            self.current_file = old_file;

            Ok(output)
        } else {
            Err(Error::new(
                io::ErrorKind::NotFound,
                format!("Block {} not found", included_block_name),
            )
            .into())
        }
    }

    fn include_verbose(
        &mut self,
        included_block_name: &str,
        params: &Option<Vec<String>>,
    ) -> Result<String> {
        if self.block_items.get(included_block_name).is_some() {
            let mut output = String::new();

            if self.config.debug {
                output.push_str(&self.get_indent());
                output.push_str(
                    format!("<!-- Including block {} -->\n", included_block_name).as_str(),
                );
            }

            let old_file = self.current_file.clone();
            self.current_file = included_block_name.to_string();

            output.push_str(self.construct_by_name(included_block_name)?.as_str());

            self.current_file = old_file;

            Ok(output)
        } else {
            Err(Error::new(
                io::ErrorKind::NotFound,
                format!("Block {} not found", included_block_name),
            )
            .into())
        }
    }

    fn title(&self, title: &String) -> Result<String> {
        Ok(format!("<h1>{}</h1>", title))
    }

    fn block(
        &mut self,
        style: &Option<String>,
        html_type: &Option<String>,
        items: &[BlockItem],
    ) -> Result<String> {
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

    fn markdown(&self, markdown: &str) -> Result<String> {
        Ok(markdown::to_html(markdown))
    }

    fn code(&self, code: &String) -> Result<String> {
        Ok(format!("<pre><code>\n{}\n</code></pre>", code))
    }

    fn image(&self, image: &String, alt: &Option<String>) -> Result<String> {
        let alt = match alt {
            Some(what) => what,
            None => "",
        };

        Ok(format!("<img src=\"{}\" alt=\"{}\" />", image, alt))
    }

    fn text(&self, text: &String) -> Result<String> {
        Ok(text.to_string())
    }

    fn link(&mut self, text: &String, url: &String, link_style: &LinkStyle) -> Result<String> {
        match link_style {
            LinkStyle::Explicit {
                color,
                underline,
                visited_color,
            } => {
                let mut normal_style: HashMap<String, String> = HashMap::new();
                let mut hover_style: HashMap<String, String> = HashMap::new();

                normal_style.insert("color".to_string(), color.normal.to_string());
                if let Some(hover) = &color.hover {
                    hover_style.insert("color".to_string(), hover.to_string());
                } else {
                    hover_style.insert("color".to_string(), color.normal.to_string());
                }

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

                let class = format!(
                    "link-{}-{}",
                    color.normal.to_string().trim_start_matches('#'),
                    {
                        if *underline {
                            "underline"
                        } else {
                            "none"
                        }
                    }
                );

                self.generated_styles
                    .insert(format!("{}:link", &class), normal_style);
                self.generated_styles
                    .insert(format!("{}:visited", &class), visited_style);
                self.generated_styles
                    .insert(format!("{}:hover", &class), hover_style.clone());
                self.generated_styles
                    .insert(format!("{}:active", &class), hover_style);

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

    fn br(&self) -> Result<String> {
        Ok("<br />".into())
    }

    fn get_indent(&self) -> String {
        let mut indent = String::new();
        for _ in 0..self.indent_level {
            indent.push_str(self.config.indent_string);
        }
        indent
    }

    fn for_each(&mut self, values: &[String], items: &[BlockItem]) -> Result<String> {
        let mut output = String::new();

        for value in values {
            self.current_loop_value = value.clone();

            for item in items {
                output.push_str(&self.construct_block(item)?);
            }
        }

        Ok(output)
    }

    fn for_each_file(&mut self, pattern: &str, items: &[BlockItem]) -> Result<String> {
        let mut output = String::new();

        let options = glob::MatchOptions {
            case_sensitive: false,
            require_literal_separator: false,
            require_literal_leading_dot: false,
        };

        let pattern = self.config.input_dir.to_str().unwrap().to_string() + "/" + pattern;

        let files = glob::glob_with(&pattern, options).unwrap();

        for entry in files {
            let entry = entry.unwrap();
            let file_name = entry.file_name().unwrap().to_str().unwrap();

            self.current_file = file_name.to_owned();
            self.current_loop_value = file_name.to_owned();

            for item in items {
                output.push_str(&self.construct_block(item)?);
            }
        }

        Ok(output)
    }

    fn loop_value(&self) -> Result<String> {
        Ok(self.current_loop_value.clone())
    }

    fn loop_value_filename(&self) -> Result<String> {
        let output = self.current_loop_value.clone();

        let output = Path::new(&output)
            .file_stem()
            .unwrap_or_else(|| OsStr::new(""))
            .to_str()
            .unwrap()
            .to_string();

        Ok(output)
    }

    fn process_special_values(&mut self, value: &str) -> Result<String> {
        let mut s = value.to_string();
        let cached_filename = &self.loop_value_filename().ok();
        let cached_loop_value = &self.loop_value()?;

        let v_regex = Regex::new(r"([^\\]|^)(\$loop_value)([[:^word:]]|$)")?;

        if let Some(filename) = cached_filename {
            let re = Regex::new(r"([^\\]|^)(\$loop_value_filename)([[:^word:]]|$)")?;

            s = re
                .replace_all(&s, |caps: &Captures| {
                    format!("{}{}{}", &caps[1], filename, &caps[3])
                })
                .to_string();

            s = s.replace("\\$loop_value_filename", "$loop_value_filename");
        }

        s = v_regex
            .replace_all(&s, |caps: &Captures| {
                format!("{}{}{}", &caps[1], &cached_loop_value, &caps[3])
            })
            .to_string();

        s = s.replace("\\$loop_value", "$loop_value");
        Ok(s)
    }

    pub fn get_generated_styles(&self) -> String {
        let mut output = String::new();

        for (class, style) in self.generated_styles.iter() {
            output.push_str(&format!("{} {{\n", class));

            for (key, value) in style.iter() {
                output.push_str(&format!("\t{}: {};\n", key, value));
            }

            output.push_str("}\n\n");
        }
        output
    }
}
