# Closure lint suppressions in the Android native crate

The Dylint lint `avoidable_closure` (`package/linter/monochromatic-lint-closure`) runs over this crate as the rule
`rust/avoidable-closure`.
The owner's decision `doc/decision/rust-closure-lint-dylint.md` reports a closure unless it captures state and is a
direct argument whose callee parameter requires `'static`.
The only suppression is rustc's own attribute on the statement that holds the closure:
`#[cfg_attr(dylint_lib = "monochromatic_lint_closure", expect(avoidable_closure, reason = "..."))]`.
`expect` makes rustc report `unfulfilled_lint_expectations` as soon as it suppresses nothing,
so a suppression cannot outlive its closure.
Configuration cannot do it instead:
the rule's settings give a severity and a library path per file pattern and no allow-list of callees
(`package/linter/monochromatic-lint/src/rust_rule_settings.rs`),
and lowering the severity would also let a replaceable closure in (`AGENTS.md` LN7).

This crate carries one suppression.
Every other closure that the rule found was replaced by a named function passed by path
(`PlayerError::no_audio_track`,
`OsStr::to_str`,
`CodecParameters::audio`,
`decode_failed`,
`install_subscriber`),
a loop (`track_by_id`),
a `match`,
or a value that implements `JniBody` (`ReadJavaString`,
`ArrayLength`,
`CopyArrayRegion`,
`PeakReplyBody`).

## `EnvUnowned::with_env` in `jni_call.rs`

`run_with_env` hands the work it was given to the JNI crate's `EnvUnowned::with_env`.
The signature is
`pub fn with_env<F, T, E>(&mut self, f: F) -> EnvOutcome<'local, T, E> where F: FnOnce(&mut Env<'local>) -> Result<T, E>`
(`jni-0.22.4/src/env.rs`,
line 4793).
It is the only entry that lends the `Env` the JNI calls live on,
and it takes a callable.
`F` has no `'static` bound,
so the lint's exemption for a state-capturing closure that the callee keeps does not apply.
The callable must carry the `body` value into the call,
so the closure captures `body`.
A named function cannot take its place,
because a function item has no captured state,
and no loop or `match` can,
because the call to `with_env` is the thing that produces the `Env`.
The crate offers no entry that takes a value in place of a callable,
and `FnOnce` cannot be implemented for a struct on stable Rust.

The five call sites that used to pass their own closures to `run_with_env` now pass `JniBody` values,
so this one closure stands for all of them.
`run_with_env` also stays the single place that logs a JNI error and a caught panic.
A control run that removed the attribute made the rule report this closure
(`jni_call.rs`,
the `let outcome = env.with_env(...)` statement),
so the suppression is needed and used.

## Build configuration

`file-enforcer.config.jsonc` lists this crate in `onlyDirs` of the rule `cargo-dylint`
(and the rule's `manifests` now include `package/*/*/*/Cargo.toml`,
the depth of this crate),
so `Cargo.toml` declares the `dylint_lib` cfg name that the attribute uses.
