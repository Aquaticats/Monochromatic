# Forbidden-strings rule port review

Date:
 2026-07-17.

Companion to `doc/planning/forbidden-strings-engine-migration.md`,
 covering step 4 of its rollout sequence
 (port the committed rule files,
 review the semantic diff).

This document classifies every change the one-time port made when rewriting the two
committed forbidden-strings rule files into the `forbidden-regex` dialect.
The ported files are staged beside the originals,
 not yet loaded by the scanner:

- `package/forbidden-strings/cli/data/builtin-rules.ported.txt`
  (from `forbidden_strings::BUILTIN_RULES`,
   embedded from `data/builtin-rules.txt`).
- `forbidden-strings.append.ported.txt`
  (from the repo-root `forbidden-strings.append.txt`).

The gitignored local rule files (`forbidden-strings.local.txt`,
 `forbidden-strings.append.local.txt`) are out of scope;
 they are covered by the cutover runbook and are never read here.

## How to reproduce

The port is scripted by a temporary bin in the benchmark sidecar:

```sh
# from the repo root
mise run //package/forbidden-strings/core.bench:build
cargo run --release --manifest-path package/forbidden-strings/core.bench/Cargo.toml --bin dialectport
```

The bin reads the two committed sources,
 rewrites each `/PATTERN/FLAGS` line,
 writes the two
`.ported.txt` files,
 prints a per-rule change report,
 then strict-compiles every ported rule
through `forbidden_regex::RegexSet::new` (never `compile_lenient`) and exits nonzero listing
the source line of any rule that fails.
It is removed after the migration cutover.

The three-casing expansion lives in the sidecar module `src/caseexpand.rs`,
 applied inside
`dialect_body` before the shared normalizer runs.

## Summary of the diff

The builtin file has 861 lines,
 of which 259 are regex rules;
 the committed append file has 18 lines,
 of which 2 are regex rules.
Literal,
 comment,
 and blank lines pass through byte-identically,
 and every rule keeps its original 1-based line number
 (load-bearing for the later differential validation).

Counts,
 measured by the port bin:

- 235 of the 259 builtin regex rules had their pattern text rewritten;
  the remaining 24 differ only by structural regrouping or a dropped flag.
- 16 builtin rules changed match semantics
  (15 had an unbounded quantifier bounded to the cap;
  2 were reshaped,
   rule 172 also being one of the 15).
- 172 builtin rules had their inline case-insensitivity three-casing-expanded,
  an approximately-preserving change (see below),
   no longer counted as a semantic change.
- The 2 append rules changed only by dropping the `m` flag,
  which is a no-op under the always-multiline engine.

The 16 semantic changes fall into two groups
 (rule 172 belongs to both):

- 15 rules had an unbounded quantifier bounded to the cap of 512.
- 2 rules (curl and mongodb) were reshaped.

Everything else the port did preserves or approximately preserves the matched-input set;
 those adaptations are listed under "Approximately-preserving adaptations"
 and "Semantics-preserving adaptations".

## Semantic changes

For any listed line number,
 the exact before and after are the line-aligned pair
 `sed -n 'Np' data/builtin-rules.txt` and `sed -n 'Np' data/builtin-rules.ported.txt`;
 representative and high-value pairs are shown inline below.

### Quantifier bounding to the cap of 512 (15 rules)

The engine rejects unbounded repetition.
Per the settled decision,
 `*` becomes `{0,512}`,
 `+` becomes `{1,512}`,
 and `{n,}` becomes `{n,512}`.
A secret longer than the cap no longer matches,
 which for these rules means an unusually long base64 blob,
 continuation gap,
 or value.

The 15 affected rules and their unbounded operator:

- 44:
  `[a-zA-Z0-9+/]{250,}` becomes `{250,512}`.
- 172:
   reshaped curl rule (next section);
   its kept value alternation bounds several `{3,}`.
- 255:
  `[a-z0-9]{100,}` becomes `{100,512}`.
- 291:
  `[A-Za-z0-9=_\-,/+]{100,}` becomes `{100,512}`.
- 299:
   two `\s*` become `\s{0,512}`.
- 437:
  `ey[...]{17,}` twice becomes `{17,512}`,
   and `[...]{10,}` becomes `{10,512}`.
