use reqwest::Client;
use serde_json::Value;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::error::Error;
use rand::Rng;
use std::time::Instant;
use std::io;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let start = Instant::now();
    let json_content = std::fs::read_to_string("urls.json").map_err(|e| Box::new(e) as Box<dyn Error>)?;
    println!("解析urls.json文件中...");
    let tasks = parse_tasks_from_json(&json_content)?;
    println!("开始寻找合适的网络线路下载...");
    for (task_name, urls) in tasks {
        let result = download_urls(&task_name, urls).await;
        match result {
            Ok(_) => println!("{} 下载完成！", task_name),
            Err(err) => eprintln!("{} 下载失败: {}", task_name, err),
        }
    }
    println!("所有下载任务完成！耗时：{:?}", start.elapsed());
    wait_for_enter();
    Ok(())
}

fn parse_tasks_from_json(json_content: &str) -> Result<BTreeMap<String, Vec<String>>, Box<dyn Error>> {
    let json: Value = serde_json::from_str(json_content)?;

    let mut tasks = BTreeMap::new();

    if let Some(object) = json.as_object() {
        for (task_name, value) in object {
            if let Some(urls_array) = value.as_array() {
                let mut urls = Vec::new();
                for url_value in urls_array {
                    if let Some(url) = url_value.as_str() {
                        urls.push(url.to_string());
                    } else {
                        return Err(format!("Invalid URL in task '{}'", task_name).into());
                    }
                }
                tasks.insert(task_name.to_string(), urls);
            } else {
                return Err(format!("Invalid URL list in task '{}'", task_name).into());
            }
        }
    } else {
        return Err("Invalid JSON format".into());
    }

    Ok(tasks)
}

async fn download_urls(task_name: &str, mut urls: Vec<String>) -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let mut failed_urls = HashSet::new();

    while !urls.is_empty() {
        // 从未失败的链接中随机选择一个
        let mut rng = rand::thread_rng();
        let random_index = rng.gen_range(0..urls.len());
        let url = urls[random_index].clone();

        if failed_urls.contains(&url) {
            // 如果链接已经失败过，则跳过
            urls.retain(|x| x != &url);
            continue;
        }

        let head_result = client.head(&url).send().await;
        if !head_result.is_ok() {
            println!("HEAD {} 失败，跳过", url);
            failed_urls.insert(url.clone());
            urls.retain(|x| x != &url);
            continue;
        }

        let result = client.get(&url).send().await;

        match result {
            Ok(res) => {
                if res.status() != 200 {
                    println!("GET {} 失败，状态码 {}，跳过", url, res.status());
                    failed_urls.insert(url.clone());
                    urls.retain(|x| x != &url);
                    continue;
                }
            }
            Err(e) => {
                println!("GET {} 失败: {}，跳过", url, e);
                failed_urls.insert(url.clone());
                urls.retain(|x| x != &url);
                continue;
            }
        }

        let file_name;
        if task_name.to_lowercase().starts_with("clash") {
            file_name = format!("{}.yaml", task_name); // 使用任务名称作为文件名
        } else {
            file_name = format!("{}.json", task_name);
        }
        let res = client.get(&url).send().await?;
        let bytes = res.bytes().await?;

        if let Err(e) = std::fs::write(&file_name, bytes) {
            return Err(Box::new(e) as Box<dyn Error>);
        }

        print!("{} ", url);
        io::stdout().flush().expect("刷新输出缓冲区失败");
        return Ok(());
    }
    Err("所有链接都下载失败".into())
}

fn wait_for_enter() {
    let mut input = String::new();
    print!("按下Enter键关闭窗口...");
    io::stdout().flush().expect("刷新输出缓冲区失败"); // 刷新输出缓冲区
    io::stdin().read_line(&mut input).expect("读取输入失败");
}