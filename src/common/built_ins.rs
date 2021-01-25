use std::collections::HashMap;
use std::path::Path;
use crate::backend::interpreter::DataType;

// Contains all built-in function and constant names
pub struct BuiltInFunctionList {
    built_in_functions: HashMap<Vec<char>, String>,
}

impl BuiltInFunctionList {
    pub(crate) fn new() -> Self {
        let mut functions_map: HashMap<Vec<char>, String> = HashMap::new();
        // this functions are built-in
        let function_list = vec!["_লিস্ট-পুশ", "_লিস্ট-পপ", "_রিড-লাইন", "_এরর", "_স্ট্রিং-স্প্লিট",
                                 "_স্ট্রিং-জয়েন", "_টাইপ", "_রিড-ফাইল", "_রাইট-ফাইল", "_ডিলিট-ফাইল",
                                 "_নতুন-ডাইরেক্টরি", "_রিড-ডাইরেক্টরি", "_ডিলিট-ডাইরেক্টরি"];
        for f_name in function_list {
            functions_map.insert(f_name.chars().collect(), f_name.to_string());
        }

        BuiltInFunctionList {
            built_in_functions: functions_map,
        }
    }

    pub(crate) fn is_built_in(&self, function_name: &Vec<char>) -> bool {
        if self.built_in_functions.contains_key(function_name) {
            true
        } else { false }
    }

    pub(crate) fn get_name(&self, function_name: &Vec<char>) -> String {
        self.built_in_functions.get(function_name).unwrap().clone()
    }

    pub(crate) fn built_in_fn_list_push(arguments: Vec<DataType>, lists: &mut Vec<Vec<DataType>>) -> DataType {
        if arguments.len() == 2 {
            let list = arguments[0].clone();

            if let DataType::List(index) = list {
                let push_value = arguments[1].clone();
                let actual_list = lists.get_mut(index).unwrap();
                actual_list.push(push_value);
            } else {
                panic!("Datatype must be array to push value")
            }

        } else if arguments.len() == 3 {
            let list = arguments[0].clone();

            if let DataType::List(index) = list {
                let push_at = arguments[1].clone();
                let push_value = arguments[2].clone();
                let actual_list = lists.get_mut(index).unwrap();

                if let DataType::Num(push_at_i_f) = push_at {
                    let push_at_u = push_at_i_f as usize;
                    actual_list.insert(push_at_u, push_value);
                } else { panic!("Index must evaluate to number type"); }

            } else { panic!("Datatype must be array to push value") }

        } else { panic!("Function requires two arguments") }

        return DataType::Nil;
    }

    pub(crate) fn built_in_fn_list_pop(arguments: Vec<DataType>, lists: &mut Vec<Vec<DataType>>) -> DataType {
        if arguments.len() == 1 {
            let list = arguments[0].clone();

            if let DataType::List(index) = list {
                let actual_list = lists.get_mut(index).unwrap();
                actual_list.pop();
            } else { panic!("Datatype must be array to push value")}

        } else if arguments.len() == 2 {
            let list = arguments[0].clone();

            if let DataType::List(index) = list {
                let pop_at = arguments[1].clone();
                let actual_list = lists.get_mut(index).unwrap();

                if let DataType::Num(pop_at_i_f) = pop_at {
                    let pop_at_i = pop_at_i_f as usize;
                    actual_list.remove(pop_at_i);
                }

            } else { panic!("Datatype must be array to push value") }

        } else { panic!("Function requires one argument") }

        return DataType::Nil;
    }

    pub(crate) fn built_in_fn_read_line(arguments: Vec<DataType>) -> DataType {
        if arguments.len() == 0 {
            let mut input = String::new();
            match std::io::stdin().read_line(&mut input) {
                Ok(_) => return DataType::String(input.trim_end().into()),
                Err(e) => panic!("{}", e),
            }
        } else { panic!("Function requires zero argument") }
    }

