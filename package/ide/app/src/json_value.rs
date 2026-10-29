//! The one place where the repository's ban on `Result::unwrap` is lifted for a macro expansion.

/// What:
///  Build a JSON value from literal syntax,
///  exactly as `serde_json::json!` does.
///  The expansion is wrapped in a `let` statement that carries the one lint allowance.
/// Why:
///  `json!` expands every string,
///  number,
///  or interpolated expression it holds into `to_value(&value).unwrap()`.
///  The repository's root `clippy.toml` disallows `Result::unwrap`,
///  and clippy reports the call at the `json!` invocation although the source holds no `unwrap`.
///  No clippy option skips code that an external macro produced,
///  so the allowance sits on the statement that holds the expansion and covers nothing else.
///  A hand-written `unwrap` beside it is still reported.
///  Serialising a string,
///  a number,
///  or a value built from `json!` literals cannot fail.
///  Evidence and the alternatives that were rejected:
///  `doc/troubleshooting/serde-json-macro-clippy-disallowed-unwrap.md`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const value = { label: 'scripted edit', edit: { changes: {} } };
/// ```
#[macro_export]
macro_rules! json_value {
    ($($json:tt)+) => {{
        #[allow(
            clippy::disallowed_methods,
            reason = "json! expands to to_value(..).unwrap(); see doc/troubleshooting/serde-json-macro-clippy-disallowed-unwrap.md"
        )]
        let value = ::serde_json::json!($($json)+);
        value
    }};
}
