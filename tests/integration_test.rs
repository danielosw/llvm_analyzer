use llvm_analyzer::block::{Block, BroadType, SpecificType};
use llvm_analyzer::expression::Expression;
use llvm_analyzer::processing::{process_function, process_ll};
static MAIN_LL: &str = "define i32 @main(i32 %argc, i32 %argv) {
        %varT = add i32 1, 0
        %noArgs = icmp eq i32 %argc, 1
        br i1 %noArgs, label %lbl_t, label %lbl_f
lbl_t:
        %varT = add i32 1, 0
        br label %end
lbl_f:
        %varF = add i32 2, 0
        br label %end
end:
        %var = phi i32 [%varT, %lbl_t], [%varF, %lbl_f]
        ret i32 %var
}
define i32 @function2(i32 %argc) {
        %noArgs = icmp eq i32 %argc, 1
        br i1 %noArgs, label %lbl_t, label %lbl_f
lbl_t:
        %varT = add i32 1, 0
        br label %end
lbl_f:
        %varF = call i32(i32,i32) @main  (i32 1, i32 2)
        br label %end

end:
        %var = phi i32 [%varT, %lbl_t], [%varF, %lbl_f]
        ret i32 %var
}";
#[test]
fn process_ll_extracts_functions() {
    let functions = process_ll(MAIN_LL.to_string()).expect("failed to process LLVM IR file");

    assert_eq!(functions.len(), 2);
    assert!(functions.contains_key("@main"));
    assert!(functions.contains_key("@function2"));

    let main_lines = &functions["@main"];
    assert_eq!(
        main_lines.first().unwrap(),
        "define i32 @main(i32 %argc, i32 %argv) {"
    );
    assert_eq!(main_lines.get(1).unwrap(), "        %varT = add i32 1, 0");
    assert_eq!(main_lines.get(12).unwrap(), "        ret i32 %var");

    let function2_lines = &functions["@function2"];
    assert!(
        function2_lines
            .iter()
            .any(|line| line.contains("define i32 @function2"))
    );
    assert!(function2_lines.iter().any(|line| line.contains("@main")));
}
#[test]
fn process_function_extracts_blocks() {
    let functions = process_ll(MAIN_LL.to_string()).expect("failed to process LLVM IR file");
    let main_lines = &functions["@main"];
    let (blocks, main_name) = process_function(main_lines.clone(), 1);
    assert_eq!(blocks.len(), 4);
    assert_eq!(main_name, "main");
}

#[test]
fn expression_default_is_unknown() {
    let expression = Expression::default();

    assert_eq!(expression.broad_type, BroadType::Unkown);
    assert_eq!(expression.specific_type, SpecificType::Unkown);
    assert!(expression.text.is_empty());
}

#[test]
fn expression_constructor_preserves_values() {
    let expression = Expression::new(
        "ret i32 %value".to_string(),
        BroadType::End,
        SpecificType::Ret,
    );

    assert_eq!(expression.text, "ret i32 %value");
    assert_eq!(expression.broad_type, BroadType::End);
    assert_eq!(expression.specific_type, SpecificType::Ret);
}

#[test]
fn expression_classifies_llvm_text() {
    let cases = [
        (
            "define i32 @main() {",
            SpecificType::Define {
                name: "@main".to_string(),
                dtype: "i32".to_string(),
                args: "".to_string(),
            },
            BroadType::Start,
        ),
        (
            "entry:",
            SpecificType::Label {
                name: "entry".to_string(),
            },
            BroadType::Start,
        ),
        (
            "call i32 @main()",
            SpecificType::Call {
                func_name: "@main".to_string(),
                args: "".to_string(),
                calltype: "i32".to_string(),
                sendto: None,
            },
            BroadType::Call,
        ),
        ("ret i32 0", SpecificType::Ret, BroadType::End),
        ("}", SpecificType::Endbracket, BroadType::End),
        (
            "%value = add i32 1, 2",
            SpecificType::Add {
                result_reg: "value".to_string(),
                operand_a: "1".to_string(),
                operand_b: "2".to_string(),
                operand_type: "i32".to_string(),
            },
            BroadType::Numberic,
        ),
    ];

    for (text, specific_type, broad_type) in cases {
        let expression = Expression::new_with_text(text.to_string());

        assert_eq!(expression.specific_type, specific_type, "{text}");
        assert_eq!(expression.broad_type, broad_type, "{text}");
    }

    let branch = Expression::new_with_text("br label %end".to_string());
    assert!(matches!(branch.specific_type, SpecificType::Br { .. }));
    assert_eq!(branch.broad_type, BroadType::End);
}