- 440:
  `[...]{40,}` becomes `{40,512}`.
- 506:
  `[a-z0-9]+` becomes `{1,512}`.
- 544:
   two `\s*` become `\s{0,512}`,
   and `.{8,}` becomes `.{8,512}`.
- 621:
  `[\s\S-]{64,}` becomes `{64,512}`.
- 699:
  `[A-Z0-9]+`,
  `\d+`,
   and `[a-z0-9]+` become `{1,512}`.
- 702:
  `[a-zA-Z0-9-]*` becomes `{0,512}`.
- 714:
   three `\d+` and one `[a-fA-F\d]+` become `{1,512}`.
- 720:
  `[\w\/\\+-]{100,}` becomes `{100,512}`.
- 828:
  `[A-Z0-9a-z_-]+` becomes `{1,512}`.

Rule 518 previously appeared here;
 its reshape now drops those repeats rather than bounding them,
 so it is no longer a quantifier-bounding change.

### Rule 172 reshape (the curl basic-auth rule)

Rule 172 is the only cross-line rule.
The cutover decision dropped the leading `\bcurl\b` context and the up-to-five-line
continuation window entirely,
 keeping the `(?:-u|--user)` option,
 its separator,
 and the `user:pass` value alternation on a single line.
The credential pair is the payload;
 `curl` is context,
 and the continuation window cannot survive the line-at-a-time model.
The later [curl authentication option boundary decision][curl-auth-user-boundary]
 partially supersedes that reshape:
line start or horizontal whitespace must now precede the kept option.
Within the kept tail,
 the value alternation's `{3,}` bounds become `{3,512}`,
 the capturing groups become non-capturing,
 and the trailing `\z` becomes `$`.

```text
before:  /\bcurl\b(?:.*|.*(?:[\r\n]{1,2}.*){1,5})[ \t\n\r](?:-u|--user)(?:=|[ \t]{0,5})("(: ... )|'( ... )'|( ... ))(?:\s|\z)/
cutover: /(?:(?:-u)|(?:--user))(?:=|[ \t]{0,5})(?: ... {3,512} ... )(?:\s|$)/
current: /(?:^|[ \t])(?:(?:-u)|(?:--user))(?:=|[ \t]{0,5})(?: ... {3,512} ... )(?:\s|$)/
```

The full current form is line 179 of
`package/forbidden-strings/cli/data/builtin-rules.txt`.
Compared with the original,
 the reshape broadens the rule
 (it flags a separately tokenized `-u user:pass` credential on any single line,
 without requiring `curl`)
 and drops cross-line correlation with the curl command.
This preserves detection on an option-bearing continuation line,
while rejecting option-like substrings embedded inside larger words or paths.

### Rule 518 reshape (the mongodb connection-string rule)

Rule 518 is reshaped to the credential-bearing core only:

```text
before: /\b(mongodb(?:\+srv)?://(?:[!-9;-~]{3,50}):(?:[!-?A-~]{3,88})@(?:(?:[a-zA-Z0-9][\w.-]+| ... host, port, replica-set, path ... ))(?:['"\s;\x60]|\\[nr]|$)/
after:  /\bmongodb(?:\+srv)?://[!-9;-~]{3,50}:[!-?A-~]{3,88}@/
```

The kept core is the mongodb scheme,
 the bounded user-information phase,
 the bounded password phase,
 and the `@` delimiter that ends the credential.
Everything after `@` (the host-or-IP alternation,
 optional port,
 comma-separated replica-set
 list,
 path,
 and trailing delimiter group) is dropped.

Adopted rationale
 (pi advisor B,
 aligned with the standing over-matching preference):

> Preserve the original credential payload,
>  but omit non-secret URI suffix validation to avoid
> determinization blow-up and to cover valid,
>  templated,
>  and partially constructed connection
> strings.

The reshape is the fix for the one rule that previously failed strict compile.
The original nested host-or-IP alternation with several wide-range repeats
 (IPv4 octets,
 optional ports,
 and a comma-separated replica-set list)
 determinized past the engine's DFA state cap of 20000.