    pub(crate) fn built_in_fn_error(arguments: Vec<DataType>) -> String {
        if arguments.len() == 1 {
            let error = arguments[0].clone();
            match error {
                DataType::String(err_message) => err_message,
                _ => panic!("_এরর() functions arguments must be string"),
            }
        } else {
            panic!("_এরর() function expects one argument");
        }
    }

    pub(crate) fn built_in_fn_string_split(arguments: Vec<DataType>, lists: &mut Vec<Vec<DataType>>) -> DataType {
        if arguments.len() == 2 {
            let string = arguments[0].clone();
            let split_by = arguments[1].clone();
            match (string, split_by) {
                (DataType::String(string), DataType::String(split_by)) => {
                    let mut splitted_string: Vec<&str> = string.split(&split_by).collect();
                    // For some reason split with "" causes splits to have "" at benginning and end
                    // Thats why removes character at start finish
                    if splitted_string[0] == "" && splitted_string[splitted_string.len() - 1] == "" {
                        splitted_string.remove(0);
                        splitted_string.remove(splitted_string.len() - 1);
                    }
                    let splitted_string: Vec<DataType> = splitted_string.iter()
                        .map(|s| DataType::String(String::from(s.clone()))).collect();
                    lists.push(splitted_string);
                    return DataType::List(lists.len() - 1);
                },
                _ => panic!("_স্ট্রিং-স্প্লিট() functions arguments must be string"),
            }
        } else {
            panic!("_স্ট্রিং-স্প্লিট() function expects two argument");
        }
    }

    pub(crate) fn built_in_fn_string_join(arguments: Vec<DataType>, lists: &mut Vec<Vec<DataType>>) -> DataType {
        if arguments.len() == 2 {
            let list_of_strings = arguments[0].clone();
            let join_by = arguments[1].clone();
            match (list_of_strings, join_by) {
                (DataType::List(list_index), DataType::String(join_by)) => {
                    let string_list = lists.get(list_index).unwrap();
                    let mut strings: Vec<String> = Vec::new();
                    for string in string_list {
                        if let DataType::String(string) = string.clone() {
                            strings.push(string);
                        } else { panic!("_স্ট্রিং-জয়েন() functions only accepts list of strings"); }
                    }
                    let joined_string = strings.join(&join_by);
                    return DataType::String(joined_string);
                },
                _ => panic!("_স্ট্রিং-জয়েন() functions arguments must be list and string"),
            }
        } else {
            panic!("_স্ট্রিং-জয়েন() function expects two argument");
        }
    }

    pub(crate) fn built_in_fn_type(arguments: Vec<DataType>) -> DataType {
        if arguments.len() == 1 {
            let data = arguments[0].clone();
            match data {
                DataType::Num(_) => DataType::String(String::from("_সংখ্যা")),
                DataType::Bool(_) => DataType::String(String::from("_বুলিয়ান")),
                DataType::String(_) => DataType::String(String::from("_স্ট্রিং")),
                DataType::List(_) => DataType::String(String::from("_লিস্ট")),
                DataType::NamelessRecord(_) => DataType::String(String::from("_রেকর্ড")),
                DataType::Function(_) => DataType::String(String::from("_ফাং")),
                DataType::Nil => DataType::String(String::from("_শূন্য")),
            }
        } else {
            panic!("_টাইপ() function expects one argument");
        }
    }

    pub(crate) fn built_in_fn_read_file(arguments: Vec<DataType>) -> DataType {
        if arguments.len() == 1 {
            let path_data = arguments[0].clone();
            match path_data {
                DataType::String(p) => {
                    let path = Path::new(&p);
                    if path.is_relative() {
                        panic!("Cannot read file with relative file")
                    }
                    let read_result = std::fs::read_to_string(path);
                    match read_result {
                        Ok(content) => DataType::String(content),
                        Err(e) => panic!("{}", e.to_string()),
                    }
                },
                _ => panic!("_রিড-ফাইল() function's path argument must be of type string"),
            }
        } else {
            panic!("_রিড-ফাইল() function expects one argument");
        }
    }

