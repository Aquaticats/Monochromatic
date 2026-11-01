// edition:2024
// Adversarial shapes: shadowed names, nested closures, macro-expanded calls, `move` without captures, `let _ = n`,
// trait-object callees and function pointer parameters.

use std::thread;

fn apply(f: fn(i32) -> i32) -> i32 { f(1) }
fn keep<F: FnOnce() + 'static>(f: F) { f() }

macro_rules! spawn_it {
    ($e:expr) => {
        std::thread::spawn($e)
    };
}

macro_rules! closure_inside_macro_body {
    ($n:expr) => {
        // verdict: reported
        std::thread::spawn(move || 1 + 1)
    };
}

macro_rules! capturing_closure_inside_macro_body {
    ($n:ident) => {
        // verdict: allowed
        std::thread::spawn(move || $n + 1)
    };
}

fn reported_parameter_shadows_outer_name() {
    let n = 1;
    // The closure parameter `n` shadows the outer `n`; the body uses only the parameter, so nothing is captured.
    // The inner closure `|n: i32| n + 1` is let-bound and captures nothing either, so the line has two reports.
    // verdict: reported 2
    let _ = thread::spawn(move || { let f = |n: i32| n + 1; f(2) });
}

fn reported_inner_local_shadows_outer_name() {
    let n = 1;
    // The closure declares its own `n`; the outer `n` is never mentioned inside, so nothing is captured.
    // verdict: reported
    let _ = thread::spawn(move || { let n = 3; n + 1 });
    let _ = n;
}

fn nested_outer_and_inner_are_classified_alone() {
    let offset = 2;
    let base = 10;
    // The outer closure captures `offset` and `base`, both through the inner closure or directly.
    // verdict: allowed
    let _ = thread::spawn(move || {
        // The inner closure is an argument of `map`, which does not keep it.
        // verdict: reported
        (1..3).map(|x| x + offset).sum::<i32>() + base
    });
}

fn nested_capture_free_inner_in_kept_outer() {
    let s = String::new();
    // verdict: allowed
    keep(move || {
        // verdict: reported
        let g = || 1 + 1;
        let _ = (g(), &s);
    });
}

struct Pair { a: String, b: String }

fn reported_two_fields_of_one_variable_are_listed_once() {
    let p = Pair { a: String::new(), b: String::new() };
    // Edition 2021 precise capture records `p.a` and `p.b` separately; the message names the variable `p` once.
    // verdict: reported
    let _ = (0..1).map(|_| p.a.len() + p.b.len()).count();
}

fn reported_move_closure_capturing_nothing() {
    // verdict: reported
    let _ = thread::spawn(move || 7);
}

fn reported_let_underscore_use_captures_nothing() {
    let n = 5;
    // Edition 2021 and later: `let _ = n` does not read `n`, so the closure captures nothing.
    // verdict: reported
    let _ = thread::spawn(move || { let _ = n; });
}

fn allowed_let_underscore_reference_captures() {
    let n = 5;
    // `let _ = &n` does read `n`, so the closure captures it.
    // verdict: allowed
    let _ = thread::spawn(move || { let _ = &n; });
}

fn allowed_macro_expanded_spawn_with_user_closure() {
    let state = String::new();
    // verdict: allowed
    let _ = spawn_it!(move || state.len());
}

fn reported_macro_expanded_capture_free_closure() {
    // verdict: reported
    let _ = spawn_it!(move || 3);
}

fn reported_closure_written_inside_macro_body_without_captures() {
    let _ = closure_inside_macro_body!(1);
}

fn allowed_closure_written_inside_macro_body_with_capture() {
    let n = 1;
    let _ = capturing_closure_inside_macro_body!(n);
}

fn reported_closure_passed_to_function_pointer_parameter() {
    // A function pointer parameter only accepts a capture-free closure.
    // verdict: reported
    let _ = apply(|x| x + 1);
}

fn reported_receiver_position() {
    let n = 1;
    // The closure is the receiver of the call, not an argument.
    // verdict: reported
    let _ = (move || n).clone()();
}

fn main() {}
