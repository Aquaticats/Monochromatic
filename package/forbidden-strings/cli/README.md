# forbidden-strings

Linear-time deny-list scanner for Git repos,
 built on the in-house `forbidden-regex` engine
(`package/forbidden-strings/core`).
 A native Rust binary with a sub-commit-budget
startup,
 it scans working-tree files line by line against a deny list of literals and
restricted-dialect regexes and reports each match as an opaque,
 redacted finding.

Rules split into a baseline embedded in the binary
(`data/builtin-rules.txt`,
 activated by `--builtin-rules`),
 a committed shared appendix
(`forbidden-strings.append.txt`),
 a per-repo gitignored appendix
(`forbidden-strings.append.local.txt`),
 or a CI-only secret (`FORBIDDEN_STRINGS_LIST`).
The matched substring,
 the surrounding line,
 and the rule pattern are never printed in
failure output,
 so a rule body that would itself leak if committed (a customer name,
 an
unreleased project codename,
 a pre-disclosure partner ID) can live as an appendix or CI
secret without exposure on public CI logs.

## What's different

- **Native binary startup.** Rust with `lto = true`,
   `codegen-units = 1`,
   `opt-level = 3`,
  `panic = "unwind"`,
   `overflow-checks = true`,
   `strip = true`.
   No Node startup and no WASM init.
   Explicit regex rules compile once per distinct content;
  later invocations reload compact exact-literal groups plus regex-only engine bytes and rebuild one Aho-Corasick
  matcher,
  which meets the measured sub-100 ms pre-commit budget.
- **Linear-time matching.** The engine is derivative and product based with no
  backtracking,
   so no rule combination can exhibit catastrophic-backtracking behaviour.
  A set-level SIMD prefilter lets clean lines skip per-rule work.
- **Set-algebra rules.** Intersection `A & B` and complement `~(A)` are first-class in
  the dialect,
   so "match X but not Y" needs no lookaround.
   PCRE-family engines
  (gitleaks,
   trufflehog,
   secretlint,
   plain RE2) cannot do this;
   their workaround is
  per-rule allowlists,
   which scale badly.
- **Sensitive rules can live out-of-band.** The committed baseline holds non-sensitive
  rules;
   the gitignored appendix and the CI-only `FORBIDDEN_STRINGS_LIST` secret hold
  sensitive rules.
   Failure output never prints the matched substring,
   the surrounding
  line,
   or the rule pattern,
   so a rule body itself can be a secret.

## When to pick something else

`forbidden-strings` deliberately omits features other scanners ship as core capabilities:

- **CEL-based post-match filtering** (entropy thresholds,
   BPE token efficiency,
   git-author
  predicates,
   file-path globs,
   string allowlists).
   No equivalent here.
- **Async HTTP validation**.
   No way to call a provider API to confirm a detected secret
  is live;
   staleness review is on you.
- **Git history scanning**.
   The walker enumerates working-tree files only.
   No equivalent
  of `gitleaks git` that scans every diff in every commit.
- **SARIF / JSON / CSV output**.
   Hits go to stderr as plain text;
   no machine-readable
  format for GitHub code-scanning upload or CI dashboards.
- **Per-rule path scoping**.
   Every rule runs against every non-skipped file;
   the scanner
  cannot apply rule X only to YAML files.
- **Per-rule allowlists**.
   No way to say "rule X but skip when it matches in path Y".
- **No streaming or stdin input.** Files only;
   the walker enumerates from disk.

If you need any of those,
 betterleaks or gitleaks is the right tool.

## Prerequisites

- **Rust toolchain**.
   Install via mise:
   `mise install rust`.
- **mise** itself,
   since build commands are `mise run` tasks.

## Build

```sh
mise run //package/forbidden-strings/cli:build
```

The release binary lands at `package/forbidden-strings/cli/target/release/forbidden-strings`.
The cli-git native wrapper links the scanner in-process,
 so it does not use this binary and nothing needs to be on
`$PATH`;
 `rulesFile` in the root `cli-git.config.jsonc` names the rules file.

## Setup

The scanner needs exactly one rules file at scan time.
 How you produce it is up to you.

### Without file-enforcer (most consumers)

Put rules in a file named `forbidden-strings.local.txt` at the repo root
(named tail-format sections,
 or legacy one-rule-per-line;
 see "Rule file format"),
 or
pass `--rules <PATH>` / set `FORBIDDEN_STRINGS_RULES=<PATH>` to point at any other path.
That is the whole setup.
 For a zero-file start,
 pass `--builtin-rules` to scan with the
embedded betterleaks-ported baseline (see "Built-in baseline" below).
 Add the file to
`.gitignore` if the rules themselves are sensitive;
 otherwise commit it.
 The "Rule file
format" section describes the line syntax.
 In CI,
 materialise the file from a secret (see
"GitHub Actions" below) so the rule bodies never enter version control.

### With file-enforcer (this monorepo's workflow)

Inside the Monochromatic monorepo,
 no rules file exists at the repository root
