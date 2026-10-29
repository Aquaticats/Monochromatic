# Slint 1.17 generated tail returns break denied Clippy `implicit_return` builds

## Symptom

A Rust crate that denies `clippy::implicit_return` compiles its maintained source,
then fails when `slint::include_modules!()` includes Slint-generated Rust.
Clippy reports the generated file under `target/<profile>/build/<crate>/out/app.rs`:

```txt
error: missing `return` statement
  --> target/release/build/terminal/.../out/app.rs:1910:14
   |
1910 |              :: core :: result :: Result :: Ok (self_rc) }
   |              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#implicit_return
```

`package/terminal/app/Cargo.toml:9-12` and
`package/music-player/desktop-app/Cargo.toml:9-12` keep the lint denied for package-owned Rust.
The failure appears only when each binary includes Slint's generated bindings.

## Root cause

Slint's public macro does not expand to generated tokens directly.
[`slint-ui/slint@v1.17.0`][slint-v1.17-source] `api/rs/slint/lib.rs:369-372` includes the file named by
`SLINT_INCLUDE_GENERATED`:

```rust
macro_rules! include_modules {
    () => {
        include!(env!("SLINT_INCLUDE_GENERATED"));
    };
}
```

The Rust generator deliberately emits several Clippy allowances,
but the version 1.17 header does not include the restriction-group `implicit_return` lint.
`slint-ui/slint@v1.17.0` `internal/compiler/generator/rust.rs:281-286` emits:

```rust
pub(super) fn generate_module_header() -> TokenStream {
    quote! {
        #![allow(non_snake_case, non_camel_case_types)]
        #![allow(unused_braces, unused_parens)]
        #![allow(clippy::all, clippy::pedantic, clippy::nursery)]
        #![allow(unknown_lints, if_let_rescope, tail_expr_drop_order)]
```

`clippy::all`,
`clippy::pedantic`,
and `clippy::nursery` do not contain restriction lints.
Clippy declares `implicit_return` in the restriction group at
[`rust-lang/rust-clippy@3211e31`][clippy-source] `clippy_lints/src/implicit_return.rs:13-40`:

```rust
declare_clippy_lint! {
    /// Checks for missing return statements at the end of a block.
    // ...
    pub IMPLICIT_RETURN,
    restriction,
    "use a return statement like `return expr` instead of an expression"
}
```

The generated implementation then uses tail expressions by design.
For example,
`slint-ui/slint@v1.17.0` `internal/compiler/generator/rust.rs:386-395` ends a generated constructor with:

```rust
pub fn new_with_existing_window(
    window: &slint::Window,
) -> ::core::result::Result<Self, slint::PlatformError> {
    // ...
    ::core::result::Result::Ok(Self(inner))
}
```

Clippy's `check_fn` passes non-unit function bodies to the implicit-return walker at
`rust-lang/rust-clippy@3211e31` `clippy_lints/src/implicit_return.rs:238-256`.
An `include!` file belongs to the consumer crate,
so the generated functions inherit the consumer's denied lint.

The initial hypothesis that an attribute on `slint::include_modules!()` would scope the lint was wrong.
rustc reports that the attribute is ignored on that macro invocation,
and the generated diagnostics remain.
A real module declaration is the required lint-level boundary.

## Verification

The incident was reproduced with:

- Slint `1.17.0`,
  crates.io `slint` checksum
  `a54a407d1a0cbaa71d830ae7c236064b171368ea18db3b51ea3f2ce3f19519ee`;
- Slint tag `v1.17.0`,
  commit `fdde7a535305d2ab2d4072dee637bad186a49723`;
- Clippy `0.1.99` from rustc `1.100.0-nightly` commit
  `c656540d6467dee1381f0cbd882412d6bd1cd5ae`;
- Fedora Linux x86-64.

The standalone probe used these files:

```toml
# Cargo.toml
[package]
name = "slint-implicit-return-probe"
version = "0.1.0"
edition = "2024"
publish = false

[lints.clippy]
implicit_return = "deny"
needless_return = "allow"

[dependencies]
slint = "=1.17.0"

[build-dependencies]
slint-build = "=1.17.0"

[workspace]
```

```rust
// build.rs
fn main() {
    slint_build::compile("ui/app.slint").expect("Slint generation should succeed");
}
```

```rust
// src/main.rs
slint::include_modules!();

fn main() {
    let _unused_window_result = AppWindow::new();
}
```

```slint
// ui/app.slint
export component AppWindow inherits Window {}
```

```toml
# mise.toml
[tasks.lint]
run = "cargo clippy -- --deny warnings"
```

