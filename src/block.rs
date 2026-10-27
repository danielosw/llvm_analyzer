use crate::block::BroadType::Other;
use crate::expression;
use regex::{self, Regex};
use std::cmp::Ordering;
use std::sync::LazyLock;
macro_rules! make_static {
    ($expr:expr) => {{ LazyLock::new(|| $expr) }};
}
macro_rules! define_regex {
    ($name:ident, $text:expr) => {
        static $name: LazyLock<regex::Regex> = make_static!({ Regex::new($text).ok().unwrap() });
    };
}
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

define_regex!(
    REGEX_NUMERIC,
    r"%(?P<RegisterName>.*?)\s{0,}=\s{0,}(?P<OperatorType>.*?) (?P<NumberType>.*?) \s{0,}(?P<NumberA>[0-9]*?*?),\s{0,}(?P<NumberB>[0-9]*?*?)$"
);
define_regex!(REGEX_BR, r"\bbr.*?\b");
define_regex!(
    REGEX_NEW_BR,
    r"^\s*br (?:label (?P<sendto1>[^\s,]+)|(?P<matchtype>[^\s]+) (?P<checkvar>[^,\s]+), label (?P<sendto2>[^,\s]+), label (?P<sendto3>[^\s,]+))\s*$"
);
define_regex!(
    REGEX_DEFINE,
    r"^\s*define\s+(?P<type>\S+)\s+(?P<name>@[^(]+)\((?P<args>[^)]*)\)"
);
define_regex!(REGEX_BLOCK_LABEL, r"^\s*(?P<name>.*?):");
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
    r"%(?P<outvar>.*?) {0,}= {0,}call (?P<calltype>.*?) @(:?P<funcname>.*?)\b"
);
define_regex!(
    REGEX_SINK,
    r"\bcall (?P<calltype>.*?) @(:?P<funcname>.*?)\(.*? (?P<varname>.*?)\)"
);
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
    Call,
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
impl SpecificType {
    pub fn arithmetic_parts(&self) -> Option<(&'static str, &str, &str, &str, &str)> {
        match self {
            Self::Add {
                result_reg,
                operand_a,
                operand_b,
                operand_type,
            } => Some(("add", result_reg, operand_type, operand_a, operand_b)),
            Self::Sub {
                result_reg,
                operand_a,
                operand_b,
                operand_type,
            } => Some(("sub", result_reg, operand_type, operand_a, operand_b)),
            Self::Div {
                result_reg,
                operand_a,
                operand_b,
                operand_type,
            } => Some(("div", result_reg, operand_type, operand_a, operand_b)),
            Self::Mul {
                result_reg,
                operand_a,
                operand_b,
                operand_type,
            } => Some(("mul", result_reg, operand_type, operand_a, operand_b)),
            _ => None,
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