(see `doc/decision/gitignore-negations.md`).
 The pieces:

- The betterleaks baseline ships inside the scanner binary
  (`data/builtin-rules.txt`,
   regenerated by the two-stage chain behind
  `mise run //package/forbidden-strings/cli:generate:rules`:
   the TS porter `src/mise.port-betterleaks.ts`,
   then the `dialectport` bin in
  `package/forbidden-strings/core.bench/src/bin/`);
   repo invocations activate
  it with `--builtin-rules` (the cli-git policy sets `builtinRules: true`).
- `forbidden-strings.append.txt` is the committed shared appendix of non-sensitive
  repo-wide rules.
- `forbidden-strings.append.local.txt` is the per-repo additions.
   Gitignored,
   free-form,
  edited by hand.
   Place sensitive literals (codenames,
   customer names,
   partner IDs) here.
- `.cache/forbidden-strings.rules.txt` is the authoritative runtime text consumed by the scanner.
  File-enforcer concatenates the two appendixes into the gitignored `.cache/` scratch dir.
  The generated root `mise.toml` `[env]` points `FORBIDDEN_STRINGS_RULES` at it
  (absolute via `{{config_root}}`).
  Do not edit directly.
- A compiled runtime artifact lives under the current user's native cache root,
  partitioned by scanner version,
  platform,
  and SHA-256 of the exact runtime text.
  It stores exact-literal groups directly and precompiled engine bytes only for explicit regex rules.
  File-enforcer invokes `forbidden-strings compile-rules` after generating the text when the
  release scanner exists.
  A later scan repairs a missing or rejected artifact automatically.

Run `mise run file-enforcer` after editing either appendix to regenerate the runtime text
and eagerly compile its cache artifact.
The generator is `generateForbiddenStringsRules` in `file-enforcer.config.ts`.
During fresh setup before the release scanner exists,
file-enforcer reports that eager compilation was skipped;
the first later scan compiles and publishes the artifact.
A stale pre-0.4.0 release binary exists but does not understand `compile-rules`,
so file-enforcer fails instead of silently claiming eager compilation;
rebuild the scanner before rerunning file-enforcer.
 If you
fork this scanner into a project that does not use file-enforcer,
 drop the appendix split
and follow the single-file workflow above.

## Usage

```sh
# scan a specific file list (uses ./forbidden-strings.local.txt by default)
forbidden-strings path/to/file other/file

# scan every working-tree file (.gitignore respected; .git/.jj skipped)
forbidden-strings --all
```

The rules list is resolved in this order:
`--rules <LIST>` flag (highest),
then `FORBIDDEN_STRINGS_RULES` environment variable,
then `./forbidden-strings.local.txt` in the current working directory.
The flag replaces the variable; the two are never merged.
A list with one entry and no trailing `?` is a plain path, the only form that existed before lists.
See "Rules list" below for the grammar.

Every runtime-rules scan uses read-write caching by default with no opt-out.
`FORBIDDEN_STRINGS_CACHE_DIR` may set an absolute cache root.
Without the override,
the scanner uses `$XDG_CACHE_HOME` or `$HOME/.cache` on XDG-oriented Unix,
`$HOME/Library/Caches` on macOS,
and `%LOCALAPPDATA%` on Windows.

```sh
# explicit path
forbidden-strings --rules ./other-rules.txt --all

# via env var (CI-friendly: materialize from a secret, then run)
FORBIDDEN_STRINGS_RULES=./materialized.txt forbidden-strings --all

# eagerly compile one rules file without scanning
forbidden-strings compile-rules --rules ./other-rules.txt

# print version and exit
forbidden-strings --version    # or -V
```

`compile-rules` requires an explicit rules list (the same grammar as `--rules`),
derives the user-cache destination,
and prints nothing on success except one `info` log line per skipped optional entry.
It reuses a valid artifact and otherwise compiles,
rechecks the source snapshot,
and atomically publishes owner-only cache data.

Cache misses and rejected artifacts fall back to authoritative text and emit one compact
JSON warning on stderr before findings.
A failed cache write emits another warning but does not invalidate the already compiled scan.
Warnings contain only closed reason and recovery tokens,
never source paths,
cache paths,
digests,
rule text,
or operating-system error text.

One reason differs by platform.
When a regular file sits where the cache root directory should be,
the first warning's reason is `unreadable` on Unix targets and `missing` on Windows,
because Unix reports a not-a-directory error for a path below a file,
while Windows reports the same path-not-found error as for an absent directory.

### Rules list

`--rules`, `compile-rules --rules`, and `FORBIDDEN_STRINGS_RULES` carry a rules list:
one or more rules files, read in order and scanned as one rule set.
The grammar is the whole definition; there is no escape syntax.

```text
list  := entry ( "," entry )*
entry := path [ "?" ]
path  := one or more characters, none of them ","
```

- The value is split on every `,`.
  Nothing is trimmed, so `a.txt, b.txt` names the file ` b.txt` (with its leading space).
