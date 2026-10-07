use clap::Parser;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
mod block;
mod proccessing;
use crate::block::Block;
use crate::proccessing::{proccess_ll, process_function};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    filename: String,
}
mod modname {}
fn output_to_file(
    blocks: HashMap<std::string::String, Block>,
    function_name: &str,
) -> std::io::Result<()> {
    let mut outputfile = File::create("./".to_owned() + function_name + ".dot")?;
    let mut output = "digraph {\n".to_string();
    let mut tempvec: Vec<&Block> = blocks.values().collect();
    tempvec.sort();
    for block in &tempvec {
        // label = "\\n".join(blocks[key]).replace("\n", "\\n").replace("{", "").replace("}", "")
        let label = block
            .lines
            .join("\\n")
            .replace("\n", "\\n")
            .replace("{", "")
            .replace("}", "");

        output += &("\t\tNode".to_owned()
            + &block.number.to_string()
            + " [shape=record,label=\""
            + &label
            + "\"]\n");
    }
    for block in tempvec {
        for (j, i) in block.connections.clone().into_iter().enumerate() {
            let target = blocks.get(&i).unwrap();

            output += &("\t\tNode".to_owned()
                + &block.number.to_string()
                + " -> Node"
                + &target.number.to_string()
                + " [label="
                + &j.to_string()
                + "];\n");
        }
    }
    output += "}";
    outputfile.write_all(output.as_bytes())?;
    outputfile.flush()?;
    Ok(())
}
fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let file_name = "./".to_owned() + &args.filename;
    let functions = proccess_ll(file_name)?;
    for function_list in functions.values() {
        process_function(function_list.to_vec())?;
    }
    Ok(())
}
