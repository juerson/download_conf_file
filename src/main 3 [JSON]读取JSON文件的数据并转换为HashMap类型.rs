use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::io::{self, Write};

fn parse_json_file(file_path: &str) -> HashMap<String, HashMap<String, Vec<String>>> {
    // 读取文件内容
    let mut file = match File::open(file_path) {
        Ok(file) => file,
        Err(_e) => {
            println!("找不到文件{}。", file_path);
            wait_for_enter();
            std::process::exit(1);
        }
    };
    let mut json_str = String::new();
    file.read_to_string(&mut json_str)
        .expect("Failed to read file");

    // 将 JSON 字符串解析为 serde_json::Value
    let json_value: Value = serde_json::from_str(&json_str).expect("Failed to parse JSON");

    // 将 serde_json::Value 转换为 HashMap
    if let Value::Object(map) = json_value {
        let mut my_dict: HashMap<String, HashMap<String, Vec<String>>> = HashMap::new();

        for (outer_key, outer_value) in map {
            if let Value::Object(inner_map) = outer_value {
                let inner_dict: HashMap<String, Vec<String>> = inner_map
                    .into_iter()
                    .filter_map(|(inner_key, inner_value)| {
                        if let Value::Array(arr) = inner_value {
                            Some((
                                inner_key,
                                arr.into_iter()
                                    .filter_map(|val| val.as_str().map(String::from))
                                    .collect(),
                            ))
                        } else {
                            None
                        }
                    })
                    .collect();

                my_dict.insert(outer_key, inner_dict);
            }
        }

        return my_dict;
    } else {
        panic!("JSON is not an object");
    }
}

// 辅助函数
fn wait_for_enter() {
    print!("按Enter键，退出程序！");
    io::stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
}

fn main() {
    let file_path = "urls.json";
    let my_dict = parse_json_file(file_path);

    for (key, value) in &my_dict {
        println!("Outer Key: {}", key);

        for (inner_key, inner_values) in value {
            println!("  Inner Key: {}", inner_key);
            println!("  Inner vlaue: {:?}", inner_values);
        }
    }
}
