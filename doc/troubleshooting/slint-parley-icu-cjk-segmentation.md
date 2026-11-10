# Slint/parley debug builds spam "ICU4X data error: No segmentation model for language: ja" on CJK filenames

A Slint app whose femtovg renderer lays out text with parley 0.9 floods stderr
with `ICU4X data error: No segmentation model for language: ja` (one line per
text-layout pass) when it displays Japanese,
 Chinese,
 Thai,
 Khmer,
 Lao,
 or
Burmese text.
 The text still renders.
 The flood appears only in DEBUG builds and
is not controllable with `RUST_LOG`.
 Enabling `icu_provider`'s `logging` feature
silences it.

## Symptom

Opening a folder of tracks with Japanese filenames (music-player on macOS,
 run as
the debug binary) prints,
 hundreds of times:

```text
ICU4X data error: No segmentation model for language: ja
ICU4X data error: No segmentation model for language: ja
...
```

- One line per layout pass per complex-script run,
   so it scales with redraws (627
  to 771 lines observed across a few seconds of one folder load).
- The window and text render fine;
   this is stderr noise,
   not a crash.
- `RUST_LOG=off`,
   `RUST_LOG=error`,
   and `RUST_LOG=warn,icu_provider=off` do NOT
  reduce it (still ~630 lines).
- Release builds print zero such lines.

The language tag in the message varies by script:
 `ja` (Chinese/Japanese),
 `th`
(Thai),
 `km`,
 `lo`,
 `my`.

## Root cause

A three-layer chain:
 parley omits CJK dictionary data,
 icu_segmenter warns it
cannot segment,
 and icu_provider's debug-only log shim turns that warn into a raw
`eprintln!`.

### 1. parley builds a line segmenter without complex-script data

`parley-0.9.0/src/analysis/mod.rs:56` (and `:63`,
 `:70`) constructs the line
segmenter with the non-complex constructor:

```rust
LineSegmenter::new_for_non_complex_scripts(opt)
```

This bundles no dictionary or LSTM model for the languages that need one
(Chinese/Japanese share a dictionary;
 Thai/Khmer/Lao/Burmese have their own).
parley does this deliberately to keep the binary small.
 Slint's femtovg renderer
uses parley for text layout,
 so every Slint femtovg app inherits this.

### 2. icu_segmenter warns when asked to segment a complex run it has no model for

When laying out a run of Japanese,
 icu_segmenter calls into its complex path.
`icu_segmenter-2.2.0/src/complex/mod.rs:84` selects a model by language and,
 on a
miss,
 attaches a display context to a `DataError`:

```rust
fn select(&self, language: Language) -> Option<DictOrLstmBorrowed<'data>> {
    const ERR: DataError = DataError::custom("No segmentation model for language");
    match language {
        // ...
        Language::ChineseOrJapanese => self.ja.map(DictOrLstmBorrowed::Dict).or_else(|| {
            ERR.with_display_context("ja");
            None
        }),
        // ...
    }
}
```

The `with_display_context("ja")` result is discarded;
 it is called purely for the
side effect of logging.
 `self.ja` is `None` because of layer 1,
 so this fires for
every Japanese run.

### 3. icu_provider's log shim is a raw `eprintln!` in debug builds

`DataError::with_display_context` logs through icu_provider's internal `log`
module (`icu_provider-2.2.0/src/error.rs:223`):

```rust
pub fn with_display_context<D: fmt::Display + ?Sized>(self, context: &D) -> Self {
    // ...
    log::warn!("{self}: {context}");
    // ...
}
```

That `log` is NOT the `log` crate by default.
 `icu_provider`'s `logging` feature
gates a real-`log` reexport,
 and `log` is an optional dependency
(`icu_provider-2.2.0/Cargo.toml:95` `logging = ["dep:log"]`,
 `:136`
`[dependencies.log]` optional).
 With `logging` OFF,
 icu_provider substitutes one
of two internal shims (`src/lib.rs:189`):

```rust
#[cfg(feature = "logging")]
pub use log;

#[cfg(all(
    not(feature = "logging"),
    all(debug_assertions, feature = "alloc", not(target_os = "none"))
))]
pub mod log {
    extern crate std;
    pub use std::eprintln as warn;   // and error/info/debug/trace
}

#[cfg(all(
    not(feature = "logging"),
    not(all(debug_assertions, feature = "alloc", not(target_os = "none"))),
))]
pub mod log {
    macro_rules! _internal_noop_log { ($($t:expr),*) => {}; }
    pub use crate::_internal_noop_log as warn;   // and the rest
}
```

So with `logging` off:

- DEBUG build (`debug_assertions` on) and `alloc`:
   `warn!` is `std::eprintln!`,
  an UNCONDITIONAL stderr write that ignores any `log` filter.
   This is the flood,
  and it explains why `RUST_LOG` has no effect.
- RELEASE build (`debug_assertions` off):
   `warn!` is `_internal_noop_log`,
   which
  expands to nothing.
   No output.

The "earlier hypothesis was wrong" note for the next investigator:
 this looked
like a macOS-specific regression from adding the CoreAudio backend,
 and like
