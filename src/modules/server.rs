pub mod http_server {
    use crate::modules::cli;
    use crate::modules::fan;
    use crate::modules::log;
    use crate::modules::temp;
    use regex::Regex;
    use std::io::Read;
    use std::io::Write;
    use std::net::{TcpListener, TcpStream};
    use std::sync::Arc;
    use std::sync::Mutex;
    // use std::thread;

    fn key_value(key: &str, value: &str, shraed_automode: &Arc<Mutex<bool>>) -> String {
        /*
         * speed,get => get cur fan's speed
         * speed,[0-9] => set cur fan's speed
         * temp,get => get cpu's temp
         * help,get => get help
         */
        match key {
            "speed" => {
                if value == "get" {
                    return fan::fan(10u8).unwrap().to_string();
                }
                if value == "auto" {
                    *(shraed_automode.lock().unwrap()) = true;
                    return "auto".to_string();
                }
                *(shraed_automode.lock().unwrap()) = false;
                let speed: u8 = value.parse().unwrap();
                return fan::fan(speed).unwrap().to_string();
            }
            "temp" => {
                if value == "get" {
                    return temp::get_temp().unwrap_or(0.0).to_string();
                }
                return format!("unkown value : {value}");
            }
            "help" => {
                if value == "get" {
                    return self::help();
                }
                return format!("unkown value : {value}");
            }
            other_key => format!("unkown key : {other_key}"),
        }
    }

    fn handle_client(mut stream: TcpStream, tx: &Arc<Mutex<bool>>) {
        let mut buffer = [0; 1024];
        stream.read(&mut buffer).unwrap();

        let request = String::from_utf8_lossy(&buffer[..]);
        let response = if let Some((key, value)) = parse_query_string(&request) {
            key_value(&key, &value, tx)
        } else {
            "Invalid key-value pair".to_string()
        };
        let response = format!("HTTP/1.1 200 OK\r\n\r\n{}\n", response);
        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }

    fn parse_query_string(request: &str) -> Option<(String, String)> {
        let pattern = Regex::new(r"^GET /\?(\w+)=(\w+)&(\w+)=(\w+)").unwrap();
        let mut key = String::from("");
        let mut value = String::from("");
        let mut err = String::from("");
        if let Some(cp) = pattern.captures(request) {
            match cp.get(1).unwrap().as_str() {
                "key" => key = cp.get(2).unwrap().as_str().to_string(),
                "value" => value = cp.get(2).unwrap().as_str().to_string(),
                other => err = format!("unexpected key : {other}"),
            }
            match cp.get(3).unwrap().as_str() {
                "key" => key = cp.get(4).unwrap().as_str().to_string(),
                "value" => value = cp.get(4).unwrap().as_str().to_string(),
                other => err = format!("unexpected key : {other}"),
            }
            if err != "" {
                log::err(&format!("parse query string error : {}", err));
                return None;
            }
        }
        Some((key.to_string(), value.to_string()))
    }

    pub fn main(addr: &String, shraed_automode: Arc<Mutex<bool>>) {
        let listener = TcpListener::bind(addr).unwrap();
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            handle_client(stream, &shraed_automode);
        }
    }

    pub fn help() -> String {
        let addr_info = format!(
            "GET http://{}?key=<key>&value=<value>",
            cli::get_server_addr()
        );
        let addr_info = format!("|{:^55}|", addr_info);
        let info = vec![
            "┌-------------------------------------------------------┐",
            "| key |value|      desc           |         demo        |",
            "|-------------------------------------------------------|",
            "|speed| get | get cur fan's speed | key=speed&value=get |",
            "|speed| auto| auto set by cpu tmp | key=speed&value=auto|",
            "|speed| 0-9 | set cur fan's speed | key=speed&value=9   |",
            "|temp | get | get cpu's temp      | key=temp&value=get  |",
            "|help | get | get help            | key=help&value=get  |",
            "|-------------------------------------------------------|",
            &addr_info,
            "└-------------------------------------------------------┘",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
        .join("\n");
        return info;
    }
}

mod auto_fan {
    use crate::modules::{fan, log, temp};
    use std::sync::{Arc, Mutex};
    use std::{thread, time::Duration};
    pub fn main(shraed_automode: Arc<Mutex<bool>>) {
        loop {
            let auto_mode = *(shraed_automode.lock().unwrap());
            if auto_mode == true {
                let cur_level = fan::fan(10).unwrap();
                let next_level = temp::get_fan_level();
                if next_level != cur_level {
                    // fan::fan(next_level).unwrap();
                    while let Err(err) = fan::fan(next_level) {
                        log::err(&format!("auto fan error : {}", err));
                        thread::sleep(Duration::from_secs(1));
                    }
                    log::info(&format!("auto fan : {} -> {}", cur_level, next_level));
                }
            }
            //
            thread::sleep(Duration::from_secs(5));
        }
    }
}

pub mod server {
    use super::{auto_fan, http_server};
    use crate::modules::{cli, console, fan, log};
    use std::{
        sync::{Arc, Mutex},
        thread,
    };

    pub fn main(flash_interval_millis: u64) {
        let addr = cli::get_server_addr();
        let auto_mode = cli::is_auto();

        let shraed_automode = Arc::new(Mutex::new(auto_mode));
        let shraed_automode1 = Arc::clone(&shraed_automode);
        let shraed_automode2 = Arc::clone(&shraed_automode);
        let shraed_automode3 = Arc::clone(&shraed_automode);

        thread::spawn(move || {
            // 监听http请求
            let shraed_automode = Arc::clone(&shraed_automode1);
            if let Err(_) = std::panic::catch_unwind(|| {
                http_server::main(&addr, shraed_automode);
            }) {
                std::process::exit(-1);
            }
        });
        thread::spawn(move || {
            // init the fan
            fan::fan(0).unwrap();
            // 自动调节风扇
            let shraed_automode = Arc::clone(&shraed_automode2);
            if let Err(_) = std::panic::catch_unwind(|| {
                auto_fan::main(shraed_automode);
            }) {
                std::process::exit(-2);
            }
        });
        let thread_render = thread::spawn(move || {
            // 绘制控制台
            let shraed_automode = Arc::clone(&shraed_automode3);
            console::main(shraed_automode, flash_interval_millis);
        });
        log::info(&format!("fan-rs start at {} ~\n", cli::get_server_addr()));
        thread_render.join().unwrap();
        log::info(&format!("fan-rs exit ~\n"));
    }
}
