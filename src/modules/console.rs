use crate::modules::cli;
use crate::modules::fan;
use crate::modules::server::http_server::help as http_server_help;
use crate::modules::temp;
use crossterm::{
    cursor,
    event::{
        read, DisableBracketedPaste, DisableFocusChange, EnableBracketedPaste, EnableFocusChange,
        Event, KeyCode, KeyEvent, KeyModifiers,
    },
    execute, queue,
    terminal::{self, disable_raw_mode, enable_raw_mode},
};
use std::io::{self, stdout};
use std::sync::Arc;
use std::sync::Mutex;
use std::{thread, time};

use super::log;

static mut AUTO_MODE: bool = false;
static mut SCREEN_Y: u16 = 0;
static mut CPU_USAGE_INTERVAL_MILLIS: u64 = 100;

fn clear() {
    execute!(stdout(), terminal::Clear(terminal::ClearType::All)).unwrap();
    execute!(stdout(), cursor::MoveTo(0, 0)).unwrap();
}

fn render(force: bool) -> u16 {
    let mut part = false; // 只展示部分信息

    let mut ii = 0;
    execute!(stdout(), cursor::MoveTo(0, ii)).unwrap();
    ii += 1;

    unsafe {
        if SCREEN_Y == 0 {
            SCREEN_Y = terminal::size().unwrap().1;
        }
        if SCREEN_Y < 12 {
            println!("height of your terminal is too small to show any info");
            return 0;
        } else if SCREEN_Y < 23 {
            part = true;
            println!("height of your terminal is too small to show all info");
            execute!(stdout(), cursor::MoveTo(0, ii)).unwrap();
            ii += 1;
        }
    }

    let auto_mode = unsafe { AUTO_MODE };
    // execute!(stdout(), cursor::MoveTo(0, 1)).unwrap();
    let fan_level = fan::fan(10).unwrap();
    let help_body = http_server_help();
    let tmp = format!("{:>5.2}°C", temp::get_temp().unwrap_or(0.0));
    let usage = format!(
        "{:>6.2}%",
        temp::get_cpu_usage(unsafe { CPU_USAGE_INTERVAL_MILLIS })
    );

    if part == false {
        // 不是展示部分信息
        for line in help_body.split("\n") {
            if force == true {
                println!("{}", line);
            }
            execute!(stdout(), cursor::MoveTo(0, ii)).unwrap();
            ii += 1;
        }
    }
    let __version = format!(
        "| {:<20} {:>18} |",
        "Author: ".to_string() + env!("CARGO_PKG_AUTHORS"),
        "Version: ".to_string() + env!("CARGO_PKG_VERSION")
    );

    for line in vec![
        "┌-----------------------------------------┐".to_string(),
        // "|   Author : SuperYY     Version : 0.0.1  |".to_string(),
        __version.to_string(),
        "|-----------------------------------------|".to_string(),
        format!(
            "|auto mode : {:6}    fan_level : {:6} |",
            auto_mode, fan_level
        ),
        format!("|cpu  temp : {}   cpu usage : {}|", tmp, usage),
        format!("|log  path : {:29}|", cli::get_log_path().chars().take(29).collect::<String>()),
        "└-----------------------------------------┘".to_string(),
    ] {
        println!("{}", line);
        execute!(stdout(), cursor::MoveTo(0, ii)).unwrap();
        ii += 1;
    }

    for line in vec![
        "┌-----------------------------------------┐",
        "|   press 0~9 to manually set fan level   |",
        "|   press  a  to enable auto fan mode     |",
        "|   press  q  to quit                     |",
        "└-----------------------------------------┘",
    ] {
        if force == true {
            println!("{}", line);
        }
        execute!(stdout(), cursor::MoveTo(0, ii)).unwrap();
        ii += 1;
    }
    return ii;
}

fn print_events(shraed_automode: &Arc<Mutex<bool>>) -> io::Result<()> {
    loop {
        let event = read()?;
        if let Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers,
            kind: _,
            state: _,
        }) = event
        {
            if modifiers.contains(KeyModifiers::CONTROL) {
                clear();
                break;
            }
        }
        if event == Event::Key(KeyCode::Char('q').into()) {
            clear();
            break;
        }
        if let Event::Resize(_x, _y) = event {
            unsafe {
                SCREEN_Y = _y;
            }
            clear();
            render(true);
        }
        for key in 0..=9 {
            if event == Event::Key(KeyCode::Char(std::char::from_digit(key, 10).unwrap()).into()) {
                *(shraed_automode.lock().unwrap()) = false;
                println!("set speed at {}", key);
                log::info(&format!("[keyboard] set speed at {}", key));
                fan::fan(key as u8).unwrap();
                break;
            } else if event == Event::Key(KeyCode::Char('a').into()) {
                println!("auto mode on");
                log::info(&format!("[keyboard] auto mode on"));
                *(shraed_automode.lock().unwrap()) = true;
                break;
            }
        }
    }
    disable_raw_mode().unwrap();
    Ok(())
}

pub fn main(shraed_automode: Arc<Mutex<bool>>, flash_interval_millis: u64) {
    unsafe {
        AUTO_MODE = cli::is_auto();
    }
    let flash_interval_millis = unsafe {
        if CPU_USAGE_INTERVAL_MILLIS >= flash_interval_millis {
            1
        } else {
            flash_interval_millis - CPU_USAGE_INTERVAL_MILLIS
        }
    };
    enable_raw_mode().unwrap();
    let mut stdout = io::stdout();
    queue!(
        stdout,
        EnableBracketedPaste,
        cursor::Hide,
        EnableFocusChange
    )
    .unwrap();
    let shared_automode2 = Arc::clone(&shraed_automode);
    thread::spawn(move || {
        clear();
        render(true);
        loop {
            thread::sleep(time::Duration::from_millis(flash_interval_millis));
            let auto_mode = *(shared_automode2.lock().unwrap());
            unsafe {
                AUTO_MODE = auto_mode;
            }
            render(false);
        }
    });
    if let Err(e) = print_events(&shraed_automode) {
        log::err(&format!("print_events error : {:?}", e));
    }
    queue!(
        stdout,
        DisableBracketedPaste,
        cursor::Show,
        DisableFocusChange
    )
    .unwrap();
    disable_raw_mode().unwrap();
}
