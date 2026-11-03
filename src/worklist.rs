use std::collections::{HashMap, HashSet, VecDeque};

use clap::builder::Str;

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
fn merge_predecessor_outputs(
    block: &Block,
    blocks: &[Block],
    out_state: &HashMap<String, HashMap<String, String>>,
) -> HashMap<String, String> {
    let mut result = HashMap::new();

    for predecessor in blocks {
        if predecessor.connections.contains(&block.name) {
            if let Some(output) = out_state.get(&predecessor.name) {
                result.extend(output.clone());
            }
        }
    }

    result
}
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct BlockState {
    pub Variables: HashMap<String, String>,
    pub RanFunctions: Vec<(String, String)>,
}
fn transfer_block(block: Block, new_in: HashMap<String, String>) {
    let state: HashMap<String, String> = new_in.clone();
    let mut currentstate: BlockState = BlockState {
        Variables: HashMap::new(),
        RanFunctions: Vec::new(),
    };
    for line in block.lines {
        if let Some(calls) = line.clone().getcall() {
            currentstate.RanFunctions.push(calls);
        }
        if let Some(assign) = line.assignment() {
            // update all cases of assign.1 with assign.0
        }
    }
}
fn analyze(blocks: Vec<Block>) {
    let mut in_state: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut out_state: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut worklist: VecDeque<Block> = VecDeque::new();
    let name_to_block: HashMap<String, Block> = worklist
        .iter()
        .map(|f| (f.name.clone(), f.clone()))
        .collect();
    blocks.iter_mut().for_each(|f| {
        f.update_setvars();
    });
    for block in blocks {
        in_state.insert(block.name.clone(), HashMap::new());
        out_state.insert(block.name.clone(), HashMap::new());
        worklist.push_back(block);
    }

    while let Some(block) = worklist.pop_front() {
        let new_in: HashMap<String, String> =
            merge_predecessor_outputs(&block, &blocks, &out_state);

        let new_out = transfer_block(block, new_in.clone());

        let changed = in_state[&block.name] != new_in || out_state[&block.name] != new_out;

        if changed {
            in_state.insert(block.name.clone(), new_in);
            out_state.insert(block.name.clone(), new_out);

            for successor in &block.connections {
                worklist.push_back(name_to_block[successor].clone());
            }
        }
    }
}
pub fn worklist(mut blocks: Vec<Block>) -> Vec<Block> {
    blocks.iter_mut().for_each(|f| {
        f.update_setvars();
    });
    let mut worklist: Vec<Block> = blocks.clone();
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