    pub(crate) fn built_in_fn_write_file(arguments: Vec<DataType>) -> DataType {
        if arguments.len() == 2 {
            let path_data = arguments[0].clone();
            let content_data = arguments[1].clone();
            match (path_data, content_data) {
                (DataType::String(p), DataType::String(content)) => {
                    let path = Path::new(&p);
                    if path.is_relative() {
                        panic!("Cannot write file with relative file")
                    }
                    let write_result = std::fs::write(path, content);
                    match write_result {
                        Ok(_) => DataType::Bool(true),
                        Err(e) => panic!("{}", e.to_string()),
                    }
                },
                _ => panic!("_রাইট-ফাইল() function's both argument must be of type string"),
            }
        } else {
            panic!("_রাইট-ফাইল() function expects two argument");
        }
    }

    pub(crate) fn built_in_fn_delete_file(arguments: Vec<DataType>) -> DataType {
        if arguments.len() == 1 {
            let path_data = arguments[0].clone();
            match path_data {
                DataType::String(p) => {
                    let path = Path::new(&p);
                    if path.is_relative() {
                        panic!("Cannot delete file with relative file")
                    }
                    let delete_result = std::fs::remove_file(path);
                    match delete_result {
                        Ok(_) => DataType::Bool(true),
                        Err(e) => panic!("{}", e.to_string()),
                    }
                },
                _ => panic!("_ডিলিট-ফাইল() function's argument must be of type string"),
            }
        } else {
            panic!("_ডিলিট-ফাইল() function expects one argument");
        }
    }

    pub(crate) fn built_in_fn_create_dir(arguments: Vec<DataType>) -> DataType {
        if arguments.len() == 1 {
            let path_data = arguments[0].clone();
            match path_data {
                DataType::String(p) => {
                    let path = Path::new(&p);
                    if path.is_relative() {
                        panic!("Cannot create dir with relative file")
                    }
                    let create_dir_result = std::fs::create_dir_all(path);
                    match create_dir_result {
                        Ok(_) => DataType::Bool(true),
                        Err(e) => panic!("{}", e.to_string()),
                    }
                },
                _ => panic!("_ক্রিয়েট-ডাইরেক্টরি() function's argument must be of type string"),
            }
        } else {
            panic!("_ক্রিয়েট-ডাইরেক্টরি() function expects one argument");
        }
    }

    pub(crate) fn built_in_fn_read_dir(arguments: Vec<DataType>) -> Vec<String> {
        if arguments.len() == 1 {
            let path_data = arguments[0].clone();
            match path_data {
                DataType::String(p) => {
                    let path = Path::new(&p);
                    if path.is_relative() {
                        panic!("Cannot read dir with relative file")
                    }
                    let create_dir_result = std::fs::read_dir(path);
                    match create_dir_result {
                        Ok(paths) => {
                            let mut all_files_dirs: Vec<String> = Vec::new();
                            for path in paths {
                                let file_dir_name =  path.unwrap().file_name().to_str().unwrap().to_string();
                                all_files_dirs.push(file_dir_name);
                            }
                            all_files_dirs
                        },
                        Err(e) => panic!("{}", e.to_string()),
                    }
                },
                _ => panic!("_রিড-ডাইরেক্টরি() function's argument must be of type string"),
            }
        } else {
            panic!("_রিড-ডাইরেক্টরি() function expects one argument");
        }
    }

    pub(crate) fn built_in_fn_delete_dir(arguments: Vec<DataType>) -> DataType {
        if arguments.len() == 1 {
            let path_data = arguments[0].clone();
            match path_data {
                DataType::String(p) => {
                    let path = Path::new(&p);
                    if path.is_relative() {
                        panic!("Cannot delete directory with relative file")
                    }
                    let delete_result = std::fs::remove_dir_all(path);
                    match delete_result {
                        Ok(_) => DataType::Bool(true),
                        Err(e) => panic!("{}", e.to_string()),
                    }
                },
                _ => panic!("_ডিলিট-ডাইরেক্টরি() function's argument must be of type string"),
            }
        } else {
            panic!("_ডিলিট-ডাইরেক্টরি() function expects one argument");
        }
    }
}