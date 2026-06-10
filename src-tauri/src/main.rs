//! Llaunchpad binary entry point.
//!
//! On desktop we hide the console window in release builds. The whole
//! app lives in the library crate (`llaunchpad_lib`) so it can also
//! be reused from a mobile or test harness.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    llaunchpad_lib::run();
}
