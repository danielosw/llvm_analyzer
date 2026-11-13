use clap::Parser;
use crossbeam::channel::{self as cc};
use eecs_677_project_3::block::{Block, EscapeIntOrReg};
use eecs_677_project_3::processing::{process_function, process_ll};
use eecs_677_project_3::worklist::analyze;
use std::cmp::max;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, prelude::*};
use std::thread::available_parallelism;
use std::thread::spawn;
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
    let mut number = 0;
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
        number += 1;
    }
    for block in tempvec {
        for (j, i) in block.connections.clone().into_iter().enumerate() {
            println!("{} -> {}", block.number, i);
            let target = blocks.get(&i.escape_int_or_reg());
            // if the target does not exist and it starts with @ assume it is a function call and add a node for it
            if target.is_none() && i.starts_with("@") {
                output += &("\t\tNode".to_owned()
                    + &number.to_string()
                    + " [shape=record,label=\""
                    + &i.escape_int_or_reg()
                    + "\"]\n");
            }

            output += &("\t\tNode".to_owned()
                + &block.number.to_string()
                + " -> Node"
                + &target
                    .map(|b| b.number.to_string())
                    .unwrap_or_else(|| number.to_string())
                + " [label="
                + &j.to_string()
                + "];\n");
            number += 1;
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
    let (s, r) = cc::unbounded();
    // get the number of threads per function
    // right now this is just functions_len/threads, but a better way should be added
    let default_parallelism_approx_result = available_parallelism();
    let default_parallelism_approx = match default_parallelism_approx_result {
        Ok(nonezero) => nonezero.get(),
        Err(_error) => 1,
    };
    let threads_per_function = max(1, default_parallelism_approx / functions.len());
    functions.values().for_each(|f| {
        let lines = f.clone();
        let sender = s.clone();

        spawn(move || {
            let (blocks, function_name) =
                process_function(lines, threads_per_function.try_into().unwrap());
            sender.send((blocks, function_name)).unwrap();
            drop(sender);
        });
    });
    let function_list: Vec<_> = (0..functions.len())
        .map(|_| {
            r.recv().map_err(|error| {
                std::io::Error::other(format!("failed to receive function result: {error}"))
            })
        })
        .collect::<std::io::Result<_>>()?;
    for (blocks, function_name) in function_list {
        let mut tempvec: Vec<Block> = blocks.values().map(|f| f.to_owned()).collect();
        tempvec.sort();
        let results = analyze(tempvec);
        let _instates = results.0;
        let out_states = results.1;
        if out_states.values().any(|f| {
            f.ran_functions.iter().any(|g| {
                g.get("@SINK")
                    .is_some_and(|args| args.iter().any(|arg| arg == "@SOURCE"))
            })
        }) {
            println!("Found flow!")
        } else {
            println!("No flow!")
        }
        output_to_file(blocks.clone(), &function_name)?;
    }
    Ok(())
}
