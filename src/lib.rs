//! Json file parser library
//!
//! # Installation
//! ```toml
//! ...
//! [dependencies]
//! rsjson = "0.6.0";
//! ```
//! or run
//! ```bash
//! cargo add rsjson
//! ```
//!
//! # Importation
//! ```rust
//! use rsjson;
//! ```
//!
//! # Code example
//! - read and parse a json file
//! ```rust
//! let json: Result<rsjson::Json, String> = rsjson::Json::fromFile("/path/to/file.json");
//! ```
//!
//! - read and parse a json structure from a string
//! - the string can be both "normal" and raw
//! ```rust
//! let json: Result<rsjson::Json, String> = rsjson::json!(
//!     r#"{
//!         "key" : "value",
//!         "second_key" : ["one", "two"]
//!     }"#
//! );
//! ```
//! - in both previous cases, remeber to handle the eventual error (e.g. using `match`) or to call `unwrap()`
//!
//!
//! - create an empty json instance
//! ```rust
//! let json = rsjson::Json::new();
//! ```
//!
//! - add a node
//! ```rust
//! json.addNode(
//!     rsjson::Node::new(
//!         "nodeLabel",
//!         rsjson::NodeContent::Int(32)
//!     )
//! );
//! ```
//!
//! - edit a node's content
//! ```rust
//! json.setContent(
//!     "nodeLabel",
//!     rsjson::NodeContent::Bool(true)
//! );
//! ```
//!
//! - remove a node
//! ```rust
//! json.remove(
//!     "nodeLabel"
//! );
//! ```
//!
//! - check the existance of a label
//! ```rust
//! let exists: bool = json.has("nodeLabel");
//! ```

#![allow(non_snake_case, unused_assignments)]

use std::{fs, path};
use std::collections::HashSet;

const DIGITS: [&str; 11] = [
    "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "."
];

#[derive(Debug, PartialEq)]
enum Token {
    String(String),
    Int(usize),
    Float(f32),
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
    Colon,
    Comma,
    Bool(bool),
    Null
}

impl Token {
    fn toString(&self) -> String {
        match self {
            Token::String(string) => string.clone(),
            _ => String::new()
        }
    }
}

struct Parser {
    tokens: Vec<Token>,
    index: usize,
    text: String ,
    len: usize
}

impl Parser {
    fn new(text: String) -> Parser {
        return Parser {
            tokens: Vec::<Token>::new(),
            index: 0_usize,
            len: (&text).len(),
            text: text
        }
    }

    fn get(&mut self) -> String {
        match self.text.get(self.index..self.index+1) {
            Some(c) => c.to_string(),
            None => {
                panic!("Non utf8 character found, which is not accepted")
            }
        }
    }

    fn checkNotEnd(&self) -> bool {
        self.index != self.len
    }

    fn parse(&mut self) -> bool {
        self.skipNull();
        while self.checkNotEnd() {
            let mut current = self.get();

            if current == "\"" {
                self.index += 1;

                let mut value = String::new();

                while self.checkNotEnd() {
                    current = self.get();

                    if current.as_str() == "\"" && (&self.text[self.index-1..self.index] != "\\") {
                        break

                    } else if current.as_str() == "\"" && (&self.text[self.index-1..self.index] == "\\" && &self.text[self.index-2..self.index-1] == "\\") {
                        break
                    }

                    value += current.as_str();
                    self.index += 1;
                }

                if ! self.checkNotEnd() {
                    return true;
                }
                self.index += 1;

                self.tokens.push(Token::String(value));

            } else if self.get() == ":" {
                self.tokens.push(Token::Colon);
                self.index += 1;

            } else if self.get() == "," {
                self.tokens.push(Token::Comma);
                self.index += 1;

            } else if self.get() == "{" {
                self.tokens.push(Token::OpenBrace);
                self.index += 1;

            } else if self.get() == "}" {
                self.tokens.push(Token::CloseBrace);
                self.index += 1;

            } else if self.get() == "[" {
                self.tokens.push(Token::OpenBracket);
                self.index += 1;

            } else if self.get() == "]" {
                self.tokens.push(Token::CloseBracket);
                self.index += 1;

            } else if DIGITS.contains(&self.get().as_str()) {
                let mut value = String::new();

                while self.checkNotEnd() && DIGITS.contains(&self.get().as_str()) {
                    value += self.get().as_str();
                    self.index += 1;
                }

                if ! self.checkNotEnd() {
                    return true;
                }

                if value.contains(".") {
                    self.tokens.push(Token::Float(value.parse::<f32>().unwrap()))

                } else {
                    self.tokens.push(Token::Int(value.parse::<usize>().unwrap()))
                }

            } else if self.get() == "t" || self.get() == "f" || self.get() == "n" {
                if self.len - self.index - 4 > 0 && &self.text[self.index..self.index + 4] == "true" {
                    self.tokens.push(Token::Bool(true));
                    self.index += 4;

                } else if self.len - self.index - 4 > 0 && &self.text[self.index..self.index + 4] == "null" {
                    self.tokens.push(Token::Null);
                    self.index += 4;

                } else if self.len - self.index - 5 > 0 && &self.text[self.index..self.index + 5] == "false" {
                    self.tokens.push(Token::Bool(false));
                    self.index += 5;

                } else {
                    return true
                }
            }
            self.skipNull();
        }

        false
    }

