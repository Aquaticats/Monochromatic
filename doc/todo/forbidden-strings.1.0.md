# Forbidden strings: road to 1.0

Release-readiness checklist for promoting `package/forbidden-strings/cli` from `0.1.8` to a
`1.0` we would be comfortable publishing and asking others to pin against.

This complements the original design rationale in [forbidden-strings.md](./forbidden-strings.md);
that document covers why the tool exists and how it works.
 This one covers what is missing
before a `1.0` semver commitment is honest.

Current state verified 2026-05-28:
 `cargo test --release` passes (190 unit plus 19 integration,
0 failures),
 `cargo clippy --release -- -D warnings` is clean,
 and the crate publishes via the
OIDC trusted-publishing workflow.
 Performance figures are recorded in `PERF.md` (most recent
recorded run 2026-05-16:
 9.4 ms cold start,
 56.6 ms full `--all` on this repo);
 a spot-check on
2026-05-28 reproduced the same order of magnitude (sub-10 ms start,
 roughly 65 ms `--all`),
 which
fits inside the pre-commit and pre-push budgets the README targets.
 The design doc's original
5 s / 500 ms budgets are several tens of times larger than these numbers,
 but the perf docs
themselves note those budgets are no longer the relevant yardstick,
 so this checklist treats
performance as well-characterized and not a 1.0 gap;
 the items below are about contract,
 coverage,
and documentation.

A `1.0` tag is a promise:
 a scanner whose security model is written down (what it guarantees and,
just as importantly,
 what it explicitly does not),
 that does not leak the rules it is given,
 that
takes a clear position on the content it does not scan,
 with a stable CLI surface,
 a stable
output contract,
 gated correctness,
 and a crate that installs and documents cleanly for someone
outside this monorepo.
 Items are tiered by how load-bearing they are.
 Tier 0 is the security
story:
 the one unambiguous correctness blocker (the load-path disclosure,
 which is undocumented
and contradicts the headline guarantee) plus the documentation that bounds expectations.
 Tier 1
is a set of already-documented false-negative tradeoffs that a 1.0 should consciously accept or
tighten rather than a list of bugs to fix.

## Tracking

