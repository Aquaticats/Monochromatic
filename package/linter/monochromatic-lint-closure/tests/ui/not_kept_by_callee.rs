// edition:2024
// Closures that stay reported although they may capture state: the callee does not keep the closure, the closure is
// not a direct argument, or it captures nothing. Functions named `allowed_*` must produce no report, functions named
// `reported_*` must produce at least one.

use std::thread;

fn run_unbounded<F: FnOnce() + Send>(f: F) { f() }
fn run_static<F: FnOnce() + 'static>(f: F) { f() }
fn run_boxed_short<'a>(mut f: Box<dyn FnMut() + 'a>) { f() }

macro_rules! go {
    ($e:expr) => {
        std::thread::spawn($e)
    };
}

fn reported_catch_unwind() {
    let v = vec![1];
    // verdict: reported
    let _ = std::panic::catch_unwind(|| v.len());
}
fn reported_iterator_adaptor() {
    let v = vec![1, 2];
    let offset = 2;
    // verdict: reported
    let w: Vec<i32> = v.iter().map(|x| x + offset).collect();
    drop(w);
}
fn reported_let_bound_then_spawn() {
    let state = String::new();
    // verdict: reported
    let f = move || state.len();
    let _ = thread::spawn(f);
}
fn reported_capture_free_thread_spawn() {
    // verdict: reported
    let _ = thread::spawn(move || 1 + 1);
}
fn reported_capture_free_for_static_parameter() {
    // verdict: reported
    run_static(|| ());
}
fn reported_callee_without_static() {
    let s = String::new();
    // verdict: reported
    run_unbounded(move || drop(s));
}
fn reported_box_with_short_lifetime() {
    let s = String::new();
    // verdict: reported
    run_boxed_short(Box::new(|| { let _ = &s; }));
}
fn reported_scoped_threads() {
    let v = vec![1];
    // verdict: reported
    thread::scope(|s| {
        // verdict: reported
        s.spawn(|| v.len());
    });
}
fn reported_closure_in_assert_token_tree() {
    let v = vec![1, 2];
    let offset = 1;
    // verdict: reported
    assert!(v.iter().any(|x| *x > offset));
}
fn reported_returned_closure(n: i32) -> impl Fn() -> i32 {
    // verdict: reported
    return move || n + 1;
}
fn reported_called_in_place() {
    let n = 1;
    // verdict: reported
    let _ = (move || n + 1)();
}
fn reported_inner_closure_while_outer_spawn_closure_is_allowed() {
    let offset = 2;
    // verdict: allowed
    let _ = thread::spawn(move || {
        let v = vec![1, 2];
        // verdict: reported
        v.iter().map(|x| x + offset).sum::<i32>()
    });
}
macro_rules! call_unbounded {
    ($e:expr) => {
        run_unbounded($e)
    };
}

fn reported_closure_through_local_macro_wrapper_without_static() {
    let state = String::new();
    // verdict: reported
    call_unbounded!(move || drop(state));
}
fn allowed_async_block_is_not_a_closure_expression() {
    let s = String::new();
    // Neither the let-bound async block nor the async fn body below is a closure the author wrote.
    let _future = async move { let _ = &s; };
}
async fn async_fn_body_is_not_a_closure_expression(name: String) -> usize {
    return name.len();
}
fn allowed_closure_through_local_macro_wrapper() {
    let state = String::new();
    // verdict: allowed
    let _ = go!(move || state.len());
}

fn main() {}
