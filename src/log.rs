use std::fmt::Display;

pub fn log_error<T: Display>(val: T) {
    println!("ERROR: {}", val);
}
