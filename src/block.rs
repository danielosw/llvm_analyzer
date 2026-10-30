use crate::expression;
use std::{cmp::Ordering, fmt};
// superclass of string
pub trait IntOrReg {
    fn is_int(&self) -> bool;
    fn is_reg(&self) -> bool;
}
// returns the int if in returns reg with leading % removed if reg
pub trait EscapeIntOrReg {
    fn escape_int_or_reg(&self) -> String;
}

impl EscapeIntOrReg for String {
    fn escape_int_or_reg(&self) -> String {
        if self.is_reg()
            && let Some(stripped) = self.strip_prefix('%')
        {
            return stripped.to_string();
        }
        self.clone()
    }
}

impl IntOrReg for String {
    fn is_int(&self) -> bool {
        self.parse::<i32>().is_ok()
    }

    fn is_reg(&self) -> bool {
        self.starts_with('%')
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BroadType {
    Call,
    Start,
    End,
    Numberic,
    Other,
    Unkown,
}
#[derive(Clone, Debug, Eq, PartialEq)]

pub enum SpecificType {
    Alloca,
    Getelementpointer,
    Load {
        load_to: String,
        load_from: String,
        load_type: String,
    },
    Store {
        store_type: String,
        store_from: String,
        store_to: String,
    },
    Ret,
    Icmp,
    Br {
        sendto1: String,
        matchtype: Option<String>,
        checkvar: Option<String>,
        sendto2: Option<String>,
    },
    Add {
        result_reg: String,
        operand_a: String,
        operand_b: String,
        operand_type: String,
    },
    Sub {
        result_reg: String,
        operand_a: String,
        operand_b: String,
        operand_type: String,
    },
    Div {
        result_reg: String,
        operand_a: String,
        operand_b: String,
        operand_type: String,
    },
    Mul {
        result_reg: String,
        operand_a: String,
        operand_b: String,
        operand_type: String,
    },
    Phi,
    // the mess here is that their is basiclly two diffrent calls, one is a regular call one one is a assignment call, the assignemnt call is the one that has a register on the left side of the equal sign, the regular call is just a call with no equal sign
    // and we want to handle both in the same type
    Call {
        func_name: String,
        args: String,
        calltype: String,
        sendto: Option<String>,
    },
    Define {
        dtype: String,
        name: String,
        args: String,
    },
    Label {
        name: String,
    },
    Endbracket,
    Unkown,
}
impl fmt::Display for SpecificType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Br {
                sendto1,
                matchtype,
                checkvar,
                sendto2,
            } => {
                if let Some(matchtype) = matchtype {
                    if let Some(checkvar) = checkvar {
                        if let Some(sendto2) = sendto2 {
                            write!(
                                f,
                                "br {} {}, label {}, label {}",
                                matchtype, checkvar, sendto1, sendto2
                            )
                        } else {
                            write!(f, "br {} {}, label {}", matchtype, checkvar, sendto1)
                        }
                    } else {
                        write!(f, "br label {}", sendto1)
                    }
                } else {
                    write!(f, "br label {}", sendto1)
                }
            }
            Self::Add {
                result_reg,
                operand_a,
                operand_b,
                operand_type,
            } => write!(
                f,
                "%{} = add {} {}, {}",
                result_reg, operand_type, operand_a, operand_b
            ),
            Self::Sub {
                result_reg,
                operand_a,
                operand_b,
                operand_type,
            } => write!(
                f,
                "%{} = sub {} {}, {}",
                result_reg, operand_type, operand_a, operand_b
            ),
            Self::Div {
                result_reg,
                operand_a,
                operand_b,
                operand_type,
            } => write!(
                f,
                "%{} = div {} {}, {}",
                result_reg, operand_type, operand_a, operand_b
            ),
            Self::Mul {
                result_reg,
                operand_a,
                operand_b,
                operand_type,
            } => write!(
                f,
                "%{} = mul {} {}, {}",
                result_reg, operand_type, operand_a, operand_b
            ),
            Self::Define { dtype, name, args } => {
                write!(f, "define {} {}({}) {{", dtype, name, args)
            }
            Self::Label { name } => write!(f, "{}:", name),
            Self::Endbracket => write!(f, "}}"),
            Self::Call {
                func_name,
                args,
                calltype,
                sendto,
            } => {
                if let Some(sendto) = sendto {
                    // example:   %secret = call i32 () @SOURCE()
                    write!(f, "%{} = call {} {}({})", sendto, calltype, func_name, args)
                } else {
                    // example:   call i32 @SOURCE()
                    write!(f, "call {} {}({})", calltype, func_name, args)
                }
            }
            Self::Store {
                store_type,
                store_from,
                store_to,
            } => {
                write!(f, "store {} {}, ptr %{}", store_type, store_from, store_to)
            }
            Self::Load {
                load_to,
                load_from,
                load_type,
            } => {
                write!(f, "%{} = load {}, ptr %{}", load_to, load_type, load_from)
            }
            _ => unimplemented!(),
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct Block {
    // Name of block.
    pub name: String,
    // The lines that make the body of the function
    pub lines: Vec<expression::Expression>,
    // The connections the function has to other blocks
    pub connections: Vec<String>,
    // The number, only used for output
    pub number: i32,
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
    pub fn new() -> Self {
        Self::default()
    }
    pub fn get_lines_vec(&self) -> Vec<String> {
        self.lines
            .iter()
            .map(|expression| expression.text.clone())
            .collect()
    }
}
