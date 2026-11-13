use crate::block::{self, SpecificType};
use crate::expression::Expression;

use crate::block::Block;

use crossbeam::channel::{self as cc};
use std::collections::HashMap;
use std::thread::spawn;

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
                function.clear();
            } else {
                function.push(i);
            }
        }
    }
    Ok(functions)
}

pub fn process_function(
    lines: Vec<String>,
    threads: i32,
) -> (
    std::collections::HashMap<std::string::String, block::Block>,
    String,
) {
    let mut inblock = false;
    let mut entry_found = false;
    let mut blocknames: Vec<String> = Vec::new();
    let mut blocks: HashMap<String, Block> = HashMap::new();
    let mut functionname = "".to_owned();
    let mut blocknum = 0;
    let mut block: Block = Block::new();
    let mut usedtheads = 0;
    let (s, r) = cc::unbounded();
    let mut expression_list: Vec<_> = Vec::new();
    let mut remaining_expressions = lines.len();
    println!("total threads: {}", threads);

    for (x, i) in lines.clone().into_iter().enumerate() {
        let real = x;
        let sender = s.clone();
        if usedtheads >= threads {
            expression_list.push(
                r.recv()
                    .map_err(|error| {
                        std::io::Error::other(format!("failed to receive function result: {error}"))
                    })
                    .unwrap(),
            );

            usedtheads -= 1;

            remaining_expressions -= 1;
        }
        spawn(move || {
            let expression = Expression::new_with_text(i.clone());
            sender.send((expression, real)).unwrap();
            drop(sender);
        });
        usedtheads += 1;
    }

    let mut received: Vec<(Expression, usize)> = (0..remaining_expressions)
        .map(|_| {
            r.recv().map_err(|error| {
                std::io::Error::other(format!("failed to receive function result: {error}"))
            })
        })
        .collect::<std::io::Result<_>>()
        .unwrap_or_else(|error| {
            panic!("failed to receive function result: {error}");
        });

    expression_list.append(&mut received);
    expression_list.sort_by_key(|(_, index)| *index);

    for (expression, _) in expression_list {
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
                    entry: !entry_found,
                    ..Default::default()
                };
                if !entry_found {
                    entry_found = false
                }
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
                    entry: !entry_found,

                    ..Default::default()
                };
                if !entry_found {
                    entry_found = false
                }
                block.lines.push(expression);
                blocknum += 1;
                inblock = true;
                continue;
            }
        } else {
            // we are in a block
            block.lines.push(expression.clone());
            if expression.specific_type == SpecificType::Ret {
                blocks.insert(block.clone().name, block.clone());
                inblock = false;
                continue;
            }
            if matches!(expression.specific_type, SpecificType::Br { .. }) {
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
                continue;
            }

            //
        }
    }
    if functionname.strip_prefix("@").is_some() {
        (blocks, functionname.strip_prefix("@").unwrap().to_string())
    } else {
        (blocks, functionname)
    }
}
