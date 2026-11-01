# markdownlint-cli2 0.22.1: wrapper args can OOM, and config globs can hang on symlinked workspaces

## Status: replaced by cli-markdown-lint

`markdownlint-cli2` has been retired from this repo in favour of
`@monochromatic-dev/cli-markdown-lint` (`package/cli/markdown-lint`),
 a
purpose-built Markdown and MDX linter.
 The root `lint`/`format` tasks now call
`lint:markdown`/`format:markdown`,
 which run the new tool from TypeScript source
under Bun.

The replacement removes the failure modes documented below by construction:

- It only ever opens `.md` and `.mdx` files,
   so a multi-gigabyte binary can
  never be read as Markdown (no OOM).
- Its internal walk filters by extension and honours `.gitignore` directly,
  with no dot-remap footgun and no fast-glob symlink traversal.
- It runs TypeScript source directly under Bun,
   so there is no build-dist-before-lint
  step (issue #231 is closed).
- It lints `.mdx`,
   which `markdownlint-cli2` never covered.

Measured runtime:
 the new tool lints the whole tree (457 `.md`/`.mdx` files) in
one process in about 3.2 seconds,
 versus `markdownlint-cli2`'s chunked
0.4 to 0.7 seconds per 25-file batch (and the hangs and OOMs below for the
no-arg and dot-plus-arg shapes).

The rest of this document is kept as the record of why the old tool was dropped.

## Symptom

Two slow paths have appeared in this repo.

### Hard-coded dot plus forwarded args reads non-markdown files

A mise task whose `run` field hard-codes `markdownlint-cli2 .` works when invoked
with no extra arguments,
 but OOMs after 30 to 50 seconds when invoked with
positional arguments via `mise run lint:markdownlint -- <path>`:

```text
$ mise run lint:markdownlint -- package/oxlint-plugin/stylistic/README.md
[//:lint:markdownlint] $ markdownlint-cli2 . package/oxlint-plugin/stylistic/R...
markdownlint-cli2 v0.22.1 (markdownlint v0.40.0)
Finding: . package/oxlint-plugin/stylistic/README.md !node_modules/** !dist/** !.dist/** !bak/**
<--- Last few GCs --->
[927285:0x285b2000]    26112 ms: Mark-Compact 4016.7 (4124.1) -> 3992.6 (4127.6) MB, ...
FATAL ERROR: Ineffective mark-compacts near heap limit Allocation failed - JavaScript heap out of memory
```

Reproducible with the binary directly,
 no mise involvement needed:

```sh
markdownlint-cli2 . package/oxlint-plugin/stylistic/README.md  # OOMs
markdownlint-cli2 . README.md                                   # OOMs
markdownlint-cli2 . .                                           # OOMs
markdownlint-cli2 .                                             # works
markdownlint-cli2 package/oxlint-plugin/stylistic/README.md    # works
```

The failure shape is any time `.` appears alongside another positional argument.

### Config recursive glob hangs in discovery

After this repo moved whole-tree coverage into `.markdownlint-cli2.jsonc` via
`globs: ["**/*.md"]`,
 no-arg linting reached a different slow path:
`markdownlint-cli2` spends more than 60 seconds in globby discovery before it
prints `Linting:`.

A mistaken version probe also triggers that path because `markdownlint-cli2` has
no `--version` option:

```text
$ mise exec -- markdownlint-cli2 --version
markdownlint-cli2 v0.22.1 (markdownlint v0.40.0)
Finding: --version **/*.md !node_modules/** !dist/** !.dist/** !bak/**
  !package-paused/** !package-deprecated/** !.out-of-scope/**
```

Direct file linting through the current task is fast because the task passes
`--no-globs` for explicit paths:

```text
$ mise run lint:markdownlint -- README.md
markdownlint-cli2 v0.22.1 (markdownlint v0.40.0)
Finding: README.md !**/node_modules/** !node_modules/** !dist/** !.dist/**
  !bak/** !package-paused/** !package-deprecated/** !.out-of-scope/**
Linting: 1 file(s)
Summary: 0 error(s)
```

That command completed in 0.383 seconds on this checkout.

## Root cause

The two symptoms come from two related globbing traps.

### 1. The dot-only remap is gated on `globPatterns.length === 1`

`markdownlint-cli2.mjs:28` defines the substitute pattern,
 and
`markdownlint-cli2.mjs:219` gates the remap on the patterns array having exactly
one element:

```js
const dotOnlySubstitute = '*.{md,markdown}';
// ...
if ((globPatterns.length === 1) && (globPatterns[0] === '.')) {
  // Substitute a more reasonable pattern
  globPatterns[0] = dotOnlySubstitute;
}
```

The `--help` output documents this case:

```text
Dot-only glob:
- The command "markdownlint-cli2 ." would lint every file in the current
  directory tree which is probably not intended
- Instead, it is mapped to "markdownlint-cli2 *.{md,markdown}" which lints
  all Markdown files in the current directory
- To lint every file in the current directory tree, the command
  "markdownlint-cli2 **" can be used instead
```

When mise's `run = "markdownlint-cli2 ."` is invoked with `--`-forwarded args,
mise appends them,
 producing `markdownlint-cli2 . <path>`.
 Now `length === 2`,
the guard fails,
 and `.` is passed verbatim to globby.
 globby then walks every
file in the working tree,
 not just markdown files.
 The substitute pattern was
the only thing limiting the match to `*.{md,markdown}`.

### 2. Matched files are read into memory and parsed as markdown

`markdownlint-cli2.mjs:478-481` collects all matches into a single in-memory
array,
 and each file is later read fully into a string for parsing:

```js
const files = [
  ...await globby(filteredGlobPatterns, globbyOptions,),
  ...filteredLiteralFiles,
];
```

In this repo,
 the dot-plus-arg walk pulled in a 3.97 GB qcow2 VM disk image at
`package/vm-builder/cli/output/qcow2/disk.qcow2` plus other
multi-megabyte binaries.
 Reading the qcow2 into a string allocates about 4 GB on
V8's heap,
 which hits Node's default heap limit before the file is fully loaded.
The mark-compact loops in the OOM trace are V8 trying to reclaim space inside
that single string allocation.

### 3. Config globs are appended unless `--no-globs` is present

`markdownlint-cli2.mjs:413-418` appends top-level `globs` from the config to the
CLI patterns unless the invocation passes `--no-globs`:

```js
if (!noGlobs) {
  // Append any globs specified in markdownlint-cli2 configuration
  const globs = baseMarkdownlintOptions.globs || [];
  appendToArray(globPatterns, globs);
}
```

This is why a single explicit file argument can still pull in the whole
configured `**/*.md` tree unless the wrapper passes `--no-globs`.

### 4. markdownlint-cli2 does not disable symlink following for globby

`markdownlint-cli2.mjs:445-454` constructs the globby options without a
`followSymbolicLinks` override:

```js
const globbyOptions = {
  "absolute": true,
  "cwd": baseDir,
  "dot": true,
  "expandNegationOnlyPatterns": false,
  gitignore,
  ignoreFiles,
  "suppressErrors": true,
  fs
};
```

`fast-glob/out/settings.js:31`,
 used by globby,
 defaults symlink following to
true:

```js
this.followSymbolicLinks = this._getValue(this._options.followSymbolicLinks, true);
```

This repo has workspace package `node_modules` symlinks that can loop back into
other workspace packages:

```text
package/test-harness/core/node_modules/@monochromatic-dev/config-tsdown -> ../../../../config/tsdown
package/config/tsdown/node_modules/@monochromatic-dev/test-harness-core -> ../../../../test-harness/core
```

The repeated path exists and resolves back to `package/config/tsdown`:

```text
package/test-harness/core/node_modules/@monochromatic-dev/config-tsdown
  /node_modules/@monochromatic-dev/test-harness-core
  /node_modules/@monochromatic-dev/config-tsdown
```

A recursive `**/*.md` discovery can therefore traverse the symlinked workspace
graph before markdownlint starts linting files.

### 5. `gitignore: true` is not enough to prune this traversal

`markdownlint-cli2.mjs:981` enables globby gitignore handling only when config
sets `gitignore: true`:

```js
const gitignore = (baseMarkdownlintOptions.gitignore === true);
```

This repo does set `gitignore: true`,
 but `.gitignore:34` contains a negation:

```gitignore
!*.config.js
```

`globby/utilities.js:355-381` refuses to pass gitignore patterns down to
fast-glob when any negation is present:

```js
export const convertPatternsForFastGlob = (patterns, usingGitRoot, normalizeDirectoryPatternForFastGlob) => {
  // Determine which patterns are safe to pass to fast-glob
  // If there are negation patterns, we can't pass file patterns to fast-glob
  // because fast-glob doesn't understand negations and would filter out files
  // that should be re-included by negation patterns.
  // If we're using git root, patterns are relative to git root not cwd,
  // so we can't pass them to fast-glob which expects cwd-relative patterns.
  // We only pass patterns to fast-glob if there are NO negations AND we're not using git root.

  if (usingGitRoot) {
    return []; // Patterns are relative to git root, not cwd
  }

  const result = [];
  let hasNegations = false;

  // Single pass to check for negations and collect positive patterns
  for (const pattern of patterns) {
    if (isNegativePattern(pattern)) {
      hasNegations = true;
      break; // Early exit on first negation
    }

    result.push(normalizeDirectoryPatternForFastGlob(pattern));
  }

  return hasNegations ? [] : result;
};
```

The gitignore matcher still filters results,
 but it cannot stop fast-glob from
walking symlinked directories first.
 The markdownlint config needs its own
positive ignore pattern that fast-glob can use while traversing.

### 6. `--version` is parsed as a glob

`markdownlint-cli2.mjs:905-910` recognizes `--help` and `--no-globs`,
 then
returns unknown args as glob patterns:

```js
} else if (arg === "--help") {
  shouldShowHelp = true;
} else if (arg === "--no-globs") {
  noGlobs = true;
} else {
  return true;
}
```

There is no `--version` branch in that argument parser,
 so `--version` becomes a
literal glob pattern and the configured `**/*.md` glob is appended.

## Verification

Tested against:

- `markdownlint-cli2@0.22.1` (commit `996abf6` in this repo's clone of
  `DavidAnson/markdownlint-cli2`)
- `markdownlint@0.40.0` (the bundled engine version)
- Node 22.
  x via the mise-managed toolchain

Patterns that work cleanly:

- `markdownlint-cli2 .` (lone argument,
   remap applies)
- `markdownlint-cli2 package/oxlint-plugin/stylistic/README.md`
- `markdownlint-cli2 README.md AGENTS.md`
- `markdownlint-cli2 --no-globs README.md`
- `mise run lint:markdownlint -- README.md`,
   measured at 0.383 seconds
- 329 git-unignored Markdown files through `mise run lint:markdownlint -- <files>`,
  chunked in 25-file batches,
   measured at 0.402 to 0.744 seconds per chunk

Patterns that fail with OOM in this repo:

- `markdownlint-cli2 . <any-other-arg>` (remap bypassed,
   full walk)
- `markdownlint-cli2 . .` (same,
   the second `.` does not collapse)
- `markdownlint-cli2 packages` (directory arg,
   full sub-walk)

Patterns that hung in recursive discovery before the nested `node_modules` fix:

- `markdownlint-cli2` with config `globs: ["**/*.md"]`,
   `gitignore: true`,
   and
  only root-level `node_modules/**` in `ignores`
- `markdownlint-cli2 --version` with the same config,
   because `--version` is an
  input pattern
- direct globby probe with `['**/*.md', '!node_modules/**', '!dist/**',
  '!.dist/**', '!bak/**', '!package-paused/**', '!package-deprecated/**',
  '!.out-of-scope/**']`,
   which timed out at 60 to 120 seconds

Bounded probes that isolate the symlink cause:

```text
globby default followSymbolicLinks with current old ignores: timed out at 60 to 120 seconds
globby followSymbolicLinks: false with current old ignores: 0.158 seconds, 328 files
globby default followSymbolicLinks plus !**/node_modules/**: 0.148 seconds, 328 files
```

The OOM is environment-dependent:
 a tree without multi-gigabyte files would
still get the wrong file set,
 including binaries and lockfiles linted as
markdown,
 but would not necessarily crash the process.
 The symlink traversal
slowdown is workspace-layout-dependent:
 a tree without symlinked package
`node_modules` directories would not hit that recursive discovery cost.

## Verified workarounds

### A. Make the mise task argument-aware and use config globs for no-arg linting

`mise.no-env.toml` is the source file;
 `mise.toml` is generated from it by
file-enforcer.

```toml
[tasks."lint:markdownlint"]
hide = true
description = "Lint Markdown with markdownlint-cli2, optionally restricted to specific files"
usage = 'arg "[args]" var=#true help="Markdown file paths (when provided, only those files are linted)"'
run = """
let args = ($env.usage_args? | default '' | str trim | split row ' ' | where {|p| $p != '' })
if ($args | is-empty) {
  markdownlint-cli2
} else {
  markdownlint-cli2 --no-globs ...$args
}
"""
```

Tradeoffs:

- File paths with spaces break because the split is on the literal space
  character.
   Source-tree markdown paths in this repo do contain spaces under
  package type-test fixtures.
   Passing those paths through this task still needs
  the same quoting repair used by the root `test` task.
- No-arg linting now covers the configured recursive `**/*.md` tree,
   not only
  root-level `*.{md,markdown}`.
   That is the desired coverage,
   but it makes the
  ignore rules performance-critical.
- Explicit file linting does not load config `globs`;
   this is intentional so
  focused lint invocations stay focused.

### B. Add an explicit nested `node_modules` ignore to markdownlint-cli2 config

`.markdownlint-cli2.jsonc`:

```jsonc
"globs": [
  "**/*.md",
],
"gitignore": true,
"ignores": [
  "**/node_modules/**",
  "node_modules/**",
  "dist/**",
  ".dist/**",
  "bak/**",
  "package-paused/**",
  "package-deprecated/**",
  ".out-of-scope/**",
],
```

Tradeoffs:

- The ignore is explicit because gitignore cannot prune this traversal when
  negations are present.
   Future generated package-install directories with a
  different name need their own markdownlint ignore.
- The root-level `node_modules/**` entry remains for readability and for older
  tooling that does not interpret `**/node_modules/**` the same way.

### C. Use `--no-globs` and pass explicit files only

For task definitions that need to accept arbitrary file lists,
`markdownlint-cli2 --no-globs <file> [<file>...]` disables the config-level
`globs` array and lints only the explicit arguments.

Tradeoffs:

- Loses positive globs configured in `.markdownlint-cli2.jsonc` for that
  invocation.
   This is desired for focused file linting.
- Does not protect against a literal `.` argument.
   `markdownlint-cli2 --no-globs .
  README.md` still asks globby to expand `.`.

## What does not work

- Adding `!package/vm-builder/cli/output/**` and `!**/*.qcow2` to the
  config `ignores`:
   tested,
   still OOMs.
   Other large binaries in the tree,
  including Rust debug binaries at 43 to 94 MB each and a 37 MB git pack,
   keep
  heap pressure high enough to crash even without the qcow2.
- Expecting `markdownlint-cli2` to filter to text files by extension before
  reading:
   it does not.
   globby's match is filename-pattern only;
   files that
  match positive globs and miss negative globs are opened regardless of content
  type.
- Setting `gitignore: true` without an explicit nested `**/node_modules/**`
  ignore:
   `.gitignore` negations keep globby from pushing ignore patterns into
  fast-glob traversal,
   so symlinked package directories can still be walked
  before filtering.
- Calling `markdownlint-cli2 --version`:
   that is not a version option in
  `markdownlint-cli2@0.22.1`;
   it becomes an input pattern.

## Why we do not file this upstream

Walking the five constraints:

1. **Is it really upstream's fault?
   ** Partly.
    The
   `globPatterns.length === 1` guard is intentional;
    the `--help` output
   documents it.
    The recursive symlink traversal is a composition of
   markdownlint-cli2 not exposing globby's `followSymbolicLinks`,
    fast-glob's
   default,
    globby's correctness-preserving handling of negated gitignore
   patterns,
    and this repo's symlinked workspace layout.
2. **Can upstream fix it?
   ** Only by changing documented or default behaviour.
   The dot remap could fire whenever `.` appears in positional patterns,
    but
   that changes `markdownlint-cli2 . <file>` semantics.
    markdownlint-cli2 could
   expose a `followSymbolicLinks` option or default it to false,
    but that changes
   recursive glob coverage for users who intentionally lint symlink targets.
3. **Are they supporting this use case?
   ** No for the dot-plus-arg wrapper shape.
   The `--help` text explicitly warns that `.` means every file unless the
   special one-argument remap fires.
    The recursive config glob is supported,
    but
   this repo's symlink cycle is a consumer workspace concern.
4. **Will they likely fix it?
   ** Not from current signals.
    The maintainer already
   chose to document the dot-only behaviour rather than broaden the remap,
    and
   globby's negation handling is a correctness guard rather than a missed
   optimization.
5. **Have we prototyped a minimal fix compatible with their architecture?
   ** No.
   For the dot case,
    the minimal patch is easy but semantically breaking.
    For
   the symlink case,
    a compatible fix needs new markdownlint-cli2 option surface
   or a globby default change,
    both broader than this repo's immediate failure.

All five do not hold.
 The workaround belongs at this repo's boundary and is
applied in `.markdownlint-cli2.jsonc`,
 `mise.no-env.toml`,
 and generated
`mise.toml`.

## Draft upstream issue (do not file as-is)

Kept for audit.
 If upstream signals change on constraints 3 or 4,
 re-evaluate
before filing.

````md
**Title:** Document or guard against `.` plus another arg bypassing the dot-only remap

**Labels:** documentation, enhancement

**Body:**

The dot-only remap in `markdownlint-cli2.mjs:219` only fires when
`globPatterns.length === 1`. When `.` is paired with another positional
argument, for example via a shell alias or build-system task that hard-codes
`markdownlint-cli2 .` and forwards user args, the remap is bypassed and `.` is
passed verbatim to globby, walking every file in the tree.

The `--help` text already warns about the dot-only case being "probably not
intended". Two improvements would make the trap less sharp:

1. Extend the warning in `--help` to mention that the remap is gated on
   `length === 1`, so users who hard-code `.` in a wrapper understand why
   passing extra args silently changes the semantics.
2. Optionally, treat `.` as the substitute pattern whenever it appears alone
   among positional arguments, still allowing `'**'` for true recursion. This
   would tighten the trap at the cost of one documented breaking case.

**Reproduction:**

In a directory with a deeply nested tree containing any large binary file,
simulated below with a sparse 4 GB file:

```sh
mkdir tmp
cd tmp
echo '# test' > a.md
truncate -s 4G big.bin
markdownlint-cli2 . a.md
# FATAL ERROR: Ineffective mark-compacts near heap limit Allocation failed
```

**Suggested fix:**

In `markdownlint-cli2.mjs`, near line 219:

```js
// Substitute the dot whenever it appears as a positional argument, not only
// when it is the sole one.
for (let i = 0; i < globPatterns.length; i++) {
  if (globPatterns[i] === '.')
    globPatterns[i] = dotOnlySubstitute;
}
```

Tradeoff: any caller relying on `markdownlint-cli2 . <file>` to mean "lint the
tree and also this file" would need to switch to `**`, which is already the
documented recommendation. Tests in `test/` would need a new case covering the
multi-arg dot scenario.
````
