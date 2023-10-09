use reqwest::Client;
use std::collections::HashSet;
use std::error::Error;
use std::io::{self, BufRead};
use std::time::Instant;
use std::io::Write;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if !std::path::Path::new("urls.txt").exists() {
        println!("urls.txt 文件不存在...");
        wait_for_enter();
        return Ok(());
    }
    let start = Instant::now();
    let file = std::fs::File::open("urls.txt").map_err(|e| Box::new(e) as Box<dyn Error>)?;
    let reader = io::BufReader::new(file);

    let mut failed_urls = HashSet::new();
    let mut successful_urls = Vec::new();

    let mut non_empty = false; // 判断是否有下载的链接
    // 指定要保存文件的文件夹路径
    let save_folder = "downloaded";

    for line in reader.lines() {
        let url = line.map_err(|e| Box::new(e) as Box<dyn Error>)?;
        if !url.trim().is_empty() {
            non_empty = true; // 有下载链接的，将状态改为Ture
            if download_url(&url, &mut failed_urls, save_folder).await.is_ok() {
                successful_urls.push(url);
            }
        }
    }

    if !successful_urls.is_empty() {
        let ok_content = successful_urls.join("\n");
        let successful_url_path = format!("{}/ok.txt", save_folder);
        std::fs::write(successful_url_path, ok_content)?;
        println!("有效链接：在{}/ok.txt文件中！", save_folder);
    }
    if !non_empty {
        println!("urls.txt 文件为空...");
    } else {
        println!("所有下载任务已经完成！用时：{:?}", start.elapsed());
    }
    wait_for_enter();
    Ok(())
}

async fn download_url(url: &str, failed_urls: &mut HashSet<String>, save_folder: &str) -> Result<(), Box<dyn Error>> {
    let client = Client::new();

    if failed_urls.contains(url) {
        // 如果链接已经失败过，则跳过
        return Ok(());
    }

    let head_result = client.head(url).send().await;
    if !head_result.is_ok() {
        println!("HEAD {} 失败，跳过", url);
        failed_urls.insert(url.to_string());
        return Ok(());
    }

    let result = client.get(url).send().await;

    match result {
        Ok(res) => {
            if res.status() != 200 {
                println!("GET {} 失败，状态码 {}，跳过", url, res.status());
                failed_urls.insert(url.to_string());
                return Ok(());
            }
        }
        Err(e) => {
            println!("GET {} 失败: {}，跳过", url, e);
            failed_urls.insert(url.to_string());
            return Ok(());
        }
    }

    // 从 URL 中截取文件名
    let file_name = if let Some(last_slash) = url.rfind('/') {
        let original_file_name = &url[(last_slash + 1)..];
        let mut unique_file_name = format!("{}/{}", save_folder, original_file_name.to_string());
        let mut count = 1;

        // 检查是否存在同名文件，如果存在，则添加数字以区别
        while std::path::Path::new(&unique_file_name).exists() {
            let extension = if let Some(dot) = original_file_name.rfind('.') {
                &original_file_name[dot..]
            } else {
                ""
            };

            unique_file_name = format!("{}/{}-{}{}", save_folder, &original_file_name[..(original_file_name.len() - extension.len())], count, extension);
            count += 1;
        }

        unique_file_name
    } else {
        "unknown_filename".to_string() // 如果找不到斜杠，默认使用 "unknown_filename"
    };

    // 获取数据和保存文件
    let res = client.get(url).send().await?;
    let bytes = res.bytes().await?;

    fs::create_dir_all(save_folder)?; // 确保文件夹存在

    if let Err(e) = std::fs::write(&file_name, bytes) {
        return Err(Box::new(e) as Box<dyn Error>);
    }
    println!("{} 下载成功！", url);
    return Ok(());
}

fn wait_for_enter() {
    let mut input = String::new();
    print!("按下Enter键关闭窗口...");
    io::stdout().flush().expect("刷新输出缓冲区失败"); // 刷新输出缓冲区
    io::stdin().read_line(&mut input).expect("读取输入失败");
}