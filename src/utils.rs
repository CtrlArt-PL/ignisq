use chrono::Local;
use colored::*;

pub fn get_ts() -> ColoredString {
    Local::now()
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
        .truecolor(128, 128, 128)
}
