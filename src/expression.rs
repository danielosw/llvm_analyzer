use crate::block::BroadType::Other;
use regex::{self, Regex};
use std::fmt;
use std::sync::LazyLock;

use crate::block::SpecificType;
macro_rules! make_static {
    ($expr:expr) => {{ LazyLock::new(|| $expr) }};
}
macro_rules! define_regex {
    ($name:ident, $text:expr) => {
        static $name: LazyLock<regex::Regex> = make_static!({ Regex::new($text).ok().unwrap() });
    };
}
define_regex!(REGEX_END_BRACKET, r"^\s*}\s*$");

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

use crate::block::BroadType;

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
        match self.specific_type {
            SpecificType::Br { .. }
            | SpecificType::Endbracket
            | SpecificType::Define { .. }
            | SpecificType::Label { .. }
            | SpecificType::Add { .. }
            | SpecificType::Sub { .. }
            | SpecificType::Div { .. }
            | SpecificType::Mul { .. } => {
                // Use the specific tpye fmt implementation
                write!(formatter, "{}", self.specific_type)
            }
            _ => write!(formatter, "{}", self.text),
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
            let captures = REGEX_DEFINE.captures(&self.text).unwrap();
            self.specific_type = SpecificType::Define {
                dtype: captures.name("type").unwrap().as_str().to_string(),
                name: captures.name("name").unwrap().as_str().to_string(),
                args: captures.name("args").unwrap().as_str().to_string(),
            };
        } else if REGEX_BLOCK_LABEL.is_match(&self.text) {
            let captures = REGEX_BLOCK_LABEL.captures(&self.text).unwrap();
            self.specific_type = SpecificType::Label {
                name: captures.name("name").unwrap().as_str().to_string(),
            };
        } else if REGEX_RETURN.is_match(&self.text) {
            self.specific_type = SpecificType::Ret;
        } else if REGEX_BR.is_match(&self.text) {
            let captures = REGEX_NEW_BR.captures(&self.text).unwrap();
            if let Some(sendto1) = captures.name("sendto1") {
                // Handle the case where there's a simple branch to a label
                self.specific_type = SpecificType::Br {
                    sendto1: sendto1.as_str().to_string(),
                    matchtype: None,
                    checkvar: None,
                    sendto2: None,
                };
            } else if let Some(matchtype) = captures.name("matchtype") {
                // Handle the case where there's a conditional branch
                self.specific_type = SpecificType::Br {
                    sendto1: captures.name("sendto2").unwrap().as_str().to_string(),
                    matchtype: Some(matchtype.as_str().to_string()),
                    checkvar: captures.name("checkvar").map(|v| v.as_str().to_string()),
                    sendto2: captures.name("sendto3").map(|v| v.as_str().to_string()),
                };
            }
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
            SpecificType::Define { .. } | SpecificType::Label { .. } => {
                self.broad_type = BroadType::Start
            }
            SpecificType::Ret | SpecificType::Br { .. } => self.broad_type = BroadType::End,
            SpecificType::Endbracket => self.broad_type = BroadType::End,
            SpecificType::Add { .. }
            | SpecificType::Div { .. }
            | SpecificType::Mul { .. }
            | SpecificType::Sub { .. } => self.broad_type = BroadType::Numberic,

            _ => self.broad_type = Other,
        }
    }
}
