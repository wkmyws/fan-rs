use clap::Parser;
#[derive(Debug, clap::Parser)]
#[command(author, version, about, arg_required_else_help(true))]
pub struct Cli {
    /// set the fan speed
    #[arg(short = 's', long, default_value_t = 0, value_parser = clap::value_parser!(u8).range(0..=9))]
    speed: u8,

    /// show cpu temperature
    #[arg(short = 't', long, default_value_t = false)]
    temp: bool,

    /// enable http service
    #[arg(short = 'l', long, default_value_t = false)]
    server: bool, // 启用服务

    /// bind http port
    #[arg(long,default_value_t=String::from("127.0.0.1:8567"))]
    addr: String,

    /// set the refresh interval for terminal information
    #[arg(long, default_value_t = 1000)]
    interval: u64, // 终端信息刷新间隔ms

    /// Automatically adjust the fan speed
    #[arg(short = 'a', long)]
    auto: Option<bool>, // 根据温度自动调整风速

    /// log file path
    #[arg(long, default_value_t = String::from("~/.fan-rs.log"))]
    log: String, // 日志文件路径
}

pub fn is_cli() -> bool {
    return Cli::parse().server == false;
}

pub fn is_auto() -> bool {
    if let Some(a) = Cli::parse().auto {
        return a;
    } else {
        if is_cli() {
            return false;
        } else {
            return true;
        }
    }
}

pub fn get_speed() -> u8 {
    return Cli::parse().speed;
}

pub fn get_temp() -> bool {
    return Cli::parse().temp;
}

pub fn get_server_addr() -> String {
    return Cli::parse().addr;
}

pub fn get_log_interval_millis() -> u64 {
    return Cli::parse().interval;
}

pub fn get_log_path() -> String {
    let log_path = Cli::parse().log;
    // use std::path;
    // let log_path = path::Path::new(&log_path);
    // return String::from(log_path.to_str().unwrap());
    let log_path = expanduser::expanduser(&log_path).unwrap();
    return String::from(log_path.to_str().unwrap());
}