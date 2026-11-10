# Momoa 3.2.6: an unpaired `\u` surrogate in Rust JSONC parsing reaches undefined behavior

Status:
 **source diagnosis and isolated prototype verified**.
 The original unsafe
parser was never executed on surrogate input.
 A checked pre-fix instrument and
patched parser were exercised in a bounded container;
 the patch and tests are
local evidence,
 not an upstream release.
 No external filing was sent.

## Symptom

A Rust caller supplying JSONC such as `r#"{"s":"\uD800"}"#` to
`momoa::jsonc::parse` can reach `char::from_u32_unchecked(0xD800)`.
The input contains **one** JSON-source backslash before `u`.
This is undefined behavior under [Rust's `char::from_u32_unchecked` safety contract][rust-char];
it is not a normal parser error whose precise message can be quoted.
No runtime manifestation of the original unsafe implementation was observed.
The checked pre-fix and patched-parser outcomes are recorded under Verification.
A high surrogate in a purported pair,
 such as `\uD83D\uDE00`,
 also reaches
the unchecked conversion of its first code unit before pairing can occur.
This document does not claim a particular runtime manifestation.

## Root cause

The source inspected was a private upstream clone at
`/var/home/user/temp/agent/momoa-2026-09-24` (commit `8dfb563`).
The clone's Rust manifest says `version = "3.2.6"` at `rust/Cargo.toml:3`:

```toml
name = "momoa"
version = "3.2.6"
```

The primary agent measured SHA-256 on the clone and the downloaded published 3.2.6 source:
`rust/src/parse.rs` matched at `1f63390201ea871c4f7d6e5ddc69d71394a2b459d0f032f3c327507f41cbfea3`;
`rust/src/readers.rs` matched at `4b15e59cb354a65ecd2fa9e8bfb99260a6638183e7a7e34d70abed69ee763b6d`.
This pins the unsafe call chain to the published source,
 not just a branch with the same manifest version.

1.  `rust/src/readers.rs:79-113` validates the escape's next four characters as
    ASCII hexadecimal digits,
     but does not reject the surrogate range:

    ```rust
    Some('u') => {
        len += 1;
        it.next();
        // next four digits must be hexadecimals
        for _i in 0..4 {
            match it.next() {
                Some(nc) if nc.is_ascii_hexdigit() => len += 1,
                Some(nc) => {
                    let new_cursor = cursor.advance(len);
                    return Err(MomoaError::UnexpectedCharacter {
                        c: nc,
                        line: new_cursor.line,
                        column: new_cursor.column,
                    });
                }
    ```

2.  `rust/src/tokens.rs:119-140` delegates quoted strings to that reader and
    returns a string token when it succeeds:

    ```rust
    '"' => {
        let read_result = read_string(it, &cursor);
        if read_result.is_err() {
            return Some(Err(read_result.err().unwrap()));
        }
        let new_cursor = read_result.unwrap();
        self.cursor = new_cursor;
        return Some(Ok(Token {
            kind: TokenKind::String,
    ```

3.  `rust/src/parse.rs:295-365` processes a `String` token,
     reads four digits,
    converts them to `u32`,
     and invokes unchecked scalar construction.
     Hexadecimal
    syntax alone does not establish that the integer is a Unicode scalar:

    ```rust
    'u' => {
        let mut hex_sequence = String::with_capacity(4);
        for _ in 0..4 {
            match &it.next() {
                Some(hex_digit) => hex_sequence.push(*hex_digit),
                _ => panic!("Should never reach here."),
            }
        }
        let char_code =
            u32::from_str_radix(hex_sequence.as_str(), 16).unwrap();
        // actually safe because we can't have an invalid hex sequence at this point
        let unicode_char = unsafe { char::from_u32_unchecked(char_code) };
        value.push_str(format!("{}", unicode_char).as_str());
    }
    ```

    The source comment equates valid hex with a valid scalar;
     the safe standard
    library constructor returns `None` for surrogates.
     The parser does not
    combine a high and low escape before this conversion.

4.  `rust/src/parse.rs:498-500` constructs and runs this parser;
     the public JSONC
    wrapper in `rust/src/lib.rs:33-44` passes `Mode::Jsonc` to it:

    ```rust
    pub fn parse(text: &str, mode: Mode, options: Option<ParserOptions>) -> Result<Node, MomoaError> {
        let mut parser = Parser::new(text, mode, options);
        parser.parse()
    }
    ```

    ```rust
    pub fn parse(text: &str) -> Result<ast::Node, MomoaError> {
        parse::parse(text, Mode::Jsonc, None)
    }
    ```

    The analogous `json::parse` wrapper at `rust/src/lib.rs:22-24` uses the
    same `parse::parse` function with `Mode::Json`,
     so the boundary is not
    exclusive to JSONC.

[Rust's `char` documentation][rust-char] and [the Rust Reference][rust-reference-char]
exclude surrogates `0xD800..=0xDFFF` from valid `char` values;
 the Reference
calls construction of an invalid `char` immediate undefined behavior.
 This is a source-level safety finding,
 not an observed execution result.

## Verification

- Scope:
   momoa Rust crate 3.2.6 from clone commit `8dfb563`.
  The primary agent measured matching SHA-256 values for published `parse.rs` and `readers.rs`.
- The primary agent ran the std-only safe control via
  `mise run test:momoa-surrogate:control` under `~/temp/agent`.
  Its output was `checked char conversion: surrogate rejected, scalar accepted`.
  It asserts `char::from_u32(0xD800) == None` and
  `char::from_u32(0x41) == Some('A')`.
   The corresponding safe assertions are:

  ```rust
  // std-only; does not import or execute Momoa
  fn main() {
      assert_eq!(char::from_u32(0xD800), None);
      assert_eq!(char::from_u32(0x41), Some('A'));
  }
  ```

- The safe control was expanded to compile and exercise a source-wide `\u`
  guard.
   `mise run test:momoa-surrogate:control` now prints
  `checked scalar and conservative escape guard: pass`.
  The guard rejects actual JSON Unicode escapes (`\uD800`,
   `\u0041`),
  a harmless escaped backslash,
   and a comment containing `\u`;
  it accepts plain text and raw Unicode.
   This checks the guard predicate,
  not an end-to-end Momoa parse.
- **Do not run** a Momoa parse of the surrogate samples on the real host.
  A pre-patch failure and post-patch success must be established only in
  a private,
   credential-free container bounded to 2 GiB and 2 CPUs,
   after
  inspecting the invoked command tree.
   The original unchecked parser was
  never executed on surrogate input;
   only checked instrumentation and the
  patched parser were run in that container.

### Isolated prototype execution manifest

A fresh private clone was created under
`~/temp/agent/upstream-prototype.AxhMuxdU/momoa` at commit `8dfb563`.
Its `origin` was verified as `humanwhocodes/momoa` and its push URL disabled.
The pre-fix probe temporarily replaced the unchecked constructor in
`rust/src/parse.rs` with a checked conversion that returned an error before
creating an invalid `char`.
 The final [prototype patch](momoa-rust-unpaired-surrogate.patch)
replaces that instrumentation with checked surrogate-pair decoding.
A separate `rust/examples/surrogate_safety.rs` calls the public JSONC parser.
The original unsafe call was **not** executed on the suspect input.

- Fetch phase:
   `mise run fetch` in the private scratch root invoked
  `cargo fetch --locked --manifest-path momoa/rust/Cargo.toml` with a private
  `CARGO_HOME`.
   Cargo fetched dependencies with network access but did not
  execute crate source.
   Mise first installed Rust 1.98.1 into that private
  Cargo home and invoked `rustc -V`;
   the container run used its own pinned
  Rust 1.97.1 image.
   The private cache measured 37 MiB after fetch and after
  the test runs;
   no credentials were copied.
- Container execution:
   `mise run test:safe-instrumented` invokes `cargo run
  --locked --offline --example surrogate_safety` in a rootless container using
  local `docker.io/library/rust:1.97-bookworm` image digest
  `sha256:77fac8b98f9f46062bb680b6d25d5bcaabfc400143952ebc572e924bcbedc3fa`.
  The image source and toolchain are not part of the published Momoa crate.
- Inputs:
   only the disposable clone mounted read-only at `/work/momoa` and
  the private Cargo registry mounted at `/cargo-home`.
  No repository checkout,
   real home directory,
   ambient credentials,
   or
  network access enters the container.
   The filesystem root is read-only.
  Cargo writes build artifacts to a 1 GiB `/target` tmpfs and temporary data
  to a 256 MiB `/tmp` tmpfs.
   The registry bind mount remains writable for
  Cargo locking and is not a strict disk-quota boundary;
   this deviation is
  limited to the inspected private cache,
   not user or shared state.
- Bounds:
   `--memory=2g`,
   `--memory-swap=2g`,
   `--cpus=2`,
  `--pids-limit=128`,
   `--ulimit nofile=256:256`,
   and native Podman
  `--timeout=600`.
   Stop if an undeclared write,
   subprocess,
   network attempt,
  crash,
   or resource bound appears;
   do not retry unsafe input outside isolation.
- Inspected command tree:
   `rust/Cargo.toml` and `Cargo.lock` select
  `serde`,
   `thiserror`,
   `wasm-bindgen`,
   `serde-wasm-bindgen` and their
  normal/build dependencies (saved in private `momoa-dependencies.txt`).
  No `build.rs`,
   network,
   or subprocess call was found in `rust/src/`.
  The pinned build scripts for `serde`,
   `serde_core`,
   `serde_json`,
  `wasm-bindgen`,
   `wasm-bindgen-shared`,
   `proc-macro2`,
   and `thiserror`
  were read before execution.
   The resolved `quote` 1.0.35 package has no
  `build.rs`;
   the previously inspected `quote` 1.0.47 script was not the
  selected artifact.
   These build scripts write to Cargo's `OUT_DIR`,
  query compiler or target settings,
   invoke `rustc` probes,
   or call
  `git rev-parse HEAD` from the private registry copy;
   no other declared
  subprocess or network endpoint was found.
   Release workflows,
   npm tasks,
  benchmarks,
   and third-party test scripts are not invoked.

The upstream Rust suite was separately run as `mise run test:upstream`
with the same container image,
 mounts,
 network policy,
 and bounds,
 invoking
`cargo test --locked --offline`.
 `rust/tests/parse_test.rs:500-506` reads
fixture files under the disposable clone;
 a search of `rust/tests` found no
other process or network path.
 The test dependency graph adds `glob`,
`test-case`,
 and its proc macros,
 with no new build scripts in the inspected
resolved graph.
 The bounded full Rust suite then passed:
 the parser integration
suite reported 52 passes and token suite 41 passes;
 no test failed.
 The
`should_parse_json_files` glob is not empty in this checkout (the primary agent
measured 51 matching `.txt` fixtures).
 The separate public-entry example was
run via `cargo run --example surrogate_safety`,
 not by `cargo test`.

The checked pre-fix instrumentation was run through
`mise run test:safe-instrumented` in this container.
 It exited 101 after
compiling the pinned crate and its dependencies.
 The public-entry example
asserted that a valid pair parses,
 and failed at
`rust/examples/surrogate_safety.rs:36`:
`assertion failed: momoa::jsonc::parse(r#"{"s":"\uD83D\uDE00"}"#).is_ok()`.
The checked instrumentation returns an ordinary error on the high half;
**this is a safe positive control,
 not execution of the original undefined behavior**.
This is the safe failing control for the patched result that follows.

The disposable clone was then changed to decode a high/low `\u` pair into
one checked scalar and return `MomoaError::UnexpectedElement` for isolated
halves.
 Running the same bounded `mise run test:safe-instrumented` command
exited 0 and printed `safe surrogate consumer cases passed`.
The example was expanded and rerun in the same bounded container.
It exited 0 and printed `safe surrogate consumer cases passed` after checking
isolated high/low errors in values and an isolated high key,
 a high half followed by a
non-low escape,
 a valid pair in a key,
 `\u0041`,
 and a literal escaped
backslash.
 It also inspected the parsed string and asserted the valid pair
decodes to `😀`.
This does not prove every JSON string or escape path works;
the repair targets invalid `char` construction only.
In particular,
 the unchanged `rust/src/parse.rs:327` uses
`text.trim_matches('"')`;
this prototype does not test trailing escaped
quotes or claim general string-decoder correctness.
 No original unchecked
parser invocation occurred in any run.
 The prototype uses the generic
`UnexpectedElement` error at the string token's start,
 not a diagnostic naming
the surrogate or its exact escape offset.

A further rerun of the patched example in the same bounded container exited 0
and printed `safe surrogate consumer cases passed`.
It asserted the specific
`UnexpectedElement` variant for isolated high and low halves in values and
keys,
 interrupted or mismatched pairs,
 and a pair followed by a lone low
half.
 It checked decoded boundary pairs `U+10000` and `U+10FFFF`,
neighboring
BMP scalars,
 `U+0000`,
mixed-case hexadecimal,
 adjacent pairs with surrounding
text,
 an object key,
 ASCII `\u0041`,
and a literal escaped backslash.
 Truncated
and nonhex escapes returned errors without panic.
 The original unsafe parser
was not executed on any surrogate input.
 The [updated patch](momoa-rust-unpaired-surrogate.patch)
passed `git apply --check` against the clean clone again.

### Reconstruct the safe control

The [checked pre-fix patch](momoa-rust-unpaired-surrogate.prepatch.patch)
and the [final prototype patch](momoa-rust-unpaired-surrogate.patch)
apply independently to a clean checkout of Momoa commit `8dfb563`.
Each includes the same public-entry example.
 `git apply --check` passed on
both patches against the clean published-source-matching clone.
**Never run the unmodified parser** on a surrogate escape.

An agent can create two separate private scratch clones,
 disable their push
URLs,
 apply the checked patch to one and the final patch to the other,
 and
fetch the pinned `rust/Cargo.lock` dependencies to a private Cargo home.
The following is the actual container command shape used for each clone.
Replace `SCRATCH` with the agent-created private root;
 the source mount is
read-only and the container has no user repository or real home mount:

```sh
# doc/troubleshooting/momoa-rust-unpaired-surrogate.md
podman run \
  --memory=2g --memory-swap=2g --cpus=2 --timeout=600 \
  --pids-limit=128 --ulimit nofile=256:256 --rm \
  --network=none --read-only --security-opt label=disable \
  --tmpfs /target:rw,size=1g --tmpfs /tmp:rw,size=256m \
  --volume "$SCRATCH/momoa:/work/momoa:ro" \
  --volume "$SCRATCH/cargo-home:/cargo-home:rw" \
  --env CARGO_HOME=/cargo-home --env CARGO_TARGET_DIR=/target \
  --env HOME=/tmp --workdir /work/momoa/rust \
  docker.io/library/rust@sha256:77fac8b98f9f46062bb680b6d25d5bcaabfc400143952ebc572e924bcbedc3fa \
  cargo run --locked --offline --example surrogate_safety
```

The checked patch was rerun on its separate disposable clone with the
expanded example:
 the earlier invalid cases returned expected errors,
then `decoded_value` failed at `rust/examples/surrogate_safety.rs:52`
with `valid JSONC: Unexpected element found. (1:6)` (exit 101).
This is a safe valid-pair control;
 it did not construct an invalid `char`.
The final patch passed the expanded example (exit 0).
The same container command with `cargo test --locked --offline` ran the
upstream Rust suite successfully on the final patch.
These are verification instructions for an agent,
 not a request for the
user to supply credentials or execute them.

### Patterns that avoid the unsafe conversion (source-derived; patched subset tested)

- `r#"{"s":"plain"}"#`:
  no Unicode escape enters the `u` arm of
  `rust/src/parse.rs:326-348`.
- `r#"{"s":"\u0041"}"#`:
  the escape represents scalar `A`;
  the
  patched public-entry example accepted it;
   the unpatched parser was not run.
- `r#"{"s":"\\uD800"}"#`:
  two JSON-source backslashes encode a literal
  backslash before `uD800`;
  the escaped-backslash arm is distinct from the
  Unicode arm in `rust/src/parse.rs:324-351`.

### Patterns unsafe to pass to the unpatched parser (source-derived; original not run)

- `r#"{"s":"\uD800"}"#`:
  isolated high surrogate in a value.
- `r#"{"s":"\uDFFF"}"#`:
  isolated low surrogate in a value.
- `r#"{"\uD800":0}"#`:
  property name reaches `parse_string` through
  `rust/src/parse.rs:455-470`:

  ```rust
  Ok(_) => {
      // name: value
      let name = self.parse_string()?;
  ```
- `r#"{"s":"\uD83D\uDE00"}"#`:
  the first of an otherwise paired set
  is converted alone in `rust/src/parse.rs:326-348`.

These are **not** a catalog of observed error variants:
 undefined behavior
has no reliable diagnostic.
 No unsafe reproducer was executed on the host.

## Verified workarounds

The following conservative guard was compiled and exercised in the std-only
`~/temp/agent/momoa-surrogate-control.rs` harness.
 Call it at every consumer
entry point **before** any Momoa parse call;
 no other path may bypass it.
The guard was not wired to an unpatched Momoa parser;
 that integration remains
unverified.
 Only the separate patched parser was run inside the container.

```rust
// Consumer-side guard; does not import or execute Momoa.
fn guard_unicode_escapes(source: &str) -> Result<(), &'static str> {
    if source.contains(r"\u") {
        return Err("Unicode escapes refused at this boundary");
    }
    Ok(())
}
```

The harness verified that this predicate rejects every input containing
`\u`,
including `\uD800`,
and accepts examples without that sequence.
Source inspection of `rust/src/parse.rs:326-348` shows the guard removes all
syntactically valid Unicode escape paths to unchecked scalar construction
when it is applied before parsing.
 It also rejects harmless `\u0041`,
literal escaped backslashes,
 comments containing `\u`,
and invalid text that
would already fail parsing.
 This over-rejection is deliberate.
Callers needing Unicode escape support or exact acceptance semantics must
choose a different parser or implement and validate a full JSONC-aware guard.

## What does not work

These alternatives were rejected by source inspection,
 not runtime trials:

- Replacing `r#"{"s":"\uD800"}"#` with `r#"{"s":"\\uD800"}"#` is
  not a reproduction of the same input:
   the latter has a literal backslash
  value,
   per `rust/src/parse.rs:324-351`.
  It avoids the problematic path
  by changing the parsed data,
   not by fixing the parser.
- Trusting `jsonc::tokenize` to accept a string before calling
  `jsonc::parse` is not a surrogate guard:
   `rust/src/tokens.rs:119-140`
  only invokes `read_string`,
  whose `rust/src/readers.rs:79-113` check
  establishes hexadecimal syntax,
   not scalar validity.
- Substituting safe `char::from_u32` alone without handling surrogate pairs
  would avoid unchecked construction but would reject valid JSON pairs.
  The corresponding design must define pairing and isolated-surrogate errors.
- Running an unpatched surrogate parse on the host is unsafe,
   not an
  acceptable verification method.

## Upstream filing decision

The `.out-of-scope/` directory was listed.
 Its entries include
`cargo-workspace.md` and `typescript-project-references.md`,
but no Momoa
or unpaired-surrogate exemption;
 there is no listed exemption to cite.
No external issue,
 PR,
 or comment was posted.

Repository-scoped GitHub web searches covered open and closed issues and PRs
for [surrogate][issue-surrogate],
[from_u32_unchecked][issue-unsafe],
[unicode][issue-unicode],
and [unpaired][issue-unpaired],
with corresponding
PR searches ([surrogate][pr-surrogate],
[from_u32_unchecked][pr-unsafe],
[unicode][pr-unicode],
and [unpaired][pr-unpaired]).
 No matching report was identified.
The [Unicode offsets PR #204][pr-offsets] concerns byte offsets of
multi-byte characters,
 not escaped surrogate scalar construction.
 Its
maintainer response asks that Rust and JS behavior stay aligned,
 which
matters to a future fix but is not a rejection of this safety report.
The primary agent also ran repository-scoped `gh search issues` for `surrogate`,
`from_u32_unchecked`,
`unicode`,
and `unpaired`;
each returned no matching issue.
It ran `gh search prs` for `surrogate`,
`from_u32_unchecked`,
`unicode`,
and `unpaired`.
Those returned only unrelated dependency updates and Unicode-offset work,
including [PR #204][pr-offsets],
not this unchecked surrogate construction.
No duplicate was found in the inspected queries;
 source and tracker state can change.

1.  **Upstream fault:
     yes.**
    The unchecked conversion follows source-level
    acceptance of surrogate hex digits,
     not a caller's unsafe block;
    see `rust/src/readers.rs:79-113` and `rust/src/parse.rs:326-351` excerpts.
2.  **Upstream can fix it:
     yes.**
    The `parse_string` boundary in
    `rust/src/parse.rs:295-365` can replace unchecked construction and
    handle pairs and errors;
     no architectural impossibility was found.
3.  **Supported use case:
     yes.**
    `rust/README.md` at the reported commit
    documents `jsonc::parse` and JSONC comment handling;
     `rust/src/lib.rs:33-44`
    exposes that API.
     The README gives a `jsonc::parse(code)` example.
4.  **External contributions welcome:
     soft yes.**
    The root README invites
    development;
     `.github/ISSUE_TEMPLATE/BUG_REPORT.md` requests a
    reproduction and expects `bug, needs repro` labels.
     No root
    `CONTRIBUTING.md` was found at this commit and `.github/` listed
    `ISSUE_TEMPLATE/` and `workflows/`,
    with no PR template in that
    listing.
     [PR #204][pr-offsets] includes a substantive maintainer
    response to an external contributor.
     No AI-assisted filing ban was
    found in the checked materials;
     that is not proof no other policy exists.
5.  **Likely fix:
     soft yes.**
    No comparable won't-fix signal was found.
    [Parser history][parse-history] and [reader history][reader-history]
    show work in these paths;
     `gh` tracker queries found no matching report.
    The [3.2.6 release notes][release-326] name provenance publication only;
    the [3.2.5 notes][release-325] name dependency and whitespace fixes.
    Neither states a refusal to fix surrogate handling.
6.  **Minimal architecture-compatible prototype:
     yes,
     isolated.**
    A fresh
    private clone of commit `8dfb563` was origin-checked and had its push URL
    disabled.
     The [checked pre-fix patch](momoa-rust-unpaired-surrogate.prepatch.patch)
    and [final prototype patch](momoa-rust-unpaired-surrogate.patch)
    separately apply to clean source.
     The final patch replaces unchecked
    scalar construction,
     decodes valid surrogate pairs,
    and errors on isolated halves.
     `git apply --check` succeeded against the
    clean 3.2.6 clone.
     A safe checked pre-fix instrument failed the valid-pair
    assertion (exit 101);
     the patched public-entry example passed in the
    bounded offline container (exit 0),
     including the decoded `😀` value.
    The bounded upstream Rust suite also passed.
     The original unsafe branch
    was not executed on the suspect input,
     even in the container.

All filing constraints have supporting evidence,
 with the stated limitation
that the pre-fix result is a **checked instrument**,
not observed undefined
behavior.
 Default policy is not to file;
 posting externally still requires
the user's authorization.
 The following draft is kept locally,
 not sent.

### New-issue draft (not sent)

~~~md
# Rust 3.2.6 JSON/JSONC parser constructs a surrogate `char` with unchecked conversion

Labels: bug, needs repro

In momoa Rust 3.2.6, `jsonc::parse` of `r#"{"s":"\uD800"}"#` (one
JSON-source backslash) reaches undefined behavior by source inspection.
Do not run the original unsafe parse on any host. A std-only control
confirms `char::from_u32(0xD800) == None`. A checked pre-fix instrument
failed on the valid surrogate-pair case, and the patched parser accepted
the pair while rejecting isolated halves in a bounded container.

Source trace: `rust/src/readers.rs:91-113` accepts the four ASCII hex
digits following `\u` with `Some(nc) if nc.is_ascii_hexdigit() => len += 1`.
`rust/src/tokens.rs:119-140` invokes `read_string(it, &cursor)` to
produce `TokenKind::String`. `rust/src/parse.rs:326-351` then does:

```rust
let char_code = u32::from_str_radix(hex_sequence.as_str(), 16).unwrap();
let unicode_char = unsafe { char::from_u32_unchecked(char_code) };
```

`rust/src/parse.rs:498-500` calls `parser.parse()`, and
`rust/src/lib.rs:33-44` exposes `parse::parse(text, Mode::Jsonc, None)`
through `jsonc::parse`. A paired high surrogate reaches the unsafe call
before the low half can be combined. Rust documents surrogate construction
through this API as undefined behavior.

Suggested fix: at `rust/src/parse.rs:326-348`, decode valid pairs to
one Unicode scalar and report isolated high/low surrogate errors instead
of constructing an invalid `char`. Add tests for ASCII escapes, valid
pairs, isolated high/low halves, escaped backslashes, and object keys.
Maintain documented Rust/JS behavior parity where possible.

The local prototype patch is recorded in the repository troubleshooting
document. `cargo run --locked --offline --example surrogate_safety` passed
after the patch; `cargo test --locked --offline` passed the Rust integration
and token suites. Repository-scoped duplicate searches found no matching
issue or PR. No upstream issue or PR has been sent.
~~~

[rust-char]: https://doc.rust-lang.org/std/primitive.char.html#method.from_u32_unchecked
[rust-reference-char]: https://doc.rust-lang.org/reference/types/char.html
[issue-surrogate]: https://github.com/humanwhocodes/momoa/issues?q=is%3Aissue+surrogate
[issue-unsafe]: https://github.com/humanwhocodes/momoa/issues?q=is%3Aissue+from_u32_unchecked
[issue-unicode]: https://github.com/humanwhocodes/momoa/issues?q=is%3Aissue+unicode
[issue-unpaired]: https://github.com/humanwhocodes/momoa/issues?q=is%3Aissue+unpaired
[pr-surrogate]: https://github.com/humanwhocodes/momoa/pulls?q=is%3Apr+surrogate
[pr-unsafe]: https://github.com/humanwhocodes/momoa/pulls?q=is%3Apr+from_u32_unchecked
[pr-unicode]: https://github.com/humanwhocodes/momoa/pulls?q=is%3Apr+unicode
[pr-unpaired]: https://github.com/humanwhocodes/momoa/pulls?q=is%3Apr+unpaired
[pr-offsets]: https://github.com/humanwhocodes/momoa/pull/204
[release-326]: https://github.com/humanwhocodes/momoa/releases/tag/momoa-rs-v3.2.6
[release-325]: https://github.com/humanwhocodes/momoa/releases/tag/momoa-rs-v3.2.5
[parse-history]: https://github.com/humanwhocodes/momoa/commits/main/rust/src/parse.rs/
[reader-history]: https://github.com/humanwhocodes/momoa/commits/main/rust/src/readers.rs/
