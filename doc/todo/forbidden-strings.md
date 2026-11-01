# Forbidden strings historical enforcement design

## Status

The Rust scanner,
out-of-band rule model,
redacted findings,
and independent CI are implemented.
Cli-git now owns local commit,
landed-commit,
and manual-push policy enforcement.
Issue `#357` removed the superseded hk/Pkl layer.
Every hk command,
Pkl snippet,
and rollout step in this document is historical evidence rather than current setup guidance.
Current commands and architecture live in `package/forbidden-strings/cli/README.md`,
`package/forbidden-strings/cli/PERF.md`,
`package/git-policy/cli/README.md`,
and `.github/workflows/forbidden-strings.yml`.

This record explains the original plan for blocking enumerable literal/regex strings from being introduced in commits,
both locally and in CI.

Release-readiness work for promoting the shipped crate to `1.0` lives in a separate
checklist:
 [forbidden-strings.1.0.md](./forbidden-strings.1.0.md).
This document deliberately does not name any of the forbidden terms,
because every committed file in this repo (including this plan) is itself a place
the rule must not appear.

## Goal and threat model

Block accidental introduction of a small,
 enumerable list of forbidden literal/regex strings
into any committed file:
 source,
 docs,
 configs,
 commit messages.
The defended-against case is **slip-ups by trusted contributors**,
 not adversarial commits.
Pre-commit hooks are bypassable with `--no-verify`,
 and trivial obfuscation
(string concatenation,
 base64,
 lookalike Unicode) defeats literal scanning;
the CI mirror is the actual line of defense and must be required by branch protection on `main`.

A defining constraint that separates this from generic deny-list tooling:
**the forbidden terms themselves are sensitive enough that they cannot appear in any committed file**,
including the file that defines the rules.
This rules out the dogfooded gitleaks/hk pattern (where rule files contain the literal patterns
and are simply excluded from scanning by glob),
 and forces an out-of-band storage architecture.

Out of scope:

- Credential/secret detection -- existing repo-level tooling and provider-side scanning cover that
- Content moderation by category (politics,
   brand,
   legal) -- regex deny-lists are a tarpit for that;
  use code review and threat-modeled per-PR checklists instead
- Already-committed history -- requires a separate `git filter-repo` operation and force-push,
  out of scope here

## Performance budget

The scanner runs on every commit (locally) and every PR (CI),
so its overhead bounds how often contributors will tolerate it
before reaching for `--no-verify`.

Concrete budget for this repo:

- Tracked corpus:
   **~12.4 MB across 2688 files**,
   ~200k lines
- Expected rule set:
   **~1k rules** mixing literals and regexes,
  some of which would be catastrophic-backtracking shaped under a backtracking engine
- **Full-repo scan budget:
   under 5s wall time,
   latest snapshot only**
  (history scanning is out of scope,
   see threat model)
- Pre-commit hot path (changed files only):
   under 500 ms
- Required throughput floor:
   ~2.5 MB/s sustained;
  target ~10× headroom (~25 MB/s) so future repo growth doesn't ratchet us back

These numbers eliminate any solution with significant per-invocation startup cost
or per-file constant overhead:
1k rules and 2.7k files multiply small constants into seconds.

Measured performance against the shipped binary lives in
`package/forbidden-strings/cli/PERF.md`.
As of 2026-05-02 the implementation is ~300x under the full-repo budget,
so the budgets above describe upper bounds the design honors with comfortable headroom,
not targets the implementation is striving toward.

## Historical decision: Rust scanner with a superseded hk adapter

Adopt **hk** (`https://hk.jdx.dev/`,
 `jdx/hk`) as the git hook runner