- An entry ending in `?` may be absent.
  An absent optional entry is skipped and logged once at `info` level
  (`optional rules file is absent; skipped`, naming the path).
  Every other entry must exist; a missing or unreadable one exits 2 and names its path.
- Exactly one trailing `?` is the marker.
  A path that itself ends in `?` is therefore written with a second one (`name??` is the optional file `name?`);
  it cannot be named as a required entry.
  A `?` anywhere else in a path is an ordinary character.
- A path containing `,` cannot be named.
  That tradeoff came with choosing a delimited list over repeating the flag.
  Reach such a file through a symbolic link or a relative path without the comma.
- An empty value, an empty entry (`a,,b`, a leading or trailing comma, a bare `?`), and a path named twice exit 2
  with the entry's position, before any file is read.
- A value with no `,` and no trailing `?` is one required entry equal to the whole value, which is exactly what it
  meant before lists existed.
  The only values whose meaning changed are those containing a `,` or ending in `?`.
- Relative entries resolve against the working directory, like the single path always did.
  The monorepo anchors each entry with mise's `{{config_root}}`.
- A path that is not valid UTF-8 can only be given through a single-entry `FORBIDDEN_STRINGS_RULES` in an embedding
  host; the command line and `std::env::var` carry UTF-8 only.

How the files combine:

- Every present file is joined in list order into one text that is compiled once.
  Rule ids and the `--builtin-rules` offset follow that order.
- All present files must be in one format (tail sections or legacy lines).
  Joining a legacy file after a tail file would turn its lines into the body of the last section,
  so a mix exits 2 naming both files.
  A file with only blank lines and comments joins with either format.
- A BOM is stripped from each file before the join, and a newline is added after any file that lacks one,
  so a file's last line never fuses with the next file's first.
- Section names must stay unique across the files.
  A failure in the joined text is re-diagnosed file by file, so the error names the file and a line number local
  to it (`rules <path>: line 5: ...`); a repeated section name names both files.
- If no file is present because every entry is optional and absent, the scan exits 2
  (`no rules loaded`), unless `--builtin-rules` makes the baseline the whole rule set.
  Scanning with no rules would pass every file.
- `--all` skips every listed file, not only the first.

Cache identity is the SHA-256 of the joined text, with the scanner version and platform.
The compiled rules are a pure function of that text, so the key is complete:
an edit to any file, a file added or removed (an optional file appearing or disappearing), and a changed order
all change the text and miss, while an unchanged list hits.
Paths are deliberately not in the key, so two checkouts or worktrees with identical rules share one artifact.
A one-file list joins to the file's exact bytes, so its artifact is the one a plain path always used.
Before an artifact is published, every listed file is read again; a file that changed, vanished, or appeared
since the first read blocks publication and emits the `write-failed` warning.

Embedding hosts parse the same value with `forbidden_strings::parse_rules_list` or build `RulesEntry` values
directly, then call `Scanner::load_entries`; `Scanner::skipped_optional_rules` lists the skipped files for hosts
that install no log subscriber.
`Scanner::load` keeps the one-path contract.

### Built-in baseline (`--builtin-rules`)

The binary embeds the betterleaks-ported baseline ruleset.
 The text form
(`data/builtin-rules.txt`) is exported as the library constant
`forbidden_strings::BUILTIN_RULES`;
 the scan path loads the baseline from a serialized
`RegexSet` precompiled at build time (compiling the full baseline at each startup is not
viable,
 so the cost is paid once during the build).
 It is pure opt-in:
 without the flag the
scanner never reads it,
 so existing invocations behave exactly as before the flag existed.

With `--builtin-rules`:

- The baseline is appended after the resolved rules file.
   Baseline findings render
  as `rule=<name>` (the upstream betterleaks id),
   so they identify themselves
  regardless of position.
   Unnamed legacy rules fall back to numeric ids that do not
  shift:
   your rules keep ids `0..user_len` and the baseline takes `user_len..`.
   A
  runtime rule whose section name collides with a baseline name fails the load
  closed (the finding token must stay unambiguous).
- When no rules file resolves at all
  (no `--rules`,
   no env var,
   and no `./forbidden-strings.local.txt` in cwd),
  the baseline alone is the ruleset;
   passing the flag is itself the configuration.
- An explicitly named missing file
  (`--rules <path>` or the env var pointing at a path that does not exist)
  still exits 2:
   silently scanning without your rules would be a false-clean result.
  Only an entry marked optional with a trailing `?` (see "Rules list") may be absent.

```sh
# zero-file quick start: scan the tree with the embedded baseline only
forbidden-strings --builtin-rules --all

# your rules plus the baseline
forbidden-strings --builtin-rules --rules ./rules.txt --all
```

`--all` and positional files are mutually exclusive in practice:
 if both are passed,
 the
walker output silently overwrites the positional list.
 Use one or the other.

## Rule file format

Two formats,
 autodetected per file on the first significant line;
 one file never mixes
them.

### Tail format (named sections)

A rule file is what `tail --verbose -n +1` over per-rule files would produce:
 a
