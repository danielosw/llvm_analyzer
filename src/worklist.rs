use std::{
    collections::{HashMap, VecDeque},
    fmt,
};

use crate::block::Block;
fn merge_predecessor_outputs(
    block: &Block,
    blocks: &[Block],
    out_state: &HashMap<String, BlockState>,
) -> BlockState {
    let mut result: BlockState = BlockState::new();

    for predecessor in blocks {
        if predecessor
            .connections
            .iter()
            .any(|connection| connection.trim_start_matches('%') == block.name)
            && let Some(output) = out_state.get(&predecessor.name)
        {
            let resulting = &mut result;
            for i in output.variables.iter() {
                if resulting.variables.contains_key(i.0) {
                    if resulting.variables.get(i.0).unwrap() == i.1 {
                        continue;
                    }
                    resulting.variables.insert(i.0.clone(), i.1.clone());
                } else {
                    resulting.variables.insert(i.0.clone(), i.1.clone());
                }
            }
            for j in output.ran_functions.iter().to_owned() {
                let function: (&String, &Vec<_>) = j.iter().next().unwrap();
                let argtemp = function.1.to_owned();

                let realargs: Vec<String> = argtemp
                    .iter()
                    .map(|f| resolve_variable(&resulting.variables, f))
                    .collect();
                let call = HashMap::from([(function.0.to_owned(), realargs)]);
                if !resulting.ran_functions.contains(&call) {
                    resulting.ran_functions.push(call);
                }
            }
        }
    }

    result
}
#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct BlockState {
    // from to
    pub variables: HashMap<String, String>,
    // func args
    pub ran_functions: Vec<HashMap<String, Vec<String>>>,
}
impl BlockState {
    pub fn new() -> Self {
        Self::default()
    }
}
impl fmt::Display for BlockState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "Variables:")?;
        for (name, value) in &self.variables {
            writeln!(formatter, "{}:{}", name, value)?;
        }

        writeln!(formatter, "Function uses:")?;
        for function in &self.ran_functions {
            let (name, args) = function.iter().next().unwrap();
            writeln!(formatter, "{}:{}", name, args.join(","))?;
        }

        Ok(())
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
            let realargs: Vec<String> = args
                .iter()
                .map(|f| resolve_variable(&currentstate.variables, f))
                .collect();
            let call = HashMap::from([(calls.0, realargs)]);
            if !currentstate.ran_functions.contains(&call) {
                currentstate.ran_functions.push(call);
            }
        }
        if let Some(assign) = line.assignment() {
            currentstate.variables.insert(assign.0, assign.1);
        }
    }
    currentstate
}

fn resolve_variable(variables: &HashMap<String, String>, variable: &str) -> String {
    let mut resolved = variable.to_owned();
    let mut seen = std::collections::HashSet::new();
    while let Some(next) = variables.get(&resolved) {
        if !seen.insert(resolved.clone()) {
            break;
        }
        resolved = next.clone();
    }
    resolved
}

pub fn analyze(
    blocks: Vec<Block>,
    function_list: Vec<(HashMap<String, Block>, String)>,
    function_names: Vec<String>,
) -> (HashMap<String, BlockState>, HashMap<String, BlockState>) {
    let mutblocks = blocks.clone();
    let mut in_state: HashMap<String, BlockState> = HashMap::new();
    let mut out_state: HashMap<String, BlockState> = HashMap::new();
    let mut worklist: VecDeque<Block> = VecDeque::new();

    let name_to_block: HashMap<String, Block> = mutblocks
        .iter()
        .map(|block| (block.name.clone(), block.clone()))
        .collect();
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
                if let Some(successor_block) = name_to_block.get(successor.trim_start_matches('%'))
                {
                    worklist.push_back(successor_block.clone());
                }
            }
        }
    }
    (in_state, out_state)
}
