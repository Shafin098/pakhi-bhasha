use pakhi::common::io::{MockIO, IO};
use std::io::Write;
use serial_test::serial;

fn create_file(file_name: &str, lines: Vec<&str>) {
    let current_dir = std::env::current_dir().unwrap();
    let tmp_dir = current_dir.join("tmp");
    std::fs::create_dir_all(&tmp_dir).unwrap();
    let mut file = std::fs::File::create(tmp_dir.join(file_name)).unwrap();
    let l: String = lines.join("\n");
    file.write_all(l.as_bytes()).unwrap()
}

fn run_module(module_name: &str, mut io: MockIO) {
    let root_path = std::env::current_dir().unwrap().join("tmp");
    let module_path = root_path.join(module_name);
    pakhi::start_pakhi(module_path.to_str().unwrap().parse().unwrap(), &mut io);
    clean_test_tmp_dir();
    io.assert_all_true();
}

fn clean_test_tmp_dir() {
    let current_dir = std::env::current_dir().unwrap();
    let tmp_dir = current_dir.join("tmp");
    std::fs::remove_dir_all(tmp_dir).unwrap()
}

#[test]
#[serial]
fn module_import() {
    create_file("test.pakhi", vec![
        r#"মডিউল ম = "module.pakhi";"#,
        "দেখাও ম/ক;",
    ]);
    create_file("module.pakhi", vec![
        "নাম ক = ২;",
        "দেখাও ক;",
    ]);

    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("২");
    mock_io.expect_println("২");
    run_module("test.pakhi", mock_io);
}

#[test]
#[serial]
#[should_panic(expected="Cyclic module dependency. Can't import root.pakhi from module.pakhi")]
fn module_import_cyclic() {
    create_file("root.pakhi", vec![
        r#"মডিউল ম = "module.pakhi";"#,
    ]);
    create_file("module.pakhi", vec![
        r#"মডিউল ম = "root.pakhi";"#,
    ]);

    let mock_io: MockIO = MockIO::new();
    run_module("root.pakhi", mock_io);
}