//! What:
//!  Runs a future on a current-thread runtime next to a task that counts every millisecond,
//! and says how often that task ran.
//! Why:
//!  A wait that holds its thread leaves the counting task no turn,
//! so a count above zero shows that the wait was awaited on the
//!  runtime and never slept the thread (the controls of the waits converted in `doc/planning/git-policy-cli-full-tokio.md`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! let ticks = 0; const timer = setInterval(() => ticks++, 1); const value = await future; clearInterval(timer); return [value, ticks];
//! ```

use std::time::Duration;

/// What:
///  Runs `future` on a current-thread runtime,
/// where one thread runs both the future and the counting task,
/// and returns the
///  future's value with the count.
/// Why:
///  One thread for both is what makes a blocking wait visible:
/// with a worker thread the counting task would run beside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [value, ticks] = await withATickerBeside(future);
/// ```
pub(crate) fn with_a_ticker_beside<Value>(future: impl std::future::Future<Output = Value>) -> (Value, u32) {
    let runtime: tokio::runtime::Runtime =
        tokio::runtime::Builder::new_current_thread().enable_all().build().expect("a current-thread runtime");
    let ticks: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let value: Value = runtime.block_on(async {
        let ticker = async {
            loop {
                tokio::time::sleep(Duration::from_millis(1)).await;
                ticks.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }
        };
        let outcome: Value = tokio::select! {
            value = future => value,
            () = ticker => unreachable!("the ticker never ends"),
        };
        return outcome;
    });
    return (value, ticks.load(std::sync::atomic::Ordering::SeqCst));
}
