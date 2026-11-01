// edition:2024
// Closures that capture state and are direct arguments of a parameter that requires `'static`.
// A function named `allowed_*` must produce no report. Its twin named `reported_*_without_static` has the same source
// with the `'static` requirement removed, and must produce a report (the positive control).

use std::thread;

struct Window;
impl Window {
    fn on_ping(&self, mut f: impl FnMut(i32) -> () + 'static) { f(1); }
    fn on_ping_unbounded(&self, mut f: impl FnMut(i32) -> ()) { f(1); }
}

struct Timer;
impl Timer {
    fn start(&self, mode: u8, interval: u32, mut callback: impl FnMut() + 'static) { let _ = (mode, interval); callback(); }
    fn start_unbounded(&self, mode: u8, interval: u32, mut callback: impl FnMut()) { let _ = (mode, interval); callback(); }
}

fn run_param<F: FnOnce() + 'static>(f: F) { f() }
fn run_param_unbounded<F: FnOnce()>(f: F) { f() }
fn run_where<F>(f: F) where F: FnOnce() + 'static { f() }
fn run_where_unbounded<F>(f: F) where F: FnOnce() { f() }
fn run_impl(mut f: impl FnMut() + Send + 'static) { f() }
fn run_impl_unbounded(mut f: impl FnMut() + Send) { f() }
fn run_boxed_explicit(mut f: Box<dyn FnMut() + 'static>) { f() }
fn run_boxed_implicit(mut f: Box<dyn FnMut()>) { f() }
fn run_boxed_short<'a>(mut f: Box<dyn FnMut() + 'a>) { f() }
fn run_generic_box<F: FnMut() + 'static>(mut f: Box<F>) { f() }
fn run_generic_box_unbounded<F: FnMut()>(mut f: Box<F>) { f() }

trait Spawner {
    fn go(&self, f: impl FnOnce() + 'static);
    fn go_unbounded(&self, f: impl FnOnce());
}
trait DynSpawner {
    fn go_boxed(&self, f: Box<dyn FnOnce() + 'static>);
    fn go_boxed_short<'a>(&self, f: Box<dyn FnOnce() + 'a>);
}
struct Pool;
impl Spawner for Pool {
    fn go(&self, f: impl FnOnce() + 'static) { f() }
    fn go_unbounded(&self, f: impl FnOnce()) { f() }
}

struct Holder<F> { f: Option<F> }
impl<F: FnMut() + 'static> Holder<F> { fn set(&mut self, f: F) { self.f = Some(f); } }
struct HolderUnbounded<F> { f: Option<F> }
impl<F: FnMut()> HolderUnbounded<F> { fn set(&mut self, f: F) { self.f = Some(f); } }

struct Stored { f: Box<dyn Fn() + 'static> }
impl Stored { fn new(f: Box<dyn Fn() + 'static>) -> Stored { Stored { f } } }
struct StoredShort<'a> { f: Box<dyn Fn() + 'a> }
impl<'a> StoredShort<'a> { fn new(f: Box<dyn Fn() + 'a>) -> StoredShort<'a> { StoredShort { f } } }

trait Task: 'static { fn run(self); }
impl<F: FnOnce() + 'static> Task for F { fn run(self) { self() } }
fn submit<T: Task>(t: T) { t.run() }
trait TaskUnbounded { fn run(self); }
impl<F: FnOnce()> TaskUnbounded for F { fn run(self) { self() } }
fn submit_unbounded<T: TaskUnbounded>(t: T) { t.run() }

fn keep_async<F: AsyncFnOnce() + 'static>(f: F) { let _ = f; }
fn keep_async_unbounded<F: AsyncFnOnce()>(f: F) { let _ = f; }

fn allowed_thread_spawn() {
    let state = String::from("x");
    // verdict: allowed
    let _ = thread::spawn(move || state.len());
}
fn allowed_thread_builder_spawn() {
    let state = String::from("x");
    // verdict: allowed
    let _ = thread::Builder::new().spawn(move || state.len());
}
fn allowed_parenthesised_argument() {
    let s = String::new();
    // verdict: allowed
    let _ = thread::spawn((move || s.len()));
}
fn allowed_setter_with_impl_trait() {
    let w = Window;
    let n = 5;
    // verdict: allowed
    w.on_ping(move |x| { let _ = x + n; });
}
fn reported_setter_with_impl_trait_without_static() {
    let w = Window;
    let n = 5;
    // verdict: reported
    w.on_ping_unbounded(move |x| { let _ = x + n; });
}
fn allowed_timer_start_third_argument() {
    let t = Timer;
    let n = 1;
    // verdict: allowed
    t.start(0, 10, move || { let _ = n + 1; });
}
fn reported_timer_start_without_static() {
    let t = Timer;
    let n = 1;
    // verdict: reported
    t.start_unbounded(0, 10, move || { let _ = n + 1; });
}
fn allowed_generic_parameter() {
    let s = String::new();
    // verdict: allowed
    run_param(move || drop(s));
}
fn reported_generic_parameter_without_static() {
    let s = String::new();
    // verdict: reported
    run_param_unbounded(move || drop(s));
}
fn allowed_where_clause() {
    let s = String::new();
    // verdict: allowed
    run_where(move || drop(s));
}
fn reported_where_clause_without_static() {
    let s = String::new();
    // verdict: reported
    run_where_unbounded(move || drop(s));
}
fn allowed_impl_trait_send_static() {
    let s = String::new();
    // verdict: allowed
    run_impl(move || { let _ = &s; });
}
fn reported_impl_trait_send_without_static() {
    let s = String::new();
    // verdict: reported
    run_impl_unbounded(move || { let _ = &s; });
}
fn allowed_box_dyn_explicit_static() {
    let s = String::new();
    // verdict: allowed
    run_boxed_explicit(Box::new(move || { let _ = &s; }));
}
fn allowed_box_dyn_default_object_bound() {
    let s = String::new();
    // verdict: allowed
    run_boxed_implicit(Box::new(move || { let _ = &s; }));
}
fn reported_box_dyn_short_lifetime() {
    let s = String::new();
    // verdict: reported
    run_boxed_short(Box::new(|| { let _ = &s; }));
}
fn allowed_generic_box_parameter() {
    let s = String::new();
    // verdict: allowed
    run_generic_box(Box::new(move || { let _ = &s; }));
}
fn reported_generic_box_parameter_without_static() {
    let s = String::new();
    // verdict: reported
    run_generic_box_unbounded(Box::new(move || { let _ = &s; }));
}
fn allowed_trait_method_concrete_receiver() {
    let s = String::new();
    // verdict: allowed
    Pool.go(move || drop(s));
}
fn reported_trait_method_concrete_receiver_without_static() {
    let s = String::new();
    // verdict: reported
    Pool.go_unbounded(move || drop(s));
}
fn allowed_trait_method_generic_receiver<T: Spawner>(t: &T) {
    let s = String::new();
    // verdict: allowed
    t.go(move || drop(s));
}
fn reported_trait_method_generic_receiver_without_static<T: Spawner>(t: &T) {
    let s = String::new();
    // verdict: reported
    t.go_unbounded(move || drop(s));
}
fn allowed_trait_method_ufcs() {
    let s = String::new();
    // verdict: allowed
    Spawner::go(&Pool, move || drop(s));
}
fn reported_trait_method_ufcs_without_static() {
    let s = String::new();
    // verdict: reported
    Spawner::go_unbounded(&Pool, move || drop(s));
}
fn allowed_trait_method_dyn_receiver(t: &dyn DynSpawner) {
    let s = String::new();
    // verdict: allowed
    t.go_boxed(Box::new(move || drop(s)));
}
fn reported_trait_method_dyn_receiver_short_lifetime(t: &dyn DynSpawner) {
    let s = String::new();
    // verdict: reported
    t.go_boxed_short(Box::new(move || drop(s)));
}
fn allowed_impl_level_bound() {
    let s = String::new();
    let mut h = Holder { f: None };
    // verdict: allowed
    h.set(move || { let _ = &s; });
}
fn reported_impl_level_bound_without_static() {
    let s = String::new();
    let mut h = HolderUnbounded { f: None };
    // verdict: reported
    h.set(move || { let _ = &s; });
}
fn allowed_constructor_taking_boxed_closure() {
    let s = String::new();
    // verdict: allowed
    let h = Stored::new(Box::new(move || { let _ = &s; }));
    (h.f)();
}
fn reported_constructor_taking_boxed_closure_without_static() {
    let s = String::new();
    // verdict: reported
    let h = StoredShort::new(Box::new(|| { let _ = &s; }));
    (h.f)();
}
fn reported_parenthesised_argument_without_static() {
    let s = String::new();
    // verdict: reported
    run_param_unbounded((move || drop(s)));
}
fn allowed_supertrait_bound() {
    let s = String::new();
    // verdict: allowed
    submit(move || drop(s));
}
fn reported_supertrait_without_static() {
    let s = String::new();
    // verdict: reported
    submit_unbounded(move || drop(s));
}
fn allowed_async_closure() {
    let s = String::new();
    // verdict: allowed
    keep_async(async move || { let _ = &s; });
}
fn reported_async_closure_without_static() {
    let s = String::new();
    // verdict: reported
    keep_async_unbounded(async move || { let _ = &s; });
}

fn main() {}
