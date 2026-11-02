use std::collections::{HashMap, HashSet};

use crate::block::Block;
fn resolve(mut name: String, values: HashMap<String, String>) -> String {
    let mut seen: HashSet<String> = HashSet::new();
    while !seen.contains(&name) && values.contains_key(&name) {
        seen.insert(name.clone());
        name = <std::string::String as Clone>::clone(&values[&name]);
    }
    name
}
fn leakcheck(mut name: String, values: HashMap<String, String>, badname: String) -> bool {
    let mut assignedtosource: bool = false;

    let mut seen: HashSet<String> = HashSet::new();
    while !seen.contains(&name) && (values.contains_key(&name)) {
        if name == "@SOURCE" {
            assignedtosource = true;
        }
        seen.insert(name.clone());
        name = <std::string::String as Clone>::clone(&values[&name]);
        if name == badname && assignedtosource {
            return true;
        }
        if !name.starts_with("%") {
            assignedtosource = false;
        }
    }
    false
}
pub fn worklist(blocks: Vec<Block>) -> Vec<Block> {
    let mut worklist = blocks.clone();
    let _blocknames: Vec<String> = worklist.iter().map(|f| f.name.clone()).collect();
    let name_to_block: HashMap<String, Block> = worklist
        .iter()
        .map(|f| (f.name.clone(), f.clone()))
        .collect();
    while let Some(mut block) = worklist.pop() {
        for connection in &block.connections {
            if let Some(connected_block) = name_to_block.get(connection) {
                let tempblock = block.clone();
                let blockrefrenced: HashSet<&String> = tempblock.refrenced.iter().collect();
                let connectedrefs: HashSet<&String> = connected_block.refrenced.iter().collect();
                let merged_referenced: HashSet<&String> = blockrefrenced
                    .union(&connectedrefs)
                    .map(|f| f.to_owned())
                    .collect();
                let mut list_setvars = block.setvars.clone();
                for setvar in &connected_block.setvars {
                    if !list_setvars.contains(setvar) {
                        list_setvars.push(setvar.clone());
                    }
                }
                if merged_referenced != blockrefrenced || list_setvars != block.setvars {
                    let blockref = &mut block.refrenced;
                    let blocksetvars = &mut block.setvars;
                    *blockref = merged_referenced
                        .iter()
                        .map(|f| f.to_owned().to_owned())
                        .collect();
                    *blocksetvars = list_setvars;
                    worklist.push(block.clone());
                } else {
                    continue;
                }
            } else {
                // its prob a function call
                // which is not handled in this step
                continue;
            }
        }
    }
    let mut values: HashMap<String, String> = HashMap::new();
    let _sources: HashSet<String> = HashSet::new();
    let _sinks: Vec<String> = Vec::new();
    for block in blocks {
        for assignment in block.setvars {
            values.extend(assignment);
        }
    }
    if leakcheck("@SOURCE".to_owned(), values, "@SINK".to_owned()) {
        println!("Leak found");
    } else {
        println!("No leak");
    }
    worklist
}