`==> name <==` header line opens a section whose body is one rule.
 The name (first
character `[a-z0-9]`,
 then any of `[a-z0-9.-]`,
 unique within the loaded file) is the
rule's identity:
 findings render it as `rule=<name>`.

- A body with exactly one significant line (non-blank,
   not `#`-leading) is classified
  by the two line shapes below,
   exactly as in the legacy format.
- A body with more than one significant line is one verbatim regex pattern handed to
  the always-verbose engine as-is (blank lines and first-column `#` comments inside it
  are the engine's own to consume).
- Any line whose trimmed form starts with `==>` without being exactly a strict header
  fails the load closed with its line number (case typos,
   indentation,
   missing-space
  arrows,
   and genuine arrow content alike);
   write genuine `==> ` content in a regex
  body with the `[=]=> ` reshape.
- Empty sections,
   duplicate names,
   and significant content before the first header
  are fail-closed load errors,
   each reported by line number only.

### Legacy format (one rule per line)

A file whose first significant line is not a strict header parses line-by-line,
 byte-for-byte as before;
 such rules carry no name and findings fall back to
`rule=N` numeric ids.
 Two line shapes (shared with single-line tail sections):

- A bare line is a **case-sensitive literal**.
   It is escaped into the engine's verbose
  dialect and matched as a substring.
   A literal under eight bytes is word-boundary
  gated at each end whose neighbouring byte is an ASCII word byte,
   so a short code
  like `ACR` matches the whole token `ACR` but no longer fires glued inside
  `ACRYLIC`;
   eight bytes and longer match anywhere,
   and over-matching in that
  direction is a ratified preference.
   If a short literal must match glued substrings
  too,
   write it as a regex (`/ACR/`).
- A line of the shape `/PATTERN/FLAGS` is a **regex** in the `forbidden-regex` dialect.
  The first `/` and the last `/` delimit the pattern.
   `FLAGS` is a trailing run of
  ASCII-lowercase letters;
   if the trailing run is not all-lowercase,
   the whole line is
  treated as a literal instead (so `/foo/I` is a literal scan for the seven bytes `/foo/I`,
  not a case-insensitive regex).

Empty and whitespace-only lines are ignored.
 A line whose first non-whitespace byte is `#`
is a comment.
 One leading UTF-8 BOM is stripped from the source.
 An empty source (no
non-blank,
 non-comment line) is a rule-file error.

### Flags policy

The engine is always in multiline and verbose mode,
 so the only accepted flags are the
ones those two modes already imply:

- `m` (multiline) and `x` (verbose) are accepted as no-ops and dropped.
- **Any other flag letter is a hard, fail-closed load error.** Silently dropping an `i` or
  an `s` would change match semantics (case folding,
   dot-matches-newline),
   so the loader
  rejects the whole ruleset rather than weaken a rule.
   Need one of those locally?
   Restructure
  the pattern (for case-insensitivity,
   spell the alternatives:
   `[Aa][Bb][Cc]`).

### Supported constructs

The dialect is a deliberately restricted subset (see
`package/forbidden-strings/core/README.md` for the full engine spec):

- Literals and the escapes `\t`,
   `\b` (word boundary),
   backslash-escaped metacharacters,
  and backslash-escaped whitespace.
- Character classes:
   `[abc]`,
   `[a-z]`,
   `[a-zA-Z]`,
   negated `[^...]`,
   and the shorthands
  `\d \w \s \D \W \S` (usable inside classes too).
- `.` matches any byte except a newline.
- Grouping and alternation:
   `(?:a|b)`.
   Groups are **non-capturing only**.
- Bounded repetition:
   `a?`,
   `a{3}`,
   `a{3,6}`.
- Anchors:
   `^`,
   `$`,
   `\b`.
   The word set is ASCII `[A-Za-z0-9_]`;
   `^` and `$` anchor at
  line boundaries.
- Set algebra:
   intersection `&`,
   complement `~(...)`.

Matching is an unanchored search over a single line's raw bytes:
 a pattern matches if it
matches any substring.
 Because verbose mode is always on,
 unescaped whitespace outside
character classes is ignored,
 so a rule may be written across several physical characters
for readability;
 to match a literal space use `\ `,
 `\t`,
 or `[ ]`.

### Set-algebra operators

Two top-level set operators that pure-PCRE engines lack:

- `A & B` (intersection):
   matches strings matched by both `A` and `B`.
- `~(A)` (complement):
   matches strings that do NOT match `A`.

Operators `&` and `|` take single-atom operands:
 a literal,
 a class,
 `.`,
 an anchor,
 a
`(?:...)` group,
 or a `~(...)`.
 A concatenation or a quantified atom must be wrapped in
`(?:...)` to be an operand,
 so there is no operator precedence to remember.
 `~(...)` is
always parenthesized.
 A pattern that can match the empty string is rejected (unanchored,
 it
would match everything),
 so `~(Y)` alone is rejected while `(?:X) & ~(Y)` with a concrete
`X` compiles.
 Example:
 ban any five-digit key except the all-zeros placeholder:

```text
/(?:key_[0-9]{5}) & ~(key_00000)/
```

### Rejected at compile time (fail-closed)

Anything outside the supported set is a hard compile error naming the offending rule's
opaque index,
 and the whole load fails closed (a bad ruleset never degrades to a partial
scan):
 `*`,
 `+`,
 unbounded `{n,}`,
 `\xNN` byte escapes,
 capturing `(`,
 lookaround and
inline-flag groups (`(?` not followed by `:`),
 backreferences,
 unknown escapes,
 unbalanced
brackets,
 stacked quantifiers,
 `{n,m}` with `n` greater than `m`,
 repetition whose
expansion exceeds the engine's cap,
 and any pattern that can match the empty string.

Load errors are redacted:
 a compile error carries only the opaque rule index (0-based position in the
compiled set) and the engine's own static reason;
 a tail-format structural error (near-header,
 duplicate name,
 empty section,
 pre-header content) carries only a source line number;
 rule text appears in neither.
 The redacted error type is `LoadError` in `src/rule/frx/error.rs`.

## Output

For each content violation:

```text
PATH:LINE rule=<token>
```

For each pathname violation:

```text
PATH:name:SEGMENT rule=<token>
```

- `LINE` is the 1-based content line number.
- `SEGMENT` is the 1-based position of the directory name or filename within
  the logical path. The root and navigation markers (`.`, `..`) are not names.
  On Windows,
  the parts of the native volume prefix (`C:`, `\\server\share`, `\\?\C:`, `\\.\COM1`, and the other prefix forms)
  are not names either,
  whatever their spelling;
  the first component after the prefix is segment 1.
  Each segment is matched separately, so a rule cannot match across a `/` boundary.
  Pathname scanning is always on for every selected file, with no disable flag.
  It can make previously clean paths fail with exit code 1.
- An offending segment is replaced in full with `[REDACTED]` in every finding,
  including content findings and read errors for the same file.
  Other segments stay visible; no column range is reported.
  Control characters and colons in visible segments are escaped to protect
  the output protocol. A line break within a name produces a redacted,
  fail-closed diagnostic because the engine's anchored rules require a single line.
- The ordinary file-selection and exclusion rules still apply.
  Repository files use repository-relative names.
  Explicit external paths or standalone scans check all segments of the supplied pathname.
- `--name-path PATH` pairs each positional content file with a logical name,
  in argument order, for integrations reading a historical or staged file from a
  private temporary location. This form cannot accompany `--all` and requires
  one name per file. Its findings append `input=N` (0-based operand index) to
  correlate a redacted path with its candidate without disclosing the name.
- The rule token is the rule's **section name** for tail-format rules (and the
  betterleaks id for baseline rules),
   or the 0-based numeric engine id for unnamed
  legacy rules.
   Unnamed runtime rules take ids `0..user_len`;
   under
  `--builtin-rules` the baseline's unnamed ids are offset past them.
   The finding is
  columnless:
   the engine reports per-line rule indices,
   not spans,
   so no
  `COL_START..COL_END` segment appears.
