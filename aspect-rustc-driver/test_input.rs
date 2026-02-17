//! Test input file for aspect-rustc-driver
//!
//! This file contains various functions to test MIR extraction

pub fn public_function(x: i32) -> i32 {
    x * 2
}

fn private_function(s: &str) -> String {
    s.to_uppercase()
}

pub async fn async_function(id: u64) -> Result<String, ()> {
    Ok(format!("User {}", id))
}

pub fn generic_function<T: Clone>(value: T) -> T {
    value.clone()
}

mod api {
    pub fn fetch_data(url: &str) -> Vec<u8> {
        url.as_bytes().to_vec()
    }

    pub fn process_data(data: &[u8]) -> String {
        String::from_utf8_lossy(data).to_string()
    }
}

mod internal {
    fn helper_function() -> bool {
        true
    }
}

pub struct Calculator;

impl Calculator {
    pub fn add(a: i32, b: i32) -> i32 {
        a + b
    }

    pub fn multiply(&self, a: i32, b: i32) -> i32 {
        a * b
    }
}
