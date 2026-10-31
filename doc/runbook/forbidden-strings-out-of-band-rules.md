# Forbidden-strings out-of-band rules

How the forbidden-strings deny-list is assembled after the de-root cutover,
 and the maintainer-only and contributor-only steps that keep the sensitive
 rules out of version control.

## Where the rules live

Three layers feed the scanner,
 and only one of them is a checked-in rules file:

- The betterleaks-ported baseline ships inside the `forbidden-strings` binary.
  `package/forbidden-strings/cli/build.rs` precompiles
  `package/forbidden-strings/cli/data/builtin-rules.txt` into a serialized
  `RegexSet` at build time,
  and every scan activates it with the `--builtin-rules` flag.
  There is no baseline file to place on disk.
- The committed shared appendix `forbidden-strings.append.txt` at the
  repository root holds non-sensitive,
   repo-wide rules that every clone and
  CI run share.
- The gitignored per-repo appendix `forbidden-strings.append.local.txt` at the
  repository root holds sensitive literals that must never enter version
  control (codenames,
   customer or partner identifiers,
   politically charged
  strings).

There is no rules file at the repository root anymore.
`.config/mise/conf.d/20-env.toml` points `FORBIDDEN_STRINGS_RULES` at the two appendixes directly,
 by absolute path through `{{config_root}}`,
 with a trailing `?` on the gitignored one so a clone without it still scans;
 `cli-git.config.jsonc` lists the same two files in its `rulesFile` array.
 Nothing is generated from them.
The design decision is recorded in `doc/decision/gitignore-negations.md`,
 and the de-rooting landed in commit `58995afff`.

## Set up a local appendix (contributor)

Create `forbidden-strings.append.local.txt` at the repository root if it is absent
 (nothing scaffolds it; `file-enforcer` only keeps its mode at 0600 once it exists).

Add one rule per line to `forbidden-strings.append.local.txt`;
 the next scan reads it.
The file is already covered by `.gitignore`;
 confirm with `git check-ignore forbidden-strings.append.local.txt`
 before adding anything sensitive.
Non-sensitive shared rules belong in the checked-in
 `forbidden-strings.append.txt` instead,
 so that every clone and CI run inherit them.

## Refresh the CI secret (maintainer)

Continuous integration has no local appendix on disk,
 so its sensitive equivalent is the `FORBIDDEN_STRINGS_LIST` repository secret.
`.github/workflows/forbidden-strings.yml` composes the runtime rules file
 from `forbidden-strings.append.txt` plus that secret under `RUNNER_TEMP`,
 and reads the secret with `printenv` (never shell interpolation) so its
 bytes never reach a logged command line.

The authoritative source for the secret is the maintainer's local appendix.
Push it without displaying its contents:

```sh
gh secret set FORBIDDEN_STRINGS_LIST < forbidden-strings.append.local.txt
```

GitHub secrets are write-only;
 there is no read-back.
Redirecting the file straight into `gh secret set` keeps the rule bytes off
 the terminal and out of shell history arguments.
Refresh the secret whenever the local appendix changes,
 and after any dialect migration that rewrites the local rules.

## Dialect expectations

The scanner runs the restricted `forbidden-regex` dialect,
 not full PCRE.
Both appendixes and the secret must obey it:

- Two shapes per line.
  A bare line is a case-sensitive literal substring.
  A line shaped `/PATTERN/FLAGS` is a regex whose first and last `/` delimit
  the pattern;
  if the trailing flag run is not all ASCII-lowercase the whole line is
  treated as a literal instead.
- `FLAGS` accepts only `m` and `x`,
  which the always-multiline,
   always-verbose engine treats as no-ops and
  drops.
  Any other flag letter (for example `i` or `s`) is a hard,
   fail-closed load
  error;
  the loader rejects the whole ruleset rather than silently change match
  semantics.
  For case-insensitivity,
   spell the alternatives (`[Aa][Bb][Cc]`) or use the
  three-casing form the baseline port uses.
- The loader is strict and fail-closed.
  Unbounded quantifiers (`*`,
   `+`,
   `{n,}`),
   capturing groups,
   lookaround,
  inline-flag groups,
   backreferences,
   and `\xNN` byte escapes are compile
  errors that name only an opaque rule index,
   never the rule text.
  A pattern that can match the empty string is rejected.
- Empty lines,
   whitespace-only lines,
   and lines whose first non-whitespace
  byte is `#` are ignored.

The full construct list is in `package/forbidden-strings/cli/README.md`
 and the engine spec in
 `package/forbidden-strings/core/README.md`.

## Porting existing out-of-band rules to the new dialect

Rules written for the old resharp dialect (trailing `/i` flags,
 inline
 `(?i)` groups,
 unbounded quantifiers) do not load under the strict `0.2.0`
 loader and must be ported once.
The porter that performed the committed-file port,
 `dialectport`,
 lives in the benchmark sidecar at
 `package/forbidden-strings/core.bench/src/bin/dialectport.rs`.
Issue #390 removed it (and its `caseexpand` support module) after the cutover
 settled;
 both were recovered when the baseline adopted the tail rule-file
 format,
 and `dialectport` is now the standing stage two of
 `mise run //package/forbidden-strings/cli:generate:rules`
 (stage one is the TS porter `src/mise.port-betterleaks.ts`,
 which emits the tail-format intermediate the dialect port rewrites).
The per-rule semantics the port applies (quantifier bounding,
 three-casing
 expansion of inline case-insensitivity,
 the two reshapes) are documented in
 `doc/planning/forbidden-strings-rule-port-review.md`.

## Verify

After changing either appendix or the secret:

- Local:
  make a throwaway commit or run `git cli-git check` on a scratch file
  to confirm the appendixes load and the gate still fires on a canary.
- CI:
   the `forbidden-strings` workflow scans changed files on every pull
  request and the full tree on every push to `main`;
  a green run confirms the composed ruleset loaded,
   and its output stays
  redacted to path,
   line,
   and rule index.
