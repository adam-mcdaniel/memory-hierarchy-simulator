pub mod cache;
pub mod config;
pub mod dc;
pub mod l2;
pub mod pagetable;
pub mod tlb;
pub mod trace;

pub use cache::*;
pub use config::*;
pub use dc::*;
pub use l2::*;
pub use pagetable::*;
pub use tlb::*;
pub use trace::*;

use std::io::{BufRead, BufReader, Read};

/// Read a line from the buffer, and panic if it is not equal to the given text.
pub(crate) fn get_header<R>(buffer: &mut BufReader<R>, text: &str)
where
    R: Read,
{
    let mut line = String::new();
    while line.trim() == "" {
        buffer.read_line(&mut line).unwrap();
    }

    if line.trim() != text {
        panic!("Expected \"{}\", got \"{}\"", text, line.trim());
    }
}

/// Read a line from the buffer. Treat the line as a key value pair. If the key doesn't match the given text, panic.
/// Return the key and the value parsed as a decimal number.
pub(crate) fn get_decimal<R>(buffer: &mut BufReader<R>, text: Option<&str>) -> Option<(String, u64)>
where
    R: Read,
{
    let mut line = String::new();
    while line.trim() == "" {
        if buffer.read_line(&mut line).unwrap() == 0 {
            return None;
        }
    }
    let mut split = line.split(':');
    let first = split.next().unwrap().trim();
    if let Some(text) = text {
        if split.clone().count() != 1 {
            panic!("Expected \"{}: {{number}}\", got \"{}\"", text, line.trim());
        }
        // Confirm that there is only one item in the split to consume (the decimal value)
        if first != text {
            panic!(
                "Expected \"{:?}: {{number}}\", got \"{}\"",
                text,
                line.trim()
            );
        }
    }
    Some((
        first.to_owned(),
        split.next().unwrap().trim().parse::<u64>().unwrap(),
    ))
}

/// Read a line from the buffer. Treat the line as a key value pair. If the key doesn't match the given text, panic.
/// Return the key and the value parsed as a hexadecimal number.
pub(crate) fn get_hexadecimal<R>(
    buffer: &mut BufReader<R>,
    text: Option<&str>,
) -> Option<(String, u64)>
where
    R: Read,
{
    let mut line = String::new();
    while line.trim() == "" {
        if buffer.read_line(&mut line).unwrap() == 0 {
            return None;
        }
    }
    let mut split = line.split(':');
    let first = split.next().unwrap().trim();
    if let Some(text) = text {
        // Confirm that there is only one item in the split to consume (the hex value)
        if split.clone().count() != 1 {
            panic!("Expected \"{}: {{number}}\", got \"{}\"", text, line.trim());
        }
        if first != text {
            panic!(
                "Expected \"{:?}: {{number}}\", got \"{}\"",
                text,
                line.trim()
            );
        }
    }
    let hex_str = split.next().unwrap().trim();
    Some((first.to_owned(), u64::from_str_radix(&hex_str, 16).unwrap()))
}

/// Read a line from the buffer. Treat the line as a key value pair. If the key doesn't match the given text, panic.
/// Return the key and the value parsed as a boolean value (read as "y" for true or "n" for false).
pub(crate) fn get_bool<R>(buffer: &mut BufReader<R>, text: Option<&str>) -> Option<(String, bool)>
where
    R: Read,
{
    let mut line = String::new();
    while line.trim() == "" {
        if buffer.read_line(&mut line).unwrap() == 0 {
            return None;
        }
    }
    let mut split = line.split(':');
    let first = split.next().unwrap().trim();
    if let Some(text) = text {
        // Confirm that there is only one item in the split to consume (the "y/n" value)
        if split.clone().count() != 1 {
            panic!("Expected \"{}: {{number}}\", got \"{}\"", text, line.trim());
        }
        if first != text {
            panic!(
                "Expected \"{:?}: {{number}}\", got \"{}\"",
                text,
                line.trim()
            );
        }
    }
    let value = split.next().unwrap().trim();
    if value == "y" || value == "Y" {
        Some((first.to_owned(), true))
    } else if value == "n" || value == "N" {
        Some((first.to_owned(), false))
    } else {
        panic!("Expected \"{{bool}}\", got \"{}\"", value);
    }
}
