use reqwest::Client;
use std::collections::HashSet;
use std::error::Error;
use std::io::{self, BufRead};
use std::time::Instant;
use std::io::Write;
use std::fs;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let infile = "url.txt";
    // 检查是否存在 url.txt 文件
    if !std::path::Path::new(infile).exists() {
        println!("{} 文件不存在...", infile);
        wait_for_enter();
        return Ok(());
    }

    // 打开文件并创建 BufReader
    let start = Instant::now();
    let reader = open_file(infile)?;

    // 初始化集合和向量
    let mut failed_urls = HashSet::new();
    let mut successful_urls = Vec::new();

    // 判断是否有下载链接
    let mut non_empty = false;
    // 指定保存文件的文件夹路径
    let save_folder = "output";

    // 遍历文件中的每一行
    for line in reader.lines() {
        let url = line.map_err(|e| Box::new(e) as Box<dyn Error>)?;
        if !url.trim().is_empty() {
            non_empty = true;
            if download_and_track(&url, &mut failed_urls, &mut successful_urls, save_folder).await.is_ok() {
                successful_urls.push(url);
            }
        }
    }

    // 保存成功的链接到文件
    if !successful_urls.is_empty() {
        save_successful_urls(&successful_urls, save_folder)?;
        println!("\n当前有效的链接已经写入文件：{}/valid_url.txt ", save_folder);
    }

    // 打印信息
    print_completion_message(&infile, non_empty, start);

    // 等待用户按下回车键
    wait_for_enter();
    Ok(())
}

// 打开文件并返回 BufReader
fn open_file(file_path: &str) -> Result<io::BufReader<std::fs::File>, Box<dyn Error>> {
    std::fs::File::open(file_path)
        .map(io::BufReader::new)
        .map_err(|e| Box::new(e) as Box<dyn Error>)
}

// 下载并追踪失败和成功的 URL
async fn download_and_track(
    url: &str,
    failed_urls: &mut HashSet<String>,
    successful_urls: &mut Vec<String>,
    save_folder: &str,
    ) -> Result<(), Box<dyn Error>> {
    if failed_urls.contains(url) {
        return Ok(());
    }
    // 下载 URL 对应的文件(并且写入文件中)，还有将成功的链接和失败分别添加到向量和集合中
    match download_url(url, failed_urls, save_folder).await {
        Ok(()) => {
            successful_urls.push(url.to_string());
            Ok(())
        }
        Err(_) => {
            failed_urls.insert(url.to_string());
            Ok(())
        }
    }
}

// 将成功的链接保存到文件
fn save_successful_urls(successful_urls: &[String], save_folder: &str) -> Result<(), Box<dyn Error>> {
    let ok_content = successful_urls.join("\n");
    let successful_url_path = format!("{}/valid_url.txt", save_folder);
    std::fs::write(successful_url_path, ok_content)?;
    Ok(())
}

// 打印完成消息
fn print_completion_message(infile: &str, non_empty: bool, start: Instant) {
    if !non_empty {
        println!("{} 文件为空！", infile);
    } else {
        println!("所有下载任务已经完成！耗时：{:?}\n", start.elapsed());
    }
}

// 异步函数：下载 URL 对应的文件(并且写入文件中)
async fn download_url(url: &str, failed_urls: &mut HashSet<String>, save_folder: &str) -> Result<(), Box<dyn Error>> {
    // 如果链接已经失败过，则跳过
    if failed_urls.contains(url) {
        return Ok(());
    }
    // 创建一个 HTTP 客户端
    let client = Client::new();
    // 发送 GET 请求并等待结果
    let result = client.get(url).send().await;

    // 处理请求结果
    match result {
        // 如果请求成功，并且状态码为 200
        Ok(res) if res.status() == 200 => {
            // 生成唯一的文件名
            let file_name = generate_unique_filename(url, save_folder);
            // 获取响应的字节数据
            let bytes = res.bytes().await?;
            // 确保保存文件的文件夹存在
            fs::create_dir_all(save_folder)?;
            // 将字节数据写入文件
            fs::write(&file_name, bytes)?;

            // 打印成功消息
            println!("{} 下载成功！", url);
        }
        // 如果请求成功，但状态码不为 200
        Ok(res) => {
            // 打印失败消息，并记录失败的 URL
            println!("GET {} 失败，状态码 {}，跳过", url, res.status());
            failed_urls.insert(url.to_string());
        }
        // 如果请求失败
        Err(e) => {
            // 打印失败消息，并记录失败的 URL
            println!("GET {} 失败: {}，跳过", url, e);
            failed_urls.insert(url.to_string());
        }
    }

    Ok(())
}

// 确定文件名（必要时添加编号），文件后缀，截取于链接的后面
fn generate_unique_filename(url: &str, save_folder: &str) -> String {
    // 从 URL 提取文件名
    let original_file_name = url.rsplit('/').next().unwrap_or("unknown");

    let mut count = 1;

    // 分割文件名和扩展名
    if let Some((filename, suffix)) = original_file_name.split_once('.') {
        let mut unique_file_name = format!("{}/{}_{}.{}", save_folder, filename, count, suffix);

        // 检查现有文件名，必要时添加编号
        while Path::new(&unique_file_name).exists() {
            count += 1;
            unique_file_name = format!("{}/{}_{}.{}", save_folder, filename, count, suffix);
        }
        return unique_file_name;
    }

    // 如果找不到扩展名，则在文件名后添加一个数字
    let mut unique_file_name = format!("{}/{}_{}", save_folder, original_file_name, count);

    // 检查现有文件名，必要时添加编号
    while Path::new(&unique_file_name).exists() {
        count += 1;
        unique_file_name = format!("{}/{}_{}", save_folder, original_file_name, count);
    }

    unique_file_name
}

// 辅助函数
fn wait_for_enter() {
    let mut input = String::new();
    print!("按下Enter键关闭窗口！");
    io::stdout().flush().expect("刷新输出缓冲区失败"); // 刷新输出缓冲区
    io::stdin().read_line(&mut input).expect("读取输入失败");
}