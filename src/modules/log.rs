use crate::modules::cli;
use chrono::{self, Datelike, Timelike};

fn get_cur_time() -> String {
    // 使用当地时间
    let local: chrono::DateTime<chrono::Local> = chrono::Local::now();
    let year = local.year();
    let month = local.month();
    let day = local.day();
    let hour = local.hour();
    let minute = local.minute();
    let second = local.second();
    let milliseconds = local.timestamp_millis() % 1000;
    format!(
        "[{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}]",
        year, month, day, hour, minute, second, milliseconds
    )
}

fn write_to_file(msg: &str) {
    let path = cli::get_log_path();
    // if path == "/dev/null" {
    //     return;
    // }
    use std::fs::OpenOptions;
    use std::io::Write;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .unwrap();
    // 每次写入一行
    file.write_all(format!("{}\n", msg).as_bytes()).unwrap();
}

pub fn info(msg: &str) {
    write_to_file(&format!("[INFO]{} {}", get_cur_time(), msg));
}

pub fn err(msg: &str) {
    write_to_file(&format!("[ERROR]{} {}", get_cur_time(), msg));
}

pub fn warn(msg: &str) {
    write_to_file(&format!("[WARN]{} {}", get_cur_time(), msg));
}
