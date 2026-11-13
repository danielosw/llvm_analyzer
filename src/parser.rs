use crate::block::SpecificType::{self, Comment, Unkown};

fn without_register_prefix(value: &str) -> String {
    value
        .trim()
        .strip_prefix('%')
        .unwrap_or(value.trim())
        .to_string()
}

pub fn parse(line: String) -> Option<SpecificType> {
    let line: &str = line.trim();
    if line.is_empty() {
        return Some(Unkown);
    }
    let tokens: Vec<String> = line
        .split([' ', ',', '='])
        .filter(|token| !token.is_empty())
        .map(|f| f.to_string())
        .collect();
    let start_token = tokens.first()?.to_string();
    let length = tokens.len();
    if start_token == ";" {
        return Some(Comment);
    }
    if line == "}" {
        return Some(SpecificType::Endbracket);
    }
    if start_token == "ret" {
        return Some(SpecificType::Ret);
    }
    if start_token == "call" {
        // we know its the form call (?P<type>.*?)(?P<func>@.*?)\((?P<args>.*?)\)
        let func_start = line.find('@')?;
        let func_end = line[func_start..].find('(')? + func_start;
        let args_end = line.rfind(')')?;
        return Some(SpecificType::Call {
            func_name: line[func_start..func_end].to_string(),
            args: line[func_end + 1..args_end].to_string(),
            calltype: line["call".len()..func_start].trim().to_string(),
            sendto: None,
        });
    }
    if start_token == "br" {
        if tokens.get(1)? == "label" {
            return Some(SpecificType::Br {
                sendto1: tokens.get(2)?.to_string(),
                matchtype: None,
                checkvar: None,
                sendto2: None,
            });
        }
        return Some(SpecificType::Br {
            sendto1: tokens.get(4)?.to_string(),
            matchtype: Some(tokens.get(1)?.to_string()),
            checkvar: Some(tokens.get(2)?.to_string()),
            sendto2: Some(tokens.get(6)?.to_string()),
        });
    }
    //    r"^\s*define\s+(?P<type>\S+)\s+(?P<name>@[^(]+)\((?P<args>[^)]*)\)"
    if start_token == "define" {
        let func_start = line.find('@')?;
        let args_start = line[func_start..].find('(')? + func_start;
        let args_end = line.rfind(')')?;
        return {
            Some(SpecificType::Define {
                dtype: tokens.get(1)?.to_string(),
                name: line[func_start..args_start].to_string(),
                args: line[args_start + 1..args_end].to_string(),
            })
        };
    }
    if length == 1 && start_token.ends_with(":") {
        // r"^\s*(?P<name>.*?):
        return Some(SpecificType::Label {
            name: start_token.strip_suffix(":")?.to_owned(),
        });
    }
    //     r"store (?P<storetype>.*?) (?P<storefrom>.*?), {0,}ptr (?P<storeto>%.*?)$"
    if start_token == "store" {
        return Some(SpecificType::Store {
            store_type: tokens.get(1)?.to_string(),
            store_from: tokens.get(2)?.to_string(),
            store_to: tokens.get(length - 1)?.to_string(),
        });
    }

    //     r"((?P<sendto>%.*?){0,1} {0,}=){0,1} {0,}call (?P<type>.*?)(?P<func>@.*?)\((?P<args>.*?)\)"
    if start_token.starts_with("%") {
        if length >= 5 && tokens.get(1)? == "call" {
            let func_start = line.find('@')?;
            let func_end = line[func_start..].find('(')? + func_start;
            let args_end = line.rfind(')')?;
            let equals = line.find('=')?;
            return Some(SpecificType::Call {
                func_name: line[func_start..func_end].to_string(),
                args: line[func_end + 1..args_end].to_string(),
                calltype: line[equals + 1..func_start]
                    .trim()
                    .strip_prefix("call")?
                    .trim()
                    .to_string(),
                sendto: Some(tokens.first()?.to_string()),
            });
        }
        //    r"(?P<loadto>%.*?) {0,}= {0,}load (?P<type>.*?), {0,}ptr (?P<loadfrom>%.*?)$"

        if length >= 5 {
            // r"^\s*br (?:label (?P<sendto1>[^\s,]+)|(?P<matchtype>[^\s]+) (?P<checkvar>[^,\s]+), label (?P<sendto2>[^,\s]+), label (?P<sendto3>[^\s,]+))\s*$"

            if tokens.get(1)? == "load" {
                return Some(SpecificType::Load {
                    load_to: tokens.first()?.to_string(),
                    load_from: tokens.get(length - 1)?.to_string(),
                    load_type: tokens.get(3)?.to_string(),
                });
            }
            // r"%(?P<RegisterName>.*?)\s{0,}=\s{0,}(?P<OperatorType>.*?) (?P<NumberType>.*?) \s{0,}(?P<NumberA>[0-9]*?*?),\s{0,}(?P<NumberB>[0-9]*?*?)$"

            let numeric_tokens: Vec<&String> =
                tokens.iter().filter(|token| !token.is_empty()).collect();
            let potential_opperator = numeric_tokens.get(1)?.as_str();
            if potential_opperator == "add"
                || potential_opperator == "sub"
                || potential_opperator == "div"
                || potential_opperator == "mul"
            {
                return match potential_opperator {
                    "add" => Some(SpecificType::Add {
                        result_reg: without_register_prefix(numeric_tokens.first()?),
                        operand_a: numeric_tokens.get(3)?.to_string(),
                        operand_b: numeric_tokens.get(4)?.to_string(),
                        operand_type: numeric_tokens.get(2)?.to_string(),
                    }),
                    "sub" => Some(SpecificType::Sub {
                        result_reg: without_register_prefix(numeric_tokens.first()?),
                        operand_a: numeric_tokens.get(3)?.to_string(),
                        operand_b: numeric_tokens.get(4)?.to_string(),
                        operand_type: numeric_tokens.get(2)?.to_string(),
                    }),
                    "div" => Some(SpecificType::Div {
                        result_reg: without_register_prefix(numeric_tokens.first()?),
                        operand_a: numeric_tokens.get(3)?.to_string(),
                        operand_b: numeric_tokens.get(4)?.to_string(),
                        operand_type: numeric_tokens.get(2)?.to_string(),
                    }),
                    "mul" => Some(SpecificType::Mul {
                        result_reg: without_register_prefix(numeric_tokens.first()?),
                        operand_a: numeric_tokens.get(3)?.to_string(),
                        operand_b: numeric_tokens.get(4)?.to_string(),
                        operand_type: numeric_tokens.get(2)?.to_string(),
                    }),
                    _ => unreachable!(),
                };
            }
        }
        return Some(Unkown);
    }

    Some(Unkown)
}