GitHub milestone:
 [forbidden-strings 1.0](https://github.com/Aquaticat/Monochromatic/milestone/1).
This document stays the canonical detail;
 the issues link back to the tier items here.
 Item to
issue map:

- Items 1 to 4 (load-path disclosure fix plus redaction test):
   #217
- Item 5 (security-model README section):
   #218
- Item 7 (bound per-file memory):
   #219
- Items 8 to 9 (CI gating plus cross-platform matrix):
   #122 (reused)
- Item 10 (ship LICENSE):
   #220
- Item 11 (MSRV plus metadata):
   #221
- Item 12 (CHANGELOG):
   #222
- Item 13 (stability policy):
   #223
- Items 14 to 18 (rule-loading and CLI foot-guns):
   #224
- Items 20 to 22 (README links plus install docs):
   #225
- Item 25 (resharp fail-closed regression tests):
   #226,
   related #158 (reused)
- Items 26 to 27 (CI fuzz-smoke plus advisory scan):
   #227
- Item 29 (caller-side sorting plus perf guard):
   #228
- Items 6,
   19,
   23,
   24,
   28 (human decisions):
   #229

## Tier 0: security model, guarantees, and disclosure

The README's central promise is that the matched substring,
 the surrounding line,
 and the rule
pattern are never printed in failure output,
 so a sensitive rule body (a customer name,
 an
unreleased codename,
 a pre-disclosure partner ID) can live in the gitignored appendix or the
`FORBIDDEN_STRINGS_LIST` CI secret without leaking on public CI logs.
 The scan-time path honors
this (`emit_hit` and the `rule={N} engine error` lines print only path,
 line/column,
 and the
opaque rule index).
 The rule-load path does not.
 It runs on every invocation,
 local and CI,
before any file is scanned,
 and reproduces as follows (verified 2026-05-28 against the
`0.1.8` release binary with a throwaway rules file):

```text
$ forbidden-strings --rules sensitive.txt target.txt
forbidden-strings: pre-validator nested_complement rejected rule "SECRET_CODENAME_NimbusFalcon&~(~(a))"
forbidden-strings: rule on line 1 (resharp): nested complement `~(~(...))` at byte offset 31: ...

$ forbidden-strings --rules other.txt target.txt
forbidden-strings: rule on line 1 (resharp): Parse(ParseError { kind: ClassUnclosed, pattern: "PartnerID_(unclosed_[group", ... })
```

Both lines go to stderr,
 the exact stream the README markets as leak-safe and the stream CI
captures.
 Three distinct emission paths leak:

1.  Redact the rule source from the pre-validator diagnostics.
     Eleven sites in `src/rules.rs`
    (lines 567,
     613,
     653,
     660,
     694,
     723,
     754,
     790,
     818,
     846,
     875) emit
    `eprintln!("forbidden-strings: pre-validator <name> rejected rule {:?}", src)`,
     printing the
    full rule pattern.
     None of these are documented;
     the README only documents the redacted
    `rule on line N (resharp): <static reason>` form.
     Replace `src` with the rule index,
     or gate
    the source echo behind an explicitly-unsafe debug env var that is off by default and
    documented as not-for-CI.
     The static `reason` strings themselves are safe (they describe the
    shape,
     for example "complement body contains \b",
     and do not quote the pattern);
     the byte
    offset in some reasons ("at byte offset 31") is a minor structural leak to reconsider while
    fixing.
2.  Stop echoing the pattern through engine compile-error Debug.
     `compile_rule_src` formats
    resharp failures as `format!("(resharp): {:?}", e)` (`src/rules.rs:946`) and the regex-crate
    path formats failures as `format!("(regex): {:?}", e)` (`src/rules.rs:1061`).
     Both
    `resharp::Error` and `regex::Error` embed the offending pattern text in their `Debug` and
    `Display` output (reproduced above:
     `pattern: "PartnerID_..."`).
     Most rules (257 of 259 in
    the ported corpus) take one of these two paths,
     so a single malformed sensitive rule dumps
    its body.
     Map these errors to a redacted message that keeps the error kind and offset but not
    the pattern bytes.
3.  Reconcile the line-number disclosure with the documented contract.
     Load errors are prefixed
    `rule on line N` (`src/rules.rs:1324`),
     a direct index into the secret file,
     whereas the
    README and the hit format promise only an "opaque rule index".
     Decide whether the line
    number is acceptable for the load path and align the documentation,
     or switch to the same
    opaque index used at scan time.
4.  Add a redaction regression test,
     the systemic fix that prevents recurrence.
     Feed a rules
    file containing a distinctive sentinel token through every error path (each pre-validator
    shape,
     a resharp compile error,
     a regex-crate compile error,
     an empty/comment-only file,
     a
    read error) and assert the sentinel bytes never appear on stdout or stderr.
     The invariant the
    entire security model rests on currently has no automated guard,
     which is why this regressed
    unnoticed.
5.  Document the security model in the shipped README,
     including the explicit non-guarantees.
    Today the threat model lives only in the internal design doc
    ([forbidden-strings.md](./forbidden-strings.md)):
     the tool defends against accidental
    introduction of enumerable forbidden literals by trusted contributors,
     not against
    adversarial commits.
     Because matching is literal and byte/codepoint exact with no
    normalization,
     it is trivially bypassed,
     and that is by design rather than a defect.
     State
    the bypasses as security boundaries,
     not bugs:
     a forbidden string written with Unicode
    homoglyphs (Cyrillic `а` for Latin `a`),
     in a different Unicode normalization form (NFC
    versus NFD,
     NFKC versus NFKD),
     with zero-width or bidi control characters spliced in,
     or via
    string concatenation,
     base64 / hex encoding,
     or character-code arrays,
     will not match.
    Equally state the operational boundaries:
     the local pre-commit hook is bypassable with
    `--no-verify`,
     so the required CI check is the actual enforcement line;
     already-committed
    history is out of scope (it needs `git filter-repo`);
     and,
     once items 1 through 4 land,
     the
    one positive guarantee is that rule bodies are never printed in failure output.
     A 1.0
    security tool must put its guarantees and its non-guarantees where users read them,
     so nobody
    mistakes accident-prevention for adversarial defense.
     Promote this from the design doc into
    a README "Security model" section and link the two.

## Tier 1: documented false-negative coverage gaps (accept or tighten)

The scanner's other core job is to not miss forbidden content.
 It applies a fail-closed
principle to engine errors (a rule that panics inside resharp becomes a synthetic
`rule={N} engine error` hit that fails the run),
 but it does not apply that principle to files
it cannot fully scan.
 Several paths drop content with no diagnostic and exit 0,
 so a forbidden
string in an affected file passes the gate with no runtime signal.
 Unlike Tier 0,
 every one of
these is already described in the README "Walker behaviour" section,
 and the 8 KiB cap is a
deliberate performance tradeoff,
 so this tier is not a bug list.
 The 1.0 decision is whether
"documented in the README but silent at runtime" is an acceptable posture for a security gate,
 or
whether the fail-closed principle should extend to coverage so a skipped file is visible (a
stderr diagnostic,
 optionally a configurable nonzero exit) and not only findable by reading the
docs.

6.  Decide the posture on coverage gaps:
     accept them as documented limitations,
     or extend
    fail-closed so a file that cannot be fully scanned produces a visible diagnostic at runtime.
    The behaviors in question,
     all currently documented and silent:
    -   The 8 KiB binary tail cap.
         `read_with_binary_check` (`src/lib.rs:337-357`) scans only the
        first 8 KiB of any file whose first 8 KiB contains a NUL byte and that is larger than
        8 KiB.
         This is intentional for performance (it caps per-file work on large binary blobs)
        and is documented in both the README and the function comment.
         The 1.0 action is to
        confirm acceptance and keep the limitation prominent,
         not to remove the cap;
         if anything
        is added,
         it is an opt-in diagnostic noting that a file was tail-capped.
    -   Non-UTF-8 file paths are dropped in both walk sources:
         the `WalkBuilder` path at
        `src/walk.rs:343` (`if let Some(s) = e.path().to_str()`,
         no else arm) and the gix-index
        union at `src/walk.rs:519` (`continue`).
         Documented as "Non-UTF-8 paths silently dropped".
    -   Per-entry walker errors are skipped (`src/walk.rs:287-288`),
         as is any entry whose
        `file_type()` errors (`:322`,
         `unwrap_or(false)`).
         Documented as "Per-entry walker errors
        silently skipped".
         A directory the walker cannot enter drops its whole subtree.
    -   Symlinked directories are not descended (`follow_links` defaults to false,
         not
        overridden).
         Documented as "Symlinks NOT followed".
         Confirm this is the intended boundary
        or follow links with a cycle guard.
7.  Bound per-file memory.
     The NUL-free branch of `read_with_binary_check` (`src/lib.rs:355`)
    reads the entire file into a `Vec<u8>` with no cap,
     and files fan out across the rayon pool,
    so up to one large file per thread can be resident at once.
     A single pathological multi-GB
    tracked text file (or several) can exhaust host memory during `--all`.
     This one is not
    documented as a tradeoff and is the genuine latent item in this tier.
     The perf plan
    deliberately rejected a size-based skip (a secret must not be skippable by being in a large
    file),
     so the fix is bounded or streaming reads that still scan the whole file,
     not a size
    cap that skips it.

## Tier 2: release engineering and packaging

8.  Add a Rust CI workflow that gates merges on `cargo test`,
     `cargo clippy -- -D warnings`,
    and `cargo fmt --check`.
     Today the only workflow touching cargo is
    `.github/workflows/cargo-publish.yml`,
     and it runs `cargo package` (compile-verify) only;
    tests and lint never run in CI,
     so a regression lands green.
     Tracked as issue #122
    (open,
     ready-for-human),
     not yet done.
9.  Run the test suite cross-platform,
     not just on the dev machine.
     `cargo-publish.yml` builds
    eight target triples (linux gnu/musl,
     arm gnu/musl,
     macOS x86/arm,
     windows-msvc x86/arm) but never
    runs `cargo test` on any of them.
     Path canonicalization,
     symlink handling,
     the `gix-index`
    read,
     and Windows path separators are all claimed-supported in the README yet exercised only
    under Linux.
10. Ship the license text inside the crate.
     `Cargo.toml` sets `license = "LGPL-3.0-or-later"`
    but its `include` list is `src/**/*.rs`,
     `Cargo.toml`,
     `Cargo.lock`,
     `README.md` only,
     so the
    published `.crate` carries no license text at all.
     LGPL-3.0 incorporates GPL-3.0 by reference
    (the LGPL text reads "incorporates the terms and conditions of version 3 of the GNU General
    Public License"),
     so a compliant distribution ships both files,
     not just the LGPL one.
     Copy
    `LICENSES/LGPL-3.0-or-later.txt` and `LICENSES/GPL-3.0-or-later.txt` into the package and add
    both to `include`;
     confirm with `cargo package --list` that they appear in the `.crate`.
     This
    is purely mechanical (the repo is uniformly LGPL-3.0-or-later,
     so there is no license choice
    to make),
     hence `ready-for-agent`.
11. Declare and test a minimum supported Rust version.
     `Cargo.toml` has no `rust-version`,
     so
    there is no stated MSRV and nothing pins it.
     Pick a floor,
     add `rust-version`,
     and add an
    MSRV job to the CI matrix from item 8.
     While editing metadata,
     fill the empty `authors` and
    `documentation` fields.
12. Write a `CHANGELOG.md`.
     The `0.1.0` through `0.1.8` history is undocumented outside git log.
    A 1.0 release with a semver commitment needs release notes a consumer can read before
    bumping.
13. Write a stability policy and state it in the README.
     Declare which surfaces are stable under
    semver (CLI flags,
     the exit-code contract `0`/`1`/`2`,
     the `PATH:LINE:COL_START..COL_END
    rule=N` output line,
     and the redaction guarantee once Tier 0 is fixed) and which are
    explicitly not (the `forbidden_strings` lib API:
     `run_cli_from_env`,
     the `fuzzing`-gated
    `fuzz_api`).
     Without this,
     every internal refactor risks looking like a breaking change.

## Tier 3: correctness and UX foot-guns

Each of these is already admitted in the README as a known quirk.
 For 1.0 they should be fixed,
or,
 where intentional,
 kept and documented with rationale rather than as a surprise.

14. Uppercase or otherwise invalid regex flags silently degrade to a literal scan.
     `/foo/i` is a
    regex;
     `/foo/I` is a literal match for the six-byte string `/foo/I`,
     with no load-time error
    (the classifier rejects non-`[a-z]` flags at `src/rule/parse.rs:150` and falls through to
    literal handling at `:209`).
     A rule author who fat-fingers a capital flag gets silently wrong
    behavior.
     Warn or error at load.
15. `--all` plus positional file arguments silently discards the positional list (the walker
    output overwrites it).
     Fix:
     reject the combination with a usage error (exit 2).
16. A bare `//` rule compiles to `(?-flags:)` and matches the empty string at every position,
    flagging every file.
     Fix:
     detect and reject this shape at load.
17. A UTF-8 BOM at the start of the rules file is not stripped,
     so the first rule silently
    carries a leading `\u{FEFF}` and never matches as intended.
     Fix:
     strip the BOM on load.
18. The rules file and `--all` resolve relative to the current working directory,
     not the git
    repository root (default `./forbidden-strings.local.txt`;
     the walker calls `list_files(".")`
    at `src/lib.rs:705`).
     Run the tool from a subdirectory and the default rules file is not
    found and `--all` scans only that subtree,
     which is surprising for a tool described as a
    git-repo scanner.
     Walk up to the repository root,
     or keep cwd-relative and emit an explicit
    "no rules file found at <cwd>" error plus a documented note.
19. Identical rules are not deduplicated;
     each fires its own `rule=N` hit.
     Decision needed:
     dedup
    at load,
     or keep current behavior and document it as intentional.

## Tier 4: documentation and distribution

20. Fix the stale README pointer.
     README lines 258 through 264 reference
    `TROUBLESHOOTING.resharp.md` "in the repository root";
     the file now lives at
    `doc/troubleshooting/resharp.md` (the `--help` text already points there correctly).
     The
    root file does not exist,
     so the pointer is dead.
21. Fix README relative links for the published crate.
     `PERF.md`,
     `FUZZING.md`,
     and
    `doc/troubleshooting/resharp.md` are not in the `include` set,
     so every relative link to
    them 404s on the crates.
    io and `docs.rs` rendered README,
     which is the first page a prospective
    user sees.
     Either bundle those files or rewrite the links to absolute repository URLs in the
    shipped README.
22. Add install and distribution docs for consumers outside the monorepo.
     The README documents
    only the in-repo `mise run ...:build` path.
     A 1.0 should document `cargo install
    forbidden-strings`,
     downloading a prebuilt release archive,
     and verifying its SLSA build
    provenance with `gh attestation verify` (the publish workflow already produces these
    attestations).
23. Decide the rustdoc story.
     There are zero `///` doc comments in `src/` (confirmed:
     0
    doc-tests),
     so the `docs.rs` page renders effectively empty.
     Either add crate-level and API
    rustdoc,
     or mark the lib `#[doc(hidden)]` and configure `docs.rs` to surface the README,
     so the
    published documentation page is not blank.

## Tier 5: dependency and soundness stability

24. Resolve the pre-1.0 dependency exposure.
     `gix-hash = "0.25"` and `gix-index = "0.51"`
    are `0.x` crates that break on minor bumps,
     and the `gix` family churns frequently.
    A 1.0 inherits their instability.
     The former resharp exposure this item used to
    track is retired:
     the engine migration (issues #375 through #389) replaced
    resharp with the in-house `forbidden-regex` engine
    (`package/forbidden-strings/core`),
     removing the dependency entirely rather
    than pinning around its panic shapes.
     What replaces it is not an upstream
    dependency risk but own-engine maturity:
     `forbidden-regex` is itself a young,
    actively developed `0.x` crate (published at `0.1.0`),
     so the open question is
    whether its compile-time soundness guarantees,
     its fuzz coverage (#386),
     and
    its differential validation against the old engine (#387) are enough to anchor
    a `forbidden-strings` 1.0,
     or whether `forbidden-regex` should reach its own
    stability milestone first.
     Decide an explicit policy:
     pin tightly,
     keep
    `Cargo.lock` committed for the binary (already done),
     and decide whether
    `forbidden-strings` 1.0 should wait on `forbidden-regex` maturing further.
25. Close the open-endedness of the resharp pre-validators.
     `src/rules.rs:931` notes shapes "the
    pre-validator does not yet know,
    " with the `catch_unwind` fail-closed net as backstop.
     For
    1.0,
     formally adopt the fail-closed contract as a permanent guarantee (upstream resharp fixes
    are not assured),
     add a regression test for every known-bad shape currently listed in
    `Cargo.toml` and `doc/troubleshooting/resharp.md`,
     and document it as part of the stability
    promise from item 13.
26. Add fuzzing to CI.
     `FUZZING.md` states "CI integration is deferred.
    " The tool's core safety
    claims (linear-time matching,
     no catastrophic backtracking,
     fail-closed on a panicking rule)
    rest on fuzz coverage that currently runs only on-demand locally.
     Add a scheduled or nightly
    bounded fuzz-smoke job that exercises the seven soundness targets,
     so a regression in those
    invariants surfaces without a manual run.
27. Add a dependency-advisory scan (`cargo audit` or `cargo deny`) to CI.
     For a security tool,
    an unguarded known-vulnerable transitive dependency directly undercuts the value proposition,
    and there is currently nothing watching for one.

## Tier 6: scope lock

28. Lock the non-goals and reconsider the highest-value ones.
     The README's "When to pick
    something else" lists SARIF/JSON output,
     stdin/streaming input,
     git-history scanning,
     per-rule
    path scoping,
     per-rule allowlists,
     and CEL post-match filtering as deliberately omitted.
     For a
    1.0,
     commit these formally as non-goals or pull the adoption-blocking ones into scope.
    Machine-readable output (SARIF or JSON for GitHub code-scanning upload) is the most
    adoption-relevant.
     Also resolve the still-open scope questions in
    [forbidden-strings.md](./forbidden-strings.md) (which file classes are scanned,
     whether
    commit messages are scanned,
     whether a path-exclude list is needed).
29. Document caller-side sorting and add a performance-regression guard.
     Cross-file hit ordering
    is rayon-scheduler-determined (stable on a given input but not sorted),
     which makes CI
    snapshot-diffing awkward;
     callers that need deterministic reports should pipe the diagnostic output into `sort`
    in their own wrapper instead of adding a sorted-output mode to the tool.
     Separately,
     the numbers in `PERF.md` are
    maintained by hand with no automated guard,
     so a future change could silently blow the
    sub-100ms pre-commit budget;
     add a lightweight perf-regression check.

## Suggested sequencing

Tier 0 is the gate.
 Until the load path stops printing rule bodies (items 1 through 3,
 guarded
by the redaction test in item 4) and the security model is written where users read it (item 5),
the tool both breaks the guarantee it is sold on and leaves its non-guarantees implicit,
 so
no 1.0 ships.
 Tier 1 is a decision the 1.0 must make explicitly (accept the documented coverage
gaps,
 or extend fail-closed to them) rather than a set of fixes,
 with item 7 (unbounded memory)
the one genuine latent fix in it.
 The Tier 2 items are the next true blockers:
 without CI gating
(8,
 9),
 a license (10),
 and a stated stability contract (13),
 the version number is the only
thing that would change at 1.0.
 Tier 3 foot-guns are cheap,
 individually small,
 and each removes
a documented surprise.
 Tier 4 is a day of documentation work.
 Tier 5 and the scope lock in Tier 6
carry the most judgment and should be decided deliberately,
 since they define what the 1.0
promise actually covers.

## A note on completeness

This list grew through review,
 and several corrections sharpened it.
 The first pass (22 items)
missed the Tier 0 disclosure defect because it verified redaction only on the scan path and
trusted the README's claim for the load path instead of reading the load,
 pre-validate,
 and
compile-error code;
 that is a genuine undocumented defect.
 A prompt to look past leak vectors
surfaced the Tier 1 coverage gaps.
 A further correction noted that those gaps are not undiscovered
bugs:
 the 8 KiB tail cap is an intentional perf tradeoff,
 and every Tier 1 behavior is already
documented in the README "Walker behaviour" section,
 so Tier 1 was reframed from "soundness
blocker" to an explicit accept-or-tighten decision.
 A final note added the security-model
documentation (item 5):
 the threat model,
 including the explicit non-guarantees (Unicode
homoglyphs,
 normalization forms,
 zero-width characters,
 concatenation,
 encoding,
 all of which
defeat literal matching by design),
 lived only in the internal design doc and belongs in the
shipped README.
 The lesson across these:
 distinguish an undocumented contradiction of a stated
guarantee (a blocker) from a documented,
 deliberate tradeoff (a decision to ratify),
 and make the
tool's guarantees and non-guarantees explicit.
 The remaining tiers were derived from reading the
code and config directly.
 This enumeration is not asserted to be exhaustive;
 the correctness
items in Tier 0 and Tier 1 warrant a dedicated audit pass,
 and the redaction regression test in
item 4 is how the Tier 0 fix becomes durable.
 Append new findings rather than treating the count
as final.
