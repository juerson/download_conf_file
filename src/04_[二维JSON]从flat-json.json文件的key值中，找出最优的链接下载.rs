use reqwest::Client;
use serde_json::Value;
use std::{
    collections::{BTreeMap, HashSet},
    error::Error,
    io::{self, Write},
    time::Instant,
    path::Path,
    fs
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 读取 JSON 文件
    let infile = "flat-json.json";
    let start = Instant::now();
    let json_content = std::fs::read_to_string(infile)?;
    println!("解析{}文件中...", infile);
    let tasks = parse_tasks_from_json(&json_content)?;
    println!("开始寻找合适的网络线路下载...");

    // 遍历任务并下载
    for (task_name, urls) in tasks {
        match download_urls(&task_name, urls).await {
            Ok(_) => println!("{} 下载完成！", task_name),
            Err(err) => eprintln!("{} 下载失败: {}", task_name, err),
        }
    }

    println!("所有下载任务完成！耗时：{:?}", start.elapsed());
    wait_for_enter();
    Ok(())
}

// 解析 JSON 中的任务列表
fn parse_tasks_from_json(json_content: &str) -> Result<BTreeMap<String, Vec<String>>, Box<dyn Error>> {
    let json: Value = serde_json::from_str(json_content)?;

    let mut tasks = BTreeMap::new();

    if let Some(object) = json.as_object() {
        for (task_name, value) in object {
            let urls = value.as_array()
                .ok_or_else(|| format!("任务 '{}' 中的无效 URL 列表", task_name))?
                .iter()
                .map(|url_value| {
                    url_value.as_str().map(ToString::to_string).ok_or_else(|| format!("任务 '{}' 中的无效 URL", task_name))
                })
                .collect::<Result<Vec<String>, String>>()?;

            tasks.insert(task_name.to_string(), urls);
        }
    } else {
        return Err("无效的 JSON 格式".into());
    }

    Ok(tasks)
}

// 异步下载 URL 列表中的内容
async fn download_urls(task_name: &str, mut urls: Vec<String>) -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let mut failed_urls = HashSet::new();

    // 检查文件夹是否存在，不存在就创建
    create_directory_if_not_exists("output");

    while let Some(url) = urls.pop() {
        // 跳过失败的 URL 或无法发送 HEAD 请求的 URL
        if failed_urls.contains(&url) || client.head(&url).send().await.is_err() {
            println!("{} 失败，跳过", url);
            failed_urls.insert(url.clone());
            continue;
        }

        // 处理 GET 请求
        match client.get(&url).send().await {
            Ok(res) if res.status() != 200 => {
                println!("GET {} 失败，状态码 {}，跳过", url, res.status());
                failed_urls.insert(url.clone());
            }
            Err(e) => {
                println!("GET {} 失败: {}，跳过", url, e);
                failed_urls.insert(url.clone());
            }
            Ok(res) => {
                // 根据任务名称确定文件扩展名
                let file_ext = if task_name.to_lowercase().starts_with("clash") { "yaml" } else { "json" };
                let file_name = format!("output/{}.{}", task_name, file_ext);
                let bytes = res.bytes().await?;
                std::fs::write(&file_name, bytes)?;
                print!("{} ", url);
                io::stdout().flush().expect("刷新输出缓冲区失败");
                return Ok(());
            }
        }
    }
    Err("所有链接都下载失败".into())
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

// 等待用户按下 Enter 键
fn wait_for_enter() {
    let mut input = String::new();
    print!("按下 Enter 键关闭窗口！");
    io::stdout().flush().expect("刷新输出缓冲区失败");
    io::stdin().read_line(&mut input).expect("读取输入失败");
}