The kept core is deterministic because the user-information class excludes `:`
 and the password class excludes `@`,
 so the two phases and the delimiter never overlap.
The broadening also covers connection strings a complete-URI rule would miss,
 such as an interpolated host `mongodb://alice:pw@${MONGO_HOST}`
 or a concatenated one `mongodb://alice:pw@" + mongoHost`,
 because it stops validating at the `@`
 (the two-byte example password sits below the rule's three-byte minimum,
 so this review document does not itself match the rule it documents;
 a realistic password makes both shapes match).

The full pi answer is preserved at `doc/planning/forbidden-strings-rule-518-pi-advice.md`.

## Approximately-preserving adaptations

These changes narrow the matched-input set only in the over-specific direction the corpus never
 exercises,
 so they are recorded here rather than among the semantic changes;
 the later differential validation should see at most the deliberate mixed-case narrowing.

### Inline case-flag three-casing expansion (172 rules)

The engine has no case-insensitivity:
 inline `(?i)`,
`(?i:...)`,
 and `(?-i:...)` are rejected at compile time,
 and there is no flags slot to carry `i`.
Rather than strip case-insensitivity (which would narrow each rule to the exact case written),
 the port transforms each case-insensitive span into case-sensitive dialect that covers the
 three shapes people actually write:

- A keyword literal run under `(?i)` scope expands to a non-capturing three-casing alternation:
  lowercase,
   per-run-Capitalized,
   and UPPERCASE.
  A single-token keyword capitalizes as a whole (`adobe` yields `adobe|Adobe|ADOBE`);
  a multi-run token capitalizes each alphabetic run
  (`api_key` yields `api_key|Api_Key|API_KEY`,
  `x-figma-token` yields
  `x-figma-token|X-Figma-Token|X-FIGMA-TOKEN`).
- A character class under `(?i)` scope widens each letter range or single to both cases
  (`[a-z]` becomes `[a-zA-Z]`,
  `[a-f0-9]` becomes `[a-fA-F0-9]`,
   and the trailer's `[nr]`
  becomes `[nNrR]` under a whole-pattern `(?i)`).
- A single quantified letter under `(?i)` scope widens to a two-case class
  (`A{22}` becomes `[aA]{22}`,
   the `s?` of `https?` becomes `[sS]?`).

Mixed-case forms like `AdOBe_` are deliberately unmatched:
 the three-casing alternation matches only the three consistent shapes,
 not the exponential mixed-case set a per-character both-case expansion would.
This is the decided policy and the sole approximating narrowing;
 it is over-specific only against inputs no real credential uses.

Scope tracking follows PCRE.
A leading `(?i)` folds the whole pattern,
 so its value classes and trailer widen;
 a mid-group `(?i)` (as in `\b(p8e-(?i)[a-z0-9]{32})...`) folds only to the end of its group,
 so the `[nr]` in the trailing context stays case-sensitive;
 a `(?-i:...)` island (as in `(?-i:ETSY|[Ee]tsy)`) stays case-sensitive inside a folded scope.

Example,
 line 50 (`adobe`):

