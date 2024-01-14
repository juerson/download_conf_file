use serde_yaml::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::io::{self, Write};

fn parse_yaml_file(file_path: &str) -> HashMap<String, HashMap<String, Vec<String>>> {
    // 读取文件内容
    let mut file = match File::open(file_path) {
        Ok(file) => file,
        Err(_e) => {
            println!("找不到文件{}。", file_path);
            wait_for_enter();
            std::process::exit(1);
        }
    };
    let mut yaml_str = String::new();
    file.read_to_string(&mut yaml_str)
        .expect("Failed to read file");

    // 将 YAML 字符串解析为 serde_yaml::Value
    let yaml_value: Value = serde_yaml::from_str(&yaml_str).expect("Failed to parse YAML");

    // 将 serde_yaml::Value 转换为 HashMap
    if let Value::Mapping(map) = yaml_value {
        let mut my_dict: HashMap<String, HashMap<String, Vec<String>>> = HashMap::new();

        for (outer_key, outer_value) in map {
            if let Value::Mapping(inner_map) = outer_value {
                let inner_dict: HashMap<String, Vec<String>> = inner_map
                    .into_iter()
                    .filter_map(|(inner_key, inner_value)| {
                        if let Value::Sequence(seq) = inner_value {
                            Some((
                                inner_key.as_str().unwrap().to_string(),
                                seq.into_iter()
                                    .filter_map(|val| val.as_str().map(String::from))
                                    .collect(),
                            ))
                        } else {
                            None
                        }
                    })
                    .collect();

                my_dict.insert(outer_key.as_str().unwrap().to_string(), inner_dict);
            }
        }

        return my_dict;
    } else {
        panic!("YAML is not a mapping");
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
    let file_path = "urls.yaml";
    let my_dict = parse_yaml_file(file_path);

    // 获取字段 "yaml" 的值
    if let Some(inner_dict) = my_dict.get("yaml") {
        // 遍历 "yaml" 内的键值对
        for (key, values) in inner_dict {
            // 使用 key 和 values 进行后续操作
            println!("{} => {:?}", key, values);
        }
    }
}
