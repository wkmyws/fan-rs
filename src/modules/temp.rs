use std::fs::File;
use std::io::Read;
use std::io::{BufRead, BufReader};
use std::thread;
use std::time::Duration;

pub fn get_cpu_usage(millis: u64) -> f32 {

    fn read_cpu_stat() -> Vec<u64> {
        let file = File::open("/proc/stat").expect("Failed to open /proc/stat");
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line.expect("Failed to read line from /proc/stat");
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() > 0 && fields[0] == "cpu" { // 找到cpu所在的行
                // cpu user nice system idle iowait irq softirq steal guest guest_nice
                // ->
                // user nice system idle iowait irq softirq steal guest guest_nice
                return fields.iter().skip(1).map(|s| s.parse::<u64>().unwrap()).collect::<Vec<u64>>();
            }
        }
        panic!("Failed to read cpu stat");
    }

    fn calc_cpu(nums: Vec<u64>) -> (u64, u64) {
        // nums: user nice system idle iowait irq softirq steal guest guest_nice
        // ref : https://stackoverflow.com/questions/23367857/accurate-calculation-of-cpu-usage-given-in-percentage-in-linux
        let total = nums.iter().sum::<u64>();
        let idle = nums[3] + nums[4]; // idle + iowait
        // let non_idle = total - idle;
        return (total, idle);
    }

    let (pre_total, pre_idle) = calc_cpu(read_cpu_stat());
    thread::sleep(Duration::from_millis(millis));
    let (cur_total, cur_idle) = calc_cpu(read_cpu_stat());
    let diff_total = cur_total - pre_total;
    let diff_idle = cur_idle - pre_idle;
    return 100.0 * (diff_total - diff_idle) as f32 / diff_total as f32;
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
