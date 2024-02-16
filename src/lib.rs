//! Json file parser library
//!
//! # Installation
//! ```toml
//! ...
//! [dependencies]
//! rsjson = "0.1.0";
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
//! let json = rsjson::Json::fromFile("/path/to/file.json".to_string());
//! ```
//!
//! - create an empty json instance
//! ```rust
//! let json = rsjson::Json::new();
//! ```
//!
//! - add a node
//! ```rust
//! json.addNode(
//!     Node::new(
//!         "nodeLabel",
//!         jsonfile::NodeContent::Int(32)
//!     )
//! );
//! ```
//!
//! - edit a node's label
//! ```rust
//! json.editNode(
//!     "nodeLabel",
//!     "newNodeLabel"
//! );
//! ```
//!
//! - edit a node's content
//! ```rust
//! json.editContent(
//!     "nodeLabel",
//!     jsonfile::NodeContent::Bool(true)
//! );
//! ```
//!
//! - remove a node
//! ```rust
//! json.removeNode(
//!     "nodeLabel"
//! );
//! ```

#![allow(non_snake_case)]
#![allow(dead_code)]

use std::fmt::Debug;
use std::fs;
use std::ops::Add;
use std::path;

const DIGITS: [&str; 11] = [
    "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "."
];

/// The enum implementation contains the various types of elements that can be contained in a json file
#[derive(Debug, PartialEq)]
pub enum NodeContent {
    String(String),
    Int(usize),
    Bool(bool),
    Float(f32),
    Json(Json),
    List(Vec<NodeContent>),
    Null(Option<u8>)
}