- One content finding is emitted per `(line, rule)` pair; one name finding per
  `(segment, rule)` pair.
- **The matched substring, the line content, and the rule pattern are never printed.**
  Only the path,
   line number,
   and rule identity token appear,
   so a failing CI log never
  becomes a leak surface.
   Name sensitive rules with deliberately opaque section names
  (the name is the only rule-derived text that surfaces);
   a numeric legacy index is
  looked up against the local rule file.

Two synthetic findings keep the scan fail-closed:

- **Read errors.** A file that cannot be opened (broken symlink,
   permission denied,
   deleted
  during scan) produces `PATH: read error: <reason>` on stderr and counts toward the
  exit-1 total.
   A secret-scanning gate must not pass silently on a file it could not inspect.
- **Engine errors.** If the matcher panics on a file,
   the `catch_unwind` boundary in
  `scan_one_set` (`src/frx_scan.rs`) catches it and emits `PATH: engine error`,
   again
  counting toward exit 1 rather than aborting or exiting clean.

Hits go to **stderr**,
 not stdout;
 redirecting `2>/dev/null` silently loses the report.
Within a file,
 findings are emitted in set order (runtime rules before the baseline),
 then
by line;
 across files,
 ordering is rayon-scheduler-determined,
 stable on a given input but
not alphabetic.
 Callers that need deterministic cross-file reports should pipe the output
into `sort`.

Exit codes:

- `0`:
   no violations.
- `1`:
   one or more violations (real hits,
   read errors,
   or engine errors).
- `2`:
   usage error or rule-file error.

## Security model

The redaction guarantee is what lets a rule body itself be a secret.
 Two boundaries carry
it:

- **Load path.** Rule compilation reports only `LoadError` (`src/rule/frx/error.rs`),
   whose
  every variant is an opaque index plus the engine's static reason;
   no pattern bytes reach
  a diagnostic.
   The compiler builds through `RegexSet::new` / `RegexSet::from_bytes`,
  neither of which logs the pattern.
