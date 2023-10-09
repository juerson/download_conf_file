use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{self, prelude::*, BufReader, SeekFrom};
use std::path::Path;
use std::fs::OpenOptions;
use std::process;

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "updated_urls.json_link.txt";
    let output_file = "urls.json";
    let split_symbol: String = "-".repeat(105);

    // 检查输入文件是否存在，如果不存在则创建
    if !Path::new(input_file).exists() {
        println!("输入文件不存在，将创建空文件：{}", input_file);
        File::create(input_file)?;
    }

    // 检查输出文件是否存在，如果不存在则创建，并写入初始字符串
    if !Path::new(output_file).exists() {
        println!("输出文件不存在，将创建并写入初始字符串：{}", output_file);
        let mut output_file = File::create(output_file)?;
        output_file.write_all(b"{}")?; // 写入初始字符串 "{}"
        print!("按Enter键退出程序>>");
        io::stdout().flush().expect("刷新输出缓冲区失败"); // 刷新输出缓冲区
        wait_for_enter();
        process::exit(0);
    }

    let mut infile = BufReader::new(File::open(input_file)?);
    let mut outfile = OpenOptions::new()
        .read(true)
        .write(true)
        .open(output_file)?;

    // Read URLs from the input file
    let mut value = Vec::new();
    let mut line = String::new();
    println!("待更新的链接（\"{}\"文件）：", input_file);
    println!("{}", split_symbol);
    while infile.read_line(&mut line)? > 0 {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            println!("|  - {}", trimmed.to_string());
            value.push(trimmed.to_string());
        }
        line.clear();
    }
    // 如果未读取到内容，执行 wait_for_enter() 和 process::exit(1)
    if value.is_empty() {
        print!("未读取到任何内容。按Enter键退出程序>>");
        io::stdout().flush().expect("刷新输出缓冲区失败"); // 刷新输出缓冲区
        wait_for_enter();
        process::exit(1);
    }
    println!("{}", split_symbol);
    // Read JSON data from the output file
    let json_data: HashMap<String, Vec<String>> = match Path::new(output_file).exists() {
        true => serde_json::from_reader(&mut outfile)?,
        false => HashMap::new(),
    };

    let mut all_keys: Vec<String> = json_data.keys().cloned().collect();
    all_keys.sort();
    println!("下面开始更新URL到JSON文件中（文件\"{}\"中的key-value键值对情况，如下）", output_file);
    println!("{}", split_symbol);
    // Print keys and values
    for key in &all_keys {
        println!("{}:", key);
        match json_data.get(key) {
            Some(values) if values.is_empty() => {
                println!("|  - []");
            }
            Some(values) => {
                for value in values {
                    println!("|  - {}", value);
                }
            }
            None => {
                println!("|  - []");
            }
        }
    }
    println!("{}", split_symbol);
    print!("请您输入要写入JSON文件的key键名：");
    io::stdout().flush().expect("刷新缓冲区失败");
    let mut update_key = String::new();
    loop {
        io::stdin().read_line(&mut update_key)?;
        update_key = update_key.trim().to_string();
        if !update_key.is_empty() {
            break;
        }
    }

    let mut updated_json_data = json_data.clone();
    updated_json_data.insert(update_key.clone(), value);
    outfile.seek(SeekFrom::Start(0))?;
    serde_json::to_writer_pretty(&mut outfile, &updated_json_data)?;

    println!("{}", split_symbol);
    println!(
        "成功将{}文件中的所有链接，添加到JSON文件的\"{}\"键中。",
        input_file, update_key
    );
    println!("{}", split_symbol);
    print!("按下Enter键关闭当前窗口...");
    io::stdout().flush().expect("刷新输出缓冲区失败"); // 刷新输出缓冲区
    wait_for_enter();
    process::exit(0);
}

fn wait_for_enter() {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("读取输入失败");
}