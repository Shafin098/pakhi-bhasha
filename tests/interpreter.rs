use pakhi::frontend::{lexer, parser};
use pakhi::frontend::parser::Stmt;
use pakhi::common::io::{MockIO, IO};
use pakhi::backend::interpreter::Interpreter;
use pakhi::common::pakhi_error::PakhiErr;

fn src_to_ast(src_lines: Vec<&str>) -> Vec<Stmt> {
    let src: String = src_lines.join("\n");
    let src_chars: Vec<char> = src.chars().collect();
    let tokens = lexer::tokenize(src_chars, "test.pakhi".to_string()).unwrap();
    match parser::parse("test.pakhi".to_string(), tokens) {
        Ok(ast) => return ast,
        Err(e) => panic!("{:?}", e),
    }
}

fn run_assert_all_true(ast: Vec<Stmt>, mut mock_io: MockIO) -> Result<(), PakhiErr> {
    let mut interpreter = Interpreter::new(ast, &mut mock_io);
    interpreter.run()?;
    mock_io.assert_all_true();
    Ok(())
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
    if let Err(err) = run_assert_all_true(ast, mock_io) {
        panic!("{:?}", err);
    }
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
    if let Err(err) = run_assert_all_true(ast, mock_io) {
        panic!("{:?}", err);
    }
}