- **Scan path.** Content findings use `PATH:LINE rule=N` in `src/frx_scan.rs`;
   name findings use `PATH:name:SEGMENT rule=N` in `src/path_scan.rs`.
   The matched content and line text are never included.
   A name matching any rule is masked in the shared pathname before either finding is formatted.
   The fail-closed `catch_unwind` boundary emits only a masked-path engine error.

Keep sensitive rule bodies out of tracked files:
use the gitignored `forbidden-strings.append.local.txt` or the CI-only
`FORBIDDEN_STRINGS_LIST` secret,
never the committed baseline or appendix.
Treat compiled artifacts as sensitive as their source:
the runtime envelope retains exact bare-literal bytes and is confidential policy data,
not a redaction.
On Unix,
file-enforcer enforces mode `0600` on the local appendix and generated runtime text;
the scanner enforces private application directories and mode `0600` on artifacts.

In CI,
pipe secrets through `printenv` rather than interpolating them into a workflow command;
shell expansion can leak values even when log masking is enabled.
Hosted GitHub runners use their native account cache for the derived artifact.
A self-hosted or shared runner should set `FORBIDDEN_STRINGS_CACHE_DIR` to job-scoped storage
when its account cache survives jobs.

## In-process candidate scans

The Rust library now exposes `Scanner`,
`CandidateScan`,
`ScanFinding`,
and fixed-token `CacheWarning` accessors for the native cli-git rewrite.
Linux verification and retained platform limitations are recorded in
[the native scanner evidence](../../../doc/handover/scanner-native-verification.md).
These package tasks do not perform a cli-git production cutover.

`Scanner::load` reuses the current runtime-rule cache,
compiler,
and embedded baseline.
`Scanner::scan` accepts a caller-chosen opaque candidate identity,
a logical pathname,
and the exact candidate byte slice.
It does not open a worktree file or substitute a temporary filename.

Content and pathname findings stay structured until the standalone formatter runs.
Matching pathname components are masked before they become display labels.
Candidate identity therefore remains usable even when distinct candidates share a masked label.
Findings never contain matched content bytes.
The inherited binary policy inspects the first 8 KiB when that prefix contains NUL;
`scanned_bytes` reports that boundary explicitly.

The adapter accepts native `Path` values for rule files and logical candidate names.
Pathname components are matched as native bytes;
unmatched non-UTF-8 bytes are escaped only for display,
and matching components remain fully masked.
Public loader/consumer controls use separate processes with disposable home/cache state.
The library catches loading and matcher unwinds without changing the process-wide panic hook.
Embedding hosts must configure payload-free panic output before creating workers,
and their effective Cargo profile must use `panic = "unwind"`.
A dependency profile cannot impose that strategy on its consumer.
A hook controls panic output;
a separate catch boundary determines failure findings or loader errors.

The standalone executable owns its output-omitting hook and final exit-2 unwind boundary.
It does not print panic payloads,
thread names,
or caller locations.
No production environment variable enables fault injection.
This is not a production-cutover announcement.

## Scanner verification

Owning tasks run against immutable copied-source images with Git 2.56.0,
no host mounts,
no network,
user `1000:1000`,
2 GiB memory,
2 CPUs,
and 128 PIDs.
They do not replace installed production binaries.

```sh
# package/forbidden-strings/cli/mise.toml
mise run //package/forbidden-strings/cli:test:container
mise run //package/forbidden-strings/cli:test:release:container
mise run //package/forbidden-strings/cli:lint:clippy:container
mise run //package/forbidden-strings/cli:test:mutation:container
mise run //package/forbidden-strings/cli:test:mutation:pathname:container
mise run //package/forbidden-strings/cli:test:guards:container
mise run //package/forbidden-strings/cli:test:guards:observe:container
mise run //package/forbidden-strings/cli.fuzz:smoke:embedding:container
mise run //package/forbidden-strings/cli.fuzz:lint:clippy:container
mise run //package/forbidden-strings/cli:verify:markdown
```

Tests and Clippy enable all features.
Mutation covers embedding owners,
including the executable boundary;
only the pre-existing shipped-corpus compiler conformance tests are omitted from repeated mutant executions.
Normal container suites run those tests too.
No mutation branch is excluded to remove a survivor.
By a decision of 2026-10-05 about mutation timeouts,
two operator replacements are never tried in any scope:
`+=` with `*=`,
and `-=` with `/=`.
`+=` with `-=` stays active.
Appending `-- --list` to a mutation task prints the mutants its scope would test.
The pathname scope mutates only `src/path_scan.rs` and `src/path_name_bytes.rs` with the same suite.
It exits nonzero on Linux,
because two `prefix_parts` mutants can differ only on a Windows target.

Evidence under `target/verification/` contains per-file source hashes,
immutable image IDs,
commands,
tool metadata,
transcripts,
exit statuses,
and complete mutation reports or fuzz corpus/artifacts.
A repository `HEAD` in a manifest is context,
not a substitute for the copied-byte inventory.
A failed evidence copy retains its disposable container for recovery.

