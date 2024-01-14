use futures::future::join_all;
use reqwest::Client;
use serde_yaml::Value;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Read;
use std::io::{self, Write};
use std::path::Path;

// 将yaml解析为HashMap类型的数据
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

// 获取 url 内容
async fn fetch_url_content(url: &str) -> Result<String, reqwest::Error> {
    let client = Client::new();
    let response = client.get(url).send().await?;
    let body = response.text().await?;
    Ok(body)
}

// 格式化JSON
fn format_json(json_str: &str) -> String {
    let value: Value = serde_json::from_str(json_str).unwrap();
    serde_json::to_string_pretty(&value).unwrap_or_else(|_| serde_json::to_string(&value).unwrap())
}

// 下载与处理数据(主要下载数据)
async fn download_and_process_data(urls: Vec<&str>, data_file: &String) -> HashSet<String> {
    let mut tasks = Vec::new();
    for url in urls.iter() {
        let task = fetch_url_content(url);
        tasks.push(task);
    }

    let results: Vec<_> = join_all(tasks).await.into_iter().collect();
    let mut unique_contents = HashSet::new();

    for result in results {
        match result {
            Ok(content) => {
                let mut trimmed_content = String::new();
                if data_file.trim().to_lowercase() == "json" {
                    let formatted_json = format_json(&content); // 数据序列化为 JSON 格式的字符串，让其适当的缩进和换行
                    trimmed_content = formatted_json.trim().to_string();
                } else if data_file.trim().to_lowercase() == "yaml" {
                    let yaml_string = serde_yaml::to_string(&content).unwrap(); // 数据序列化为 YAML 格式的字符串，让其适当的缩进和换行
                    trimmed_content = yaml_string.trim().to_string();
                } else {
                    // None
                }
                unique_contents.insert(trimmed_content);
            }
            Err(err) => {
                eprintln!("资源下载错误: {:?}", err);
            }
        }
    }
    unique_contents
}

// 目录不存在就创建文件夹
fn create_directory_if_not_exists(directory_path: &str) {
    let dir_path = Path::new(directory_path);
    if !dir_path.exists() {
        if let Err(err) = fs::create_dir_all(dir_path) {
            panic!("创建文件夹失败: {}", err);
        }
    }
}

// 将数据写入文件（不同的数据，用不同的文件存储）
fn write_to_file(
    unique_contents: HashSet<String>,
    dir_name: &str,
    inner_key: &str,
    data_file: &str,
) {
    if unique_contents.len() >= 1 {
        for (index, content) in unique_contents.iter().enumerate() {
            let filename = format!(
                "{}/{}{}.{}",
                dir_name,
                inner_key,
                if unique_contents.len() > 1 {
                    format!("_{}", index + 1)
                } else {
                    String::new()
                },
                data_file
            );

            if let Ok(mut file) = File::create(filename.clone()) {
                writeln!(file, "{}", content).expect("Error writing to file");
                println!("  - 数据已经写入文件'{}'", filename);
            } else {
                eprintln!("  - 创建/打开文件'{}'时出现错误", filename);
            }
        }
    } else {
        println!("  - 没有下载到{}相关的数据！", inner_key);
    }
}

#[tokio::main]
async fn main() {
    let file_path = "urls.yaml";
    // 将yaml解析为HashMap类型的数据
    let my_dict = parse_yaml_file(file_path);

    // 存放的文件夹
    let dir_name = "output";
    // 检查文件夹是否存在，不存在就创建
    create_directory_if_not_exists(dir_name);

    // 遍历YAML文件中，最外层的key-value
    for (data_file, value) in &my_dict {
        // 遍历字段里面的key-vlaue（第2层）
        for (inner_key, inner_values) in value {
            // 使用 iter 和 cloned 方法将 &Vec<String> 转换为 Vec<&str>
            let urls: Vec<&str> = inner_values.iter().map(|s| s.as_str()).collect();
            let unique_contents = download_and_process_data(urls, data_file).await;
            println!(
                "{}配置文件，有{}个不相同的，将下载后的数据写入文件中...",
                inner_key,
                unique_contents.len()
            );
            // 将数据写入文件（不同的数据，用不同的文件存储）
            write_to_file(unique_contents, dir_name, inner_key, data_file);
        }
    }
	
	println!();
    wait_for_enter();
}
