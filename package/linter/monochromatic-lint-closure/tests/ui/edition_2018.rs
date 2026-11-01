// edition:2018
// Before edition 2021 a closure captures a whole variable even when it only writes `let _ = n`, so the same source
// that is reported in edition 2024 (see adversarial.rs, `reported_let_underscore_use_captures_nothing`) is allowed here.

use std::thread;

fn allowed_let_underscore_use_captures_the_variable_in_2018() {
    let n = 5;
    // verdict: allowed
    let _ = thread::spawn(move || { let _ = n; });
}

fn main() {}
