use futures::future::join_all;
use reqwest::Client;
use serde_json::Value;
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;

async fn download_resource(url: &str) -> Result<String, reqwest::Error> {
    let client = Client::new();
    let response = client.get(url).send().await?;
    let body = response.text().await?;
    Ok(body)
}

fn format_json(json_str: &str) -> String {
    let value: Value = serde_json::from_str(json_str).unwrap();
    serde_json::to_string_pretty(&value).unwrap_or_else(|_| serde_json::to_string(&value).unwrap())
}

#[tokio::main]
async fn main() {
    let urls = vec![
        "https://www.gitlabip.xyz/Alvin9999/pac2/master/singbox/1/config.json",
        "https://gitlab.com/free9999/ipupdate/-/raw/master/singbox/config.json",
        "https://www.githubip.xyz/Alvin9999/pac2/master/singbox/config.json",
        "https://fastly.jsdelivr.net/gh/Alvin9999/pac2@latest/singbox/config.json",
    ];

    let mut tasks = Vec::new();

    for url in urls.iter() {
        let task = download_resource(url);
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

    println!("独立无二的资源: {}", unique_contents.len());

    // 将每个不同的数据写入单独的文件
    for (index, content) in unique_contents.iter().enumerate() {
        let filename = format!("singbox_{}.json", index + 1);
        if let Ok(mut file) = File::create(filename.clone()) {
            writeln!(file, "{}", content).expect("Error writing to file");
            println!("数据已经写入文件'{}'", filename);
        } else {
            eprintln!("创建/打开文件'{}'时出现错误", filename);
        }
    }
}
