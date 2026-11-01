# Gating corpus text in commits with forbidden-strings

Asked by the owner on 2026-08-25, after `doc/audit/corpus-text-reached-a-public-repository.md`:
can `forbidden-strings` help reduce corpus exposure in commits?

Yes. The owner's design is the right one:
take the whole corpus, split it into sentences, and make every sentence a literal rule.
Two refinements came out of measuring it, and one blocker was found in the scanner.

## What the shape-based rule cannot do

The first instinct, matching Chinese characters, fails three independent ways.

It does not compile.
The rule dialect has no `\x{...}` escape, so `/[\x{4E00}-\x{9FFF}]{12,24}/` is rejected at load:

```text
forbidden-strings: rules cjkrule.txt: rule 0: syntax error at byte 1: unsupported escape \x
```

Run length does not separate the populations.
Corpus-bearing documents carry longest Chinese runs from 24 characters down to 2,
while files with nothing to do with the corpus reach 15.
Around 180 files under `package/translation-repair/*/src/` sit at 8 to 16,
and every one is invented: the fixtures are cat-themed by house rule.
Any threshold that catches a memorial passage fires on most of the test suite.

The scanner cannot be told to spare them.
Its README says so under "When to pick something else":
every rule runs against every non-skipped file, and there are no per-rule allowlists.

## What the literal deny-list does

Prototyped, then built from the pinned corpus at `a41fc607ea5a70d8a7625cc67d5ed8c444f53379`.

The corpus holds 92 person entries, 276 page files across three language variants,
and 2552 comment files.
Split on sentence boundaries and de-duplicated, that is 20197 distinct strings,
of which 10214 are at least 24 characters long.

### Subtract what the repository already has

A raw sentence list fires on our own source, and the reason is instructive.
The corpus repeats structural labels across dozens of entries,
and a parser has to recognise those labels to do its job,
so strings like `Contributor for this entry:` and `<summary>Original</summary>`
live legitimately in `house-policy.ts`, `apply-patch.ts` and several test files.

Counting how many entries a string appears in does NOT separate them,
which was measured and refuted:
after cleaning, one entry's copy of a label can be a whole line on its own,
so the label counts as unique-to-one-entry while being template in 25 others.

Subtracting every sentence the repository already contains does separate them,
and it is simpler than the count that failed.
Of 10214 candidates, exactly 8 appear anywhere in the 7981 tracked files.
The deny-list is the other 10206.

That subtraction carries the whole policy:

-   Corpus boilerplate our parser needs is already here, so it is subtracted out.
    Zero false positives on today's tree, by construction rather than by tuning.
-   The 185 lines the owner decided stay are already here too,
    so the guard never re-litigates them and never teaches its own bypass.
-   What remains fires exactly when corpus text that is not already here enters a commit.

Untracked run logs and built output are excluded from the subtraction on purpose.
They carry far more corpus text, around 50 files under `node_modules`,
but they are never committed, so what they hold must not shrink the deny-list.

### Where the rules live

In `forbidden-strings.append.local.txt`, which is gitignored,
so the corpus text in the rules is never itself committed.
Findings print `PATH:LINE rule=<token>` and never the matched substring,
the surrounding line, or the pattern,
which is what makes a deny-list whose bodies are the sensitive thing workable at all.

Rules are per worktree, not per repository.
`mise.toml` sets `FORBIDDEN_STRINGS_RULES` to `{{config_root}}/.cache/forbidden-strings.rules.txt`,
and `{{config_root}}` is the worktree root.
The two live worktrees already carry different generated rules files,
10652 bytes against 12635, so this is observed rather than assumed.

The commit hook scans only what a commit touches.
`package/git-policy/forbidden-strings/src/index.ts` records that
"every lifecycle now supplies only the operation's own delta,
so no post-commit narrowing happens here".
The standalone scanner walks the working tree; the policy adapter does not.

## The blocker: user rules are recompiled on every invocation

Compile cost is linear in rule count, measured on one unchanged file:

```text
    50 rules    0.21s        1000 rules    8.96s
   100 rules    0.25s        2000 rules   22.75s
   250 rules    1.20s        4000 rules   41.54s
   500 rules    2.88s        8000 rules   97.45s
```

About 12 milliseconds per rule, so a 10206-rule deny-list costs roughly two minutes.

That is fine if it is paid once per rule change, and the engine can do that:
`load_precompiled` rebuilds a serialized `RegexSet` without recompiling,
and `build.rs` already uses it,
compiling the ported baseline once at build time into `OUT_DIR/builtin-rules-precompiled.bin`.

BUT THE PRECOMPILED SLOT IS NOT REACHABLE FOR USER RULES.
`package/forbidden-strings/cli/src/frx_load.rs` states the split in its own module note:
the runtime rules file is "compiled from text at startup via `compile_from_text`",
while the builtin baseline is "embedded as a precompiled serialized `RegexSet`
and rebuilt via `load_precompiled` (never recompiled), active only under `--builtin-rules`".

The CLI exposes `--rules`, `--all` and `--builtin-rules`, and nothing that supplies
or persists precompiled bytes for the appendix.
So a large appendix costs its full compile on every commit, not once per rule change.

This is a wiring gap rather than a documentation gap.

## Decision

Adopt the literal deny-list, built as described, and DO NOT INSTALL IT YET.

Filed as `#456`, and the deny-list waits for that fix rather than taxing every commit.
Waiting costs nothing here: the owner decided separately that corpus text in commits
is not a blocker at all, and that sanitization happens once at the end
under temporarily disabled branch protection.
A guard that charges two minutes per commit to prevent something already scheduled
for a single sweep is the wrong trade.

The natural fix is to compile the appendix once where it is already generated:
`file-enforcer` writes `.cache/forbidden-strings.rules.txt` when rules change,
so it could write `.cache/forbidden-strings.rules.bin` beside it,
and the scanner could prefer the serialized set when it is newer than the text.
That turns two minutes per commit into two minutes per rule change,
which is what the design assumed all along.
Filed rather than built: it is a change to `package/forbidden-strings/cli`,
not to the package this work owns.
