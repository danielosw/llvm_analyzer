use crate::block::BroadType::Other;
use crate::parser::parse;
use std::fmt;

use crate::block::BroadType;
use crate::block::SpecificType;

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
            | SpecificType::Mul { .. }
            | SpecificType::Call { .. }
            | SpecificType::Load { .. }
            | SpecificType::Store { .. } => {
                // Use the specific tpye fmt implementation
                write!(formatter, "{}", self.specific_type)
            }
            _ => write!(formatter, "{}", self.text),
        }
    }
}

impl Expression {
    // Retuns Some(callname, args) else None
    pub fn getcall(self) -> Option<(String, String)> {
        match self.specific_type {
            SpecificType::Call {
                func_name,
                args,
                calltype: _,
                sendto: _,
            } => Some((func_name, args)),
            _ => None,
        }
    }
    // Returns Some((destination, source)) if an assignment, else None
    pub fn assignment(self) -> Option<(String, String)> {
        match self.specific_type {
            SpecificType::Call {
                func_name,
                args: _,
                calltype: _,
                sendto,
            } => {
                if let Some(realto) = sendto {
                    return Some((realto, func_name));
                }
                None
            }
            SpecificType::Load {
                load_to,
                load_from,
                load_type: _,
            } => Some((load_to, load_from)),
            SpecificType::Store {
                store_type: _,
                store_from,
                store_to,
            } => Some((store_to, store_from)),
            _ => None,
        }
    }
    pub fn new(text: String, broad_type: BroadType, specific_type: SpecificType) -> Self {
        Expression {
            text,
            broad_type,
            specific_type,
        }
    }
    pub fn new_with_text(text: String) -> Self {
        let spec = parse(text.clone());
        let realspec = spec.expect("Failed to procces text");
        let mut expression = Expression {
            text,
            broad_type: BroadType::Unkown,
            specific_type: realspec,
        };
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
    // only call after the specific type is populated
    pub fn guess_general_type(&mut self) {
        match self.specific_type {
            SpecificType::Call { .. } => self.broad_type = BroadType::Call,
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