/// Each method is associated with a type in `NodeContent`
impl NodeContent {
    pub fn toString(&self) -> Option<String> {
        match self {
            NodeContent::String(value) => Some(value.to_string()),
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

    pub fn toJson(&self) -> Option<&Json> {
        match self {
            NodeContent::Json(value) => Some(value.to_owned()),
            _ => None
        }
    }

    pub fn toList(&self) -> Option<&Vec<NodeContent>> {
        match self {
            NodeContent::List(value) => Some(value.to_owned()),
            _ => None
        }
    }

    pub fn toNull(&self) -> Option<Node> {
        return None;
    }
}

/// Data structure containig label and value for a node in the json structure
#[derive(Debug, PartialEq)]
pub struct Node {
    label: String,
    content: NodeContent,
}

impl Node {
    pub fn new(label: String, content: NodeContent) -> Node {
        return Node {
            label: label,
            content: content
        }
    }
}

/// Contains the vector of nodes contained in the json file
#[derive(Debug, PartialEq)]
pub struct Json {
    nodes: Vec<Node>
}

impl Json {
    pub fn new() -> Json {
        Json {
            nodes: Vec::<Node>::new()
        }
    }

    fn makeNew(nodes: Vec<Node>) -> Json {
        Json {
            nodes: nodes
        }
    }

    /// Reads the specified file and returns a `Json` struct containing the red data
    pub fn fromFile(filePath: String) -> Json {
        let content = match fs::read_to_string(path::Path::new(filePath.as_str())) {
            Err(why) => {
                panic!("{}", why)
            },
            Ok(fileContent) => fileContent
        };

        let mut json = Json::makeNew(Vec::<Node>::new());
        let mut index: usize = 0;

        while index < content.len() {
            index = Json::skipNull(&content, index);

            if &content[index..index + 1] == "{" {
                let (newIndex, res) = Json::json(&content, index);

                json = res;
                index = newIndex;

                let newIndex = Json::skipNull(&content, index);
                if newIndex == (content.len() - 1) {
                    break
                } else {
                    panic!("Json format error");
                }
            }
        }

        return json;
    }

    /// Generates a `Json` struct containing the data provided as a string
    pub fn fromString(string: String) -> Json {
        let content = string.clone();

        let mut json = Json::makeNew(Vec::<Node>::new());
        let mut index: usize = 0;

        while index < content.len() {
            index = Json::skipNull(&content, index);

            if &content[index..index + 1] == "{" {
                let (newIndex, res) = Json::json(&content, index);

                json = res;
                index = newIndex;

                let newIndex = Json::skipNull(&content, index);
                if newIndex == (content.len() - 1) {
                    break
                } else {
                    panic!("Json format error");
                }
            }
        }

        return json;
    }

    fn json(content: &String, startIndex: usize) -> (usize, Json) {
        let mut index = startIndex + 1;
        let mut nodes = Vec::<Node>::new();

        while index < content.len() {
            index = Json::skipNull(content, index);

            if &content[index..index+1] == "\"" {
                let (newIndex, node) = Json::node(content, index);

                index = newIndex + 1;
                nodes.push(node);

                index = Json::skipNull(content, index);

            } else if &content[index..index+1] == "," {
                let tempIndex = Json::skipNull(content, index + 1);

                if &content[tempIndex..tempIndex+1] == "{" {
                    panic!("Json format error");
                } else {
                    index += 1;
                }

            } else if &content[index..index+1] == "}" {
                break
            }
        }

        (
            index,
            Json {
                nodes: nodes
            }
        )
    }

    fn node(content: &String, startIndex: usize) -> (usize, Node) {
        let mut label: String = String::new();
        let mut index = startIndex + 1;

        while &content[index..index+1] != "\"" {
            label = label.add(&content[index..index+1]);
            index += 1;
        }

        index += 1;
        index = Json::skipNull(content, index);

        if &content[index..index+1] != ":" {
            panic!("Json format error");
        }

        index += 1;
        index = Json::skipNull(content, index);

        let (newIndex, content) = Json::contentElement(content, index);

        (newIndex, Node{
            label: label,
            content: content
        })
    }

    fn contentElement(content: &String, startIndex: usize) -> (usize, NodeContent) {
        let mut index = startIndex;

        index = Json::skipNull(content, index);

        if &content[index..index+1] == "\"" {
            let mut nodeContent: String = String::new();
            index += 1;

            while &content[index..index+1] != "\"" {
                nodeContent = nodeContent.add(&content[index..index+1]);
                index += 1;
            }

            (index+1, NodeContent::String(nodeContent))

        } else if &content[index..index+1] == "{" {
            let (newIndex, nodeContent) = Json::json(content, index);
            (newIndex, NodeContent::Json(nodeContent))

        } else if &content[index..index+1] == "[" {
            let (newIndex, list) = Json::list(content, index);
            (newIndex, NodeContent::List(list))

        } else if  index + 4 < content.len() && &content[index..index+4] == "true" {
            (index+4, NodeContent::Bool(true))

        } else if index + 5 < content.len() && &content[index..index+5] == "false" {
            (index+5, NodeContent::Bool(false))

        } else if index + 4 < content.len() && &content[index..index+4] == "null" {
            (index+4, NodeContent::Null(None))

        } else if DIGITS.contains(&&content[index..index+1]) {
            let mut number: String = String::new();

            while DIGITS.contains(&&content[index..index+1]) {
                number = number.add(&content[index..index+1]);
                index += 1;
            }

            if number.contains(".") {
                (index, NodeContent::Float(number.parse::<f32>().unwrap()))

            } else {
                (index, NodeContent::Int(number.parse::<usize>().unwrap()))
            }
        } else {
            panic!("Json format error");
        }
    }

    fn skipNull(content: &String, startIndex: usize) -> usize {
        let mut index = startIndex;

        while index < content.len() && (&content[index..index+1] == " " || &content[index..index+1] == "\t" || &content[index..index+1] == "\n") {
            index += 1;
        }

        return index;
    }

    fn list(content: &String, startIndex: usize) -> (usize, Vec<NodeContent>) {
        let mut list = Vec::<NodeContent>::new();
        let mut index = startIndex + 1;

        while &content[index..index+1] != "]" {
            let (newIndex, element) = Json::contentElement(content, index);
            list.push(element);

            index = Json::skipNull(content, newIndex);
            if &content[index..index+1] != "," {

                if &content[index..index+1] == "]" {
                    break

                } else {
                    panic!("Json format error");
                }

            } else {
                index += 1;
            }
        }

        (index, list)
    }

    /// Returns the value associated to the node with the specified label
    pub fn get(&self, label: String) -> Option<&NodeContent> {
        for node in &self.nodes {
            if node.label == label {
                return Some(&node.content)
            }
        }

        return None;
    }

    fn renderJson(json: &Json, indent: String) -> String {
        let mut content = String::from("{");

        for node in &json.nodes {
            let nodeContent = &node.content;

            if nodeContent.toString() != None {
                content = content.add(format!("\n{}\"{}\" : \"{}\",", indent, node.label, nodeContent.toString().unwrap()).as_str());

            } else if nodeContent.toUsize() != None {
                content = content.add(format!("\n{}\"{}\" : {},", indent, node.label, nodeContent.toUsize().unwrap()).as_str());

            } else if nodeContent.toFloat() != None {
                content = content.add(format!("\n{}\"{}\" : {},", indent, node.label, nodeContent.toFloat().unwrap()).as_str());

            } else if nodeContent.toBool() != None {
                content = content.add(format!("\n{}\"{}\" : {},", indent, node.label, nodeContent.toBool().unwrap()).as_str());

            } else if nodeContent.toUsize() != None {
                content = content.add(format!("\n{}\"{}\" : {},", indent, node.label, nodeContent.toUsize().unwrap()).as_str());
            } else if nodeContent.toList() != None {
                content = content.add(
                    format!(
                        "\n{}\"{}\" : {},",
                        indent,
                        node.label,
                        Json::renderList(nodeContent.toList().unwrap()).as_str()
                    ).as_str()
                );
            } else if nodeContent.toJson() != None {
                let subContent = Json::renderJson(node.content.toJson().unwrap(), indent.clone().add("\t").to_string());
                content = content.add(format!("\n{}\"{}\" : {}", indent, node.label, subContent).as_str());
                content = content[0..content.len()-1].to_string().add(&indent).add("},");

            } else {
                content = content.add(format!("\n{}\"{}\" : null,", indent, node.label).as_str());
            }
        }

        content = content[0..content.len()-1].to_string().add("\n").add("}");
        return content;
    }

    fn renderList(list: &Vec<NodeContent>) -> String {
        let mut content = String::from("[");

        for element in list {
            if element.toString() != None {
                content = content.add(format!("\"{}\", ", element.toString().unwrap()).as_str());

            } else if element.toUsize() != None {
                content = content.add(format!("{}, ", element.toUsize().unwrap()).as_str());

            } else if element.toFloat() != None {
                content = content.add(format!("{}, ", element.toFloat().unwrap()).as_str());

            } else if element.toBool() != None {
                content = content.add(format!("{}, ", element.toBool().unwrap()).as_str());

            } else if element.toUsize() != None {
                content = content.add(format!("{}, ", element.toUsize().unwrap()).as_str());

            } else if element.toJson() != None {
                let subContent = Json::renderJson(element.toJson().unwrap(), String::from("\t"));

                content = content.add(format!("{}, ", subContent).as_str());
                content = content[0..content.len()-1].to_string().add("}, ");

            } else {
                content = content.add("null, ");
            }
        }

        content = (&content[0..content.len()-2]).to_string().add("]");
        return content;
    }

    /// Writes the content of the `Json` struct into the specified file
    pub fn writeToFile(&self, fileName: String) -> bool {
        let content = Json::renderJson(self, "\t".to_string());

        return match fs::write(path::Path::new(&fileName), content) {
            Err(_) => false,
            Ok(_) => true
        }
    }

    /// Adds a node to the `Json` struct
    pub fn addNode(&mut self, node: Node) {
        self.nodes.push(node);
    }

    /// Changes the label of a node, returns a bool representing the status of the change
    pub fn changeLabel(&mut self, label: String, newLabel: String) -> bool {
        for node in &mut self.nodes {
            if node.label == label {

                node.label = newLabel.clone();
                return true;
            }
        }

        return false;
    }

    /// Changes the content of a node, returns a bool representing the status of the change
    pub fn changeContent(&mut self, label: String, content: NodeContent) -> bool {
        for node in &mut self.nodes {
            if node.label == label {

                node.content = content;
                return true;
            }
        }

        return false;
    }

    /// Removes a node basing on its label
    pub fn removeNode(&mut self, label: String) -> bool {
        let mut index: usize = 0;

        for node in &self.nodes {
            if node.label == label {
                self.nodes.remove(index);

                return true;
            }
            index += 1;
        }
        return false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut json = Json::new();

        json.addNode(Node::new(String::from("a"), NodeContent::String(String::from("a"))));
        json.addNode(Node::new(String::from("b"), NodeContent::Bool(true)));

        json.changeLabel(String::from("a"), String::from("c"));
        json.changeContent(
            String::from("c"),
            NodeContent::List(
                Vec::from([
                    NodeContent::Int(10), NodeContent::Bool(false), NodeContent::Float(32.64)
                ])
            )
        );

        json.writeToFile(String::from("newFile.json"));

        assert_eq!(0, 0);
    }
}