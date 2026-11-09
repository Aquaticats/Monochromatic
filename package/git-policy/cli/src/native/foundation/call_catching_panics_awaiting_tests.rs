//! Controls for the guarded await:
//! a return is a value,
//! and a panic in any one poll is an error carrying its words.

use super::*;

/// A runtime of the control's own,
/// one thread,
/// so no other test's tasks run on it.
fn runtime() -> tokio::runtime::Runtime {
    return tokio::runtime::Builder::new_current_thread().enable_all().build().expect("a current-thread runtime");
}

/// A future that returns gives back what it returned,
/// after as many awaits as it makes.
#[test]
fn an_await_that_returns_gives_back_its_value() {
    let value: Result<u32, String> = runtime().block_on(call_catching_panics_awaiting(async {
        let yielded: u32 = std::future::ready(1).await;
        let counted: u32 = std::future::ready(40).await;
        return counted + yielded + 1;
    }));
    assert_eq!(value, Ok(42));
}

/// A panic inside an await is the error that carries its words,
/// and the caller goes on.
#[test]
fn a_panic_inside_an_await_gives_back_its_words() {
    let caught: Result<u32, String> = runtime().block_on(call_catching_panics_awaiting(async {
        let yielded: u32 = std::future::ready(1).await;
        panic!("the awaited work broke after {yielded}");
    }));
    assert_eq!(caught, Err(String::from("the awaited work broke after 1")));
}

/// A payload that is neither text is reported as carrying no message,
/// as the guarded call of a plain function does.
#[test]
fn a_panic_without_words_says_so() {
    let caught: Result<u32, String> = runtime().block_on(call_catching_panics_awaiting(async {
        std::panic::panic_any(7_u32);
    }));
    assert_eq!(caught, Err(String::from("no message")));
}
