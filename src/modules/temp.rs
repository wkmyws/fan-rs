use std::fs::File;
use std::io::Read;
use std::io::{BufRead, BufReader};
use std::thread;
use std::time::Duration;

pub fn get_cpu_usage(millis: u64) -> f32 {
    let times = 5; // 统计5次的均值
    let interval = Duration::from_millis(millis / times);
    let mut arr_usage: Vec<f32> = vec![];

    let mut total_time_prev: u64 = 0;
    let mut idle_time_prev: u64 = 0;

    for _ in 0..times {
        let file = File::open("/proc/stat").expect("Failed to open /proc/stat");
        let reader = BufReader::new(file);

        let mut total_time: u64 = 0;
        let mut idle_time: u64 = 0;

        for line in reader.lines() {
            let line = line.expect("Failed to read line from /proc/stat");
            let fields: Vec<&str> = line.split_whitespace().collect();

            if fields.len() > 0 && fields[0] == "cpu" {
                for (i, &field) in fields.iter().enumerate() {
                    if i > 0 {
                        let time: u64 = field.parse().expect("Failed to parse CPU time");
                        total_time += time;
                        if i == 4 {
                            idle_time = time;
                        }
                    }
                }
                break;
            }
        }

        let total_delta = total_time - total_time_prev;
        let idle_delta = idle_time - idle_time_prev;

        let usage = 1.0 - (idle_delta as f32) / (total_delta as f32);

        // println!("CPU Usage: {:.2}%", usage * 100.0);
        arr_usage.push(usage);

        total_time_prev = total_time;
        idle_time_prev = idle_time;

        thread::sleep(interval);
    }
    let ans = arr_usage.iter().sum::<f32>() / arr_usage.len() as f32 * 100 as f32;
    return ans;
}

pub fn get_temp() -> Option<f32> {
    // 读取cpu温度
    if let Ok(mut file) = std::fs::File::open("/sys/class/thermal/thermal_zone0/temp") {
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        let temp: f32 = content.trim().parse().unwrap();
        return Some(temp / 1000.0);
    }
    return None;
}

pub fn get_fan_level() -> u8 {
    if let Some(temp) = get_temp() {
        if temp > 65.0 {return 9};
    }
    let cpu_usage = get_cpu_usage(2000) as i32;
    if cpu_usage < 20 {
        return 0;
    }
    if cpu_usage < 30 {
        return 2;
    }
    if cpu_usage < 35 {
        return 4;
    }
    if cpu_usage < 45 {
        return 6;
    }
    if cpu_usage < 55 {
        return 8;
    }
    return 9;
}
