use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File, OpenOptions};
use std::io::{self, prelude::*, BufReader, SeekFrom};
use std::path::Path;
use std::process;

// 创建或初始化文件，如果文件不存在则创建并写入初始内容
fn create_or_initialize_file(file_path: &str, initial_content: &[u8]) -> Result<(), Box<dyn Error>> {
    if !Path::new(file_path).exists() {
        println!("文件不存在，将创建并写入初始内容：{}", file_path);
        fs::write(file_path, initial_content)?;
    }
    Ok(())
}

// 从文件中读取 URLs
fn read_urls(file_path: &str, split_symbol: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let mut infile = BufReader::new(File::open(file_path)?);
    let mut value = Vec::new();
    let mut line = String::new();

    println!("待更新的链接(\"{}\"文件)：", file_path);
    println!("{}", split_symbol);

    while infile.read_line(&mut line)? > 0 {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            println!("| - {}", trimmed);
            value.push(trimmed.to_string());
        }
        line.clear();
    }

    if value.is_empty() {
        print!("未读取到任何内容。按Enter键退出程序！");
        io::stdout().flush().expect("刷新输出缓冲区失败");
        wait_for_enter();
        process::exit(1);
    }

    println!("{}", split_symbol);
    Ok(value)
}

// 从文件中读取 JSON 数据
fn read_json_data(file_path: &str) -> Result<HashMap<String, Vec<String>>, Box<dyn Error>> {
    let json_data: HashMap<String, Vec<String>> =
        match Path::new(file_path).exists() {
        true => serde_json::from_reader(File::open(file_path)?)?,
            false => HashMap::new(),
        };
    Ok(json_data)
}

// 打印 JSON 数据
fn print_json_data(json_data: &HashMap<String, Vec<String>>, split_symbol: &str, output_file: &str) {
    let mut all_keys: Vec<String> = json_data.keys().cloned().collect();
    all_keys.sort();

    println!("下面开始更新URL到JSON文件中（文件\"{}\"中的key-value键值对情况，如下）", output_file);
    println!("{}", split_symbol);

    for key in &all_keys {
        println!("{}:", key);

        match json_data.get(key) {
            Some(values) if values.is_empty() => println!("| - []"),
            Some(values) => values.iter().for_each(|value| println!("| - {}", value)),
            None => println!("| - []"),
        }
    }

    println!("{}", split_symbol);
}

// 更新 JSON 文件
fn update_json_file(file_path: &str, update_key: String, value: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut outfile = OpenOptions::new()
        .read(true)
        .write(true)
        .open(file_path)?;

    let mut updated_json_data = read_json_data(file_path)?;
    updated_json_data.insert(update_key.clone(), value);
    outfile.seek(SeekFrom::Start(0))?;
    serde_json::to_writer_pretty(&mut outfile, &updated_json_data)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let input_file = "url.txt";
    let output_file = "flat-json.json";
    let split_symbol: String = "-".repeat(105);
    
    // 创建或初始化输入文件和输出文件
    create_or_initialize_file(input_file, b"")?;
    create_or_initialize_file(output_file, b"{}")?;

    let value = read_urls(input_file, &split_symbol)?;
    let json_data = read_json_data(output_file);

    print_json_data(&json_data?, &split_symbol, output_file);

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
    // 更新 JSON 文件
    update_json_file(output_file, update_key.clone(), value)?;

    println!("{}", split_symbol);
    println!(
        "成功将{}文件中的所有链接，添加到JSON文件的\"{}\"键中。",
        input_file, update_key
    );
    println!("{}", split_symbol);
    print!("按下Enter键关闭当前窗口！");
    io::stdout().flush().expect("刷新输出缓冲区失败");
    wait_for_enter();
    process::exit(0);
}

// 等待用户按下 Enter 键的函数
fn wait_for_enter() {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("读取输入失败");
}