#[test]
fn expression_display_returns_source_text() {
    let expression = Expression::new_with_text("ret i32 0".to_string());

    assert_eq!(expression.to_string(), "ret i32 0");
}

#[test]
fn expression_display_formats_arithmetic_expressions() {
    let cases = [
        (
            SpecificType::Add {
                result_reg: "result".to_string(),
                operand_a: "1".to_string(),
                operand_b: "2".to_string(),
                operand_type: "i32".to_string(),
            },
            "%result = add i32 1, 2",
        ),
        (
            SpecificType::Sub {
                result_reg: "result".to_string(),
                operand_a: "4".to_string(),
                operand_b: "3".to_string(),
                operand_type: "i32".to_string(),
            },
            "%result = sub i32 4, 3",
        ),
        (
            SpecificType::Div {
                result_reg: "result".to_string(),
                operand_a: "8".to_string(),
                operand_b: "2".to_string(),
                operand_type: "i32".to_string(),
            },
            "%result = div i32 8, 2",
        ),
        (
            SpecificType::Mul {
                result_reg: "result".to_string(),
                operand_a: "3".to_string(),
                operand_b: "5".to_string(),
                operand_type: "i32".to_string(),
            },
            "%result = mul i32 3, 5",
        ),
    ];

    for (specific_type, expected) in cases {
        let expression = Expression::new(
            "source text".to_string(),
            BroadType::Numberic,
            specific_type,
        );

        assert_eq!(expression.to_string(), expected);
    }
}

#[test]
fn expression_display_formats_parsed_arithmetic_expression() {
    let expression = Expression::new_with_text("%value = add i32 1, 2".to_string());

    assert_eq!(expression.to_string(), "%value = add i32 1, 2");
}

#[test]
fn main_ll_instructions_format_correctly_after_processing() {
    let functions = process_ll(MAIN_LL.to_string()).expect("failed to process LLVM IR file");
    let expected: Vec<String> = MAIN_LL
        .lines()
        .filter(|line| {
            let instruction = line.trim();
            !instruction.is_empty() && instruction != "}"
        })
        .map(|line| line.trim().to_string())
        .collect();
    let mut formatted = Vec::new();

    for function_name in ["@main", "@function2"] {
        let (blocks, _) = process_function(functions[function_name].clone(), 1);
        let mut blocks: Vec<_> = blocks.into_values().collect();
        blocks.sort();

        for expression in blocks.into_iter().flat_map(|block| block.lines) {
            if !expression.text.trim().is_empty() {
                formatted.push(expression.to_string().trim().to_string());
            }
        }
    }

    assert_eq!(formatted, expected);
}

#[test]
fn block_stores_and_returns_expression_text() {
    let block = Block {
        name: "entry".to_string(),
        lines: vec![
            Expression::new_with_text("entry:".to_string()),
            Expression::new_with_text("ret i32 0".to_string()),
        ],
        connections: vec!["exit".to_string()],
        number: 3,
        entry: true,
        refrenced: Vec::new(),
        setvars: Vec::new(),
    };

    assert_eq!(block.get_lines_vec(), vec!["entry:", "ret i32 0"]);
    assert_eq!(block.connections, vec!["exit"]);
    assert_eq!(block.number, 3);
}

#[test]
fn block_default_is_empty() {
    let block = Block::new();

    assert!(block.name.is_empty());
    assert!(block.lines.is_empty());
    assert!(block.connections.is_empty());
    assert_eq!(block.number, 0);
}

#[test]
fn blocks_are_ordered_by_number() {
    // generate a bunch of block with random numbers and make sure they sort correctly
    let mut blocks: Vec<Block> = Vec::new();
    for i in (0..10).rev() {
        let block = Block {
            name: format!("block{}", i),
            lines: vec![Expression::new_with_text(format!("line{}", i))],
            connections: vec![format!("block{}", i + 1)],
            number: i - 1,
            entry: false,
            setvars: Vec::new(),
            refrenced: Vec::new(),
        };
        blocks.push(block);
    }

    blocks.sort();
    assert!(blocks.windows(2).all(|w| w[0].number <= w[1].number));
}
