# rolldown 1.2.13: `external` RegExps are compiled once into the Rust `regex` engine and matched natively per specifier, while the predicate form costs one napi round trip to the JS main thread per check

Rolldown's core is Rust;
the npm package is a napi binding over a prebuilt native module.
For the `external` input option this splits the two accepted forms across
two execution models:
a `RegExp` entry is decomposed at config time and recompiled as a native
Rust matcher that runs inside the parallel module scan,
while an `ExternalOptionFunction` predicate stays in JavaScript and is
called through an napi `ThreadsafeFunction` for every import specifier.
The regex-necessity audit
([issue #680](https://github.com/Aquaticats/Monochromatic/issues/680))
filed every rolldown `external` regex as replaceable by a predicate
"at similar or better time complexity",
comparing pattern shapes without tracing where each form executes.
The affected issues
(#726,
#817 through #822,
#834 through #837,
#846 through #848)
were closed as wontfix on the evidence recorded here.

## Symptom

Nothing fails at runtime.
The observable trigger is the audit's proposal itself:
replace `/^node:/u` and `/^electron(\/|$)/u` entries in rolldown configs
(and the `anchoredExternal` / `escapePackageNameForRegExp` helpers in
`package/config/rolldown/src/package-externals.ts`)
with one predicate function per config.
Two properties of that proposal contradict the recorded justifications' framing
and the audit criterion's assumptions:

- With the `RegExp` form,
  no JavaScript runs during external matching at all;
  the patterns are matched by Rust's `regex` crate
  (a finite-automaton engine that is linear time by construction,
  with no backtracking possible for any input).
- With the predicate form,
  every external check in every module-resolution task becomes an async
  round trip from Rust to the single JS main thread.
  Measured on this machine:
  roughly 8 microseconds per check,
  adding about 45 ms (a factor of about 1.4 on total build time)
  on a 2000-module fixture with about 6000 checks per build.

## Root cause

All citations are rolldown source at tag `v1.2.13`
(commit `f7de35f45bcd4cb8e4b6d8d11e74fbfb20b03127`),
which matches the installed `rolldown@1.2.13`.
Paths are relative to the rolldown repository root.

### The binding type is all-or-nothing

The napi binding accepts either an array of string-or-RegExp entries or a
single JS callback, never a mix
(`crates/rolldown_binding/src/utils/normalize_binding_options.rs:222-226`):

```rust
// crates/rolldown_binding/src/utils/normalize_binding_options.rs:222
fn normalize_external_option(
  external: Option<
    Either<Vec<BindingStringOrRegex>, JsCallback<FnArgs<(String, Option<String>, bool)>, bool>>,
  >,
) -> Option<IsExternal> {
```

So one config's migration to a predicate moves every external decision in
that config to JavaScript,
including the whole `RegExp[]` that
`packageExternals` builds per consuming package
(one anchored matcher per declared dependency).

### A RegExp entry is compiled to Rust once, at config time

The JS `RegExp` object is not retained.
At the napi boundary its `source` and `flags` strings are extracted and
compiled into a `HybridRegex`
(`crates/rolldown_binding/src/types/binding_string_or_regex.rs:48-58`):

```rust
// crates/rolldown_binding/src/types/binding_string_or_regex.rs:48
unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> napi::Result<Self> {
  unsafe {
    let value = NapiStringOrRegex::from_napi_value(env, napi_val)?;
    let value = match value {
      Either::A(inner) => StringOrRegex::String(inner),
      Either::B(inner) => {
        let reg = HybridRegex::with_flags(&inner.source, &inner.flags)?;
        StringOrRegex::Regex(reg)
      }
    };
```

`HybridRegex` tries the Rust `regex` crate first and only falls back to the
ECMAScript-compatible `regress` engine for features the `regex` crate cannot
express, such as backreferences and lookaround
(`crates/rolldown_utils/src/js_regex.rs:5-6`,
`crates/rolldown_utils/src/js_regex.rs:88-97`):

```rust
// crates/rolldown_utils/src/js_regex.rs:88
pub fn with_flags(pattern: &str, flags: &str) -> anyhow::Result<Self> {
  let regex_pattern = Self::get_regex_pattern(pattern, flags);
  match regex::Regex::new(&regex_pattern).map(Self::from_optimized) {
    Ok(reg) => Ok(reg),
    Err(_) => regress::Regex::with_flags(pattern, flags)
      .map(HybridRegexInner::Ecma)
      .map(Self)
      .map_err(anyhow::Error::from),
  }
}
```

The pattern is prefixed with `(?R{flags})`,
mapping the JS `u` flag onto the `regex` crate's Unicode mode and enabling
CRLF-aware line semantics
(`crates/rolldown_utils/src/js_regex.rs:130-135`).
The workspace pins `regex = "1.13.0"` with features
`std`, `perf`, `unicode-perl`, `unicode-case`,
and `regress = "0.12.0"`
(rolldown `Cargo.toml:229-235`).

### Matching runs inside Rust, per specifier, on parallel tasks

`IsExternal::call` matches array entries natively;
only the function arm crosses back into JS
(`crates/rolldown_common/src/inner_bundler_options/types/is_external.rs:41-48`):

```rust
// crates/rolldown_common/src/inner_bundler_options/types/is_external.rs:41
Ok(match self {
  IsExternal::StringOrRegex(patterns) => patterns.iter().any(|p| match p {
    StringOrRegex::String(s) => s == specifier,
    StringOrRegex::Regex(r) => r.matches(specifier),
  }),
  IsExternal::Fn(Some(is_external)) => {
    return is_external(specifier, importer, is_resolved).await;
  }
  IsExternal::Fn(None) => false,
})
```

Two details matter here:

- The string arm is exact equality only,
  so plain strings cannot express the `electron` plus `electron/...` prefix
  semantics of `/^electron(\/|$)/u` or the `node:` protocol prefix
  without enumerating every possible specifier.
- `r.matches` dispatches to `regex::Regex::is_match` for patterns that
  compiled through the fast path
  (`crates/rolldown_utils/src/js_regex.rs:137-143`).
  The `regex` crate is an automaton engine:
  linear time in the input for every pattern it accepts,
  with catastrophic backtracking structurally impossible.
  The `regress` fallback is also native Rust;
  neither arm ever calls into JavaScript.

`IsExternal::call` runs for every import specifier of every module,
before resolution and again after it when plugins left the id unmarked
(`crates/rolldown_plugin/src/utils/resolve_id_check_external.rs:36` and
`crates/rolldown_plugin/src/utils/resolve_id_check_external.rs:63`),
reached from the module loader's resolve path
(`crates/rolldown/src/module_loader/resolve_utils.rs:39`).
Module tasks are spawned concurrently
(`crates/rolldown/src/module_loader/module_loader.rs:354`,
`module_loader.rs:366`,
`module_loader.rs:397`),
so the array form keeps every external decision inside Rust's concurrent
scan.

### The predicate form is one napi threadsafe-function hop per check

`JsCallback` is documented in rolldown's own source as a type alias for
napi's `ThreadsafeFunction`
(`crates/rolldown_binding/src/types/js_callback.rs:45`):

```rust
// crates/rolldown_binding/src/types/js_callback.rs:45
/// `JsCallback`  is a type alias for `ThreadsafeFunction`. It represents a JavaScript function that passed to Rust side.
```

The wrapped callback awaits an `invoke_async` per call
(`crates/rolldown_binding/src/utils/normalize_binding_options.rs:229-241`):

```rust
// crates/rolldown_binding/src/utils/normalize_binding_options.rs:229
Either::B(is_external) => {
  IsExternal::Fn(Some(Arc::new(move |source, importer, is_resolved| {
    let source = source.to_string();
    let importer = importer.map(ToString::to_string);
    let is_external = Arc::clone(&is_external);
    Box::pin(async move {
      is_external
        .invoke_async((source.clone(), importer, is_resolved).into())
        .await
        .context("external option")
        .map_err(anyhow::Error::from)
    })
  })))
}
```

Each invocation marshals the specifier from a Rust tokio task to the JS
main thread's event loop,
runs the closure there,
and awaits the answer before that module's resolution can continue.
Whatever else the JS thread is doing
(plugin hooks,
watch-mode rebuild triggers)
now sits in the critical path of every external decision.

## Verification

Versions under test,
2026-10-17 on this machine:
installed `rolldown@1.2.13` with native binding
`@rolldown/binding-linux-x64-gnu@1.2.13`,
source clone at tag `v1.2.13`
(commit `f7de35f45bcd4cb8e4b6d8d11e74fbfb20b03127`),
Node v26.12.0,
Rust 1.101.0-nightly for the offline crate check.

### Harness A: the repository's patterns take the Rust `regex` fast path

A scratch crate replicating `HybridRegex::with_flags`'s fast path
(the `(?R{flags})` prefix plus `regex::Regex::new`)
with rolldown's exact `regex` version and feature set,
run offline against the cached `regex-1.13.1`:

```rust
// src/main.rs (scratch crate, regex = { version = "1.13.0", default-features = false,
//   features = ["std", "perf", "unicode-perl", "unicode-case"] })
fn main() {
    for (pattern, flags) in [
        (r"^electron(\/|$)", "u"),
        (r"^node:", "u"),
    ] {
        let full = format!("(?R{flags}){pattern}");
        match regex::Regex::new(&full) {
            Ok(re) => {
                println!("OPTIMIZED (regex crate, linear time): {full}");
                for input in ["electron", "electron/main", "electron/", "electronfoo", "myelectron", "node:path", "node:", "nodex", ""] {
                    println!("  is_match({input:?}) = {}", re.is_match(input));
                }
            }
            Err(err) => println!("REGRESS FALLBACK (would be Ecma path): {full}: {err}"),
        }
    }
}
```

Result:
both patterns compile through the `regex` crate
(`OPTIMIZED`),
never reaching the `regress` fallback,
and the match table agrees with the JS semantics the configs rely on:
`electron`,
`electron/main`,
and `electron/` match;
`electronfoo` and `myelectron` do not;
`node:path` and `node:` match;
`nodex` does not.
The flagless variants in
`package/file-manager/variant-electron/rolldown.preload.config.ts`
differ only in producing the prefix `(?R)` instead of `(?Ru)`,
which removes Unicode mode that these ASCII-only patterns do not use.

Catalog of engine selection,
from the same source:

- Optimized (`regex` crate,
  linear time guaranteed):
  every pattern this repository passes to `external`,
  since they are literals,
  anchors,
  one alternation,
  and escaped slashes.
- Ecma fallback (`regress`,
  still native Rust,
  backtracking engine):
  patterns using lookaround or backreferences.
  Rolldown's own test asserts `\d+(?!\d)` takes this path
  (`crates/rolldown_utils/src/js_regex.rs:208-211`).

### Harness B: equivalence and cost of the predicate form

The committed probe
[`rolldown-external-regex-native-rust-matching.mjs`](rolldown-external-regex-native-rust-matching.mjs)
builds a synthetic fixture
(N chained TypeScript modules,
each with 10 builtin `node:` imports,
plus an entry importing `electron` and `electron/main`)
and runs the installed rolldown twice per measurement batch:
once with `external: [/^node:/u, /^electron(\/|$)/u]`,
once with the audit-proposed predicate
`(id) => id.startsWith('node:') || id === 'electron' || id.startsWith('electron/')`.

Run:
`node doc/troubleshooting/rolldown-external-regex-native-rust-matching.mjs 400 5`
and again with `2000 3`.

Recorded results (2026-10-17, committed probe, warnings-free fixture):

- Outputs byte-identical between the two forms at both fixture sizes
  (`outputs identical: true`),
  with `electron` and `electron/main` kept external.
  The predicate buys no behaviour.
- Callback invocations for one predicate-form build:
  1204 at N=400,
  6004 at N=2000.
  Each is one threadsafe-function round trip;
  the RegExp form makes zero JS calls.
- Wall time per full build (rolldown plus generate),
  two interleaved batches,
  medians:
  - N=400,
    five runs per batch:
    array 24.0 ms and 23.0 ms;
    predicate 32.6 ms and 34.1 ms.
    Delta about 9 to 11 ms per build,
    roughly 8 to 9 microseconds per check.
  - N=2000,
    three runs per batch:
    array 104.0 ms and 104.0 ms;
    predicate 149.6 ms and 148.7 ms.
    Delta about 45 ms per build,
    roughly 7.5 microseconds per check,
    and at least four times the within-form run-to-run spread
    (array min-max 99.0 to 110.5 ms across both batches),
    so the penalty is signal,
    not noise.
- The penalty grows with specifier count because the per-check hop is
  constant-order while the native form's per-check cost is nanoseconds
  inside the parallel Rust scan.

## Verified workarounds

The standing form for this repository:
keep `RegExp` entries in rolldown `external` arrays and keep the justified
`no-restricted-syntax/no-regex` suppressions on them.
Tradeoffs,
stated so the choice stays auditable:

- Each site carries a suppression comment,
  so the lint gate records an exception instead of covering the site.
  The justifications are accurate:
  matching is anchored,
  the input is one import specifier,
  and the executing engine cannot backtrack.
- A pattern using ECMAScript-only syntax
  (lookaround,
  backreferences)
  silently falls to the `regress` engine:
  still native,
  but a backtracking engine without the linear-time guarantee.
  None of the repository's current external patterns do this.
- The array form cannot make importer-conditional decisions
  (`parentId`,
  `isResolved`);
  a config that ever needs those must take the predicate form and its
  per-check cost deliberately.

## What does not work

- **Migrating to `ExternalOptionFunction` predicates**
  (the audit's proposal for issues #726,
  #817 through #822,
  #834 through #837,
  #846 through #848):
  measured slower at every fixture size,
  moves all external decisions onto the single JS thread,
  and is all-or-nothing per config
  (`Either<Vec<BindingStringOrRegex>, JsCallback<...>>`),
  so it cannot be adopted per array entry or per site.
  For consumers of `packageExternals` it also moves the per-dependency
  anchored matchers into the JS callback path.
  Output equivalence means there is no functional gain to weigh against
  the cost.
- **Plain string entries instead of patterns**:
  the string arm is exact equality
  (`is_external.rs:43`),
  so strings cannot cover `electron/<subpath>` or the open set of `node:`
  builtins without enumerating every specifier,
  which breaks on any new subpath import.
- **Reading the audit criterion as satisfied "at similar time complexity"**:
  asymptotically both forms are linear per specifier,
  but the criterion compared pattern shapes while the two forms execute on
  different machines:
  a native automaton inside Rust's parallel scan versus a
  microsecond-order event-loop round trip serialized on the JS main thread.
  The measured build-time factor of about 1.4 at 2000 modules is the
  difference the criterion did not model.
  `doc/audit/regex-necessity.md` carries the dated correction.

## Upstream filing decision

Nothing to file;
this is not an upstream defect.
Walk of the six constraints:

1. **Is it really upstream's fault?**
   No.
   The split execution model is rolldown's deliberate design:
   `RegExp` entries are explicitly decomposed and recompiled natively,
   and rolldown's own docs comment in `js_callback.rs` describes the
   threadsafe-function machinery as the intended way to run JS options.
   The behaviour is the value proposition of a Rust bundler.
2. **Can upstream fix it?**
   Not applicable;
   there is nothing to fix.
   The predicate form's cost is inherent to calling JS from Rust.
3. **Are they supporting this use case?**
   Yes;
   both forms are supported and documented in the type surface
   (`ExternalOption = StringOrRegExp | StringOrRegExp[] | ExternalOptionFunction`).
4. **Would the repo welcome our contribution?**
   Not applicable without a defect or feature gap.
5. **Will they likely fix it?**
   Not applicable.
6. **Have we prototyped a minimal fix?**
   Not applicable;
   the only change that would help this repository is on our side
   (keeping the RegExp form),
   and it is already in place.

`.out-of-scope/` was checked:
no rolldown entry exists there,
and none is needed because no upstream filing is proposed.
The upstream tracker was not searched for duplicates because there is no
report to duplicate;
this doc records intended behaviour plus our measured cost of the
alternative,
which belongs in this repository,
not in rolldown's tracker.
