# forbidden-strings 0.3.0: a literal rule body in the appendix matches itself, failing every later commit that touches the file

Note 2026-11-09T16:55:17-05:00:
these citations in this record name paths that no longer exist.
`package/cli/forbidden-strings` is now `package/forbidden-strings/cli`,
moved 2026-10-29 in `113633b259`.
`package/git-policy/cli/src/native/scanner_selection.rs` is now
`package/git-policy/cli/src/native/candidate/scanner_selection.rs`,
moved 2026-11-03 in `b3be9b1641`.

## A note on how this document spells things

Every banned identifier below is written with one character bracketed,
 `Oxc[C]omment` for the identifier
`Oxc` followed by `Comment`.
 That is not a typo and not the identifier's real spelling:
 it is the same
workaround the document describes,
 applied to the document.
 Spelling the identifiers plainly made this file fail
the very rules it is about,
 on three lines at once,
 which is the most direct demonstration of the problem
available.
 Remove the brackets when copying anything from here into a real file or command.

## Symptom

Editing `forbidden-strings.append.txt` fails the commit on a line the edit never touched:

```text
{"schemaVersion":1,"sequence":0,"type":"finding","trigger":"pre-forward",
"policyId":"security/forbidden-strings","severity":"error",
"code":"security/forbidden-strings/forbidden-string",
"message":"Forbidden string matched at line 138 (rule crn-renamed-oxc-comment).",
"path":"forbidden-strings.append.txt","fix":"none"}
```

Line 138 was the body of the `crn-renamed-oxc-comment` rule itself,
 a bare literal.
 The commit being attempted
appended an unrelated rule further down the file.

`git commit --no-enforce-security` does not help;
 no such escape exists for this policy.
 The `--no-enforce-*`
flags `cli-git` accepts are `--no-enforce-only`,
 `--no-enforce-bulk-add`,
 `--no-enforce-require-root`,
`--no-enforce-worktree` and `--no-enforce-worktree-branch`,
 none of which reaches
`security/forbidden-strings`.

## Root cause

The scanner has no notion of a file that is exempt from scanning,
 so it reads its own rule file as ordinary
content.
 Stated in the tool's own README,
 `package/forbidden-strings/cli/README.md:95`:

```md
- **Per-rule path scoping**.
   Every rule runs against every non-skipped file;
   the scanner
  cannot apply rule X only to YAML files.
- **Per-rule allowlists**.
   No way to say "rule X but skip when it matches in path Y".
```

The policy is registered without any path exclusion,
 `cli-git.config.ts:27`:

```ts
    'security/forbidden-strings': [
      'error',
      {
        executable: './package/forbidden-strings/cli/target/release/forbidden-strings',
        builtinRules: true,
      },
    ],
```

So a rule whose body is the bare text `Oxc[C]omment` is satisfied by its own definition.
 Nothing detects this at
authoring time,
 because the policy runs against staged paths:
 the file passes until something stages it,
 and
then it fails for reasons unrelated to the change.

An earlier reading of this was wrong and is worth naming:
 the first assumption was that the newly added rule was
malformed and had broken section parsing for the rules above it.
 It had not.
 The failing rule was years older
than the edit,
 and the added section was last in the file,
 so it could not affect anything above it.
 The tell was
that the reported line number pointed at existing content.

## Verification

Version under test:
 `forbidden-strings 0.3.0`,
 built at
`package/forbidden-strings/cli/target/release/forbidden-strings`.

Harness,
 run against a disposable directory rather than the repository:

```bash
D=$(mktemp --directory "${HOME}/temp/agent/fsprobe.XXXXXXXX")
cat forbidden-strings.append.txt forbidden-strings.append.local.txt > "$D/rules.txt"
printf 'const Oxc[C]omment = 1;\n' > "$D/hit-one.ts"
printf 'const ParsedComment = 3;\n' > "$D/clean.ts"
for f in hit-one clean; do
  printf '%-12s ' "$f"
  ./package/forbidden-strings/cli/target/release/forbidden-strings \
    --rules "$D/rules.txt" "$D/$f.ts" > /dev/null 2>&1 \
    && echo "exit 0 (no violation)" || echo "exit 1 (violation)"
done
printf '%-12s ' "appendix"
./package/forbidden-strings/cli/target/release/forbidden-strings \
  --rules "$D/rules.txt" forbidden-strings.append.txt > /dev/null 2>&1 \
  && echo "exit 0 (no violation)" || echo "exit 1 (violation)"
rm --recursive --force "$D"
```

Patterns that self-match,
 so the appendix fails on its own content,
 shown here unbracketed in description only:

-   a bare literal body,
     the identifier `Oxc[C]omment` written with no brackets at all;
-   a regex body whose pattern text contains the string it matches,
     an alternation of the same identifiers
    written with no brackets.

Patterns that do not self-match,
 while matching the same target text,
 and these are literal:

