//! Research rustc Callbacks API to find TyCtxt access method

#![feature(rustc_private)]

extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;

use rustc_driver::{Callbacks, Compilation, RunCompiler};
use rustc_interface::{interface, Config};
use rustc_middle::ty::TyCtxt;
use rustc_session::config;

struct TyCtxtResearchCallbacks {
    found_tcx: bool,
}

impl Callbacks for TyCtxtResearchCallbacks {
    fn config(&mut self, config: &mut interface::Config) {
        println!("=== config() callback ===");
        println!("Config available, but no TyCtxt yet");
    }

    // Try to implement after_crate_root_parsing if it exists
    // This will cause a compile error if the signature is wrong,
    // which helps us discover the correct signature
}

impl TyCtxtResearchCallbacks {
    fn new() -> Self {
        Self { found_tcx: false }
    }
}

fn main() {
    println!("=== TyCtxt Research ===");
    println!("Testing different callback methods...");

    let args: Vec<String> = std::env::args().collect();
    let mut callbacks = TyCtxtResearchCallbacks::new();

    RunCompiler::new(&args, &mut callbacks).run();
}
