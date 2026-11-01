//! What:
//!      This module file is the private namespace around Rust emitted by
//!           Slint.
//!  The inner lint attributes apply only inside this namespace,
//!           while package-owned Rust in `main.rs` and its sibling modules
//!           remains under the manifest's denied `implicit_return` and shadow
//!           lints.
//! Why:
//!       Slint 1.17 emits tail-expression returns and rebinds generated
//!           helpers (`self_rc`,
//!  `_self`,
//!  `the_struct`,
//!  ...) over each other.
//!           Its generated header already exempts several Clippy groups but not
//!           these restriction lints,
//!  so this boundary carries the exemption
//!           until Slint includes them itself.
//!  The boundary lives in its own
//!           file (the pattern `main.rs` already uses for `ui_progress.rs` and
//!           friends) so the generated-code exemption and its attribute lines
//!           stay out of `main.rs`'s max-lines budget;
//!  see
//!           `doc/troubleshooting/slint-generated-rust-implicit-return.md`.
//! Gotcha:
//!    The direct attribute on `slint::include_modules!()` is ignored by
//!           rustc;
//!  a module boundary is required for the lint level to apply,
//!           and this file is that module.
//!  The exemption is stacked as two
//!           `#![allow]` attributes because one merged list would overflow the
//!           100-column format width.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export * from './app.slint.generated';
//! ```
#![allow(clippy::implicit_return)]
#![allow(clippy::shadow_reuse, clippy::shadow_same, clippy::shadow_unrelated)]

// What:     `slint::include_modules!()` includes build-time generated Rust.
// Why:      `AppWindow` and related UI bindings come from Slint markup.
//
// In TS you'd write (pseudocode):
// ```ts
// export * from './app.slint.generated';
// ```
slint::include_modules!();