Run:

```sh
mise run lint
```

### Failing catalog

- Direct `slint::include_modules!()` with `implicit_return = "deny"` produced
  58 generated-code diagnostics.
- `#[allow(clippy::implicit_return)]` directly on the macro produced
  `unused attribute 'allow'` and the same 58 generated-code diagnostics.
- Running `cargo clippy --fix` changed the current profile's ignored `target/.../out/app.rs`,
  but a release build regenerated an unfixed file and failed again.

### Passing catalog

- Wrapping `slint::include_modules!()` in a private module carrying
  `#[allow(clippy::implicit_return)]`,
  then importing its public bindings,
  completed with no diagnostics.
- Adding `clippy::implicit_return` to Slint's generated module header,
  rebuilding the compiler from the `v1.17.0` source tree,
  and rerunning the direct-include probe completed with no diagnostics.
- Package-owned functions with explicit `return` statements remain checked because
  both workarounds scope the allowance to generated code only.

## Verified workarounds

### Isolate generated bindings in a private module

Both affected binaries use this consumer-owned boundary
(the allowance list grew when the shadow lints were denied fleet-wide;
see "The shadow lints reach the same generated code"):

```rust
#[allow(clippy::implicit_return)]
#[allow(clippy::shadow_reuse, clippy::shadow_same, clippy::shadow_unrelated)]
mod slint_generated {
    slint::include_modules!();
}

use slint_generated::*;
```

The two crates apply it in their own shapes:
`package/terminal/app/src/main.rs` keeps the boundary inline in `main.rs`
with one wrapped `#[allow(...)]` listing all four lints,
and `package/music-player/desktop-app` carries the boundary in the sibling module file
`src/slint_generated.rs` with two stacked `#![allow(...)]` inner attributes.
Both `Cargo.toml` files continue denying `implicit_return` and the three shadow lints
for every maintained module.

Tradeoff:
generated bindings gain one namespace and a root import.
The wildcard import is needed because Slint determines the exported binding set from the markup.

### Patch Slint's generated module header

The upstream-compatible patch is one line:

```diff
diff --git a/internal/compiler/generator/rust.rs b/internal/compiler/generator/rust.rs
@@
-        #![allow(clippy::all, clippy::pedantic, clippy::nursery)]
+        #![allow(clippy::all, clippy::pedantic, clippy::nursery, clippy::implicit_return)]
```

The path-dependency probe failed before this patch and passed after it.

The same generated-header location would carry the three shadow lints
named in "The shadow lints reach the same generated code".

Tradeoff:
consumers need a patched Slint build or a release containing the change.
The repository therefore keeps the private-module workaround while using crates.io Slint `1.17.0`.

## The shadow lints reach the same generated code

