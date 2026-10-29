# gtk4-rs 0.11: bumping the feature level to v4_12 turns CssProvider::load_from_data into a deprecation warning that fails clippy -D warnings builds

Raising the gtk4 crate's API feature gate (here `v4_10` to `v4_12`,
 needed for
`ListView::scroll_to`) newly exposes deprecation attributes for APIs deprecated at or below
the new level.
`CssProvider::load_from_data` is deprecated since GTK 4.12,
 so a crate that compiled clean at
`v4_10` starts failing `cargo clippy -- --deny warnings` at `v4_12` with no code change of its
own.

Found while building `package/file-manager/variant-gtk-sticky`.

## Symptom

```txt
warning: use of deprecated method `gtk4::CssProvider::load_from_data`: Since 4.12
```

emitted for each call site;
 under this repo's `lint:clippy` task
(`cargo clippy --release -- --deny warnings`) the warnings are errors and the build fails.
Dependencies compiled into the same build graph with unified features (here the original
`file-manager` crate,
 still written against `v4_10`) emit the same warnings,
 but as
warnings only,
 since `-D warnings` applies to the local crate.

## Root cause

Cargo unifies features additively across a build graph,
 so enabling `v4_12` anywhere enables
it for every gtk4 user in that graph.
gtk4-rs gates deprecation attributes on those version features;
 at `v4_12`,
`load_from_data` carries `#[deprecated = "Since 4.12"]` (mirroring the C API,
 which replaced
it with `gtk_css_provider_load_from_string`).
The interaction is by design on both sides;
 the footgun is that a feature bump made for one
API (`scroll_to`) changes the lint status of unrelated existing code.

## Verification

Environment:
 gtk4-rs 0.11 over system GTK 4.22.4,
 rustc via the repo toolchain.

- `gtk4 = { version = "0.11", features = ["v4_10"] }` plus `provider.load_from_data(CSS)`:
  builds and clippy-passes.
- Same code with `features = ["v4_12"]`:
   two deprecation warnings (one per call site),
  clippy exits nonzero under `--deny warnings`.
- Replacing with `provider.load_from_string(CSS)`:
   clean at `v4_12`.

## Verified workarounds

- Use the replacement API:
   `load_from_string` (same argument,
   same behavior for UTF-8 CSS).
  Applied in `package/file-manager/app/src/style.rs` and
  `package/file-manager/variant-gtk-sticky/src/style.rs`.
  Both packages expose GTK 4.12 APIs,
  so standalone and unified build graphs use the same non-deprecated call.
  Tradeoff:
  GTK 4.12 is now the minimum supported API level.

## What does not work

- `#[allow(deprecated)]` at the call site:
   works mechanically but violates this repo's
  lint-suppression policy for a case with a drop-in replacement;
   not used.

## Upstream filing decision

`.out-of-scope/` was checked:
 no gtk4-rs exemption exists.

1. Really upstream's fault?
    No;
    deprecation gating on version features is the intended design,
   and the replacement exists.
2. Can upstream fix it?
    Nothing to fix.
3. Supported use case?
    Yes;
    documented deprecation with named successor.
4. Would the repo welcome the contribution?
    Not evaluated;
    constraint 1 fails.
5. Will they likely fix it?
    Nothing to fix.
6. Prototyped minimal fix?
    Not applicable;
    consumer-side one-line rename recorded above.

Decision:
 nothing to file.
