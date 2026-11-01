// edition:2024
// A dependency-style crate whose exported macros expand to closures and calls. It stands in for a macro from another
// crate: spans produced by its expansions count as external to the fixture that uses it.

#[macro_export]
macro_rules! spawn_arg {
    ($e:expr) => {
        std::thread::spawn($e)
    };
}

#[macro_export]
macro_rules! apply_arg {
    ($e:expr) => {
        std::iter::once(1).map($e).count()
    };
}

#[macro_export]
macro_rules! make_capture_free_closure {
    () => {
        std::thread::spawn(move || 7)
    };
}

#[macro_export]
macro_rules! make_borrowing_closure {
    ($v:expr) => {
        $v.iter().map(|x| x + $v.len()).count()
    };
}
