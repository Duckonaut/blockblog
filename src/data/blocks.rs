use serde::{Deserialize, Serialize};

use super::colors::LinkColor;

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
    #[serde(rename = "include")]
    IncludeVerbose { 
        path: String,
        params: Option<Vec<String>>,
    },
    #[serde(rename = "title")]
    Title(String),
    #[serde(rename = "block")]
    Block {
        #[serde(rename = "style")]
        style: Option<String>,
        #[serde(rename = "html_type")]
        html_type: Option<String>,
        #[serde(rename = "items")]
        items: Vec<BlockItem>,
    },
    #[serde(rename = "markdown")]
    Markdown(String),
    #[serde(rename = "code")]
    Code(String),
    #[serde(rename = "image")]
    Image {
        #[serde(rename = "path")]
        path: String,
        #[serde(rename = "alt")]
        alt: Option<String>,
    },
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
    #[serde(rename = "$for_each")]
    ForEach {
        #[serde(rename = "pattern")]
        pattern: Option<String>,
        #[serde(rename = "values")]
        values: Option<Vec<String>>,
        #[serde(rename = "items")]
        items: Vec<BlockItem>,
    },
    #[serde(rename = "$loop_value")]
    LoopValue,
    #[serde(rename = "$loop_value_filename")]
    LoopValueFileName
}


