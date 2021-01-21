use std::collections::HashMap;

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
                                 "_ক্রিয়েট-ডাইরেক্টরি"];
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

    //pub(crate) fn is_built_in_with_str(&self, function_name: &str) -> bool {
    //    if self.built_in_functions.contains_key(function_name) {
    //        true
    //    } else { false }
    //}
}