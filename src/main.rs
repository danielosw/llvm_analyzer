use clap::Parser;
use regex::{self, Regex};
use std::collections::HashMap;
use std::io;
use std::sync::LazyLock;
macro_rules! make_static {
    ($expr:expr) => {{ LazyLock::new(|| $expr) }};
}
macro_rules! define_regex {
    ($name:ident, $text:expr) => {
        static $name: LazyLock<std::option::Option<regex::Regex>> =
            make_static!({ Regex::new($text).ok() });
    };
}
#[derive(Clone)]
struct Block<'a> {
    // Name of block.
    name: String,
    // THe lines that make the body of the function
    lines: Vec<&'a str>,
    // The connections the function has to other blocks
    connections: Vec<Block<'a>>,
}
define_regex!(REGEX_DEFINE, r"define .*? (.*?)\(");
define_regex!(REGEX_LABEL, r"label %(.*?)(?:\,|$)");
define_regex!(REGEX_RETURN, r"\bret\b");
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    filename: String,
}
fn process_function(lines: Vec<&str>) {
    let mut inblock = false;
    let mut blocknames: Vec<&str> = Vec::new();
    let mut blocks: HashMap<String, Block> = HashMap::new();
    let mut currentname = "";
    let mut functionname = "";
    let mut currentcounter = 1;
    let mut block: Option<Block> = None;
    for i in lines {
        if (i.trim().starts_with(";")) {
            continue;
        }
        if !inblock {
            let define = REGEX_DEFINE.as_ref().unwrap().captures(i);
            if (define.is_some()) {
                // move the function name into the functionname variable
                currentname = define.unwrap().get(1).unwrap().as_str();
                functionname = currentname;
                blocknames.push(functionname);
                inblock = true;
                block = Some(Block {
                    name: currentname.to_string(),
                    lines: Vec::new(),
                    connections: Vec::new(),
                });
            }
            let labels = REGEX_LABEL.as_ref().unwrap().captures(i);
            if labels.is_some() {
                currentname = labels.unwrap().get(1).unwrap().as_str();
                blocknames.push(currentname);
                block = Some(Block {
                    name: currentname.to_string(),
                    lines: Vec::new(),
                    connections: Vec::new(),
                });
                inblock = true;
                continue;
            }
        } else {
            // we are in a block
            if REGEX_RETURN.as_ref().unwrap().find(i).is_some() {
                let mut templist = block.clone().unwrap().lines.clone();
                templist.push(i);
                let connections = block.clone().unwrap().connections.clone();
                block = Some(Block {
                    name: block.clone().unwrap().name.clone(),
                    lines: templist,
                    connections: connections,
                });
                blocks.insert(block.as_ref().unwrap().name.clone(), block.clone().unwrap());
                inblock = false;
                currentcounter = 1;
                continue;
            }
            let isBr = i.trim().starts_with("br ");
            if isBr {
                let labels = REGEX_LABEL.as_ref().unwrap().captures_iter(i);
                let captures: Vec<&str> = labels
                    .filter(|f| if f.get(1).is_some() { true } else { false })
                    .map(|f| f.get(1).unwrap().as_str())
                    .collect();
                
                let mut tempconnections = ;
                for j in captures{
                    block.unwrap().connections.
                    
                }
                }

        }
    }
}
fn main() {
    let args = Args::parse();
}
