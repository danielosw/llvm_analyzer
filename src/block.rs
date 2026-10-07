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

define_regex!(REGEX_DEFINE, r"define .*? (.*?)\(");
define_regex!(REGEX_RETURN, r"\bret\b");
define_regex!(REGEX_CALL, r"\bcall\b");
define_regex!(REGEX_BLOCK_LABEL, r"^\s*(.*?):");

#[derive(Clone, Eq, PartialEq)]
pub enum BroadType {
    Call,
    Start,
    End,
    Other,
    Unkown,
}
#[derive(Clone, Eq, PartialEq)]

pub enum SpecificType {
    Alloca,
    Getelementpointer,
    Load,
    Store,
    Ret,
    Icmp,
    Br,
    Add,
    Sub,
    Div,
    Mul,
    Phi,
    Call,
    Define,
    Label,
    Endbracket,
    Unkown,
}

#[derive(Clone, Eq, PartialEq)]
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
        formatter.write_str(&self.text)
    }
}
impl Expression {
    pub fn new(text: String, broad_type: BroadType, specific_type: SpecificType) -> Self {
        Expression {
            text,
            broad_type,
            specific_type,
        }
    }
    pub fn new_with_text(text: String) -> Self {
        let mut expression = Expression {
            text,
            broad_type: BroadType::Unkown,
            specific_type: SpecificType::Unkown,
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
        } else if self.text.trim().starts_with("br ") {
            self.specific_type = SpecificType::Br;
        } else if self.text.trim().ends_with("}") {
            self.specific_type = SpecificType::Endbracket;
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
            _ => self.broad_type = Other,
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
