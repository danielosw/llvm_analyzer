use crate::block;
use regex::{self, Regex};
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

define_regex!(REGEX_DEFINE, r"define .*? (.*?)\(");
define_regex!(REGEX_LABEL, r"label %(.*?)(?:\,|$)");
define_regex!(REGEX_BLOCK_LABEL, r"^\s*(.*?):");
define_regex!(REGEX_RETURN, r"\bret\b");
define_regex!(REGEX_CALL, r"\bcall\b");

use crate::block::Block;

use std::collections::HashMap;

pub fn process_ll(ll_text: String) -> Result<HashMap<String, Vec<String>>, std::io::Error> {
    let lines: Vec<String> = ll_text.lines().map(|s| s.to_string()).collect();
    let mut functions: HashMap<String, Vec<String>> = HashMap::new();
    let mut infunction = false;
    let mut firstfuntion = false;
    let mut name = "".to_string();
    let mut function: Vec<String> = Vec::new();
    for i in lines {
        let j = i.clone();
        let define = REGEX_DEFINE.as_ref().unwrap().captures(&j);
        if !infunction {
            if let Some(define) = define {
                name = define.get(1).unwrap().as_str().to_owned();
                function.push(i);
                infunction = true;
                firstfuntion = true;
            } else if !firstfuntion {
                name = "global".to_string();
                function.push(i);
            }
        } else {
            if i.contains("}") {
                function.push(i);
                functions.insert(name.clone().to_string(), function.clone());
                name = "".to_string();
                infunction = false;
                function = Vec::new();
            } else {
                function.push(i);
            }
        }
    }
    Ok(functions)
}

pub fn process_function(
    lines: Vec<String>,
) -> (
    std::collections::HashMap<std::string::String, block::Block>,
    String,
) {
    let mut inblock = false;
    let mut blocknames: Vec<String> = Vec::new();
    let mut blocks: HashMap<String, Block> = HashMap::new();
    let mut functionname = "".to_owned();
    let mut currentcounter = 1;
    let mut blocknum = 0;
    let mut block: Block = Block::new();
    for i in lines {
        if i.trim().starts_with(";") {
            continue;
        }
        if !inblock {
            let define = REGEX_DEFINE.as_ref().unwrap().captures(&i);
            if let Some(define) = define {
                // move the function name into the functionname variable
                functionname = define.get(1).unwrap().as_str().to_owned();
                blocknames.push(functionname.clone());
                inblock = true;
                // create a new block
                block = Block {
                    name: define.get(1).unwrap().as_str().to_owned(),
                    number: blocknum,
                    ..Default::default()
                };
                block.lines.push(i.clone());
                blocknum += 1;
            }
            let labels = REGEX_BLOCK_LABEL.as_ref().unwrap().captures(&i);
            if let Some(labels) = labels {
                let label = labels.get(1).unwrap().as_str().to_owned();
                blocknames.push(label.clone());
                // create a new block
                block = Block {
                    name: label.to_string(),
                    number: blocknum,

                    ..Default::default()
                };
                block.lines.push(i);
                blocknum += 1;
                inblock = true;
                continue;
            }
        } else {
            // we are in a block
            let is_br = i.trim().starts_with("br ");

            if REGEX_RETURN.as_ref().unwrap().find(&i).is_some() {
                block.lines.push(i);
                blocks.insert(block.clone().name, block.clone());
                inblock = false;
                currentcounter = 1;
                continue;
            }
            if is_br {
                block.lines.push(i.clone());
                let labels = REGEX_LABEL.as_ref().unwrap().captures_iter(&i);
                let captures: Vec<&str> = labels
                    .filter(|f| f.get(1).is_some())
                    .map(|f| f.get(1).unwrap().as_str())
                    .collect();
                for j in captures {
                    block
                        .connections
                        .push(j.strip_prefix('%').unwrap_or(j).to_owned());
                }
                blocks.insert(block.name.clone(), block.clone());
                inblock = false;
                currentcounter = 1;
                continue;
            }
            if REGEX_CALL.as_ref().unwrap().find(&i).is_some() {
                let callname = block.name.clone() + &currentcounter.to_string();
                let returnname = block.name.clone() + &(currentcounter + 1).to_string();
                block.connections.push(callname.clone());
                blocks.insert(block.name.clone(), block.clone());

                let call_block = Block {
                    name: callname.clone(),
                    lines: vec![i.clone()],
                    connections: vec![returnname.clone()],
                    number: blocknum,
                };
                blocknum += 1;

                blocks.insert(callname, call_block);

                let return_block = Block {
                    name: returnname.clone(),
                    number: blocknum,

                    ..Default::default()
                };
                blocknum += 1;
                blocks.insert(returnname.clone(), return_block);
                block = blocks.get(&returnname).cloned().unwrap();

                currentcounter += 2;
                continue;
            }
            block.lines.push(i);
        }
    }
    if functionname.strip_prefix("@").is_some() {
        (blocks, functionname.strip_prefix("@").unwrap().to_string())
    } else {
        (blocks, functionname)
    }
}
