# Forbidden-strings engine-swap cutover differential

Date:
 2026-07-17.

The cutover safety gate for issue #387.
This is a one-shot differential validation of the forbidden-strings engine swap
 (the old resharp / `regex`-crate hybrid at published version `0.1.9`
 versus the new `forbidden-regex` engine at `0.2.0`),
 run over the repository corpus with the committed baseline rule files on each side.
The acceptance bar is zero unexplained lost findings:
 every finding delta must map to a reviewed port semantic change from
 `doc/planning/forbidden-strings-rule-port-review.md`,
 or it is a defect in the rewrite.

This report is redacted.
It carries paths,
 line numbers,
 rule indexes,
 counts,
 and category attributions only,
 never rule text and never matched content bytes.
Rules are referenced by index:
 the 1-based source-line index shared by both committed rule files
 (the review doc's canonical rule identity),
 with the new binary's 0-based compiled index noted where it aids the mapping.

## Result

Zero unexplained lost findings across every corpus.
The engine swap introduces no fail-open regression.

- Lost findings (present old side,
   absent new side):
 zero,
   on all three corpora.
- Gained findings (absent old side,
   present new side):
 six on the full `--all` walk,
 all attributable to the two reviewed rule reshapes
 (four to the rule-518 mongodb credential-core reshape,
 two to the rule-172 curl-anchor-drop reshape).
- The gains are over-matching in the direction the standing lossiness preference accepts;
 they are recorded,
   not treated as defects.

## Binaries under test

- Old side:
 the published `forbidden-strings 0.1.9`,
 installed from crates.io into a disposable temp root
 (`cargo install forbidden-strings --version 0.1.9 --root <mktemp -d>`),
 leaving the repository untouched.
 Its regex engine is resharp;
 its output format is `PATH:LINE:COL_START..COL_END rule=N`.
- New side:
 the live release binary at
 `package/forbidden-strings/cli/target/release/forbidden-strings`
 (self-reports `0.2.0`),
 used as-is,
   never rebuilt.
 Its engine is `forbidden-regex`;
 its output is columnless `PATH:LINE rule=N`.

## Rule-file pinning and precedence

Both runs pin the rules-file precedence so the gitignored local files
 (`forbidden-strings.local.txt`,
`forbidden-strings.append.local.txt`)
 are entirely absent,
 and neither their bytes nor their derived cache
 (`.cache/forbidden-strings.rules.txt`) ever enters either scan.
The gitignored local files were never read,
 printed,
 or quoted during this work.

- The measuring shell has `FORBIDDEN_STRINGS_RULES` pointed at the local cache;
 every scan invocation ran under `env -u FORBIDDEN_STRINGS_RULES -u FORBIDDEN_STRINGS_LIST`
 to remove it.
- Old side ruleset:
 the old in-tree baseline `package/forbidden-strings/cli/data/builtin-rules.txt`,
 passed explicitly with `--rules` (highest precedence,
   and `0.1.9` has no built-in flag).
- New side ruleset:
 the embedded ported baseline via `--builtin-rules`,
 with no user rules file resolving
 (env removed,
   and no `./forbidden-strings.local.txt` exists at the repo root),
 so the baseline alone is the ruleset.

### Pin verification

The env override was confirmed to be the sole load control point,
 and the new-side baseline-only runs were confirmed to carry no user or local rules:

- A one-rule probe fixture,
   pointed at by `FORBIDDEN_STRINGS_RULES`,
 flips a controlled probe file from zero findings to one finding at `rule=0`,
 proving the env override is consulted.
- In the baseline-only run the self-match on the ported rule file's line 111
 reports the new compiled index 22 (baseline starting at 0,
   no offset);
 with a single user rule added via the env override the same finding shifts to index 23.
 The non-offset index in every differential run proves no user rule,
 and therefore no gitignored local rule,
   was loaded.
- Every new-side rule index observed across all runs is at most 258,
 inside the 259-rule baseline range,
   corroborating baseline-only loading.

## Rule numbering and the cross-side mapping

The two sides number rules differently,
 so findings were normalized to a common key of
 `path` and `line` and source-line rule index before comparison.

The scheme was established empirically with a controlled two-rule fixture
 (two bare-literal rules separated by a comment line and a blank line)
 before the mapping was trusted:

- Old `0.1.9` numbers a rule by its 1-based physical source-line number in the rules file.
 The fixture's first rule (physical line 2) reported `rule=2`;
 the second (physical line 5) reported `rule=5`.
- New `0.2.0` numbers a rule by its 0-based compiled index,
 counting only rule lines and skipping comment and blank lines.
 The same two rules reported `rule=0` and `rule=1`.

Both committed rule files are line-aligned:
 the parser confirmed identical rule-line positions across
 `builtin-rules.txt` and `builtin-rules.ported.txt`
 (259 rule lines,
 all regex,
 first at source line 41,
 last at source line 861).
This yields a bijection between the old source-line index and the new compiled index,
 built by parsing the shared rule-line positions.
The mapping was then cross-checked against real self-match findings:

- The ported rule file's line 111 self-matches under old index 111 and new index 22;
 source line 111 is compiled index 22.
- The vendored config self-matches under old index 305 and new index 85;
 source line 305 is compiled index 85.
- The curl reshape is source line 172,
   compiled index 42.
- The mongodb reshape is source line 518,
   compiled index 154.

Every cross-side identity lands on the same underlying rule,
 confirming the normalization is correct on live data.

## Meta and rule-source exclusion

A fixed set of rule-source and vendored files self-matches on rule bodies
 and is handled differently by each binary's self-skip set,
 so those paths are excluded from the port-semantics comparison
 (they are rule text,
 not corpus content):

- `package/forbidden-strings/cli/data/builtin-rules.txt`
- `package/forbidden-strings/cli/data/builtin-rules.ported.txt`
- `package/forbidden-strings/cli/data/betterleaks-default-config.toml`
- `package/forbidden-strings/cli/src/port-betterleaks-relaxations.ts`
- `forbidden-strings.append.txt`
- `forbidden-strings.append.local.txt`
- `forbidden-strings.local.txt`
- `forbidden-strings.local.example.txt`
- `.cache/forbidden-strings.rules.txt`

The new binary skips `betterleaks-default-config.toml` during `--all` by design;
 the old binary scans it.
That divergence is a self-skip tooling difference,
 not a port semantic,
 and the exclusion neutralizes it.
To keep rule coverage high despite the exclusion,
 the vendored config and both rule files were also scanned as an explicit positional set
 (positional arguments bypass the walker self-skip on both binaries),
 giving an apples-to-apples comparison on identical rule-rich input.

## Corpus

- Full `--all` walk of the working tree at `HEAD` `c8eb46466`:
 6407 tracked files,
   74.7 MiB of tracked content.
- Changed-file set:
 every path touched in the last 30 commits that still exists as a regular file,
 102 files,
   passed as identical positional arguments to both binaries.
- Rich rule-exercising set:
 the vendored `betterleaks-default-config.toml` plus both committed rule files,
 passed positionally to both binaries;
 this is the densest secret-shaped content in the repo
 and is where a lost finding across the broad rule set would surface.

## Finding counts per side

Raw counts are before the meta and rule-source exclusion;
 corpus counts are after it.

- Full `--all`:
 old raw 18 (all 18 in excluded meta files),
 new raw 7 (1 in an excluded meta file,
   6 corpus).
- Changed-file set:
 old raw 1 (excluded meta file),
 new raw 3 (1 excluded meta file,
   2 corpus).
- Rich rule-exercising set:
 old raw 19,
 new raw 19;
 after normalization the two finding sets are byte-for-byte identical
 (zero lost,
   zero gained),
 exercising source-line rules 111 and 305 with no delta.

## Delta summary by attribution category

Full `--all` walk (corpus,
 meta excluded):

- Lost:
 zero.
- Gained:
 six.
 Four map to `reshape-518-mongodb` (source-line rule 518,
   compiled index 154).
 Two map to `reshape-172-curl` (source-line rule 172,
   compiled index 42;
 rule 172 also carries the quantifier-bound,
  `\z`-to-`$`,
   and CR/LF-class-drop
 categories,
   all narrowing or neutral,
   so the gain direction is the curl reshape).

Changed-file set (corpus,
 meta excluded):

- Lost:
 zero.
- Gained:
 two,
   both `reshape-518-mongodb`
 (a subset of the `--all` gains,
   since the changed-file set contains the review doc
 but not the other gained-finding paths).

Rich rule-exercising set:

- Lost:
 zero.
- Gained:
 zero.

No delta in any corpus falls outside a reviewed category.
The quantifier-bound (15 rules),
 leading-repeat-strip (117 rules),
 three-casing inline-`(?i)` expansion (172 rules),
`\z`-to-`$` (4 rules),
 CR/LF-class-drop (2 rules),
 and `m`-flag-drop categories produced no observed
 finding delta on this corpus:
 the repository content does not exercise the inputs those adaptations touch,
 and none of them dropped a finding the corpus contains.

## Gained-finding attribution detail

Each gained finding,
 by path,
 line,
 source-line rule index,
 and category:

- `doc/planning/forbidden-strings-rule-port-review.md:172`,
 rule 518,
  `reshape-518-mongodb`.
- `doc/planning/forbidden-strings-rule-port-review.md:173`,
 rule 518,
  `reshape-518-mongodb`.
- `doc/planning/forbidden-strings-rule-518-pi-advice.md:23`,
 rule 518,
  `reshape-518-mongodb`.
- `doc/planning/forbidden-strings-rule-518-pi-advice.md:24`,
 rule 518,
  `reshape-518-mongodb`.
- `package/android-exempt-unused/cli/README.md:54`,
 rule 172,
  `reshape-172-curl`.
- `doc/troubleshooting/mise-rust-components.md:538`,
 rule 172,
  `reshape-172-curl`.

The rule-518 gains are the interpolated-host and concatenated-host connection-string
 examples that the review doc and the rule-518 advice doc contain verbatim as documentation.
The reshaped rule stops validating at the credential-terminating `@`,
 so it flags these connection strings the old full-URI rule skipped;
 this is the exact broadening the review doc records
 (interpolated hosts now covered).

The rule-172 gains were a documentation path and a container-run command
 that carried a `-u value:value` option shape without the word `curl` on the line.
The cutover reshape dropped the leading `\bcurl\b` context and continuation window,
 so it flagged the credential-pair shape on any single line.
Both gained lines matched that broadened form and neither matched the old,
 curl-anchored,
 continuation-windowed rule.
The cutover accepted those gains and the standing preference originally ratified them.

The later [curl authentication option boundary decision][curl-auth-user-boundary]
 partially supersedes that acceptance.
A separate `-u` or `--user` token may still match without `curl`,
 preserving continuation-line detection.
A `-u` substring embedded inside a larger word or path no longer matches;
 the port requires line start or horizontal whitespace before the option.

## Acceptance

- Zero unexplained lost findings:
 met.
 No corpus produced a lost finding,
   so no fail-open regression exists,
 and the deliberate mixed-case narrowing of the three-casing expansion
 was not even reached by this corpus.
- Every gained finding mapped to a reviewed port change:
 met.
 All eight distinct gains (six on `--all`,
   two of them repeated in the changed set)
 attribute to the rule-518 or rule-172 reshape.

## Reproduce

From the repository root,
 with `OLD` the installed `0.1.9` binary and
 `NEW` the live release binary:

```sh
# old side: old baseline via --rules, columns present, rule=source-line
env -u FORBIDDEN_STRINGS_RULES -u FORBIDDEN_STRINGS_LIST \
  "$OLD" --rules package/forbidden-strings/cli/data/builtin-rules.txt --all

# new side: embedded ported baseline, columnless, rule=compiled-index
env -u FORBIDDEN_STRINGS_RULES -u FORBIDDEN_STRINGS_LIST \
  "$NEW" --builtin-rules --all
```

Normalize both to `path` and `line` and source-line rule index
 (strip the old column span;
 map the new compiled index to its source line
 through the shared rule-line positions of the two committed rule files),
 then take the set difference.
The old binary rejects the `--` argument separator;
 pass positional files without it.

[curl-auth-user-boundary]: ../decision/forbidden-strings-curl-auth-user-boundary.md
