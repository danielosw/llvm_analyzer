use crate::block::{self, SpecificType};
use crate::expression::Expression;
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

        let expression = Expression::new_with_text(j.clone());
        if !infunction {
            if matches!(expression.specific_type, SpecificType::Define { .. }) {
                let define = match expression.specific_type {
                    SpecificType::Define { ref name, .. } => name.to_string(),
                    _ => unreachable!(),
                };
                name = define;
                function.push(i);
                infunction = true;
                firstfuntion = true;
            } else if !firstfuntion {
                name = "global".to_string();
                function.push(i);
            }
        } else {
            if expression.specific_type == SpecificType::Endbracket {
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
        let expression = Expression::new_with_text(i.clone());

        if !inblock {
            if matches!(expression.specific_type, SpecificType::Define { .. }) {
                functionname = match expression.specific_type {
                    SpecificType::Define { ref name, .. } => name.to_string(),
                    _ => unreachable!(),
                };
                blocknames.push(functionname.clone());
                inblock = true;
                // create a new block
                block = Block {
                    name: functionname.clone(),
                    number: blocknum,
                    ..Default::default()
                };
                block.lines.push(expression.clone());
                blocknum += 1;
            }
            if matches!(expression.specific_type, SpecificType::Label { .. }) {
                let label = match expression.specific_type {
                    SpecificType::Label { ref name } => name.to_string(),
                    _ => unreachable!(),
                };
                blocknames.push(label.clone());
                // create a new block
                block = Block {
                    name: label.to_string(),
                    number: blocknum,

                    ..Default::default()
                };
                block.lines.push(expression);
                blocknum += 1;
                inblock = true;
                continue;
            }
        } else {
            // we are in a block
            if expression.specific_type == SpecificType::Ret {
                let expression = Expression::new_with_text(i.clone());
                block.lines.push(expression);
                blocks.insert(block.clone().name, block.clone());
                inblock = false;
                currentcounter = 1;
                continue;
            }
            if matches!(expression.specific_type, SpecificType::Br { .. }) {
                let expression = Expression::new_with_text(i.clone());
                block.lines.push(expression.clone());
                if let SpecificType::Br {
                    sendto1, sendto2, ..
                } = expression.specific_type
                {
                    block.connections.push(sendto1);

                    if let Some(st2) = sendto2 {
                        block.connections.push(st2);
                    }
                }
                blocks.insert(block.name.clone(), block.clone());
                inblock = false;
                currentcounter = 1;
                continue;
            }
            if expression.specific_type == SpecificType::Call {
                let callname = block.name.clone() + &currentcounter.to_string();
                let returnname = block.name.clone() + &(currentcounter + 1).to_string();
                block.connections.push(callname.clone());
                blocks.insert(block.name.clone(), block.clone());

                let call_block = Block {
                    name: callname.clone(),
                    lines: vec![expression],
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
            block.lines.push(expression);
        }
    }
    if functionname.strip_prefix("@").is_some() {
        (blocks, functionname.strip_prefix("@").unwrap().to_string())
    } else {
        (blocks, functionname)
    }
}