#[test]
fn var_decl_num() {
    let ast = src_to_ast(vec![
        "নাম ক = ১;",
        "দেখাও ক;",
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("১");
    if let Err(err) = run_assert_all_true(ast, mock_io) {
        panic!("{:?}", err);
    }
}

#[test]
fn var_decl_string() {
    let ast = src_to_ast(vec![
        r#"নাম ক  = "testing";"#,
        "দেখাও ক;"
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("testing");
    if let Err(err) = run_assert_all_true(ast, mock_io) {
        panic!("{:?}", err);
    }
}

#[test]
fn list_single_dim_indexing() {
    let ast = src_to_ast(vec![
        "নাম ক = [১, ২, ৩];",
        "দেখাও ক[১];"
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("২");
    if let Err(err) = run_assert_all_true(ast, mock_io) {
        panic!("{:?}", err);
    }
}

#[test]
fn list_multi_dim_indexing() {
    let ast = src_to_ast(vec![
        "নাম ক = [১, [১, ২, ৩], ৩];",
        "দেখাও ক[১][১];"
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("২");
    if let Err(err) = run_assert_all_true(ast, mock_io) {
        panic!("{:?}", err);
    }
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
    if let Err(err) = run_assert_all_true(ast, mock_io) {
        panic!("{:?}", err);
    }
}

#[test]
fn built_in_fn_list_mutate_push() {
    let ast = src_to_ast(vec![
        "নাম ক = [১, ২, ৩];",
        "_লিস্ট-পুশ(ক, ৪);",
        "দেখাও ক[৩];"
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("৪");
    if let Err(err) = run_assert_all_true(ast, mock_io) {
        panic!("{:?}", err);
    }
}

#[test]
fn built_in_fn_list_push_middle() {
    let ast = src_to_ast(vec![
        "নাম ক = [১, ২, ৩];",
        "_লিস্ট-পুশ(ক, ১, ৪);",
        "দেখাও ক[১];"
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("৪");
    if let Err(err) = run_assert_all_true(ast, mock_io) {
        panic!("{:?}", err);
    }
}

#[test]
fn built_in_fn_list_pop_middle() {
    let ast = src_to_ast(vec![
        "নাম ক = [১, ২, ৩];",
        "_লিস্ট-পপ(ক, ১);",
        "দেখাও ক[১];"
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("৩");
    if let Err(err) = run_assert_all_true(ast, mock_io) {
        panic!("{:?}", err);
    }
}

#[test]
fn built_in_fn_list_len() {
    let ast = src_to_ast(vec![
        "নাম ক = [১, ২, ৩];",
        "দেখাও _লিস্ট-লেন(ক);",
        "নাম ক = [];",
        "দেখাও _লিস্ট-লেন(ক);",
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("৩");
    mock_io.expect_println("০");
    if let Err(err) = run_assert_all_true(ast, mock_io) {
        panic!("{:?}", err);
    }
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
    if let Err(err) = run_assert_all_true(ast, mock_io) {
        panic!("{:?}", err);
    }
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
    if let Err(err) = run_assert_all_true(ast, mock_io) {
        panic!("{:?}", err);
    }
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
    if let Err(err) = run_assert_all_true(ast, mock_io) {
        panic!("{:?}", err);
    }
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
    if let Err(err) = run_assert_all_true(ast, mock_io) {
        panic!("{:?}", err);
    }
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
    if let Err(err) = run_assert_all_true(ast, mock_io) {
        panic!("{:?}", err);
    }
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
    if let Err(err) = run_assert_all_true(ast, mock_io) {
        panic!("{:?}", err);
    }
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
    if let Err(err) = run_assert_all_true(ast, mock_io) {
        panic!("{:?}", err);
    }
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
    if let Err(err) = run_assert_all_true(ast, mock_io) {
        panic!("{:?}", err);
    }
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
    if let Err(err) = run_assert_all_true(ast, mock_io) {
        panic!("{:?}", err);
    }
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
    if let Err(err) = run_assert_all_true(ast, mock_io) {
        panic!("{:?}", err);
    }
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
    if let Err(err) = run_assert_all_true(ast, mock_io) {
        panic!("{:?}", err);
    }
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
    if let Err(err) = run_assert_all_true(ast, mock_io) {
        panic!("{:?}", err);
    }
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
    if let Err(err) = run_assert_all_true(ast, mock_io) {
        panic!("{:?}", err);
    }
}

#[test]
fn loop_no_new_env() {
    let ast = src_to_ast(vec![
        "লুপ {",
        "   দেখাও ১;",
        "   থামাও;",
        "} আবার;"
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("১");
    if let Err(err) = run_assert_all_true(ast, mock_io) {
        panic!("{:?}", err);
    }
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
    if let Err(err) = run_assert_all_true(ast, mock_io) {
        panic!("{:?}", err);
    }
}

#[test]
fn recursive_function_call() {
    let ast = src_to_ast(vec![
        "ফাং রি(ক) {",
        "    যদি ক > ৪ {",
        "        ফেরত ক;",
        "    }",
        "    দেখাও ক;",
        "    রি(ক + ১);",
        "} ফেরত;",
        "রি(০);",
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("০");
    mock_io.expect_println("১");
    mock_io.expect_println("২");
    mock_io.expect_println("৩");
    mock_io.expect_println("৪");
    if let Err(err) = run_assert_all_true(ast, mock_io) {
        panic!("{:?}", err);
    }
}

#[test]
#[should_panic]
fn built_in_fn_error() {
    let ast = src_to_ast(vec![
        r#"_এরর("এরর হয়েছে");"#,
        r#"দেখাও "দেখাবেনা";"#,
    ]);

    let mock_io: MockIO = MockIO::new();
    if let Err(err) = run_assert_all_true(ast, mock_io) {
        panic!("{:?}", err);
    }
}

#[test]
fn built_in_fn_string_split() {
    let ast = src_to_ast(vec![
        r#"নাম স্প্লিট = _স্ট্রিং-স্প্লিট("স্ট্রিং স্প্লিট", " ");"#,
        r#"দেখাও স্প্লিট[০];"#,
        r#"দেখাও স্প্লিট[১];"#,
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("স্ট্রিং");
    mock_io.expect_println("স্প্লিট");
    if let Err(err) = run_assert_all_true(ast, mock_io) {
        panic!("{:?}", err);
    }
}

#[test]
fn built_in_fn_string_join() {
    let ast = src_to_ast(vec![
        r#"নাম স্প্লিট = ["স্ট্রিং", "স্প্লিট"];"#,
        r#"দেখাও _স্ট্রিং-জয়েন(স্প্লিট, "-");"#,
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("স্ট্রিং-স্প্লিট");
    if let Err(err) = run_assert_all_true(ast, mock_io) {
        panic!("{:?}", err);
    }
}

#[test]
fn built_in_fn_type() {
    let ast = src_to_ast(vec![
        r#"দেখাও _টাইপ(১);"#,
        r#"দেখাও _টাইপ(মিথ্যা);"#,
        r#"দেখাও _টাইপ("১");"#,
        r#"দেখাও _টাইপ([১]);"#,
        r#"দেখাও _টাইপ(@{"১" -> ১,});"#,
        r#"নাম ক;"#,
        r#"দেখাও _টাইপ(ক);"#,
        r#"ফাং খ() {"#,
        r#"} ফেরত;"#,
        r#"দেখাও _টাইপ(খ);"#,
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("_সংখ্যা");
    mock_io.expect_println("_বুলিয়ান");
    mock_io.expect_println("_স্ট্রিং");
    mock_io.expect_println("_লিস্ট");
    mock_io.expect_println("_রেকর্ড");
    mock_io.expect_println("_শূন্য");
    mock_io.expect_println("_ফাং");
    if let Err(err) = run_assert_all_true(ast, mock_io) {
        panic!("{:?}", err);
    }
}

#[test]
fn built_in_fn_to_string() {
    let ast = src_to_ast(vec![
        r#"দেখাও _স্ট্রিং(১) == "১";"#,
        r#"দেখাও _স্ট্রিং(১.০) == "১";"#,
        r#"দেখাও _স্ট্রিং(-১.০) == "-১";"#,
        r#"দেখাও _স্ট্রিং(১৩.৩২) == "১৩.৩২";"#,
        r#"দেখাও _স্ট্রিং(-৪৩.৪৩) == "-৪৩.৪৩";"#,
        r#"দেখাও _স্ট্রিং(-০.৪৩) == "-০.৪৩";"#,
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("সত্য");
    mock_io.expect_println("সত্য");
    mock_io.expect_println("সত্য");
    mock_io.expect_println("সত্য");
    mock_io.expect_println("সত্য");
    mock_io.expect_println("সত্য");
    if let Err(err) = run_assert_all_true(ast, mock_io) {
        panic!("{:?}", err);
    }
}

#[test]
fn built_in_fn_to_num() {
    let ast = src_to_ast(vec![
        r#"দেখাও _সংখ্যা("১") == ১;"#,
        r#"দেখাও _সংখ্যা("১.০") == ১;"#,
        r#"দেখাও _সংখ্যা("-১.০") == -১;"#,
        r#"দেখাও _সংখ্যা("১৩.৩২") == ১৩.৩২;"#,
        r#"দেখাও _সংখ্যা("-৪৩.৪৩") == -৪৩.৪৩;"#,
        r#"দেখাও _সংখ্যা("-০.৪৩") == -০.৪৩;"#,
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("সত্য");
    mock_io.expect_println("সত্য");
    mock_io.expect_println("সত্য");
    mock_io.expect_println("সত্য");
    mock_io.expect_println("সত্য");
    mock_io.expect_println("সত্য");
    if let Err(err) = run_assert_all_true(ast, mock_io) {
        panic!("{:?}", err);
    }
}

#[test]
fn built_in_const_platform() {
    let ast = src_to_ast(vec![
        r#"দেখাও _প্ল্যাটফর্ম;"#,
    ]);
    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println(std::env::consts::OS);
    if let Err(err) = run_assert_all_true(ast, mock_io) {
        panic!("{:?}", err);
    }
}