//! Minimal rustc-driver example to test basic functionality

#![feature(rustc_private)]

extern crate rustc_driver;

use rustc_driver::Callbacks;

struct EmptyCallbacks;

impl Callbacks for EmptyCallbacks {
    // Empty implementation - just let rustc do its normal thing
}

fn main() {
    // Get command-line arguments
    let args: Vec<String> = std::env::args().collect();

    println!("=== Minimal rustc-driver Example ===");
    println!("Args: {:?}", args);

    // Invoke rustc with empty callbacks
    // This tests that we can successfully link rustc_driver
    let mut callbacks = EmptyCallbacks;
    rustc_driver::RunCompiler::new(&args, &mut callbacks).run();
}