-   `/Oxc[C]omment/`;
-   `/(?:INERT_MEMBERS_BY_INTERFAC[E])|(?:isInertCollectionMembe[r])/`;
-   any anchored pattern whose anchor cannot hold inside a comment line,
     such as the `^CODE:` shortcode rules,
    whose per-code rationale comments are written as `# INF: ...` and so never match at line start.

After rewriting both self-matching rules,
 the harness prints `exit 1` for each banned identifier,
 `exit 0` for
each replacement identifier,
 and `exit 0` for the appendix itself.

## Verified workarounds

Bracket one character of the pattern so the pattern text no longer matches the pattern:

```diff
 ==> crn-renamed-oxc-comment <==
 # Renamed to ParsedComment when mutation-test moved from oxc-parser to
 # yuku-parser; the old name misattributes the producer of the comment shape.
-Oxc[C]omment      <- was the bare identifier, unbracketed
+/Oxc[C]omment/    <- now a regex, bracketed exactly as shown
```

Tradeoff:
 the rule body no longer reads as the plain string it bans,
 so a reader has to know the convention to
see what is banned.
 That cost is paid once per rule and is why the appendix header now states the convention
above the first affected section.
 It also means a rule can no longer be copied verbatim out of the file as a
search term.

Applies to the checked-in appendix and equally to `forbidden-strings.append.local.txt`,
 whose bare-literal
sections have the same exposure the moment that file is scanned.

## What does not work

-   `git commit --no-enforce-security`.
     Not a flag;
     the policy has no escape hatch.
     Confirmed by enumerating the
    `--no-enforce-*` strings in `package/git-policy/cli/src`.
-   Writing the new rule as a regex while leaving older literal rules alone.
     The trip is on pre-existing content,
    so any edit to the file still fails until every self-matching rule is fixed.
-   Moving the rule to `forbidden-strings.append.local.txt`.
     That file is gitignored,
     so a `CRN` reservation put
    there never reaches other clones,
     and the same self-match applies to it anyway.
-   Relying on the policy to catch this at authoring time.
     It runs against staged paths,
     so an unstaged appendix
    edit looks fine and fails later.

## Upstream filing decision

`forbidden-strings` is a repository-owned package under `package/forbidden-strings/cli`,
 not third-party,
 so the
six-constraint upstream check does not apply and there is nothing to file on an external tracker.
 The
convention-level fix is landed;
 the tool-level fix is recorded below as a change this repository could make to
its own code.

The durable improvement would be for the scanner,
 or the `cli-git` policy wrapping it,
 to skip the resolved rules
file and its appendixes when walking.
 That removes the need for the bracketing convention entirely.
 It is not
attempted here because it changes a security scanner's skip logic,
 which deserves its own decision about whether
a file being a rule source should ever exempt its content:
 a rule file that legitimately contains a real secret
as a pattern is exactly the case the current behaviour catches.

## Update 2026-10-29: a rename or merge of unchanged content is no longer rescanned

The same class of failure refused moving the scanner package itself.
Renaming `package/cli/forbidden-strings` to a new directory re-lists every file under it as an addition at the new path,
and the policy scanned each one again.
Three files match the scanner's own rules by design:

`package/cli/forbidden-strings` moved on 2026-10-29 to `package/forbidden-strings/cli`
in `113633b259` (refactor(*):
move cli packages into product categories).

- `data/builtin-rules.txt`,
  rule `aws-amazon-bedrock-api-key-short-lived` (line 118 at the time),
   whose body is a literal that the rule matches;
- `data/betterleaks-default-config.toml`,
  whose vendored `gcp-api-key` allowlist holds example key values;
- `src/port-betterleaks-relaxations.ts`,
  whose comments hold an example connection string.

They passed at the old paths only because `SCANNER_SELF_MATCH_PATHS` in `src/native/scanner_selection.rs`
names those old paths as unscanned.
At the new paths the exemption no longer applied,
and no `--no-enforce-*` flag reaches this policy.

Fixed in the source of `monochromatic-git-policy-cli` (`src/native/candidate_known_content.rs`):
the policy no longer reads the bytes of a candidate whose exact blob a parent commit's tree already holds.
A pure rename,
a copy of an existing blob,
and a merge that brings a file only one parent has are therefore allowed.
The rule is by blob object name,
not by path or similarity,
so a rename with one changed byte is scanned in full.
`package/git-policy/cli/SPEC.md` ("Content already in history") has the rule and its security reasoning.
The controls are in `src/native/policy_forbidden_strings_known_content_tests.rs`
and `src/native/binary_known_content_tests.rs`;
the rename,
staged-rename and merge controls failed on unchanged code before the rule existed.

What this does not change:

- The fix takes effect only in an installed wrapper built from a release that carries it.
   Releases of this crate are cut by bumping its version on `main`
   (see `doc/runbook/cli-git-global-install.md` for the install side).
- An edit that changes the bytes of one of those files is still scanned.
   After the move,
  `SCANNER_SELF_MATCH_PATHS` must name the new paths before the first edit to any of the three.
