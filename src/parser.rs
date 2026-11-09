use std::collections::VecDeque;

use crate::block::SpecificType::{self, Comment, Unkown};
use crate::expression::Expression;

fn without_register_prefix(value: &str) -> String {
    value
        .trim()
        .strip_prefix('%')
        .unwrap_or(value.trim())
        .to_string()
}

pub fn parse(line: String) -> Option<SpecificType> {
    let line: &str = line.trim();
    let tokens: Vec<String> = line.split([' ', ',', '=']).map(|f| f.to_string()).collect();
    let startToken = tokens.get(0)?.to_string();
    let length = tokens.len();
    if startToken == ";" {
        return Some(Comment);
    }
    if (line == "}") {
        return Some(SpecificType::Endbracket);
    }
    if startToken == "ret" {
        return Some(SpecificType::Ret);
    }
    if startToken == "call" {
        // we know its the form call (?P<type>.*?)(?P<func>@.*?)\((?P<args>.*?)\)
        return Some(SpecificType::Call {
            func_name: tokens.get(2)?.to_string(),
            args: tokens.get(3)?.to_string(),
            calltype: tokens.get(1)?.to_string(),
            sendto: None,
        });
    }

    Some(Unkown)
}
