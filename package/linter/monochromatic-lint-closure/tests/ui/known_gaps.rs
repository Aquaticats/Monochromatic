// edition:2024
// Known gaps: shapes where the closure is kept by a callee but the lint does not see through the wrapper, so the
// closure stays reported. Each case names its gap. A future fix that recognises the wrapper turns the marker into
// `allowed` and the golden output shrinks. Cases D0 and D3c are controls without a wrapper effect.

use std::rc::Rc;
use std::sync::Arc;

fn keep<F: FnOnce() + 'static>(f: F) { f() }
fn keep_option<F: FnOnce() + 'static>(f: Option<F>) { if let Some(f) = f { f() } }
fn keep_rc(f: Rc<dyn Fn() + 'static>) { f() }
fn keep_arc(f: Arc<dyn Fn() + Send + Sync + 'static>) { f() }
fn keep_optional_box(f: Option<Box<dyn FnOnce() + 'static>>) { if let Some(f) = f { f() } }
fn keep_static_reference(f: &'static dyn Fn() -> usize) -> usize { f() }

struct FieldBox { f: Box<dyn Fn() + 'static> }
struct FieldGeneric<F: Fn() + 'static> { f: F }

fn control_unwrapped_generic_parameter() {
    let s = String::new();
    // verdict: allowed
    keep(move || drop(s));
}

fn gap_arc_new() {
    let s = String::new();
    // GAP: Arc::new(closure) into Arc<dyn Fn() + 'static>; only Box::new is looked through.
    // verdict: reported
    keep_arc(Arc::new(move || { let _ = &s; }));
}

fn keep_generic_arc<T: Send + Sync + 'static>(value: T) { drop(value) }
struct TupleWrapper<F: FnOnce() + 'static>(F);

fn gap_arc_new_into_generic_parameter() {
    let s = String::new();
    // GAP: Arc::new(closure) into a generic F: 'static; only Box::new is looked through,
    // and treating Arc::new like Box::new would turn this into an allowed case.
    // verdict: reported
    keep_generic_arc(Arc::new(move || { let _ = &s; }));
}

fn gap_tuple_struct_constructor() {
    let s = String::new();
    // GAP: a tuple struct constructor is not a declared function (DefKind::Ctor), so its F: 'static is not read.
    // verdict: reported
    let w = TupleWrapper(move || drop(s));
    (w.0)();
}

fn gap_rc_new() {
    let s = String::new();
    // GAP: Rc::new(closure) into Rc<dyn Fn() + 'static>; only Box::new is looked through.
    // verdict: reported
    keep_rc(Rc::new(move || { let _ = &s; }));
}

fn gap_some_wrapper() {
    let s = String::new();
    // GAP: Some(closure) into Option<F> with F: 'static; the callee of the closure is the constructor Some.
    // verdict: reported
    keep_option(Some(move || drop(s)));
}

fn gap_struct_field_box() {
    let s = String::new();
    // GAP: struct field of type Box<dyn Fn() + 'static>; the closure is not a call argument.
    // verdict: reported
    let h = FieldBox { f: Box::new(move || { let _ = &s; }) };
    (h.f)();
}

fn gap_struct_field_generic() {
    let s = String::new();
    // GAP: struct field of generic type F: Fn() + 'static; the closure is not a call argument.
    // verdict: reported
    let h = FieldGeneric { f: move || { let _ = &s; } };
    (h.f)();
}

fn gap_option_of_box() {
    let s = String::new();
    // GAP: Some(Box::new(closure)) into Option<Box<dyn FnOnce() + 'static>>; only the box is looked through.
    // verdict: reported
    keep_optional_box(Some(Box::new(move || drop(s))));
}

fn gap_leaked_static_reference() {
    let s = String::new();
    // GAP: Box::leak(Box::new(closure)) into &'static dyn Fn; reference parameters are not examined.
    // verdict: reported
    keep_static_reference(Box::leak(Box::new(move || s.len())));
}

fn gap_callee_is_a_generic_fn_trait_value<G: Fn(Box<dyn FnOnce() + 'static>)>(g: G) {
    let s = String::new();
    // GAP: the callee is a value of fn-trait type, so there is no declared function whose parameters could be read.
    // verdict: reported
    g(Box::new(move || drop(s)));
}

fn gap_callee_is_a_boxed_fn_trait_value(g: Box<dyn Fn(Box<dyn FnOnce() + 'static>)>) {
    let s = String::new();
    // GAP: same as above for a Box<dyn Fn(..)> value.
    // verdict: reported
    g(Box::new(move || drop(s)));
}

fn gap_callee_is_a_function_pointer(g: fn(Box<dyn FnOnce() + 'static>)) {
    let s = String::new();
    // GAP: the callee is a function pointer value; its parameter types are known but are not examined.
    // verdict: reported
    g(Box::new(move || drop(s)));
}

fn reported_capture_free_reference_closure_for_static_reference_parameter() {
    // The closure captures nothing, so it is reported whatever the parameter requires (the unwrapped twin of the
    // leaked-reference gap above).
    // verdict: reported
    keep_static_reference(&|| 1);
}

fn main() {}