something a `log` subscriber could filter.
 Both are false.
 The repo's Linux build
pins the identical `parley 0.9.0` / `icu_segmenter 2.2.0` / `icu_provider 2.2.0`,
so a Linux DEBUG build prints the same flood;
 it was simply never seen because the
Linux app is launched from the KDE `.desktop` (stderr to the journal),
 while on
macOS it was run from a Terminal.
 And no `log` subscriber is involved at all in
debug:
 the shim is a direct `eprintln!`.

## Verification

Versions under test:

- Slint git rev `85e3eb76819762cdcaa732fa87533ff896546bac` (1.17.0-dev),
   features
  `backend-winit`,
   `renderer-femtovg`,
   `renderer-software`.
- `parley 0.9.0`,
   `icu_segmenter 2.2.0`,
   `icu_provider 2.2.0`,
  `icu_segmenter_data 2.2.0` (crates.
  io).
- music-player debug binary on macOS 26.5.1,
   Apple Silicon.

Harness (a folder with one Japanese-named track;
 run headless over ssh,
 which
still drives text layout):

```bash
mkdir -p /tmp/mp-ja
cp fixtures/tone.flac "/tmp/mp-ja/日本語のテスト.flac"
run() { ( env "$@" ./target/debug/music-player /tmp/mp-ja >/tmp/t.log 2>&1 & p=$!; sleep 5; kill $p ); grep -c "No segmentation" /tmp/t.log; }
```

Fails (flood):

```text
default (no RUST_LOG):              627
RUST_LOG=off:                       633
RUST_LOG=warn,icu_provider=off:     633
RUST_LOG=error:                     633
```

Works (no flood):

```text
release build (./target/release/music-player):   0
debug build with icu_provider `logging` enabled:  0   (see workaround)
```

## Verified workarounds

### Enable icu_provider's `logging` feature via cargo feature unification (applied)

In the consuming crate's `Cargo.toml`:

```toml
icu_provider = { version = "2", features = ["logging"] }
```

The crate does not call icu_provider;
 this is purely a feature-unification shim
that flips the transitive `icu_provider/logging` on,
 replacing the debug
`eprintln!` shim with the real `log` facade.
 With no `log` subscriber installed
the `warn!` is silent,
 so the flood stops in BOTH debug and release builds.

Tradeoffs:
 adds an unused direct dependency (and the small `log` crate);
 the
version constraint must stay compatible with the transitive pin (`2.2.0` here) so
cargo unifies onto one copy rather than duplicating;
 it suppresses ALL
icu_provider warnings,
 not just this one (acceptable,
 they are data-availability
internals,
 not app errors).
 It does NOT add CJK dictionary line-breaking:
 parley
still uses `new_for_non_complex_scripts`,
 so Japanese keeps breaking between
graphemes (which is acceptable for CJK).

### Use a release build

`mise run //package/music-player/desktop-app:run` builds `--release`,
 where the
icu_provider shim is already a no-op,
 so the flood never appears.

Tradeoff:
 the fast-iteration `run-debug` path (and any `cargo run`/`target/debug`
binary) still floods without the feature workaround above.

## What does not work

- `RUST_LOG` (any value):
   the debug shim is a raw `eprintln!`,
   not routed through
  the `log` facade,
   so log filtering cannot touch it (verified:
   off/error/
  `icu_provider=off` all still print ~630 lines).
- Installing a `log` subscriber (env_logger,
   etc.) to filter it:
   same reason;
   in
  debug-without-`logging`,
   icu_provider never calls the real `log` crate.
- Bundling the CJK dictionary data (e.g. a fuller icu_segmenter data set) to get
  proper segmentation AND silence the warn:
   parley hardcodes
  `new_for_non_complex_scripts` (`analysis/mod.rs:56`),
   so it would not consult
  the extra data anyway.
   Proper CJK line-breaking needs an upstream parley change,
  not a data or feature toggle at our boundary.

## Upstream filing decision

No `.out-of-scope/` exemption matches slint,
 parley,
 or icu4x;
 checked and found
none.
 The 6-constraint check:

1.  Upstream's fault?
     Only partly,
     and by design.
     parley intentionally omits CJK
    dictionary data (`new_for_non_complex_scripts`) for binary size,
     and
    icu_provider's debug `eprintln!` shim is an intentional no-dependency dev aid.
    Neither is a defect;
     the user-facing noise is the interaction.
2.  Can upstream fix it?
     parley could expose a feature to use the full segmenter,
    or icu_segmenter could downgrade the per-call warn.
     Possible,
     but a design call.
3.  Supporting this use case?
     parley renders CJK text (it just does not
    dictionary-segment it);
     the limitation is implicit,
     not a broken promise.
4.  Welcome contribution?
     Not assessed (not reached).
5.  Will they fix?
     Unknown;
     most likely "working as intended / by design".
6.  Prototyped a minimal fix?
     The user-facing problem is fully resolved at our
    boundary by the `logging` feature;
     no upstream patch is needed.

Decision:
 do not file.
 This is a known upstream design tradeoff with a complete
consumer-side fix,
 recorded here.
 No draft issue is kept.
