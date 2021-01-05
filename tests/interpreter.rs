use pakhi::frontend::{lexer, parser};
use pakhi::frontend::parser::Stmt;
use pakhi::common::io::{MockIO, IO};
use pakhi::backend::interpreter::Interpreter;

fn src_to_ast(src_lines: Vec<&str>) -> Vec<Stmt> {
    let src: String = src_lines.join("\n");
    let src_chars: Vec<char> = src.chars().collect();
    let tokens = lexer::tokenize(src_chars);
    parser::parse("test.pakhi".to_string(), tokens)
}

fn run_assert_all_true(ast: Vec<Stmt>, mut mock_io: MockIO) {
    let mut interpreter = Interpreter::new(ast, &mut mock_io);
    interpreter.run();
    assert!(mock_io.assert_all_true());
}

#[test]
fn println_test() {
    let ast = src_to_ast(vec![
        "দেখাও ০;",
        "দেখাও ০;",
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("০");
    mock_io.expect_println("০");
    run_assert_all_true(ast, mock_io);
}

#[test]
fn print_test() {
    let ast = src_to_ast(vec![
        "_দেখাও ০;",
        "_দেখাও ০;",
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_print("০");
    mock_io.expect_print("০");
    run_assert_all_true(ast, mock_io);
}

#[test]
fn var_decl_num() {
    let ast = src_to_ast(vec![
        "নাম ক = ১;",
        "দেখাও ক;",
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("১");
    run_assert_all_true(ast, mock_io);
}

#[test]
fn var_decl_string() {
    let ast = src_to_ast(vec![
        r#"নাম ক  = "testing";"#,
        "দেখাও ক;"
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("testing");
    run_assert_all_true(ast, mock_io);
}

#[test]
fn list_single_dim_indexing() {
    let ast = src_to_ast(vec![
        "নাম ক = [১, ২, ৩];",
        "দেখাও ক[১];"
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("২");
    run_assert_all_true(ast, mock_io);
}

#[test]
fn list_multi_dim_indexing() {
    let ast = src_to_ast(vec![
        "নাম ক = [১, [১, ২, ৩], ৩];",
        "দেখাও ক[১][১];"
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("২");
    run_assert_all_true(ast, mock_io);
}

#[test]
fn list_multi_dim_mixed_indexing() {
    let ast = src_to_ast(vec![
        r#"নাম ক = [১, ২, ৩, @{"key" -> [১,২], "key_2" -> ৪,}];"#,
        r#"দেখাও ক[৩]["key"][০];"#,
        r#"দেখাও ক[৩]["key_2"];"#,
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("১");
    mock_io.expect_println("৪");
    run_assert_all_true(ast, mock_io);
}

#[test]
fn list_mutate_push() {
    let ast = src_to_ast(vec![
        "নাম ক = [১, ২, ৩];",
        "_লিস্ট-পুশ(ক, ৪);",
        "দেখাও ক[৩];"
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("৪");
    run_assert_all_true(ast, mock_io);
}

#[test]
fn list_push_middle() {
    let ast = src_to_ast(vec![
        "নাম ক = [১, ২, ৩];",
        "_লিস্ট-পুশ(ক, ১, ৪);",
        "দেখাও ক[১];"
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("৪");
    run_assert_all_true(ast, mock_io);
}

#[test]
fn list_pop_middle() {
    let ast = src_to_ast(vec![
        "নাম ক = [১, ২, ৩];",
        "_লিস্ট-পপ(ক, ১);",
        "দেখাও ক[১];"
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("৩");
    run_assert_all_true(ast, mock_io);
}

#[test]
fn list_mutate() {
    let ast = src_to_ast(vec![
        "নাম ক = [১, ২, ৩];",
        "ক[১] = ৫;",
        "দেখাও ক[১];"
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("৫");
    run_assert_all_true(ast, mock_io);
}

#[test]
fn list_consistent() {
    let ast = src_to_ast(vec![
        "নাম ক = [১, ২, ৩];",
        "নাম খ = ক;",
        "ক[১] = ২০;",
        "দেখাও খ[১];",
        "খ[১] = ৩০;",
        "দেখাও ক[১];"
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("২০");
    mock_io.expect_println("৩০");
    run_assert_all_true(ast, mock_io);
}

#[test]
fn nameless_record_literal() {
    let ast = src_to_ast(vec![
        "নাম ক =  @{",
        "\"key\" -> ১,",
        "\"key\" -> ১ + ১,",
        "};",
        "দেখাও ক;",
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_print("@{");
    mock_io.expect_print("\"key\":");
    mock_io.expect_print("২");
    mock_io.expect_print(",");
    mock_io.expect_println("}");
    run_assert_all_true(ast, mock_io);
}

#[test]
fn nameless_record_single_dim_indexing() {
    let ast = src_to_ast(vec![
        "নাম ক =  @{",
        "\"key\" -> ১,",
        "\"key\" -> ১ + ১,",
        "};",
        r#"ক["key"] = "string";"#,
        r#"দেখাও ক["key"];"#,
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("string");
    run_assert_all_true(ast, mock_io);
}

#[test]
fn nameless_record_multi_dim_indexing() {
    let ast = src_to_ast(vec![
        "নাম ক =  @{",
        "\"key\" -> @{\"key_2\" -> \"string\",},",
        "};",
        r#"দেখাও ক["key"]["key_2"];"#,
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("string");
    run_assert_all_true(ast, mock_io);
}

#[test]
fn nameless_record_multi_dim_mixed_indexing() {
    let ast = src_to_ast(vec![
        r#"নাম ক = @{"key" -> [১, ২, ৩, @{"key" -> ১,}],};"#,
        r#"দেখাও ক["key"][২];"#,
        r#"দেখাও ক["key"][৩]["key"];"#,
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("৩");
    mock_io.expect_println("১");
    run_assert_all_true(ast, mock_io);
}

#[test]
fn expression_unary() {
    let ast = src_to_ast(vec![
        "নাম ক = ১;",
        "নাম খ = -১;",
        "দেখাও -ক;",
        "দেখাও -খ;",
        "দেখাও !সত্য;",
        "দেখাও !মিথ্যা;",
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("-১");
    mock_io.expect_println("১");
    mock_io.expect_println("মিথ্যা");
    mock_io.expect_println("সত্য");
    run_assert_all_true(ast, mock_io);
}

#[test]
fn expression_and() {
    let ast = src_to_ast(vec![
        "দেখাও মিথ্যা & মিথ্যা;",
        "দেখাও মিথ্যা & সত্য;",
        "দেখাও সত্য & মিথ্যা ;",
        "দেখাও সত্য & সত্য;",
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("মিথ্যা");
    mock_io.expect_println("মিথ্যা");
    mock_io.expect_println("মিথ্যা");
    mock_io.expect_println("সত্য");
    run_assert_all_true(ast, mock_io);
}

#[test]
fn expression_or() {
    let ast = src_to_ast(vec![
        "দেখাও মিথ্যা | মিথ্যা;",
        "দেখাও মিথ্যা | সত্য;",
        "দেখাও সত্য | মিথ্যা ;",
        "দেখাও সত্য | সত্য;",
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("মিথ্যা");
    mock_io.expect_println("সত্য");
    mock_io.expect_println("সত্য");
    mock_io.expect_println("সত্য");
    run_assert_all_true(ast, mock_io);
}

#[test]
fn expression_equlaity() {
    let ast = src_to_ast(vec![
        "দেখাও মিথ্যা == মিথ্যা;",
        "দেখাও মিথ্যা != সত্য;",
        "দেখাও সত্য == মিথ্যা ;",
        "দেখাও সত্য != সত্য;",
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("সত্য");
    mock_io.expect_println("সত্য");
    mock_io.expect_println("মিথ্যা");
    mock_io.expect_println("মিথ্যা");
    run_assert_all_true(ast, mock_io);
}

#[test]
fn if_test_condition_true() {
    let ast = src_to_ast(vec![
        "যদি সত্য {",
        "   দেখাও ০;",
        "}",
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("০");
    run_assert_all_true(ast, mock_io);
}

#[test]
fn if_test_condition_false() {
    let ast = src_to_ast(vec![
        "যদি মিথ্যা {",
        "   দেখাও ১;",
        "} অথবা {",
        "   দেখাও ০;",
        "}",
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("০");
    run_assert_all_true(ast, mock_io);
}

#[test]
fn loop_test() {
    let ast = src_to_ast(vec![
        "নাম ক = ০;",
        "লুপ {",
        "   দেখাও ১;",
        "   ক = ক + ১;",
        "   যদি ক >= ৩ {",
        "       থামাও;",
        "   }",
        "} আবার;"
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("১");
    mock_io.expect_println("১");
    mock_io.expect_println("১");
    run_assert_all_true(ast, mock_io);
}

#[test]
fn function_decl_call() {
    let ast = src_to_ast(vec![
        "ফাং দ্বিগুন(সংখ্যা) {",
        "   ফেরত সংখ্যা * ২;",
        "} ফেরত;",
        "দেখাও দ্বিগুন(২);"
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("৪");
    run_assert_all_true(ast, mock_io);
}