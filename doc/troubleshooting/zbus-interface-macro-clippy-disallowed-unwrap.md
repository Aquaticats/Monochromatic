# zbus 5.19.0: the `zbus::interface` macro expands to `Result::unwrap`, tripping clippy `disallowed_methods`

An `impl` block annotated with `#[zbus::interface(...)]` makes clippy report
`use of a disallowed method std::result::Result::unwrap` in crates whose root `clippy.toml` bans that method,
although the block contains no `unwrap`.
The error points at the `#[zbus::interface(...)]` attribute line.

Found while applying the root `clippy.toml` inside the container lint of `package/nested-wayland-session/cli`.
Before the container received that file,
the ban was not applied there and the nine interface blocks of the crate passed.

## Symptom

```txt
error: use of a disallowed method `std::result::Result::unwrap`
   --> src/appearance_portal.rs:167:1
    |
167 | #[zbus::interface(name = "org.freedesktop.portal.Settings")]
    | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: handle errors with ? or explain intentional panic with expect
    = note: this error originates in the attribute macro `zbus::interface`
```

One error appears per annotated `impl` block, nine in `package/nested-wayland-session/cli/src`.

## Root cause

The macro generates a method `introspect_to_writer` on a trait impl that it emits next to the annotated block.
That method writes the introspection XML with `writeln!` into a `&mut dyn std::fmt::Write` and unwraps each result.
Source: `zbus_macros-5.19.0/src/iface.rs:994` to `:1007` and the signal and method helpers near `:1168` in the
crates.io source as vendored in the local cargo registry.

```rust
// zbus_macros-5.19.0/src/iface.rs:1007
::std::writeln!(writer, r#"{:indent$}</interface>"#, "", indent = level).unwrap();
```

The clippy lint `disallowed_methods` reads the expanded code and has no notion that this write cannot fail,
and `clippy.toml` has no option to skip code produced by an external macro.

## What does not work

An `#[allow(clippy::disallowed_methods)]` attribute placed directly above `#[zbus::interface(...)]` has no effect.
The generated trait impl is a sibling item of the annotated block, so a lint level on the block does not reach it.
Observed: all nine errors remained with that attribute on each block, in the container lint run with the root
`clippy.toml` mounted.

## Verified workaround

Wrap each annotated `impl` in an anonymous constant and put the allow on that constant:

```rust
// The allow covers only this block.
#[allow(
    clippy::disallowed_methods,
    reason = "zbus::interface expands to writeln!(..).unwrap() in its generated introspection method"
)]
const _: () = {
    #[zbus::interface(name = "org.a11y.Bus")]
    impl BusLauncher {
        fn get_address(&self) -> String {
            return self.address.clone()
        }
    }
};
```

The generated sibling item lands inside the same block, so the allow covers it and nothing else in the file.
Method privacy is unchanged, because a block is not a module.
Observed: `cargo clippy --release --all-targets -- -D warnings` with the root `clippy.toml` in effect passes,
and a planted `Result::unwrap` elsewhere in `src/lib.rs` still fails, so the allow does not widen beyond its block.

A file-level `#![allow(clippy::disallowed_methods)]` was not tried, because it would silence the ban for the
hand-written code in the same file.

## Upstream filing decision

Not filed.
`zbus` unwraps a `fmt::Write` into a string buffer on purpose, and the lint behaves as documented;
the collision is a local-policy interaction, the same shape as
`doc/troubleshooting/serde-json-macro-clippy-disallowed-unwrap.md`.