The panic tasks inject startup,
worker,
partial-load,
and post-match faults only into disposable copied source.
Protected tests must pass;
removing each output/control-flow guard must make the same consumer test fail after rebuilding.
Independent observations count the host hook after each public operation,
verify both startup and worker cases without short-circuiting,
and drive actual parallel CLI content/name fault paths.
Windows separator,
prefix-counting,
and prefix-skip policy is exercised on Linux with explicit target semantics
and the raw prefix bytes Windows's parser returns for each prefix form;
Windows's native volume-prefix parser itself runs only on a Windows host,
which no package task provides.
`mise run //package/forbidden-strings/cli:lint:clippy:windows` type-checks and lints every target,
including `#[cfg(windows)]` tests,
for `x86_64-pc-windows-gnu` on a Linux host that has that Rust target installed;
it runs no test.
See the native scanner evidence for exact results,
source snapshots,
commits,
and retained survivor limitations.

## Integration

### Local cli-git policy

Root `cli-git.config.jsonc` enables `security/forbidden-strings` at error severity.
 The
PATH-shadowed cli-git wrapper evaluates selected would-be-committed bytes before commit,
landed commit bytes before automatic push,
 and Git-native outgoing ranges before manual
push.
 Native `--no-verify` skips Git hooks but does not skip this policy.

Run an explicit read-only check through the installed wrapper with:

```sh
git cli-git check --policy security/forbidden-strings --all
```

The native wrapper links the scanner crate in-process and invokes no scanner executable.
 Scanner infrastructure failures
remain distinct exit-2 engine failures;
 findings exit `1`.

### GitHub Actions

`.github/workflows/forbidden-strings.yml` remains independent of local
wrapper state.
 It downloads the release matching the scanner crate version,
 verifies the
archive's GitHub build-provenance attestation,
 materializes the committed baseline plus
shared appendix and optional repository secret,
 then invokes the scanner binary directly.
Pull-request and merge-queue jobs scan changed files relative to `origin/main`.
 Pushes to `main` scan the tree-to-tree
delta from the newest commit whose tree a previous successful run of the workflow validated.
 The
`validated-tree-watermark-v1` marker names the workflow revisions whose push runs prove their
head tree that way.
 Only successful runs establish such a baseline, so a canceled, failed, or
timed-out run rescans everything it left unvalidated on the next push.
 With no suitable baseline
(the first marked run, a rewritten `main`, or an unavailable run list) the workflow scans the
current tree instead.
 The same precedence applies locally and in CI:
`--rules` > `FORBIDDEN_STRINGS_RULES` > `./forbidden-strings.local.txt`.

## Walker behaviour

- **`--all` semantics.** Walks the working tree via `ignore::WalkBuilder` in `src/walk.rs`:
  `.hidden(false)` (dotfiles like `.github/`,
   `.npmrc` ARE scanned),
   `.ignore(false)` (the
  `.ignore` file is NOT consulted;
   `.gitignore` stays enabled).
   Files force-added past
  `.gitignore` (`git add -f`) are recovered via an in-process `gix-index` read of
  `.git/index`;
   no git subprocess.
- **`.git/` and `.jj/` skipped.** Internal VCS state is never scanned.
- **Symlinks NOT followed.** `WalkBuilder`'s default `follow_links` is false;
   symlinked
  directories are not descended,
   symlinked files surface as a read-error synthetic hit on a
  broken target.
- **Non-UTF-8 paths silently dropped.** Index entries that are not valid UTF-8 are excluded
  from the walk.
- **Binary-file 8 KiB tail cap.** Files whose first 8 KiB contains a NUL byte are scanned
  only in the first 8 KiB.
   The leading window always runs,
   so secrets there fire;
   the tail
  past 8 KiB is skipped.
   Constant `BIN_PROBE_SIZE` and `read_with_binary_check` in
  `src/lib.rs`.
- **Self-skip set.** During `--all`,
   canonical paths are auto-skipped so rule bodies do not
  self-match:
   the materialised rules file (whatever `--rules` / env var / default resolves
  to),
   plus three generated-source paths:
  - `package/forbidden-strings/cli/data/betterleaks-default-config.toml`
  - `package/forbidden-strings/cli/data/builtin-rules.txt`
  - `package/forbidden-strings/cli/src/port-betterleaks-relaxations.ts`

  Skip is path-anchored via `std::fs::canonicalize`,
   not basename-anchored,
   so an
  unrelated file named `forbidden-strings.local.txt` in a subdirectory is still scanned.
  Paths that fail to canonicalize from the current cwd are silently dropped from the set.
  Explicit positional arguments bypass the `--all` skip;
   the scanner's own
  `forbidden-strings.*.txt` config files at cwd are skipped in both modes
  (`is_config_file_at_cwd` in `src/lib.rs`).

## Performance

The scanner is a native Rust binary tuned for a sub-commit-budget startup and linear-time
matching;
 the release profile (`Cargo.toml`) sets `lto`,
 `codegen-units = 1`,
 `opt-level = 3`,
