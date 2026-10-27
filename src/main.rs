use clap::Parser;
use eecs_677_project_3::block::{Block, EscapeIntOrReg};
use eecs_677_project_3::processing::{process_function, process_ll};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, prelude::*};
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    filename: String,
}
fn output_to_file(
    blocks: HashMap<std::string::String, Block>,
    function_name: &str,
) -> std::io::Result<()> {
    let mut outputfile = File::create("./".to_owned() + function_name + ".dot")?;
    let mut output = "digraph {\n".to_string();
    let mut tempvec: Vec<&Block> = blocks.values().collect();
    tempvec.sort();
    for block in &tempvec {
        let label = block
            .lines
            .iter()
            .map(|expression| expression.to_string())
            .collect::<Vec<String>>()
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
            println!("{} -> {}", block.number, i);
            let target = blocks.get(&i.escape_int_or_reg()).unwrap();

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
    let llfile = File::open(&file_name)?;
    let mut reader = BufReader::new(llfile);
    // read the entire file into a string
    let mut ll_text = String::new();
    reader.read_to_string(&mut ll_text)?;
    let functions = process_ll(ll_text)?;
    for function_list in functions.values() {
        let (blocks, function_name) = process_function(function_list.to_vec());
        output_to_file(blocks, &function_name)?;
    }
    Ok(())
}