```text
before: /(?i)[\w.-]{0,50}(?:adobe) ... ([a-f0-9]{32})(?:\\?['"\x60]|[\s;]|\\[nr]|$)/
after:  /(?:(?:(?:adobe)|(?:Adobe)|(?:ADOBE))) ... (?:[a-fA-F0-9]{32})(?:(?:\\?['"`])|[\s;]|(?:\\[nNrR])|$)/
```

Example,
 line 142 (a bare keyword rule):

```text
before: /(?i)CLOJARS\_[a-z0-9]{60}/
after:  /(?:(?:clojars_)|(?:Clojars_)|(?:CLOJARS_))[a-zA-Z0-9]{60}/
```

Example,
 line 249 (`etsy`),
 where the inner `(?-i:ETSY|[Ee]tsy)` case-sensitive island stays case-sensitive:

```text
before: /(?i)[\w.-]{0,50}(?:(?-i:ETSY|[Ee]tsy)) ... ([a-z0-9]{24}) .../
after:  /(?:(?:(?:ETSY)|(?:[Ee]tsy))) ... (?:[a-zA-Z0-9]{24}) .../
```

The 172 affected source lines are:

47,
 50,
 53,
 59,
 65,
 68,
 71,
 86,
 89,
 92,
 99,
 118,
 121,
 124,
 127,
 130,
 133,
 136,
 142,
 145,
148,
 154,
 157,
 160,
 163,
 166,
 169,
 175,
 181,
 184,
 187,
 190,
 199,
 202,
 205,
 208,
 211,
 214,
217,
 220,
 223,
 226,
 229,
 233,
 237,
 240,
 243,
 246,
 249,
 252,
 255,
 258,
 261,
 264,
 267,
 270,
273,
 276,
 279,
 282,
 285,
 288,
 294,
 299,
 302,
 308,
 375,
 378,
 381,
 384,
 387,
 390,
 393,
 399,
404,
 407,
 413,
 416,
 419,
 425,
 428,
 431,
 434,
 443,
 446,
 449,
 452,
 455,
 458,
 461,
 464,
 467,
470,
 473,
 476,
 479,
 482,
 485,
 488,
 491,
 494,
 500,
 503,
 509,
 521,
 524,
 527,
 530,
 533,
 539,
544,
 547,
 550,
 556,
 559,
 566,
 574,
 581,
 584,
 587,
 592,
 598,
 603,
 606,
 609,
 612,
 615,
 621,
624,
 633,
 639,
 648,
 651,
 654,
 657,
 660,
 693,
 696,
 699,
 705,
 708,
 732,
 737,
 741,
 747,
 750,
756,
 759,
 762,
 765,
 768,
 774,
 777,
 780,
 783,
 786,
 789,
 792,
 798,
 801,
 804,
 807,
 810,
 813,
816,
 819,
 822,
 825,
 828,
 831,
 834,
 837.

Six of these also bound an unbounded quantifier
 (255,
 299,
 544,
 621,
 699,
 828),
 which is the quantifier-bounding semantic change covered above.
Four of them (136,
 556,
 592,
 756) produce byte-identical output to the earlier case-stripped
 port,
 because their case-insensitive scope holds no letter to three-case and no letter range to
 widen:
 the keyword sits inside a `(?-i:...)` island or before a mid-group flag,
 and the value class lies outside the folded scope.

### Leading redundant class-repeat stripping (117 rules)

The betterleaks generic detectors prepend a nullable identifier context repeat,
 `[\w.-]{0,50}`,
 before the keyword.
Under the engine's unanchored line search a nullable leading repeat always matches the empty
string,
 so it never adds a constraint:
 a line matches `[\w.-]{0,50}(?:keyword)...` exactly when it matches `(?:keyword)...`.
The port strips it
 (including a second copy nested just inside a leading group,
 as in rules 136 and 157),
 leaving a leading `\b` or `^` anchor intact because those are real constraints.

This strip is required for the port to be usable:
 the strict compile surfaced a determinization blow-up on the un-stripped shape.
The whole-line matcher for `[\w.-]{0,50}(?:adafruit)...`,
 whose leading repeat overlaps the keyword's own character set,
 took roughly two minutes to compile for a single rule
 (measured 123 seconds for line 47,
 55 seconds for line 50);
 after stripping the redundant repeat the same rule compiles in about 100 milliseconds.

The 117 affected source lines are:

47,
 50,
 59,
 65,
 71,
 86,
 89,
 92,
 118,
 121,
 124,
 127,
 130,
 136,
 145,
 148,
 154,
 157,
 160,
 163,
166,
 169,
 175,
 181,
 184,
 187,
 190,
 202,
 205,
 208,
 214,
 217,
 220,
 223,
 240,
 243,
 246,
 249,
258,
 261,
 264,
 270,
 273,
 276,
 279,
 302,
 308,
 375,
 378,
 390,
 404,
 407,
 413,
 425,
 431,
 434,
443,
 446,
 449,
 452,
 458,
 461,
 464,
 467,
 470,
 473,
 476,
 479,
 482,
 485,
 488,
 491,
 494,
 500,
503,
 509,
 521,
 524,
 527,
 530,
 533,
 550,
 556,
 559,
 574,
 581,
 584,
 587,
 603,
 606,
 624,
 633,
648,
 651,
 660,
 693,
 732,
 737,
 747,
 750,
 756,
 759,
 762,
 768,
 774,
 777,
 780,
 783,
 786,
 789,
792,
 804,
 819,
 828,
 831,
 834,
 837.

## Semantics-preserving adaptations

These changes rewrite the pattern text but do not change which single scanned line matches.
They are recorded here for completeness;
 they are not counted among the semantic changes,
 and the later differential validation should see no finding delta from them.

### Carriage-return and line-feed class members dropped (2 rules)

Rules 172 and 440 list `\r` and `\n` as members of a character class,
 for the multi-line base64 continuation case.
The engine rejects `\r`/`\n` escapes,
 and a single scanned line never contains a newline byte
 (the scanner splits on `\n` and strips a trailing `\r`),
 so the port drops those two members.
For rule 440 the surrounding class keeps its literal backslash and every other member;
 the drop leaves an inert character out of a class that a line can never exercise.

### `\z` rewritten to `$` (4 rules)

Rules 172,
 569,
 663,
 and 720 end an alternation with `\z` (absolute end of input).
The engine has no `\z`;
 the port rewrites it to `$`,
 which under the always-multiline engine matches at end of line.
For a single scanned line the two anchor positions coincide,
 so the match set is unchanged.

### `m` flag dropped (2 append rules)

The two committed append rules (source lines 17 and 18) carry the `m` flag.
Multiline is always on in the engine,
 so the flag is a no-op and the port drops it;
 the pattern text is otherwise byte-identical.

### Always-applied dialect normalizations

Every regex rule also passed through these text rewrites,
 none of which change the matched-input set:

- POSIX class spellings expand to byte classes
  (`[[:alnum:]]` to `[A-Za-z0-9]`;
   the one occurrence is line 518,
   now dropped by the reshape).
- Capturing groups `(...)` become non-capturing `(?:...)`;
  the scanner reports per-line rule indices,
   not captures.
- An unnecessary escape of a non-metacharacter loses its backslash
  (`\"` becomes `"`,
   four times on line 544),
  and `\_` and `\x60` become the literal `_` and backtick they already denoted.
- An unescaped space or tab outside a class is escaped (`\ ` or `\t`),
  because verbose mode swallows bare whitespace
  (lines 544 and 621 carry such spaces).
- Alternation and intersection operands are wrapped in `(?:...)` where the dialect requires a
  single atom;
  already-atomic operands are left unwrapped.
  The four set-algebra rules (105,
   321,
   849,
   861) keep their `&` and `~(...)` operators,
  with each operand wrapped to a single atom.

## Compile-time finding

The strict compile answers the migration plan's open question about compile cost.
Even after the leading-repeat strip,
 the faithful full-context rules are individually expensive to determinize:
 line 440 (a base64 JWT rule) is the worst,
 followed by the private-key block (621) and the two poly rules (603,
 606),
 with a few dozen rules exceeding one second.
These costs come from large bounded repetitions over wide character sets
 (for example `{70,400}`,
`{109,269}`,
 and the cap-inflated `{n,512}` forms),
 and the three-casing expansion adds only small keyword alternations on top.

Compiling the whole ruleset at scanner startup is therefore not viable against the README's
sub-second budgets.
The migration should embed a precompiled serialized `RegexSet`
 (`to_bytes`/`from_bytes`) built once at build time,
 rather than compiling `include_str!` text at each startup;
 this resolves the first open implementation question in the migration plan.

## Verification result

The port bin strict-compiles all 261 ported regex rules
 (259 builtin,
 2 append)
 through `forbidden_regex::RegexSet::new`,
 fanned out across the available cores.
All 261 rules compile;
 the mongodb rule 518 that previously failed the DFA state cap now compiles as the reshaped
 credential core,
 so the bin exits zero.
No rule trips `EmptyMatchable`,
 and no rule is silently dropped.

169 builtin rules changed bytes from the previous ported output
 (168 of the 172 case rules,
 now three-cased instead of case-stripped,
 plus rule 518);
 the append ported file is byte-identical to the previous output.
Line alignment with `builtin-rules.txt` is preserved at 861 lines.

[curl-auth-user-boundary]: ../decision/forbidden-strings-curl-auth-user-boundary.md