    fn skipNull(&mut self) {
        let skip = [" ", "\t", "\n"];

        while self.index < self.len && skip.contains(&&self.text[self.index..self.index + 1]) {
            self.index += 1;
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeContent {
    String(String),
    Int(usize),
    Float(f32),
    Bool(bool),
    List(Vec<NodeContent>),
    Json(Json),
    Null
}

impl NodeContent {
    pub fn toString(&self) -> Option<String> {
        match self {
            NodeContent::String(value) => Some(value.to_string().replace("\"", "\\\"")),
            _ => None
        }
    }

    pub fn toUsize(&self) -> Option<usize> {
        match self {
            NodeContent::Int(value) => Some(value.to_owned()),
            _ => None
        }
    }

    pub fn toBool(&self) -> Option<bool> {
        match self {
            NodeContent::Bool(value) => Some(value.to_owned()),
            _ => None
        }
    }

    pub fn toFloat(&self) -> Option<f32> {
        match self {
            NodeContent::Float(value) => Some(value.to_owned()),
            _ => None
        }
    }

    pub fn toJson(&self) -> Option<Json> {
        match self {
            NodeContent::Json(value) => Some(value.clone()),
            _ => None
        }
    }

    pub fn toList(&self) -> Option<Vec<NodeContent>> {
        match self {
            NodeContent::List(value) => Some(value.clone()),
            _ => None
        }
    }

    pub fn toNull(&self) -> Option<Node> {
        return None;
    }
}

/// Node struct represents a pair of label (String) and content (rsjson::NodeContent)
/// implements a getter for both label and content
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    label: String,
    content: NodeContent
}

impl Node {
    pub fn new<T: ToString>(label: T, content: NodeContent) -> Node {
        Node {
            label: label.to_string().replace("\"", "\\\""),
            content: content
        }
    }

    pub fn getLabel(&self) -> String {
        return self.label.clone();
    }

    pub fn getContent(&self) -> NodeContent {
        return self.content.clone();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Json {
    nodes: Vec<Node>,
    labels: HashSet<String>
}

impl Json {
    pub fn new() -> Json {
        return Json {
            nodes: Vec::<Node>::new(),
            labels: HashSet::<String>::new()
        }
    }

    /// Reads the file at `filePath` and returns a Json struct corresponding to its content
    pub fn fromFile<T: ToString>(filePath: T) -> Result<Json, String> {
        match std::fs::read_to_string(filePath.to_string()) {
            Err(why) => Err(format!("Failed because: {why}")),
            Ok(content) => Json::fromString(content)
        }
    }

    pub fn fromString<T: ToString>(text: T) -> Result<Json, String> {
        let mut parser = Parser::new(text.to_string());
        let error = parser.parse();

        if error {
            return Err(String::from("Json format error"));
        }

        let tokens = parser.tokens;

        if tokens.get(0).unwrap() != &Token::OpenBrace {
            return Err(String::from("Json format error: missing opening curly bracket"));
        }

        let index = 1_usize;

        let (_, json, error) = Self::json(&tokens, index);
        if error {
            return Err(String::from("Json format error"));
        }

        return Ok(json.unwrap())
    }

    fn json(tokens: &Vec<Token>, startIndex: usize) -> (usize, Option<Json>, bool) {
        let mut index = startIndex;
        let mut nodes = Vec::<Node>::new();
        let mut labels = HashSet::<String>::new();

        while index < tokens.len() {
            match tokens.get(index).unwrap() {
                Token::String(_) => {
                    let (newIndex, node, error) = Self::node(&tokens, index);

                    if error {
                        return (index, None, true)
                    }

                    index = newIndex;
                    if tokens.get(index).unwrap() != &Token::CloseBrace && tokens.get(index).unwrap() != &Token::Comma {
                        return (index, None, true)

                    } else if tokens.get(index).unwrap() == &Token::Comma {
                        index += 1;
                    }

                    match node {
                        Some(node) => {
                            labels.insert(node.label.clone());
                            nodes.push(node);
                        },
                        None => {}
                    }
                },
                Token::CloseBrace => {
                    break
                }
                _ => return (index, None, true)
            }
        }
        (index, Some(Json{nodes: nodes, labels}), false)
    }

    fn list(tokens: &Vec<Token>, startIndex: usize) -> (usize, Option<NodeContent>, bool) {
        let mut index = startIndex;
        let mut content = Vec::<NodeContent>::new();

        while tokens.get(index).unwrap() != &Token::CloseBracket {
            match tokens.get(index).unwrap() {
                Token::String(string) => {
                    content.push(NodeContent::String(string.to_owned()));
                    index += 1;
                },

                Token::Int(int) => {
                    content.push(NodeContent::Int(int.to_owned()));
                    index += 1;
                },

                Token::Float(float) => {
                    content.push(NodeContent::Float(float.to_owned()));
                    index += 1;
                },

                Token::Null => {
                    content.push(NodeContent::Null);
                    index += 1;
                },

                Token::Bool(bool) => {
                    content.push(NodeContent::Bool(bool.to_owned()));
                    index += 1;
                },

                Token::OpenBrace => {
                    let (newIndex, json, error) = Self::json(tokens, index + 1);

                    if error {
                        return (index, None, true)
                    }

                    index = newIndex + 1;
                    content.push(NodeContent::Json(json.unwrap()));
                },

                Token::OpenBracket => {
                    let (newIndex, list, error) = Self::list(tokens, index);

                    if error {
                        return (index, None, true)
                    }

                    index = newIndex;
                    content.push(list.unwrap())
                },

                Token::Comma => {
                    index += 1;
                },

                _ => {
                    return (index, None, true)
                }


            }
        }
        if tokens.get(index-1).unwrap() == &Token::Comma {
            return (index, None, true);
        }

        (index, Some(NodeContent::List(content)), false)
    }

    fn node(tokens: &Vec<Token>, startIndex: usize) -> (usize, Option<Node>, bool) {
        let mut index = startIndex;
        let label = tokens.get(index).unwrap().toString();

        index += 1;
        if tokens.get(index).unwrap() != &Token::Colon {
            return (index, None, true)
        }
        index += 1;

        let mut content = NodeContent::Null;
        match tokens.get(index).unwrap() {
            Token::Null => {
                content = NodeContent::Null;
                index += 1;
            },

            Token::Int(int) => {
                content = NodeContent::Int(int.to_owned());
                index += 1;
            },

            Token::Float(float) => {
                content = NodeContent::Float(float.to_owned());
                index += 1;
            },

            Token::Bool(bool) => {
                content = NodeContent::Bool(bool.to_owned());
                index += 1;
            },

            Token::String(string) => {
                content = NodeContent::String(string.to_owned());
                index += 1;
            },

            Token::OpenBrace => {
                index += 1;
                let (newIndex, nodeContent, error) = Self::json(tokens, index);
                if error {
                    return (index, None, true)
                }
                index = newIndex + 1;
                content = NodeContent::Json(nodeContent.unwrap());
            },

            Token::OpenBracket => {
                index += 1;
                let (newIndex, list, error) = Self::list(tokens, index);

                if error {
                    return (index, None, true);
                }

                index = newIndex + 1;
                content = list.unwrap();
            }

            _ => {
                return (index, None, true)
            }
        }

        (index, Some(Node{label: label, content: content}), false)
    }

    /// Returns a vector containing all nodes in the Json object
    pub fn getAllNodes(&self) -> Vec<Node> {
        return self.nodes.clone();
    }

    /// Returns the content of the requested node
    pub fn get<T: ToString>(&self, label: T) -> Option<&NodeContent> {
        for node in &self.nodes {
            if node.label == label.to_string() {
                return Some(&node.content)
            }
        }

        return None;
    }

    /// Returns the requested node
    pub fn getNode<T: ToString>(&self, label: T) -> Option<&Node> {
        for node in &self.nodes {
            if node.label == label.to_string() {
                return Some(node);
            }
        }
        return None;
    }

    fn renderJson(json: &Json) -> String {
        let mut content = String::from("{");

        for node in &json.nodes {
            let mut label = (&node.label).to_owned();
            label = label.replace("\\", "\\\\").replace("\"", "\\\"");
            
            content = format!("{}\"{}\":{},", content, &node.label, Self::renderContent(&node.content));
        }

        if content.len() > 2 {
            format!("{}{}", content[0..content.len()-1].to_string(), "}")

        } else {
            format!("{}{}", content, "}")
        }
    }

    fn renderList(list: &Vec<NodeContent>, ) -> String {
        let mut content = String::from("[");

        for node in list {
            content = format!("{}{},", content, Self::renderContent(&node))
        }

        if content.len() > 1 {
            format!("{}{}", content[0..content.len()-1].to_string(), "]")
        } else {
            String::from("[]")
        }
    }

    pub fn renderContent(object: &NodeContent) -> String {
        match object {
            NodeContent::Bool(bool) => if *bool { String::from("true") } else { String::from("false") },
            NodeContent::Float(float) => format!("{}", float),
            NodeContent::Int(int) => format!("{}", int),
            NodeContent::Null => String::from("null"),
            NodeContent::String(string) => format!("\"{}\"", string.replace("\\", "\\\\").replace("\"", "\\\"")),
            NodeContent::List(list) => Self::renderList(&list),
            NodeContent::Json(json) => Self::renderJson(&json),
        }
    }

    /// Exports the Json struct into a Json file and writes it into `fileName`
    pub fn writeToFile<T: ToString>(&self, fileName: T) -> bool {
        let content = Json::renderJson(self);

        return match fs::write(path::Path::new(&fileName.to_string()), content) {
            Err(_) => false,
            Ok(_) => true
        }
    }

    /// Exports the Json struct into a json-formatted string
    pub fn toString(&self) -> String {
        return Json::renderJson(self);
    }

    /// Adds a node to the Json struct
    pub fn addNode(&mut self, node: Node) {
        self.nodes.push(node);
    }

    /// Changes the content of a node, returns a bool representing the status of the change
    pub fn setContent<T: ToString>(&mut self, label: T, content: NodeContent) -> bool {
        for node in &mut self.nodes {
            if node.label == label.to_string() {

                node.content = content;
                return true;
            }
        }

        return false;
    }

    /// Removes a node basing on its label
    pub fn remove<T: ToString>(&mut self, label: T) -> bool {
        let mut index: usize = 0;

        for node in &self.nodes {
            if node.label == label.to_string() {
                self.nodes.remove(index);

                return true;
            }
            index += 1;
        }
        return false;
    }

    /// Converts json to bytes
    pub fn bytes(&self) -> Vec<u8> {
        Json::renderJson(self).bytes().collect::<Vec<u8>>()
    }

    /// Checks if exists a node with provided label
    pub fn has<T: ToString>(&self, label: T) -> bool{
        self.labels.contains(&label.to_string())
    }
}

#[macro_export]
macro_rules! json {
    ( $string:expr ) => {
        Json::fromString($string)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let content = std::fs::read_to_string("./map.json");
        let j = Json::fromString(content.unwrap()).unwrap();

        println!("{:?}", j);
        return;
    }
}