Commit `53c1e01a9` denied `clippy::shadow_reuse`,
`clippy::shadow_same`,
and `clippy::shadow_unrelated` in the canonical `[lints.clippy]` policy,
and issue #604 tracked the fallout.
Slint's generated Rust
(resolved at `1.17.0` in both include-based crates' `Cargo.lock` files)
rebinds its own helpers over each other
(`self_rc`,
`_self`,
`self_weak`,
`the_struct`,
`window`,
`order`,
`visitor`,
`r#repeated_indices`),
so the included `out/app.rs` failed the new policy exactly like it failed `implicit_return`.
160 deduplicated generated findings sat behind `package/terminal/app/src/main.rs`.
The `package/music-player/desktop-app` include fails the same way:
a 2026-10-08 probe temporarily narrowed the boundary's shadow allowance
and watched the bin compile fail with 1884 errors dominated by the same binding families,
then pass again with the allowance restored.
While each crate's lib target failed to compile,
its bin target was never linted in those runs,
so the bin findings,
maintained and generated alike,
surfaced only once the lib was clean.

The private-module boundary absorbs these too:
both include-based boundaries allow the same lint set,
each in the shape its crate needs.
`package/terminal/app/src/main.rs` keeps the inline `mod slint_generated`
with one wrapped `#[allow(...)]` listing all four lints.
`package/music-player/desktop-app` carries the boundary in the sibling module file
`src/slint_generated.rs` with two stacked `#![allow(...)]` inner attributes.
The inline wrapped-list shape first used there pushed `main.rs`
past the 300-code-line `builtin(max-lines)` budget
(measured 2026-10-08:
304 code lines,
rejected),
and AGENTS.md `MXL` prescribes splitting into sibling files over compressing code to fit;
after the split,
`main.rs` measures 296 of 300 code lines.

`package/ide/app/src/native.rs` needs no shadow allowance.
Since commit `8e471dba6` (2026-10-04) its `mod ui` uses the `slint::slint!` re-export macro
instead of `slint::include_modules!()`,
so the generated code reaches the crate as a procedural-macro expansion rather than an included file,
and with Slint resolved at `1.18.1` in its `Cargo.lock`,
the 2026-10-08 run of `mise run //package/ide/app:lint:clippy`
reports no shadow findings in generated or maintained source.

Verified with `rustc 1.100.0-nightly (1303417c4 2026-09-21)`:
after these boundary allowances and the maintained-source renames tracked in issue #604,
each of the crates whose manifest carries the shadow policy passes its own Clippy task.

## What does not work

- **Attribute on the macro call**:
  rustc says `allow` is ignored on `slint::include_modules!()` and Clippy still inspects the included file.
- **Fixing `target/.../out/app.rs`**:
  the file is generated and profile-specific;
  `slint-build` overwrites it on the next relevant build.
- **Removing the package-level deny**:
  this makes generated output quiet by also permitting implicit returns in maintained Rust,
  which discards the intended policy.
- **Allowing the lint at crate root**:
  this also exempts maintained source and is broader than the generated boundary.

## Upstream filing decision

`.out-of-scope/` contains no exemption for Slint-generated Rust or Clippy diagnostics.
`.out-of-scope/cargo-workspace.md` mentions Slint only as a native dependency that may require containers,
not as an upstream-filing exclusion.

Searches across open and closed Slint issues and pull requests used
`implicit_return`,
`generated clippy lint`,
and `clippy generated Rust`.
They returned no matching report as of 2026-08-29.

1. **Is it really upstream's fault?**
   Yes.
   Slint owns the generated Rust and already emits lint allowances for generated-code style.
   The missing restriction allowance exposes consumer policy to code consumers cannot maintain.
2. **Can upstream fix it?**
   Yes.
   The verified fix changes the generated module header in one location.
3. **Are they supporting this use case?**
   Yes.
   `slint-build` plus `slint::include_modules!()` is Slint's documented Rust integration,
   and the generated header already accounts for Clippy.
4. **Would the repository welcome the contribution?**
   Yes.
   `CONTRIBUTING.md:4-6` welcomes issues and pull requests.
   The contribution guide,
   issue template,
   pull request template,
   and a [recent merged external contribution][slint-pr-11907] contain no AI-assistance ban.
5. **Will they likely fix it?**
   Yes.
   No contrary maintainer position or documented non-goal was found,
   and the change extends an existing generated-code lint boundary.
6. **Has a minimal compatible fix been prototyped?**
   Yes.
   The one-line patch passed the standalone path-dependency probe after the unmodified tag failed it.

The following draft is fileable after human review.
It discloses AI assistance even though the checked contribution policy does not require that disclosure.

~~~md
Title: Generated Rust does not suppress Clippy `implicit_return`

Slint 1.17.0 generated Rust already carries allowances for `clippy::all`,
`clippy::pedantic`, and `clippy::nursery`, but `implicit_return` belongs to the
restriction group. A consumer that denies this lint gets diagnostics from the
file included by `slint::include_modules!()`.

Minimal reproduction:

```toml
[lints.clippy]
implicit_return = "deny"
needless_return = "allow"

[dependencies]
slint = "=1.17.0"

[build-dependencies]
slint-build = "=1.17.0"
```

```rust
// build.rs
fn main() {
    slint_build::compile("ui/app.slint").expect("Slint generation should succeed");
}
```

```rust
// src/main.rs
slint::include_modules!();
fn main() {}
```

```slint
export component AppWindow inherits Window {}
```

`cargo clippy -- --deny warnings` reports `missing return statement` against
`target/.../out/app.rs`.

The source path is `internal/compiler/generator/rust.rs` in
`generate_module_header()`. This patch fixed the probe while preserving the
existing generated-code lint strategy:

```diff
-#![allow(clippy::all, clippy::pedantic, clippy::nursery)]
+#![allow(clippy::all, clippy::pedantic, clippy::nursery, clippy::implicit_return)]
```

I verified the unmodified `v1.17.0` source failed the path-dependency probe and
the patched source passed it. This report and prototype were prepared with AI
assistance and reviewed against the generated source and a runnable reproducer.
~~~

[slint-v1.17-source]: https://github.com/slint-ui/slint/tree/v1.17.0
[clippy-source]: https://github.com/rust-lang/rust-clippy/tree/3211e31c83180858d82fa2a2fa27641943ca6c7e
[slint-pr-11907]: https://github.com/slint-ui/slint/pull/11907
