
static mut CUR_SPEED: u8 = 0;
use rppal::i2c::I2c;
fn get_fd() -> Result<I2c, Box<dyn std::error::Error>> {
    let mut i2c = I2c::new()?;
    i2c.set_slave_address(0x0d)?;
    Ok(i2c)
}
fn get_speed() -> u8 {
    return unsafe { CUR_SPEED };
}
fn set_speed(fd: &mut I2c, level: u8) -> Result<u8, Box<dyn std::error::Error>> {
    // 更正大小，原本是0最小1最大，现在是0最小9最大
    // 0123456789 -> 0234567891
    let mut num = level;
    if num % 9 == 0 {
        num = 9 - num; // 调换09
    }
    num = (num + 1) % 10;
    for _ in 0..10 {
        fd.smbus_write_byte(0x08, num)?;
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    fd.smbus_write_byte(0x08, num)?;
    unsafe { CUR_SPEED = level };
    Ok(level)
}

pub fn fan(speed: u8) -> Result<u8, String> {
    // set:0-9 get:10
    if speed == 10 {
        return Ok(get_speed());
    }
    return match get_fd() {
        Ok(mut fd) => match set_speed(&mut fd, speed) {
            Ok(speed) => Ok(speed),
            Err(err) => Err(format!("Failed to set speed: {err}")),
        },
        Err(err) => Err(format!("Failed to initialize I2C: {err}")),
    };
}
