use pakhi::common::io::{MockIO, IO};
use std::io::Write;
use std::sync::{Arc, PoisonError};
use std::sync::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    static ref MUTEX: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
}

// create_file creates file ./tmp folder
fn create_file(file_name: &str, lines: Vec<&str>) {
    let current_dir = std::env::current_dir().unwrap();
    let tmp_dir = current_dir.join("__tmp");
    std::fs::create_dir_all(&tmp_dir).unwrap();
    let mut file = std::fs::File::create(tmp_dir.join(file_name)).unwrap();
    let l: String = lines.join("\n");
    file.write_all(l.as_bytes()).unwrap()
}

fn run_module(module_name: &str, mut io: MockIO) {
    let root_path = std::env::current_dir().unwrap().join("__tmp");
    let module_path = root_path.join(module_name);
    pakhi::start_pakhi(module_path.to_str().unwrap().parse().unwrap(), &mut io);
    clean_test_tmp_dir();
    io.assert_all_true();
}

fn clean_test_tmp_dir() {
    let current_dir = std::env::current_dir().unwrap();
    let tmp_dir = current_dir.join("__tmp");
    std::fs::remove_dir_all(tmp_dir).unwrap()
}

#[test]
fn module_import() {
    let _m = MUTEX.lock().unwrap_or_else(PoisonError::into_inner);
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
#[should_panic(expected="Cyclic module dependency. Can't import root.pakhi from module.pakhi")]
fn module_import_cyclic() {
    let _m = MUTEX.lock().unwrap_or_else(PoisonError::into_inner);
    create_file("root.pakhi", vec![
        r#"মডিউল ম = "module.pakhi";"#,
    ]);
    create_file("module.pakhi", vec![
        r#"মডিউল ম = "root.pakhi";"#,
    ]);

    let thread = std::thread::spawn(|| {
        let mock_io: MockIO = MockIO::new();
        run_module("root.pakhi", mock_io);
    });
    if thread.join().is_err() {
        clean_test_tmp_dir();
        panic!("Cyclic module dependency. Can't import root.pakhi from module.pakhi");
    }
}

#[test]
fn built_in_fn_read_file() {
    let _m = MUTEX.lock().unwrap_or_else(PoisonError::into_inner);
    create_file("test.txt", vec![
        "test passed",
    ]);
    create_file("test.pakhi", vec![
        "দেখাও _রিড-ফাইল(_ডাইরেক্টরি + \"./test.txt\");",
    ]);

    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("test passed");
    run_module("test.pakhi", mock_io);
}

#[test]
fn built_in_fn_write_file() {
    let _m = MUTEX.lock().unwrap_or_else(PoisonError::into_inner);
    create_file("test.pakhi", vec![
        "_রাইট-ফাইল(_ডাইরেক্টরি + \"./test.txt\", \"test passed\");",
        "দেখাও _রিড-ফাইল(_ডাইরেক্টরি + \"./test.txt\");",
    ]);

    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("test passed");
    run_module("test.pakhi", mock_io);
}

#[test]
fn built_in_fn_delete_file() {
    let _m = MUTEX.lock().unwrap_or_else(PoisonError::into_inner);
    create_file("test.txt", vec![
        "test passed",
    ])
    ;create_file("test.pakhi", vec![
        "দেখাও _ডিলিট-ফাইল(_ডাইরেক্টরি + \"./test.txt\");",
    ]);

    let mock_io: MockIO = MockIO::new();
    run_module("test.pakhi", mock_io);
    assert!(!std::path::Path::new("./tmp/test.txt").exists());
}

#[test]
fn built_in_fn_read_dir() {
    let _m = MUTEX.lock().unwrap_or_else(PoisonError::into_inner);
    // create_file creates file ./tmp folder
    create_file("test.txt", vec!["test passed"]);
    create_file("test.pakhi", vec![
        "নাম ডার = _রিড-ডাইরেক্টরি(_ডাইরেক্টরি + \"./\");",
        "দেখাও ডার[০];"
    ]);

    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("test.txt");
    run_module("test.pakhi", mock_io);
}

#[test]
fn built_in_fn_create_dir() {
    let _m = MUTEX.lock().unwrap_or_else(PoisonError::into_inner);
    create_file("test.pakhi", vec![
        "_নতুন-ডাইরেক্টরি(_ডাইরেক্টরি + \"./test\");",
        "_রাইট-ফাইল(_ডাইরেক্টরি + \"./test/test.txt\", \"test passed\");",
        "নাম ড = _রিড-ডাইরেক্টরি(_ডাইরেক্টরি + \"./test\");",
        "দেখাও ড;"
    ]);

    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("[test.txt]");
    run_module("test.pakhi", mock_io);
}

#[test]
#[should_panic]
fn built_in_fn_delete_dir() {
    let _m = MUTEX.lock().unwrap_or_else(PoisonError::into_inner);
    create_file("test.pakhi", vec![
        "_নতুন-ডাইরেক্টরি(_ডাইরেক্টরি + \"./test\");",
        "_ডিলিট-ডাইরেক্টরি(_ডাইরেক্টরি + \"./test\")",
        "_রিড-ডাইরেক্টরি(_ডাইরেক্টরি + \"./test\");",
    ]);

    let thread = std::thread::spawn(|| {
        let mock_io: MockIO = MockIO::new();
        run_module("test.pakhi", mock_io);
    });
    if thread.join().is_err() {
        clean_test_tmp_dir();
        panic!()
    }
}

#[test]
fn built_in_fn_file_or_dir() {
    let _m = MUTEX.lock().unwrap_or_else(PoisonError::into_inner);
    create_file("test.pakhi", vec![
        "_নতুন-ডাইরেক্টরি(_ডাইরেক্টরি + \"./test\");",
        "_রাইট-ফাইল(_ডাইরেক্টরি + \"./test.txt\", \"test passed\");",
        "দেখাও _ফাইল-নাকি-ডাইরেক্টরি(_ডাইরেক্টরি + \"./test.txt\");",
        "দেখাও _ফাইল-নাকি-ডাইরেক্টরি(_ডাইরেক্টরি + \"./test\");",
    ]);

    let mut mock_io: MockIO = MockIO::new();
    mock_io.expect_println("ফাইল");
    mock_io.expect_println("ডাইরেক্টরি");
    run_module("test.pakhi", mock_io);
}