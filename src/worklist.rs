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
    out_state: &HashMap<String, BlockState>,
) -> BlockState {
    let mut result: BlockState = BlockState::new();

    for predecessor in blocks {
        if predecessor.connections.contains(&block.name) {
            if let Some(output) = out_state.get(&predecessor.name) {
                let resulting = &mut result;
                for i in output.Variables.iter() {
                    if resulting.Variables.contains_key(i.0) {
                        if resulting.Variables.get(i.0).unwrap() == i.1 {
                            continue;
                        } else {
                            resulting.Variables.insert(i.0.clone(), i.1.clone());
                        }
                    } else {
                        resulting.Variables.insert(i.0.clone(), i.1.clone());
                    }
                }
                for j in output.RanFunctions.iter().to_owned() {
                    let function: (&String, &Vec<_>) = j.iter().next().unwrap();
                    let argtemp = function.1.to_owned();
                    let args: Vec<String> = argtemp
                        .iter()
                        .map(|j| {
                            let g: Vec<_> = j.split(" ").collect();
                            if g.len() > 1 {
                                g[1].to_string().to_owned()
                            } else {
                                g[0].to_string().to_owned()
                            }
                        })
                        .collect();
                    let realargs: Vec<String> = args
                        .iter()
                        .map(|f| {
                            resulting
                                .Variables
                                .get(&f.to_string())
                                .cloned()
                                .unwrap_or_else(|| f.clone())
                        })
                        .collect();
                    resulting
                        .RanFunctions
                        .push(HashMap::from([(function.0.to_owned(), realargs)]));
                }
            }
        }
    }

    result
}
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct BlockState {
    // from to
    pub Variables: HashMap<String, String>,
    // func args
    pub RanFunctions: Vec<HashMap<String, Vec<String>>>,
}
impl Default for BlockState {
    fn default() -> Self {
        Self {
            Variables: HashMap::new(),
            RanFunctions: Vec::new(),
        }
    }
}
impl BlockState {
    pub fn new() -> Self {
        Self::default()
    }
}
fn transfer_block(block: Block, new_in: BlockState) {
    let mut currentstate: BlockState = new_in.clone();
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
    let mut in_state: HashMap<String, BlockState> = HashMap::new();
    let mut out_state: HashMap<String, BlockState> = HashMap::new();
    let mut worklist: VecDeque<Block> = VecDeque::new();
    let name_to_block: HashMap<String, Block> = worklist
        .iter()
        .map(|f| (f.name.clone(), f.clone()))
        .collect();
    blocks.iter_mut().for_each(|f| {
        f.update_setvars();
    });
    for block in blocks {
        in_state.insert(block.name.clone(), BlockState::new());
        out_state.insert(block.name.clone(), BlockState::new());
        worklist.push_back(block);
    }

    while let Some(block) = worklist.pop_front() {
        let new_in: BlockState = merge_predecessor_outputs(&block, &blocks, &out_state);

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
