use eecs_677_project_3::block::{Block, BroadType, Expression, SpecificType};
use eecs_677_project_3::processing::{process_function, process_ll};
use std::cmp::Ordering;
static MAIN_LL: &str = "define i32 @main(i32 %argc) {
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
    assert_eq!(main_lines.first().unwrap(), "define i32 @main(i32 %argc) {");
    assert_eq!(main_lines.get(11).unwrap(), "        ret i32 %var");

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
    let (blocks, main_name) = process_function(main_lines.clone());
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
            SpecificType::Define,
            BroadType::Start,
        ),
        ("entry:", SpecificType::Label, BroadType::Start),
        ("call i32 @main()", SpecificType::Call, BroadType::Call),
        ("br label %end", SpecificType::Br, BroadType::End),
        ("ret i32 0", SpecificType::Ret, BroadType::End),
        ("}", SpecificType::Endbracket, BroadType::End),
        (
            "%value = add i32 1, 2",
            SpecificType::Unkown,
            BroadType::Other,
        ),
    ];

    for (text, specific_type, broad_type) in cases {
        let expression = Expression::new_with_text(text.to_string());

        assert_eq!(expression.specific_type, specific_type, "{text}");
        assert_eq!(expression.broad_type, broad_type, "{text}");
    }
}

#[test]
fn expression_display_returns_source_text() {
    let expression = Expression::new_with_text("ret i32 0".to_string());

    assert_eq!(expression.to_string(), "ret i32 0");
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
        };
        blocks.push(block);
    }

    blocks.sort();
    assert!(blocks.windows(2).all(|w| w[0].number <= w[1].number));
}
