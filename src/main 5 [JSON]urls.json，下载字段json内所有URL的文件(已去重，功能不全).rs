use futures::future::join_all;
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::io::{self, Write};

// 将json解析为HashMap类型的数据
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

async fn fetch_url_content(url: &str) -> Result<String, reqwest::Error> {
    let client = Client::new();
    let response = client.get(url).send().await?;
    let body = response.text().await?;
    Ok(body)
}

fn format_json(json_str: &str) -> String {
    let value: Value = serde_json::from_str(json_str).unwrap();
    serde_json::to_string_pretty(&value).unwrap_or_else(|_| serde_json::to_string(&value).unwrap())
}

async fn download_and_process_data(urls: Vec<&str>) -> HashSet<String> {
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
                let formatted_json = format_json(&content);
                let trimmed_content = formatted_json.trim().to_string();
                unique_contents.insert(trimmed_content);
            }
            Err(err) => {
                eprintln!("资源下载错误: {:?}", err);
            }
        }
    }

    unique_contents
}

#[tokio::main]
async fn main() {
    let file_path = "urls.json";
    let my_dict = parse_json_file(file_path);

    // 获取字段 "json" 的值
    if let Some(inner_dict) = my_dict.get("json") {
        // 遍历字段 "json" 内的键值对
        for (key, values) in inner_dict {
            // 使用 iter 和 cloned 方法将 &Vec<String> 转换为 Vec<&str>
            let urls: Vec<&str> = values.iter().map(|s| s.as_str()).collect();
            let unique_contents = download_and_process_data(urls).await;
            println!(
                "{}配置文件，有{}个不相同的，将下载后的数据写入文件中...",
                key,
                unique_contents.len()
            );
            if unique_contents.len() >= 1 {
                // 将每个不同的数据写入单独的文件
                for (index, content) in unique_contents.iter().enumerate() {
                    let filename = if unique_contents.len() > 1 {
                        format!("output/{}_{}.json", key, index + 1)
                    } else {
                        format!("output/{}.json", key)
                    };
                    if let Ok(mut file) = File::create(filename.clone()) {
                        writeln!(file, "{}", content).expect("Error writing to file");
                        println!("  - 数据已经写入文件'{}'", filename);
                    } else {
                        eprintln!("  - 创建/打开文件'{}'时出现错误", filename);
                    }
                }
            }
        }
    }
}
