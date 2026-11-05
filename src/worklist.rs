use std::collections::{HashMap, HashSet, VecDeque};

use crate::block::Block;

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
        if predecessor.connections.contains(&block.name)
            && let Some(output) = out_state.get(&predecessor.name)
        {
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

                let realargs: Vec<String> = argtemp
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

    result
}
#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct BlockState {
    // from to
    pub Variables: HashMap<String, String>,
    // func args
    pub RanFunctions: Vec<HashMap<String, Vec<String>>>,
}
impl BlockState {
    pub fn new() -> Self {
        Self::default()
    }
}
fn transfer_block(block: Block, new_in: BlockState) -> BlockState {
    let mut currentstate: BlockState = new_in.clone();
    for line in block.lines {
        if let Some(calls) = line.clone().getcall() {
            let args: Vec<String> = calls
                .1
                .split(',')
                .map(|j| {
                    let g: Vec<_> = j.split(" ").collect();
                    if g.len() > 1 {
                        g[1].to_string().to_owned()
                    } else {
                        g[0].to_string().to_owned()
                    }
                })
                .collect();
            currentstate
                .RanFunctions
                .push(HashMap::from([(calls.0, args)]));
        }
        if let Some(assign) = line.assignment() {
            currentstate.Variables.insert(assign.0, assign.1);
        }
    }
    currentstate
}
pub fn analyze(blocks: Vec<Block>) -> (HashMap<String, BlockState>, HashMap<String, BlockState>) {
    let mut mutblocks = blocks.clone();
    let mut in_state: HashMap<String, BlockState> = HashMap::new();
    let mut out_state: HashMap<String, BlockState> = HashMap::new();
    let mut worklist: VecDeque<Block> = VecDeque::new();
    let name_to_block: HashMap<String, Block> = worklist
        .iter()
        .map(|f| (f.name.clone(), f.clone()))
        .collect();
    mutblocks.iter_mut().for_each(|f| {
        f.update_setvars();
    });
    for block in &mutblocks {
        in_state.insert(block.name.clone(), BlockState::new());
        out_state.insert(block.name.clone(), BlockState::new());
        worklist.push_back(block.clone());
    }

    while let Some(block) = worklist.pop_front() {
        let new_in: BlockState = merge_predecessor_outputs(&block, &mutblocks, &out_state);

        let new_out = transfer_block(block.clone(), new_in.clone());

        let changed = in_state[&block.name] != new_in || out_state[&block.name] != new_out;

        if changed {
            in_state.insert(block.name.clone(), new_in);
            out_state.insert(block.name.clone(), new_out);

            for successor in &block.connections {
                worklist.push_back(name_to_block[successor].clone());
            }
        }
    }
    (in_state, out_state)
}
