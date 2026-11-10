# ripgrep `--glob` + `-l` confusion: file-name filtering and content matching are independent passes

## Symptom

Listing test files with rg returns far fewer results than expected,
 and
the missing files appear correlated with spaces in directory paths:

```bash
# Returns 3 files -- all in paths without spaces
rg --glob '*.test.ts' -l '\.test\.' package/

# Returns 81 files -- including paths with spaces
rg --files --glob '*.test.ts' package/
```

The spaces are a red herring;
 rg handles spaces in paths correctly.
The real reason the first command returns three results is documented
below.

## Root cause

`--glob` and `-l` participate in different stages of rg's pipeline:

- `--glob '*.test.ts'` filters **which files to search**,
   by file name.
- `-l '\.test\.'` runs a regex content search for the pattern `\.test\.`
  and lists files whose contents match.

The three files that survived `-l '\.test\.'` happen to contain the
literal four-character sequence `.test.` somewhere in their text
(typically in an import path like `'./foo.test.ts'` or a comment).
The other 78 test files contain the word `test` (from test imports,
`it(...)` calls,
 describe titles) but never the dotted form `.test.`.

The confusion arises because,
 when debugging,
 the natural instinct is
to suspect path handling (spaces,
 special characters,
 symlinks) since
the visible diff between matched and unmatched files is "path
characters.
" The actual cause (a content pattern that does not match)
is invisible until you stop searching contents and start enumerating
files.

## Verification

Demonstrate the two operations are independent:

```bash
# Files whose NAME matches the glob (no content predicate)
rg --files --glob '*.test.ts' package/

# Files whose CONTENTS match the regex (no name predicate)
rg -l '\.test\.' package/

# Intersection: name matches glob AND content matches regex
rg --glob '*.test.ts' -l '\.test\.' package/
```

The first and second commands return supersets of the third by
construction.

`rg --debug` produces a per-file trace of which path was considered
and why it was skipped,
 useful when the disconnect between expected
and actual results is not obvious from the command line alone.

## Verified workaround

Use the operation that matches the intent:

```bash
# List files whose NAME matches *.test.ts (no content search)
rg --files --glob '*.test.ts' package/

# Search file CONTENTS for "test" in files named *.test.ts
rg --glob '*.test.ts' -l 'test' package/
```

Tradeoff:
 none.
 The two commands serve different needs;
 pick the one
that asks the question you actually have.
 The pitfall is reaching for
`-l <regex>` when the goal is "list files by name",
 which `--files`
already provides without invoking the content engine.

## What does not work

- Removing spaces from directory names:
   the spaces were never the
  problem;
   the content regex never matched in the missing files
  regardless of path shape.
- Switching `\.test\.` to `test`:
   changes the result set but does not
  fix the underlying confusion;
   the new pattern matches a different
  subset of files.
   If the goal is "files whose name ends in
  `.test.ts`",
   use `--files --glob '*.test.ts'`.

## Why we do not file this upstream

ripgrep's behaviour is documented and consistent with the rest of the
content-search family (grep,
 ag).
 This entry exists for our own future
sessions,
 not because ripgrep is at fault.

1.  **Is it really upstream's fault?
    ** No. `--glob` filters file
    inclusion;
     `-l` filters by content match.
     Each option does what its
    `--help` text says.
2.  **Can upstream fix it?
    ** Nothing to fix.
3.  **Are they supporting this use case?
    ** Both operations are
    supported;
     the user's question was misframed.
4.  **Will they likely fix it?
    ** N/A.
5.  **Have we prototyped a minimal fix?
    ** N/A.

Decision:
 keep this doc as an internal reminder;
 do not file anything
upstream.