and implement the deny-list as a small **Rust binary** built on
[**resharp**](https://github.com/ieviev/resharp) (RE#),
a derivative-based,
 automata-backed regex engine with linear-time matching.
The deny-list contents live in a `.gitignore`'d **plain-text** file
each contributor maintains locally;
only the scanner crate,
 the loader,
 and an example template are committed.

The deny-list file format is one rule per line,
 with two forms:

- A bare line is a case-sensitive literal substring match
- A line of the shape `/PATTERN/FLAGS` is a regex match,
  delimited by the first `/` and the last `/`

Example shape (illustrative;
 real contents do not appear in this document):

```txt
PlaceholderLiteral
/placeholder.regex/i
```

Empty lines are ignored.
Lines starting with `#` are ignored as comments.
A literal that itself matches the shape `^/.+/[a-z]*$` must be expressed as a regex
(e.g. ban `/etc/passwd` literally with `/\/etc\/passwd/`).
There is no other syntax;
 the format is deliberately minimal.
Internally the loader escapes literals as regex source before feeding resharp,
so the engine sees a single uniform input.

### Why hk over alternatives

- **Tool-family alignment.
  **
  hk is by the mise author;
   this repo is mise-heavy.
  Single mental model,
   single maintainer ecosystem,
   unified `mise use` install path.
- **Generic step model.
  **
  hk steps run arbitrary shell commands;
   invoking a Rust binary is first-class
  (mise installs the binary;
   hk shells out).
- **One config powers local and CI.
  **
  `hk run pre-commit` locally,
   `hk check --from-ref origin/main` in CI.
  No duplicated rule definitions across husky and a GitHub Action.
- **Production-ready.
  **
  `v1.44.3` released 2026-04-30,
   ~800 stars,
   active.

### Why resharp over alternatives

- **Linear-time matching.
  **
  Derivative/automata-based,
   no backtracking.
  A 1k-rule mix that includes catastrophic-backtracking shapes
  cannot exhibit catastrophic behavior under resharp by construction.
- **Combined-pattern matching is the architectural fit.
  **
  All 1k rules can be compiled into one alternation and matched in a single pass over each file,
  which is what the perf budget requires:
  a per-rule loop with 1k rules × 2.7k files × any non-trivial constant misses 5s.
- **Native Rust binary.
  **
  Eliminates the runtime/startup overhead that breaks the budget for hosted-language scanners
  (see "Why not betterleaks" below).
- **Maintained and live.
  **
  MIT,
   ~110 stars,
   by ieviev;
   pushed within days of this writing.
  The intersection/complement primitives are bonus expressivity we don't strictly need,
  but they let future rule composition stay declarative.

### Why not the alternatives

- **betterleaks (gitleaks fork)** -- evaluated and rejected on perf.
  Has the right config story (`BETTERLEAKS_CONFIG` and `BETTERLEAKS_CONFIG_TOML` env vars
  load rules out-of-band;
   `--redact=100` suppresses matched values from output)
  and is actively maintained (`v1.1.2` 2026-04-08,
   by gitleaks' original creator).
  But measured against this repo with 1k rules and `keywords` prefilters,
  full-tree scans run **far over the 5s budget**;
  even a stdin scan of one small file warm is ~250ms,
  which is process startup + 1k-rule TOML parse before any matching.
  Aho-Corasick prefilter and re2 regex engine help but don't recover the budget.
  Also lacks native `.gitignore` (only `.gitleaksignore`),
  forcing an `xargs git ls-files` shape that interacts with file-target config-handling bugs
  (issues #85,
   #87 still open as of writing).
- **gitleaks + custom rules** -- same perf concerns as betterleaks plus
  the canonical pattern stores patterns in a TOML file in the repo;
  out-of-band is possible but cuts against the tool's grain.
- **git-secrets** -- AWS Labs project,
   last tag in 2020,
   no CI story without scripting it yourself.
  Rules live in opaque `git config` entries (good for the out-of-band requirement)
  but the project is effectively unmaintained.
- **trufflehog** -- custom regex detectors require a `keywords` filter,
  not a clean deny-list.
   Designed for credential scanning,
   not literal blocking.
- **secretlint with `@secretlint/secretlint-rule-pattern`** -- works,
  but pulls a JS-ecosystem secret-scanning framework for one rule type
  and inherits Node startup overhead.
- **TypeScript scanner with `aho-corasick` + `re2-wasm`** -- viable on smaller repos
  but on this one's corpus + rule budget the WASM regex engine is the bottleneck;
  Bun startup + WASM init + per-file overhead × 2.7k files exceeds the budget.
  Native code is the structural fix.
- **RE2 directly (without resharp)** -- linear-time same as resharp.
  Reasonable alternative;
   resharp's edge is the combined-pattern construction
  and intersection/complement primitives for future rule composition.
  Picking resharp is a soft preference;
   if resharp throughput disappoints in practice,
  swapping to RE2 is mechanical.
- **GitHub native push protection / custom patterns**:
  custom patterns require GitHub Secret Protection (paid),
  unavailable on free public repos owned by individuals.
- **Native git hook + `grep` script**:
  `AGENTS.md` prohibits bash scripts;
   perf is fine for small rule sets but degrades with 1k rules.
- **`pre-commit/pre-commit-hooks` (Python pre-commit framework)**:
  no built-in or popular community hook for arbitrary forbidden-words enforcement.
  Pulls a Python tool into a Bun/mise repo;
   tool-family mismatch.

## Historical implementation plan

### Where the deny-list lives

The deny-list cannot be committed.
Options considered:

1. **Per-contributor gitignored file.
   **
   Each contributor maintains `forbidden-strings.local.txt` themselves.
   Simplest,
    fits a small-team or solo repo,
    no shared-secret distribution problem.
   Downside:
    enforcement is uniform only by convention;
    one contributor's missing rule
   means CI cannot block that term either,
    since CI also reads from a configured location.
2. **Shared deny-list via a private channel.
   **
   Distribute via a private git submodule,
    an encrypted file with the key out-of-band,
   1Password / a secrets manager,
    or fetched at hook-runtime from an authenticated endpoint.
   Stronger consistency,
    real distribution overhead.
3. **Hash-based deny-list,
    committed.
   **
   Store SHA-256 of each forbidden literal;
    scanner hashes candidate substrings
   and compares against the hash list.
   Works only for case-sensitive exact literals;
    breaks regex matching
   unless the scanner enumerates variants.
4. **GPG-encrypted deny-list,
    committed as ciphertext.
   **
   Each contributor decrypts at hook-runtime with their own key.
   Adds a key-management step but lets the file live in the repo.

The recommended starting point is option 1 for both local and CI.
For CI,
 the file is provided as a GitHub Actions secret containing the plain-text deny-list,
written to `forbidden-strings.local.txt` at job-start;
 same path,
 different source.
This means CI enforces whatever the secret holds,
 even if a contributor's local file diverges.

### Files to add (committed)

- `hk.pkl` -- hk configuration;
   one `pre-commit` step invoking the scanner
- `package/forbidden-strings/cli/` -- Rust crate (`Cargo.toml`,
   src/`main.rs`);
  binary name `forbidden-strings`,
   depends on the `resharp` crate plus a TOML parser
- `package/forbidden-strings/cli/mise.toml` -- per-package build task
  (`cargo build --release`) and install task that drops the binary on PATH
- `forbidden-strings.local.example.txt` -- placeholder template a contributor copies
  to `forbidden-strings.local.txt` on first setup;
  contains only an obviously-fake placeholder line
- `.github/workflows/forbidden-strings.yml` -- CI mirror;
   materializes the deny-list
  from a repository secret,
   builds the scanner once (cached),
   then calls
  `hk check --from-ref origin/main`

### Files to modify

- `.gitignore` -- add `forbidden-strings.local.txt`
- Root `mise.toml` -- pin `hk` and `pkl`.
  `rust = "latest"` is already pinned and is enough;
  resharp is a library dependency declared in the scanner crate's `Cargo.toml`,
  not a `cargo:` tool (mise's `cargo:` backend installs binaries,
   not libraries).
  The scanner binary itself is built via the package's mise tasks rather than installed as a tool
- Local git-hook wiring -- handled by `hk install --global` (recommended;
   one-time per machine,
  Git 2.54+;
   the hook is a no-op in repos without `hk.pkl`) or `hk install` per-repo.
  hk replaces husky entirely;
   no `.husky/pre-commit` edit is needed
- Branch protection on `main` (configured outside the repo on GitHub)
  to require the new CI check to pass before merge

### Scanner design

The scanner is a single-binary Rust CLI.
Invocation shape:
 `forbidden-strings --rules <path> [--all] [FILE...]`.

On startup:

1. Parse the rules file (`--rules` arg,
    or `FORBIDDEN_STRINGS_RULES` env) line-by-line:
   empty lines and `#` comments are skipped;
   a line matching `^/.+/[a-z]*$` is parsed as a regex with the trailing letters as flags;
   every other non-empty line is a case-sensitive literal,
   escaped to regex source for engine input.
   Each rule retains its 1-based index from input order.
2. Build a single combined automaton via resharp from the union of all rule sources.
   Compile cost is paid once per invocation.
3. Resolve the file list:
   in `--all` mode the binary calls `git ls-files -z` itself and reads the result;
   otherwise it scans the file paths passed as positional arguments.
   The scanner reads each path from the **working tree** via plain FS reads,
   not the git index.
   This is strictly better than staged-blob reads for the threat model:
   any forbidden term that lands on disk is flagged immediately,
   even if not yet staged,
   so a term sitting in an unstaged hunk gets surfaced now rather than slipping through
   on a future `git add .`.
   Reading 12.4 MB of tracked content into RAM is sub-millisecond on modern SSDs;
   any `git cat-file --batch` approach loses to plain reads by orders of magnitude.
   Loading via mmap or buffered reads is an implementation detail.

   The only residual imprecision is the inverse case
   (stage a forbidden term,
    then edit the working tree to remove it,
   commit before the next scan),
   which requires deliberate sequencing and falls under the obfuscation cases
   the threat model already accepts as out of scope.
4. For each file,
    run the combined automaton over the content in one pass,
   collect all match offsets and the originating rule index.
5. For each hit,
    report `path:line:start-col..end-col` plus the opaque rule index
   on stderr,
    one finding per line.
6. Exit `1` if any rule fired,
    `0` otherwise.

Columns are 1-based code-unit offsets within the matched line.
The range is inclusive on both ends;
 for a hit at column 14 spanning four characters,
the report reads `path:42:14..17`.

The scanner must explicitly **not** print the matched substring,
the rule pattern,
 or any surrounding line context in failure messages:
only the path,
 line number,
 column range,
 and the opaque rule index.
Otherwise the failing CI log itself becomes a leak surface.
The column range leaks only the length of the match,
which is an accepted tradeoff for editor-jumpable output.
A contributor wanting to know which rule fired looks up the index against their local file.

#### Combined-automaton blowup fallback

A pathological union of arbitrary regexes can blow up the combined-automaton state count.
resharp's derivative-based construction is more resilient than classical NFA->DFA
but is not immune.
If a real rule set triggers a state-count explosion or measurable slowdown,
fall back to two-stage matching:
build an Aho-Corasick automaton over the rules' literal anchors (literals + regex `keywords`),
and only invoke the full regex on files where the AC stage hits.
This is a v1.1 optimization;
 v1 trusts the unified automaton and measures.

### Historical Pkl configuration sketch

```pkl
amends "package://github.com/jdx/hk/releases/download/v1.44.3/hk@1.44.3#/Config.pkl"

hooks {
  ["pre-commit"] {
    steps {
      ["forbidden-strings"] {
        check = "forbidden-strings --rules ${FORBIDDEN_STRINGS_RULES:-forbidden-strings.local.txt} {{files}}"
      }
    }
  }
  ["check"] {
    steps = (hooks["pre-commit"].steps)
  }
}
```

### Historical CI workflow sketch

A single GitHub Actions job on pull_request and push to main:

1. Checkout with full history (`fetch-depth: 0`) so `--from-ref` works
2. Install mise,
    run `mise install` to get hk and the Rust toolchain
3. Build the scanner with `mise run //package/forbidden-strings/cli:build`
   (cache `target/` keyed on `Cargo.lock` to keep CI fast)
4. Materialize `forbidden-strings.local.txt` from a repo secret
   (e.g. `secrets.FORBIDDEN_STRINGS_LIST`) -- the secret holds the full plain-text deny-list
5. `hk check --from-ref origin/main` -- runs the same steps the local hook runs,
   scoped to the changed files in the PR

The workflow YAML must not echo the secret.
GitHub masks secret values in logs,
 but the materialization step must read from
an `env:` value piped to the file (e.g. `printenv FORBIDDEN_STRINGS_LIST > path`),
not interpolate the secret into a `run:` block where shell expansion could leak it.

Make this a required check in branch protection so `--no-verify` locally cannot land on `main`.

## Caveats and known limitations

- **`--no-verify` bypasses the local hook.
  **
  This is true of every option in this category.
  CI is the enforcement;
   the hook is the fast feedback loop.
- **Obfuscation defeats literal scanning.
  **
  Anyone who wants to bypass can split the string,
   encode it,
   or use Unicode lookalikes.
  Deny-lists work for accident prevention only.
- **Working-tree reads,
   not staged blobs.
  **
  The scanner reads files from the filesystem rather than the git index.
  This is the strict-default for the threat model:
  any forbidden term on disk is surfaced immediately,
   even if currently unstaged,
  so contributors deal with it now rather than discovering it on the next `git add .`.
  The only residual imprecision is the inverse case
  (stage a term,
   then edit the working tree to remove it,
   commit before re-scanning),
  which requires deliberate sequencing and is the same category as the obfuscation cases
  the threat model already concedes as out of scope.
  In CI the checked-out tree is the content being committed,
   so the distinction is moot.
- **Already-committed instances of forbidden strings remain in history.
  **
  Searching with `git log -S` and GitHub search will still surface them.
  Cleanup requires `git filter-repo` and a force-push (a separate,
   destructive operation
  not in scope here).
- **The deny-list grows by accretion.
  **
  Without periodic review,
   false positives accumulate
  and contributors learn to `--no-verify` reflexively.
  Plan:
   review the list quarterly,
   remove unused rules,
   document why each entry exists.
- **Pkl added a config language during the retired hk phase.
  **
  Issue `#357` removed both Pkl and its IDE configuration after cli-git parity.
- **Rust toolchain becomes a build-time dependency.
  **
  `mise.toml` already pins `rust = "latest"`,
  so the cost is bounded;
   CI must cache `target/` keyed on `Cargo.lock`
  or first-run build time will dominate.
- **Combined-automaton state-count blowup.
  **
  resharp handles 1k mixed rules well in the expected case,
  but a pathological rule combination can explode automaton size.
  Mitigation:
   AC-prefilter fallback (see Scanner design).
  Detection:
   instrument the scanner to log build time and state count
  in `--verbose` mode so a slow rule addition is noticed at edit time.
- **Failure messages must be redacted.
  **
  Printing the matched substring or surrounding lines in error output
  turns a CI log into a leak surface;
  the scanner reports only `path:line:start-col..end-col` and the opaque rule index.
  The column range leaks the match length only,
   which is the accepted floor.

## Historical rollout

1. Land the Rust crate at `package/forbidden-strings/cli/`
   with the loader pointing at a `.local.txt` file that contains
   only an obviously-fake placeholder rule.
   Verify a `cargo build --release` and that the binary runs end-to-end
   on a synthetic input that exercises a literal hit,
    a regex hit,
    and a no-hit file.
2. Benchmark the v0 scanner against the full tracked corpus
   with a synthetic 1k-rule set
   (700 literals,
    200 normal regexes,
    100 catastrophic-shape regexes;
   resharp's automata-based engine handles all three classes in linear time).
   Confirm full-repo wall time is well under the 5s budget;
   if the combined automaton blows up,
    gate the regex stage behind an AC literal prefilter.
3. Land hk wired to invoke the binary,
   verify the local hook fires and the CI job runs green on a no-op PR.
4. Configure the GitHub Actions secret holding the real plain-text deny-list.
   Confirm CI fails on a test PR introducing a deny-listed term
   and that the failure log contains only `path:line:cols` plus rule index,
    never the term itself.
5. Distribute the real `forbidden-strings.local.txt` to each contributor via a private channel.
6. Enable branch protection on `main` to require the CI check.
7. Document the workflow for adding a new forbidden string in `doc/agent/forbidden-strings.md`
   (or extend an existing agent doc):
   how to update the local file,
    how to update the CI secret,
   how to format new rules,
   and how to interpret the opaque rule index in failure messages.

## Historical open questions

- **Scope of files scanned.
  **
  Source files only,
   or also documentation,
   commit messages,
   and config files?
  Default:
   every staged file.
   Excluding any class of file is a request to allow the term there,
  which should be a deliberate,
   justified decision.
- **Documents that legitimately need to discuss a forbidden term**
  (e.g. an incident postmortem referencing a customer name).
  The minimal format has no per-rule path filter,
  so this means either:
   (a) the scanner gains a global exclude list of paths
  (a second simple file) or (b) docs always use a deliberate alias.
  Decide before adding any rule whose forbidden term might appear in legitimate discussion.
- **Commit-message scanning.
  **
  hk supports a `commit-msg` hook;
  do we want the deny-list to fire there too?
  Default:
   yes,
   since commit messages are committed and historical.
- **Husky retirement.
  **
  hk supplants husky;
   the existing `.husky/pre-commit` (currently a single comment line)
  becomes dead weight once `hk install --global` is in place.
  Decide whether to delete `.husky/` entirely as a follow-up.
- **History cleanup policy.
  **
  Should we run a history search for each new rule before adding it,
  to know whether existing history already contains the term?
  If so,
   what's the policy:
   block adding the rule until history is clean,
  or accept that the rule prevents future introductions only?
- **Distribution channel for the real deny-list.
  **
  Per-contributor manual setup (simplest) versus a private submodule,
   encrypted file,
  or fetched-from-private-endpoint (more robust,
   more setup overhead).
- **Whether option 4 (GPG-encrypted committed file) is preferable to option 1 + CI secret.
  **
  Tradeoff:
   GPG adds key management for every contributor;
  CI secret only enforces in CI and depends on contributors maintaining their own copy locally.

## Prior art consulted

- hk:
   `https://hk.jdx.dev/`,
   `https://github.com/jdx/hk`
- resharp (RE#):
   `https://github.com/ieviev/resharp`
  (Rust,
   automata-based regex with intersection/complement;
   the chosen engine)
- betterleaks:
   `https://github.com/betterleaks/betterleaks`
  (gitleaks fork by gitleaks' original creator;
  has out-of-band config via `BETTERLEAKS_CONFIG`/`BETTERLEAKS_CONFIG_TOML`
  and `--redact=100` for safe output;
  rejected on perf,
   see "Why not the alternatives")
- gitleaks:
   `https://github.com/gitleaks/gitleaks` (custom rules supported,
   well-maintained)
- git-secrets:
   `https://github.com/awslabs/git-secrets` (stale,
   last tag 2020)
- trufflehog:
   `https://github.com/trufflesecurity/trufflehog`
  (custom regex requires keyword filter,
   not a pure deny-list)
- secretlint:
   `https://github.com/secretlint/secretlint`
  (`@secretlint/secretlint-rule-pattern` accepts arbitrary regex)
- pre-commit framework:
   `https://pre-commit.com/`
  (no popular community hook for arbitrary forbidden words)
- GitHub Secret Protection custom patterns:
  `https://docs.github.com/en/code-security/secret-scanning/`
  (paid;
   not available on free public repos)