`panic = "unwind"`,
 `overflow-checks = true`,
 and `strip`.
 `panic = "unwind"` and
`overflow-checks = true` are load-bearing for the fail-closed `catch_unwind` boundary,
 not
speed:
 the forbidden-regex engine documents that it expects the caller's unwind boundary,
and a wrapped overflow would otherwise let a corrupt rule fail open.

Full bench methodology and per-version regression history are in `PERF.md`.
Runtime cache verification measures first compilation separately from repeated artifact loads.
A valid artifact must preserve finding identity and matching behavior before its timing is accepted.
The cache directory has no automatic retention policy;
deleting its `forbidden-strings/` child is always safe because authoritative text regenerates it.

## Fuzzing

Coverage-guided fuzzing lives in its own package,
 `package/forbidden-strings/cli.fuzz`,
 so a
scoped nightly toolchain does not force this published crate onto nightly.
 The scanner
exposes a curated internal surface (`fuzz_api`,
 behind the `fuzzing` Cargo feature) for the
targets to drive.
 The teardown that removed the old engine also retired the gate,
 shard,
 and
routing targets that fuzzed it;
 the surviving targets are retargeted onto the
`forbidden-regex` load and scan path.
 See that package's README for prerequisites,
 mise
commands,
 the bounded-container wrapper,
 and crash-reproduction guidance.

## Architecture

- **Two-form loader.** `src/rule/frx` owns the rule-file format
  and preserves whether each rule was written as a bare literal
  or an explicit regex.
  Bare literals still escape into the verbose dialect for the public engine compiler;
  runtime scanning instead retains their exact bytes.
  `/PATTERN/FLAGS` lines and multiline tail sections remain restricted regex rules,
  validated under their original global rule ids.
- **Hybrid runtime matcher.** `src/runtime_matcher.rs` de-duplicates exact literals
  into one overlapping Aho-Corasick matcher
  and compiles only explicit regex rules into `RegexSet`.
  Both subset-local outputs map back to original ids,
  sort,
  and de-duplicate before finding attribution.
- **Content-addressed runtime cache.** `src/runtime_cache/` hashes exact authoritative text,
  selects a scanner-version and platform partition,
  validates scanner-owned envelope metadata,
  rebuilds the literal matcher from compact groups,
  and decodes optional regex-only engine bytes.
  Scan-time repair and `compile-rules` share the same atomic publisher.
- **Line-based batch scan.** `src/frx_scan.rs` splits each file's bytes into lines once
  and hands the buffer plus line-start offsets through a common matcher interface.
  Runtime sets merge Aho-Corasick and regex-subset ids;
  the built-in baseline retains `RegexSet::line_matches`.
  Each set runs under a `catch_unwind` boundary so a matcher fault fails closed as a synthetic finding.
- **Build-time baseline precompilation.** `build.rs` compiles `data/builtin-rules.txt`
  through the engine once at build time and serializes it (`to_bytes`);
  `lib.rs` embeds the blob with `include_bytes!` and the loader rebuilds it via the validating
  `from_bytes`,
  never recompiling.
  This trusted embedded path remains separate from the mutable runtime cache envelope.
  Cargo compiles build scripts' dependencies unoptimized by default,
  so `Cargo.toml` sets `build-override` `opt-level = 3` for the release and dev profiles,
  and the engine builds the baseline's rules on one worker thread per core
  (see [the Cargo build-override write-up](../../../doc/troubleshooting/cargo-build-override-opt-level.md)).
- **Concurrent load and walk.** Rule loading and `--all` file walking run concurrently via
  `rayon::join` (they share no state);
   files then fan out across the rayon thread pool for
  the parallel scan.
- **`ignore` crate walker + in-process gix-index union.** `--all` uses `ignore::WalkBuilder`
  (honouring `.gitignore`,
   `.git/info/exclude`,
   and global excludes) and unions the result
  with an in-process `gix_index::File` read of `.git/index` so `git add -f` files are still
  discovered.
   See `src/walk.rs`.
- **Bundled `data/betterleaks-default-config.toml`.** Upstream-vendored provenance for the
  betterleaks port;
   the embedded baseline is derived from it,
   and
  `port-betterleaks-relaxations.ts` records the lossy translations applied during the port.

## Script linting

The verification scripts in `bin` are TypeScript.
`mise run //package/forbidden-strings/cli:lint:scripts:types` type-checks them,
and `mise run //package/forbidden-strings/cli:lint:scripts:oxlint -- bin/<file>.ts` lints the files it is given.
The Oxlint task takes explicit paths and has no package-wide form on purpose.
A package-wide `lint:oxlint` run, measured on 2026-10-30, fails with 30 errors and 29 warnings,
all in `src/mise.port-betterleaks.ts`, which the package's `tsconfig.json` includes.
It also lints `task.ts`, which no `tsconfig.json` includes: the semantic bridge reports `project-not-found`
and a type-aware rule throws.
Making the package-wide task usable needs those two files fixed, not a looser rule.