- A literal rule body in the appendix still matches itself the moment its bytes change,
  as the sections above describe.
   The workaround there stands.

## Update 2026-11-01: the CI workflow rescanned the renamed rule sources on every push

### Symptom

The `forbidden-strings` workflow failed on every push to `main`
from run 40848310962 (2026-10-29 09:16 UTC) through run 41272422264 (`d2036021e2`,
2026-11-01).
Its "Scan changed files" step printed a notice that is not an error,
then nineteen findings in the same three files and exit 1:

```text
{"type":"forbidden-strings/cache-warning","schemaVersion":1,"reason":"missing","recovery":"compile-from-text"}
package/forbidden-strings/cli/data/betterleaks-default-config.toml:399 rule=aws-amazon-bedrock-api-key-short-lived
package/forbidden-strings/cli/data/betterleaks-default-config.toml:1251 rule=gcp-api-key
  ... one line for each of 1252 to 1266 ...
package/forbidden-strings/cli/data/builtin-rules.txt:118 rule=aws-amazon-bedrock-api-key-short-lived
package/forbidden-strings/cli/src/port-betterleaks-relaxations.ts:141 rule=mongodb-connection-string
```

The cache notice only says the runner has no compiled-rules cache yet,
so the scanner compiled the rules from text.
The same notice appeared in green runs.

### Root cause

The workflow passes the changed files to the scanner positionally.
The scanner skips the three generated rule sources only during an `--all` walk
(`build_skip_set` and `is_walker_skipped`,
applied under `if all` in `src/lib.rs`);
for positional arguments it skipped only the `forbidden-strings.*.txt` files at the working directory
(`is_config_file_at_cwd`).
The commit guard avoids the problem on its own side with `SCANNER_SELF_MATCH_PATHS`
(`package/git-policy/cli/src/native/scanner_selection.rs`),
so the workflow was the one caller with neither.

Why it started on 2026-10-29:
the last green push run was 40844905727 (head `35424427f1`,
08:49 UTC).
`113633b259` (08:55 UTC,
"move cli packages into product categories") renamed the three files.
The workflow lists changed files with `--diff-filter=ACMRT`,
which includes renames,
so the renamed files reached the scanner for the first time since they were added.
The first failing run,
40848310962,
scanned 1052 files and reported exactly these findings.

Why it stayed red:
since `9698f56938` (2026-10-20) a push run scans the delta from the newest commit an earlier successful run validated.
No run has succeeded since `35424427f1`,
so every later push rescans the same growing delta
(10272 files in run 41272422264) and meets the same files again.
An earlier edit of the rule sources,
`c5e57910f5` (2026-08-26),
also failed its run
(32937714181;
its log has expired,
so the cause is unverified),
but the next push passed,
because the push scope then was only the event's own range.

Not the cause:
`FORBIDDEN_STRINGS_RULES` in `.config/mise/conf.d/20-env.toml`,
which local runs read and CI does not (the workflow passes `--rules`).

### Verification

Scanner builds under test:
the unmodified `forbidden-strings 0.5.0` binary of the main checkout
and a debug build of the fixed source tree.

- The unmodified binary on the three files,
  positionally,
  from the repository root:
   the same nineteen findings,
  exit 1.
- The unmodified binary on the 10276 files
   `git diff --name-only --diff-filter=ACMRT 35424427f1 HEAD` lists:
  exit 1 with those findings.
- The fixed binary on the same 10276 files:
  exit 0,
  so nothing else in that delta matches.
- `tests/self_match_positional.rs`:
  three tests fail on unmodified code
   (relative paths,
  absolute paths,
  and a positive control that a real hit still fails beside the skipped sources)
   and pass after the fix.
   Three guard tests pass both before and after:
   a same-named copy under `vendor/` is still scanned,
   a rules file named positionally is still scanned,
  and `--all` still skips the sources.

### Fix

The scanner owns the list,
so the skip moved there.
`build_self_match_set` in `src/lib.rs` returns the canonical paths of the three sources
and applies to walker output and positional arguments alike.
The anchor is the discovered repository root,
so a run started in any subdirectory resolves the same three files;
a standalone run outside any repository resolves them from the working directory.
`build_skip_set` keeps only the rules files and stays `--all`-only,
so a rules file the user names is still scanned.
The match stays path-anchored by canonical path.

Limits:

- The workflow downloads `forbidden-strings-v<version>` for the `Cargo.toml` version
   and falls back to the latest release when that tag is not published.
   The fix reaches CI only in a release that carries it,
   and the watermark advances on the first green run after that.
- The list of three paths now lives in two places,
   `build_self_match_set` and `SCANNER_SELF_MATCH_PATHS`.
   Moving or renaming the package needs both updated in the same change.
- A real secret placed in one of the three files is not reported by CI or by the commit guard.
   That is the same exposure the `--all` walk and the commit guard already had.
