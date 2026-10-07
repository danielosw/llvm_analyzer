use eecs_677_project_3::block::Block;
use eecs_677_project_3::processing::{process_function, process_ll};
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
    let path = format!("{}/test1.ll", env!("CARGO_MANIFEST_DIR"));

    let functions = process_ll(MAIN_LL.to_string()).expect("failed to process LLVM IR file");

    assert_eq!(functions.len(), 2);
    assert!(functions.contains_key("@main"));
    assert!(functions.contains_key("@function2"));

    let main_lines = &functions["@main"];
    assert!(
        main_lines
            .iter()
            .any(|line| line.contains("define i32 @main"))
    );
    assert!(main_lines.iter().any(|line| line.contains("ret i32 %var")));

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
