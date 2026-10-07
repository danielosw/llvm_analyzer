use clap::Parser;
use regex::{self, Regex};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, prelude::*};
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

#[derive(Clone, Eq, PartialEq)]
struct Block {
    // Name of block.
    name: String,
    // The lines that make the body of the function
    lines: Vec<String>,
    // The connections the function has to other blocks
    connections: Vec<String>,
    // The number, only used for output
    number: i32,
}
// I am only implementing this so I can use .sort
impl Ord for Block {
    fn cmp(&self, other: &Self) -> Ordering {
        self.number.cmp(&other.number)
    }
}
impl PartialOrd for Block {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Default for Block {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            lines: Vec::new(),
            connections: Vec::new(),
            number: 0,
        }
    }
}
impl Block {
    // construct a new block with defualt values
    fn new() -> Self {
        Self::default()
    }
}
define_regex!(REGEX_DEFINE, r"define .*? (.*?)\(");
define_regex!(REGEX_LABEL, r"label %(.*?)(?:\,|$)");
define_regex!(REGEX_BLOCK_LABEL, r"^\s*(.*?):");
define_regex!(REGEX_RETURN, r"\bret\b");
define_regex!(REGEX_CALL, r"\bcall\b");

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    filename: String,
}
fn process_function(lines: Vec<String>) -> std::io::Result<()> {
    let mut inblock = false;
    let mut blocknames: Vec<String> = Vec::new();
    let mut blocks: HashMap<String, Block> = HashMap::new();
    let mut functionname = "".to_owned();
    let mut currentcounter = 1;
    let mut blocknum = 0;
    let mut block: Block = Block::new();
    for i in lines {
        if (i.trim().starts_with(";")) {
            continue;
        }
        if !inblock {
            let define = REGEX_DEFINE.as_ref().unwrap().captures(&i);
            if (define.is_some()) {
                // move the function name into the functionname variable
                functionname = define.as_ref().unwrap().get(1).unwrap().as_str().to_owned();
                blocknames.push(functionname.clone());
                inblock = true;
                // create a new block
                block = Block {
                    name: define.unwrap().get(1).unwrap().as_str().to_owned(),
                    number: blocknum,
                    ..Default::default()
                };
                block.lines.push(i.clone());
                blocknum += 1;
            }
            let labels = REGEX_BLOCK_LABEL.as_ref().unwrap().captures(&i);
            if labels.is_some() {
                let label = labels.unwrap().get(1).unwrap().as_str().to_owned();
                blocknames.push(label.clone());
                // create a new block
                block = Block {
                    name: label.to_string(),
                    number: blocknum,

                    ..Default::default()
                };
                block.lines.push(i);
                blocknum += 1;
                inblock = true;
                continue;
            }
        } else {
            // we are in a block
            let isBr = i.trim().starts_with("br ");

            if REGEX_RETURN.as_ref().unwrap().find(&i).is_some() {
                block.lines.push(i);
                blocks.insert(block.clone().name, block.clone());
                inblock = false;
                currentcounter = 1;
                continue;
            } else if isBr {
                block.lines.push(i.clone());
                let labels = REGEX_LABEL.as_ref().unwrap().captures_iter(&i);
                let captures: Vec<&str> = labels
                    .filter(|f| if f.get(1).is_some() { true } else { false })
                    .map(|f| f.get(1).unwrap().as_str())
                    .collect();

                for j in captures {
                    block
                        .connections
                        .push(j.strip_prefix('%').unwrap_or(j).to_owned());
                }
                blocks.insert(block.name.clone(), block.clone());
                inblock = false;
                currentcounter = 1;
                continue;
            } else if REGEX_CALL.as_ref().unwrap().find(&i).is_some() {
                let callname = block.name.clone() + &currentcounter.to_string();
                let returnname = block.name.clone() + &(currentcounter + 1).to_string();
                block.connections.push(callname.clone());
                blocks.insert(block.name.clone(), block.clone());

                let mut call_block = Block {
                    name: callname.clone(),
                    lines: vec![i.clone()],
                    connections: vec![returnname.clone()],
                    number: blocknum,

                    ..Default::default()
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
            } else {
                block.lines.push(i);
            }
        }
    }
    if functionname.strip_prefix("@").is_some() {
        output_to_file(blocks, functionname.strip_prefix("@").unwrap())
    } else {
        output_to_file(blocks, &functionname)
    }
}
fn output_to_file(
    mut blocks: HashMap<std::string::String, Block>,
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
        let mut j = 0;
        for i in block.connections.clone() {
            let target = blocks.get(&i).unwrap();

            output += &("\t\tNode".to_owned()
                + &block.number.to_string()
                + " -> Node"
                + &target.number.to_string()
                + " [label="
                + &j.to_string()
                + "];\n");
            j += 1;
        }
    }
    output += "}";
    outputfile.write(output.as_bytes())?;
    outputfile.flush()?;
    Ok(())
}
fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let llfile = File::open("./".to_owned() + &args.filename).unwrap();
    let reader = BufReader::new(llfile);
    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;
    let mut functions: HashMap<String, Vec<String>> = HashMap::new();
    let mut infunction = false;
    let mut firstfuntion = false;
    let mut name = "".to_string();
    let mut function: Vec<String> = Vec::new();
    for i in lines {
        let j = i.clone();
        let define = REGEX_DEFINE.as_ref().unwrap().captures(&j);
        if (!infunction) {
            if (define.is_some()) {
                name = define.unwrap().get(1).unwrap().as_str().to_owned();
                function.push(i);
                infunction = true;
                firstfuntion = true;
            } else if !firstfuntion {
                name = "global".to_string();
                function.push(i);
            }
        } else {
            if (i.contains("}")) {
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
    for (_, function_list) in functions.iter() {
        process_function(function_list.to_vec())?;
    }
    Ok(())
}
