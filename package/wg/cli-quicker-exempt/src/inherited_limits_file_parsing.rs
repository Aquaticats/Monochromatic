//! Reading the text of one cgroup limit file:
//! `io.max`,
//! `io.weight`,
//! `cpu.max`
//! and the plain limit files.

/// What one block device may be limited to,
/// and the weight the kernel uses when nothing sets one.
use super::{
    DEFAULT_IO_WEIGHT,
    DeviceLimits,
};
/// The per-device limits keyed by device.
use std::collections::BTreeMap;
/// Parse failures.
use std::io;

/// Reports a nonempty run of digits.
fn is_digits(text: &str) -> bool {
    return !text.is_empty() && text.bytes().all(is_digit_byte);
}

/// Reports an ASCII digit byte.
fn is_digit_byte(byte: u8) -> bool {
    return byte.is_ascii_digit();
}

/// Reports a `major:minor` device number.
fn is_device_number(word: &str) -> bool {
    return match word.split_once(':') {
        Some((major, minor)) => is_digits(major) && is_digits(minor),
        None => false,
    };
}

/// Parses `io.max`:
/// one line per device,
/// the `major:minor` and then `key=value` words,
/// where `max` is no limit.
/// A key this parser does not know is skipped,
/// so a newer kernel's extra key does not stop every move,
/// and a value that is neither `max` nor a number is an error.
pub fn parse_io_max(text: &str) -> io::Result<BTreeMap<String, DeviceLimits>> {
    let mut devices: BTreeMap<String, DeviceLimits> = BTreeMap::new();
    for line in text.lines() {
        let mut words = line.split_whitespace();
        let Some(device) = words.next() else {
            continue;
        };
        if !is_device_number(device) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("not an io.max device: {device:?}"),
            ));
        }
        let mut limits = DeviceLimits::default();
        for word in words {
            let Some((key, value)) = word.split_once('=') else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("not an io.max setting: {word:?}"),
                ));
            };
            let parsed = parse_limit(value)?;
            match key {
                "rbps" => limits.read_bandwidth = parsed,
                "wbps" => limits.write_bandwidth = parsed,
                "riops" => limits.read_iops = parsed,
                "wiops" => limits.write_iops = parsed,
                _unknown => {}
            }
        }
        if limits != DeviceLimits::default() {
            devices.insert(device.to_string(), limits);
        }
    }
    return Ok(devices);
}

/// Parses `io.weight`:
/// the `default` line holds the weight of the cgroup,
/// and lines for single devices are not read.
/// The kernel default is no weight to carry.
pub fn parse_io_weight(text: &str) -> io::Result<Option<u64>> {
    for line in text.lines() {
        let Some(rest) = line.strip_prefix("default ") else {
            continue;
        };
        return match rest.trim().parse::<u64>() {
            Ok(weight) if weight == DEFAULT_IO_WEIGHT => Ok(None),
            Ok(weight) => Ok(Some(weight)),
            Err(_error) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("not an io.weight default: {:?}", rest.trim()),
            )),
        };
    }
    return Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "an io.weight without a default line",
    ));
}

/// Parses a cgroup limit file:
/// `max` is unlimited and a number is the limit.
pub fn parse_limit(text: &str) -> io::Result<Option<u64>> {
    let trimmed = text.trim();
    if trimmed == "max" {
        return Ok(None);
    }
    return match trimmed.parse::<u64>() {
        Ok(number) => Ok(Some(number)),
        Err(_error) => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("not a cgroup limit: {trimmed:?}"),
        )),
    };
}

/// Parses `cpu.max`,
/// `max <period>` or `<quota> <period>`,
/// into microseconds of CPU time per second.
pub fn parse_cpu_max(text: &str) -> io::Result<Option<u64>> {
    let mut words = text.split_whitespace();
    let (Some(quota), Some(period)) = (words.next(), words.next()) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("not a cpu.max line: {:?}", text.trim()),
        ));
    };
    let Some(quota_value) = parse_limit(quota)? else {
        return Ok(None);
    };
    let Ok(period_value) = period.parse::<u64>() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("not a cpu.max period: {period:?}"),
        ));
    };
    if period_value == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "a cpu.max period of zero",
        ));
    }
    return Ok(Some(quota_value.saturating_mul(1_000_000) / period_value));
}
