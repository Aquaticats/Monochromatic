# forbidden-strings and corpus exposure: literal names yes, character shape no

Asked by the owner on 2026-08-25, after `doc/audit/corpus-text-reached-a-public-repository.md`:
can `forbidden-strings` help reduce corpus exposure in commits?

Yes, in exactly one shape:
a worktree-local deny-list of literal corpus person names.
The obvious rule, matching Chinese characters, is not merely noisy.
It does not compile, and it would be wrong if it did.

## What was measured, in the order it settled the question

### A character-range rule does not compile

The rule dialect has no `\x{...}` escape.
Loading `/[\x{4E00}-\x{9FFF}]{12,24}/` fails:

```text
forbidden-strings: rules cjkrule.txt: rule 0: syntax error at byte 1: unsupported escape \x
```

That alone ends the character-shape approach.
The dialect is deliberately restricted, fail-closed,
and rejects `*`, `+`, unbounded `{n,}`, capturing groups, lookaround and backreferences,
which is what buys its linear-time guarantee.

### Even if it compiled, run length does not separate corpus from anything else

Longest run of consecutive Chinese characters per tracked file, measured across the repo.
Corpus-bearing documents run from 24 down to 2.
Files with nothing to do with the corpus reach 15.
The two populations overlap through almost their whole range,
so no threshold divides them.

Worse, about 180 files under `package/translation-repair/*/src/` carry runs of 8 to 16,
and every one of them is INVENTED.
The fixtures are cat-themed by house rule, sunbathing kittens and chased butterflies,
so a threshold low enough to catch a memorial passage
fires on most of the package's test suite, which is edited constantly.

### The scanner has no path scoping and no allowlists

Its own README says so under "When to pick something else":
every rule runs against every non-skipped file,
and there is no way to say "rule X but skip when it matches in path Y".
`package/forbidden-strings/cli/src/mise.port-betterleaks.ts` records the same thing
from the porting side: upstream `path = '''...'''` scopes are dropped on conversion.

So a rule cannot be told to spare the fixtures.

### Rules ARE worktree-specific, which is the part that makes anything workable

`mise.toml` sets `FORBIDDEN_STRINGS_RULES = "{{config_root}}/.cache/forbidden-strings.rules.txt"`,
and `{{config_root}}` is the worktree root, not the repository.
The two live worktrees already carry different generated rules files,
10652 bytes here against 12635 in the main worktree,
so this is observed rather than inferred.

A rule can therefore exist where corpus work happens and nowhere else.

### The commit hook scans only what the commit touches

`package/git-policy/forbidden-strings/src/index.ts` takes lifecycle-selected candidates,
and its comment records that "every lifecycle now supplies only the operation's own delta,
so no post-commit narrowing happens here".
An untouched file is not re-litigated.
The standalone scanner walks the working tree, but the policy adapter does not.

### Literal rules discriminate perfectly, and their findings cannot leak

Prototyped on a throwaway repository with two named literal rules,
one Latin and one Chinese, against four candidate files.

Positive, both rules firing:

```text
corpus-doc.md:1 rule=corpus-person-name
corpus-doc-zh.md:1 rule=corpus-person-name-zh
exit=1
```

Negative, proven clean rather than merely quiet:
the SAME loaded rules against a cat fixture and a technical Chinese document
exit 0, so the probe was shown able to produce both outcomes.

The finding prints the rule's name and the line number.
It never prints the matched substring, the surrounding line, or the pattern,
which is why a rule body that would itself be sensitive can live in the deny-list at all.

## Decision

Use a deny-list of literal corpus person names,
in the gitignored `forbidden-strings.append.local.txt` of the worktree where corpus work happens.
Never a rule keyed on character ranges, run lengths, or anything else about shape.

Scope it to names NOT already committed.
The owner decided the 185 existing lines stay,
so a rule that fires on them would block routine edits to the files that carry them
and teach its own bypass.
Scoped to the unreached part of the corpus, the false-positive rate is zero by construction,
and the rule fires exactly when new corpus material enters a commit,
which is the thing that was asked for.

## What this does not do

It does not catch a passage whose people are already named in the repo.
It does not catch English prose that names nobody.
It is a tripwire on the highest-harm case, a real person's name entering a commit,
not a general corpus-text detector.
No general corpus-text detector is possible here,
because the distinguishing property is provenance and the scanner only sees content.
