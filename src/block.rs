use regex::{self, Regex};
use std::sync::LazyLock;
use std::{cmp::Ordering, fmt};

use crate::block::BroadType::Other;
macro_rules! make_static {
    ($expr:expr) => {{ LazyLock::new(|| $expr) }};
}
macro_rules! define_regex {
    ($name:ident, $text:expr) => {
        static $name: LazyLock<regex::Regex> = make_static!({ Regex::new($text).ok().unwrap() });
    };
}

define_regex!(
    REGEX_NUMERIC,
    r"%(?P<RegisterName>.*?)\s{0,}=\s{0,}(?P<OperatorType>.*?) (?P<NumberType>.*?) \s{0,}(?P<NumberA>[0-9]*?*?),\s{0,}(?P<NumberB>[0-9]*?*?)$"
);
define_regex!(REGEX_BR, r"\bbr.*?\b");

define_regex!(REGEX_LABEL, r"label %(.*?)(?:\,|$)");
define_regex!(REGEX_DEFINE, r"define .*? (.*?)\(");
define_regex!(REGEX_BLOCK_LABEL, r"^\s*(.*?):");
define_regex!(REGEX_RETURN, r"\bret\b");
define_regex!(REGEX_CALL, r"\bcall\b");
define_regex!(
    REGEX_STORE,
    r"store (?P<storetype>.*?) %(?P<storefrom>.*?), ptr %(?P<storeto>.*?)$"
);
define_regex!(
    REGEX_STORE_CONST,
    r"store (?P<storetype>.*?) (?P<storefrom>.*?), ptr %(?P<storeto>.*?)$"
);
define_regex!(
    REGEX_SOURCE,
    r"%(?P<varname>.*?) {0,}= {0,}call .*?@(.*?)\b"
);
define_regex!(REGEX_SINK, r"\bcall void @SINK\(.*? %(?P<varname>.*?)\)");
define_regex!(
    REGEX_LOAD,
    r"%(?P<loadto>.*?)\s{0,}=\s{0,}load\s{0,}(?P<type>.*?),\s{0,}ptr %(?P<loadfrom>.*?)$"
);
define_regex!(REGEX_END_BRACKET, r"^\s*}\s*$");
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
    Br,
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
    Call,
    Define,
    Label,
    Endbracket,
    Unkown,
}
impl SpecificType {}
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Expression {
    pub broad_type: BroadType,
    pub specific_type: SpecificType,
    pub text: String,
}
impl Default for Expression {
    fn default() -> Self {
        Self {
            broad_type: BroadType::Unkown,
            specific_type: SpecificType::Unkown,
            text: Default::default(),
        }
    }
}
impl fmt::Display for Expression {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.broad_type != BroadType::Numberic {
            return formatter.write_str(&self.text);
        } else {
            // reconstruct the expression
            match self.specific_type {
                SpecificType::Add {
                    ref result_reg,
                    ref operand_a,
                    ref operand_b,
                    ref operand_type,
                } => {
                    write!(
                        formatter,
                        "%{} = add {} {}, {}",
                        result_reg, operand_type, operand_a, operand_b
                    )
                }
                SpecificType::Sub {
                    ref result_reg,
                    ref operand_a,
                    ref operand_b,
                    ref operand_type,
                } => {
                    write!(
                        formatter,
                        "%{} = sub {} {}, {}",
                        result_reg, operand_type, operand_a, operand_b
                    )
                }
                SpecificType::Div {
                    ref result_reg,
                    ref operand_a,
                    ref operand_b,
                    ref operand_type,
                } => {
                    write!(
                        formatter,
                        "%{} = div {} {}, {}",
                        result_reg, operand_type, operand_a, operand_b
                    )
                }
                SpecificType::Mul {
                    ref result_reg,
                    ref operand_a,
                    ref operand_b,
                    ref operand_type,
                } => {
                    write!(
                        formatter,
                        "%{} = mul {} {}, {}",
                        result_reg, operand_type, operand_a, operand_b
                    )
                }
                _ => unreachable!(),
            }
        }
    }
}
impl Expression {
    pub fn new(text: String, broad_type: BroadType, specific_type: SpecificType) -> Self {
        Expression {
            text,
            broad_type,
            specific_type,
            ..Default::default()
        }
    }
    pub fn new_with_text(text: String) -> Self {
        let mut expression = Expression {
            text,
            broad_type: BroadType::Unkown,
            specific_type: SpecificType::Unkown,
            ..Default::default()
        };
        expression.guess_specific_type();
        expression.guess_general_type();
        expression
    }
    pub fn new_with_specific_type(text: String, specific_type: SpecificType) -> Self {
        let mut expression = Expression {
            text,
            broad_type: BroadType::Unkown,
            specific_type,
            ..Default::default()
        };
        expression.guess_general_type();
        expression
    }
    pub fn guess_specific_type(&mut self) {
        if REGEX_CALL.is_match(&self.text) {
            self.specific_type = SpecificType::Call;
        } else if REGEX_DEFINE.is_match(&self.text) {
            self.specific_type = SpecificType::Define;
        } else if REGEX_BLOCK_LABEL.is_match(&self.text) {
            self.specific_type = SpecificType::Label;
        } else if REGEX_RETURN.is_match(&self.text) {
            self.specific_type = SpecificType::Ret;
        } else if REGEX_BR.is_match(&self.text) {
            self.specific_type = SpecificType::Br;
        } else if REGEX_END_BRACKET.is_match(&self.text) {
            self.specific_type = SpecificType::Endbracket;
        } else if REGEX_NUMERIC.is_match(&self.text) {
            let caputures_lol = REGEX_NUMERIC.captures_iter(&self.text).next().unwrap();
            let number_type = &caputures_lol["OperatorType"];
            self.specific_type = match number_type {
                "add" => SpecificType::Add {
                    // regex is     r"%(?P<RegisterName>.*?)\s{0,}=\s{0,}(?P<OperatorType>.*?) (?P<NumberType>.*?) \s{0,}(?P<NumberA>[0-9]*?*?),\s{0,}(?P<NumberB>[0-9]*?*?)$"
                    result_reg: caputures_lol["RegisterName"].to_string(),
                    operand_a: caputures_lol["NumberA"].to_string(),
                    operand_b: caputures_lol["NumberB"].to_string(),
                    operand_type: caputures_lol["NumberType"].to_string(),
                },
                "sub" => SpecificType::Sub {
                    result_reg: caputures_lol["RegisterName"].to_string(),
                    operand_a: caputures_lol["NumberA"].to_string(),
                    operand_b: caputures_lol["NumberB"].to_string(),
                    operand_type: caputures_lol["NumberType"].to_string(),
                },
                "div" => SpecificType::Div {
                    result_reg: caputures_lol["RegisterName"].to_string(),
                    operand_a: caputures_lol["NumberA"].to_string(),
                    operand_b: caputures_lol["NumberB"].to_string(),
                    operand_type: caputures_lol["NumberType"].to_string(),
                },
                "mul" => SpecificType::Mul {
                    result_reg: caputures_lol["RegisterName"].to_string(),
                    operand_a: caputures_lol["NumberA"].to_string(),
                    operand_b: caputures_lol["NumberB"].to_string(),
                    operand_type: caputures_lol["NumberType"].to_string(),
                },
                _ => {
                    println!("Unkown numeric type! {}", number_type);
                    SpecificType::Unkown
                }
            };
        } else {
            self.specific_type = SpecificType::Unkown;
        }
    }
    // only call after the specific type is populated
    pub fn guess_general_type(&mut self) {
        match self.specific_type {
            SpecificType::Call => self.broad_type = BroadType::Call,
            SpecificType::Define | SpecificType::Label => self.broad_type = BroadType::Start,
            SpecificType::Ret | SpecificType::Br => self.broad_type = BroadType::End,
            SpecificType::Endbracket => self.broad_type = BroadType::End,
            SpecificType::Add {
                result_reg: _,
                operand_a: _,
                operand_b: _,
                operand_type: _,
            }
            | SpecificType::Div {
                result_reg: _,
                operand_a: _,
                operand_b: _,
                operand_type: _,
            }
            | SpecificType::Mul {
                result_reg: _,
                operand_a: _,
                operand_b: _,
                operand_type: _,
            }
            | SpecificType::Sub {
                result_reg: _,
                operand_a: _,
                operand_b: _,
                operand_type: _,
            } => self.broad_type = BroadType::Numberic,

            _ => self.broad_type = Other,
        }
    }

    pub fn resolve(self) {
        if self.broad_type == BroadType::Numberic {
            return;
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct Block {
    // Name of block.
    pub name: String,
    // The lines that make the body of the function
    pub lines: Vec<Expression>,
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
