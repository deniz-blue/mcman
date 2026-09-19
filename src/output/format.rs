use std::time::Duration;

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
const ELLIPSIS: char = '…';

pub fn columns(text: &str) -> u16 {
    text.width() as u16
}

pub fn ratio(done: u64, total: u64) -> f64 {
    if total == 0 {
        return 1.0;
    }

    done as f64 / total as f64
}

pub fn bytes(size: u64) -> String {
    let (scale, unit) = unit_of(size);

    format!("{} {unit}", scaled(size, scale))
}

pub fn byte_fraction(done: u64, total: u64) -> String {
    let (scale, unit) = unit_of(total);

    format!("{}/{} {unit}", scaled(done, scale), scaled(total, scale))
}

pub fn duration(time: Duration) -> String {
    let seconds = time.as_secs_f64();

    if seconds < 60.0 {
        format!("{seconds:.1}s")
    } else {
        format!("{}m {:02}s", time.as_secs() / 60, time.as_secs() % 60)
    }
}

pub fn clip(text: &str, width: u16) -> String {
    if columns(text) <= width {
        return text.to_owned();
    }

    let mut kept = 0;
    let mut used = width_of(ELLIPSIS);

    for character in text.chars().rev() {
        used += width_of(character);

        if used > width {
            break;
        }

        kept += character.len_utf8();
    }

    format!("{ELLIPSIS}{}", &text[text.len() - kept..])
}

fn width_of(character: char) -> u16 {
    character.width().unwrap_or(0) as u16
}

fn unit_of(size: u64) -> (f64, &'static str) {
    let mut scale = 1.0;

    for unit in UNITS {
        if (size as f64) < scale * 1024.0 {
            return (scale, unit);
        }
        scale *= 1024.0;
    }

    (scale, UNITS[UNITS.len() - 1])
}

fn scaled(size: u64, scale: f64) -> String {
    let value = size as f64 / scale;

    if scale > 1.0 && value < 100.0 {
        format!("{value:.1}")
    } else {
        format!("{}", value.round())
    }
}
