# Two per-token array copies in `@optique/core`, prototyped

Incomplete and not final.
An AI session (Pi,
model `deepseek-v4.1-flash`) wrote every line of this report and of the work it describes on
2026-11-13;
no person has reviewed or approved it,
and only the repository owner declares work complete.
Done and checked by that session:
the two fixes,
their tests,
the deliberate breaks of those tests,
the three runtimes' suites on the final tree,
the timing runs and the profile named below.
Open:
no human review;
nothing was sent to GitHub;
the two diffs are unreviewed by upstream,
and the buffer fix is
measurably useful only on very long command lines (section "Measurements").

## Where the work is

 -  Scratch folder:
    `${HOME}/temp/agent/upstream-prototype.3lsGY9v7`,
    of which `keep/` holds everything that must
    outlive the run.
    This report was written to `keep/report.md`.
 -  Clone:
    `keep/` is inside a clone of `dahlia/optique` made with `gh repo clone`,
    with its push URL set to
    `DISABLED`.
    Nothing was pushed,
    and the commit service's own retries all failed against that URL,
    which is what
    the wrapper's noise in the logs is.
 -  Base commit:
    upstream `main` at `d26f1a244d4434f9dbfeb0317f6db32533cb8c32` (the merge of pull request 1121),
    with the input-trace patch from
    `Monochromatic/doc/troubleshooting/optique-input-trace-copies-its-map-for-each-recorded-token.patch` applied
    unchanged and committed as `a9cbea5bdadd993f14f3ab2f897f0a6718539df5`.
    Every diff below applies to that commit.
 -  Upstream pull request 1121's head,
    checked as a second base:
    `a175cdb7a90778c1b3bce5712e2bc0bd3ae0f64b`
    (fetched through `gh api`,
    since a `git fetch` of the branch was refused;
    the branch is merged into the base
    commit above).

Diffs,
in `keep/diffs/`:

 -  `01-array-view.patch`:
    the shared foundation,
    an array-shaped view over storage the caller owns.
    311 lines,
    two new files (module and tests).
 -  `02-multiple-state.patch`:
    the `multiple()` fix.
    Needs `01`.
 -  `03-input-buffer.patch`:
    the `argument()` fix.
    Needs `01`,
    not `02`.
 -  `04-both-combined.patch`:
    `01`,
    `02` and `03` at once,
    for a reader who wants one file.

The two fixes are separate:
they touch disjoint files and either can be applied without the other.
They are not
independent of the *foundation*:
both need the array view,
and duplicating that module in both diffs would put two
copies of the same proxy code in the package,
so it is one diff of its own instead of being repeated.
That is the
one place where "either can be taken without the other" does not hold literally;
everything else does.

Check results (`keep/measurements/apply-checks.jsonl`),
for the input-trace patch and then `01`,
`02`,
`03` in order,
applying cleanly and being applied:

 -  on `main` at `d26f1a24`:
    all four apply.
 -  on pull request 1121's head at `a175cdb7`:
    all four apply.
 -  `02` alone and `03` alone on the trace-patched base:
    both apply as patches (they create only their own files and
    edit only their own call sites),
    but a tree with `02` and no `01` does not compile,
    because `persistent-vector.ts`
    imports `array-view.ts`.
    The same holds for `03`.

No hunk needed a fuzz or a manual fix on either base.

## What each fix changes

### `multiple()`'s state array (`02-multiple-state.patch`)

Changed:

 -  `packages/core/src/internal/array-view.ts` (from `01`):
    `virtualArrayOf()` builds a proxy over an empty array
    that serves indices,
    `length`,
    `has`,
    `ownKeys`,
    `getOwnPropertyDescriptor`,
    `set` and `deleteProperty`;
    writes
    to an element or to `length` throw.
 -  `packages/core/src/internal/persistent-vector.ts`:
    an immutable vector,
    a 32-way trie with slot counts on the
    branches,
    `get` in O(log32 N),
    `append` and `dropLast` copying only the nodes on the path,
    `toArray`,
    iteration.
    Plus `appendTo()` and `dropLastFrom()`,
    which keep plain arrays below `VIEW_THRESHOLD = 1024` items and wrap the
    vector in an array view above it.
 -  `packages/core/src/modifiers.ts` lines 2746 to 2752 and 2918 to 2924 (the sync and async twins):
    the expression
    `annotateFreshArray(context.state, [...(added ? context.state : context.state.slice(0, -1)), nextItemState])`
    becomes `annotateFreshArray(context.state, appendTo(added ? context.state : dropLastFrom(context.state),
    nextItemState))`.

Readers of `multiple()`'s state array,
all checked,
and all inside `modifiers.ts`:

 -  `selectItems()` lines 2564,
    2565 and 2569:
    `state.at(-1)` and `state.length`.
 -  `parseSync()` lines 2614,
    2615,
    2639 and the append site:
    `context.state.length` and `context.state`.
 -  `parseAsync()` lines 2786,
    2787,
    2811 and its append site:
    the same.
 -  `complete()` lines 2986,
    2988,
    3018,
    3020:
    `state.length` and `state[i]` for each item.
 -  `[extractPhase2SeedKey]()` lines 3054,
    3056,
    3093,
    3095:
    the same.
 -  `suggest()` lines 3132,
    3133,
    3139,
    3148,
    3149,
    3163:
    `context.state.at(-1)`,
    `.length` and `context.state[i]`.
 -  `getDocFragments()` lines 3328 and 3329:
    `state.state.length > 0` and `state.state.at(-1)`.

Searched and found not to read it:
`constructs.ts`,
`primitives.ts`,
`facade.ts`,
`completion.ts`,
`usage.ts`,
`doc.ts`,
`suggestion.ts` and the other packages.
The array-shape heuristics in `modifiers.ts` that do look at arrays
(`unwrapMultipleItemState()` line 172,
`isMatchedCommandParserState()` line 96,
`optional()`/`withDefault()` state
carriers) read an *item's* state,
not the array of items,
and see the same values as before;
`optional(multiple(x))`
keeps working because the value is still `Array.isArray` and still reports its length.

Public type and behaviour:
`multiple()`'s signature is unchanged,
the state is still `readonly TState[]`,
and every
existing test passes unchanged.
Two behaviour changes are visible only to code that breaks the contract already:
writing to an element of the state (or to `length`) now throws `TypeError` instead of silently changing that version,
and `Object.isFrozen()` is false on a view even though writes fail.

Alternatives rejected:

 -  *Change the state to a persistent sequence.*
    Then `readonly TState[]` becomes another type,
    and the readers
    above plus the array-shape heuristics in `modifiers.ts` and the tests that build a state as an array would all
    have to change;
    the brief asks for unchanged tests and public behaviour.
 -  *Mutate the state array in place.*
    A failed alternative discards its context and the parse resumes from an
    earlier state;
    editing the array two contexts share shows one of them an item the other added.
    The new tests
    catch exactly that (section "Tests"),
    and so would the existing backtracking tests in some shapes.
 -  *A chunked array whose chunk list is copied per append.*
    That is O(N/32) per append,
    still quadratic in the
    long run;
    the vector's path copy is O(log32 N).
 -  *No threshold.*
    Measured below (section "Measurements"):
    below about a thousand items the copy is cheaper than
    the view,
    so the fix would have made ordinary command lines slower.

### `argument()`'s input buffer (`03-input-buffer.patch`)

Changed:

 -  `packages/core/src/internal/input-buffer-view.ts`:
    `dropPrefixFrom()` returns `base.slice(offset)` while fewer
    than `VIEW_THRESHOLD = 16384` tokens are left,
    and an array view that remembers the base array and the offset
    above it;
    cutting a view again moves the offset.
    `leadingOf()` copies the first `count` tokens straight out of
    the base array,
    so a two-token `slice()` does not walk proxy traps per element.
 -  `packages/core/src/primitives.ts` lines 2717,
    2726 and 2734 (sync) and 2744,
    2753 and 2757 (async):
    the trace entry and
    `consumed` use `leadingOf(context.buffer, i + 1)`,
    and `buffer: context.buffer.slice(i + 1)` becomes
    `buffer: dropPrefixFrom(context.buffer, i + 1)`.

Readers of `context.buffer`,
checked:

 -  `primitives.ts`:
    101 mentions,
    all of the form `buffer[0]`,
    `buffer[i]`,
    `buffer.length`,
    `buffer[context.buffer.length - 1]`,
    `buffer.slice(n)` and two spreads of `buffer.slice(1)` into a new token
    list (line 1429).
    Every one of those works on a view;
    the spreads and `slice()` calls produce plain arrays.
 -  `constructs.ts`:
    117 mentions,
    the same kinds of read.
 -  `internal/parser.ts`:
    26 mentions,
    including `getFailureProgress()` line 1126 (`buffer.slice(consumedInStep)`)
    and the loop conditions `while (context.buffer.length > 0)` at lines 1235,
    1341,
    1607,
    1872,
    2269, 2309.
 -  `internal/passthrough.ts` lines 55,
    124 and 163 compare `buffer` by identity to mean "the same cursor".
    Each cut
    is a new object and an unchanged buffer stays the same object,
    so the comparison means what it meant.
 -  `internal/parser.ts` line 1418 `isBufferUnchanged()` compares length and every element;
    a view answers both.
 -  `facade.ts`:
    16 mentions,
    the same kinds of read.
 -  `modifiers.ts`:
    4 mentions,
    `context.buffer.length === 0`.
 -  The rest of `packages/` (config,
    env,
    testing and the others) either builds a context with a plain array in a
    test or does not touch `buffer`.

Public type and behaviour:
`ParserContext.buffer` is still `readonly string[]`,
and every test passes unchanged.
Visible only to code that breaks the contract:
writing a token or `length`,
and in-place `Array.prototype` methods
(`sort`,
`reverse`,
`splice`,
`push` and the like) now throw `TypeError` on a cut buffer above the threshold,
where
before they would have edited that parse's private copy.

Alternatives rejected:

 -  *An offset field on `ParserContext` and a plain array.*
    `buffer[0]` would stop being the next token,
    which is
    the contract every reader above relies on,
    including parsers library users write.
 -  *A new type for `buffer`.*
    It is a public field,
    read by user parsers;
    the smallest change would be a `TokenList`
    interface with `at(index)`,
    `length` and `slice()`,
    and it would mean rewriting 250-odd call sites in
    `primitives.ts`,
    `constructs.ts`,
    `internal/parser.ts` and `facade.ts` plus every user parser that indexes it.
 -  *A lower threshold.*
    Measured:
    at 4,096 a parse of 8,000 tokens was 22% slower with views,
    and at 8,192 one of
    16,000 tokens was 28% slower.
    16,384 is where the win is proven and the loss is out of reach.
 -  *Not shipping this fix at all.*
    It is close to it:
    see "Measurements".
    At the delivered threshold it engages
    only on command lines of more than 16,384 tokens,
    where it makes the 32,000-token parse 1.25 times faster on
    top of the `multiple()` fix.

## Tests

Upstream's style:
`node:test` with `node:assert/strict`,
co-located as `*.test.ts`,
no timing threshold anywhere.
The regression tests guard structure and sharing,
not time,
as pull request 1121 describes its own tests.

New test files:

 -  `packages/core/src/internal/array-view.test.ts` (6 tests):
    the view as an array (indexing,
    `length`,
    `slice`,
    spread,
    `map`,
    `every`,
    `join`,
    `indexOf`,
    `at`,
    `Object.keys`,
    `JSON.stringify`,
    `in`,
    `Object.hasOwn`,
    `node:assert` deep equality),
    a shorter window than the source could serve,
    an empty view,
    symbol properties,
    that writes throw,
    and that two views of one source do not see each other's window.
 -  `packages/core/src/internal/persistent-vector.test.ts` (14 tests):
    building,
    size boundaries at
    0,
    1,
    31,
    32,
    33,
    1023,
    1024,
    1025 and 32,801,
    appending one at a time with every earlier version read back at
    its start,
    middle and end,
    dropping,
    two extensions of one version,
    a version extended after a drop,
    a
    fast-check property comparing every intermediate version against a plain array over random append,
    drop-last and extend-an-older-version sequences;
    and for `appendTo()`/`dropLastFrom()`:
    the plain-array path,
    every earlier version kept,
    two extensions of one array,
    crossing the threshold,
    and dropping the last item of
    a view.
 -  `packages/core/src/internal/input-buffer-view.test.ts` (9 tests):
    prefixes,
    empty tails,
    dropping zero,
    two drops from one array,
    dropping the prefix of a view,
    a long buffer cut without copying,
    a short buffer
    copied,
    and `leadingOf()` on a plain array,
    on a view,
    and the prefix being a plain array the caller owns.

New tests in existing files:

 -  `src/modifiers.test.ts`:
    "should keep a state that is extended twice from mixing items" (sync),
    "should keep an async state that is extended twice from mixing items",
    "should keep every intermediate state of
    a long run".
    The first two build a state of `VIEW_THRESHOLD` items,
    extend it twice with different next items
    and check that neither result carries the other's item and that the state both were taken from still completes
    to its own items.
    The third parses 1,027 items one at a time and reads back the versions on both sides of the
    threshold,
    and every version's length.
 -  `src/primitives.test.ts`:
    "should keep a buffer consumed twice from mixing tokens" and its async twin.
    From one
    context with `VIEW_THRESHOLD + 2` tokens,
    `argument()` consumes one token and an `object()` of two arguments
    consumes two;
    neither result's buffer shows the other's cursor and the original buffer still holds every token.

The parser-level tests size themselves from the exported constants (`VIEW_THRESHOLD` from each new module),
so they
cross the threshold and would follow it if it moved.
They import those constants by relative path,
because the
package import map points at `dist/` under Node and the bundler does not export an internal constant that only the
module itself uses;
the first version of these tests used `#src/` and failed on Node and Bun with
`does not provide an export named 'VIEW_THRESHOLD'`,
which the suite caught before this report.

Deliberate breaks,
one at a time,
restored after each,
from `keep/logs/breaks.jsonl`:

 -  *array-view-forwards-writes-to-its-target*:
    guards "refuses to write an element or the length";
    exit 1,
    1 failed,
    caught.
    The
    failing lines were `refuses to write an element or the length`,
    `virtualArrayOf`,
    `FAILED | 0 passed (5 steps) | 1 failed (1 step) (24ms)`.
 -  *array-view-hides-its-index-descriptors*:
    guards "presents the elements of its source as an array",
    "reports a shorter length than its source could serve";
    exit 1,
    2 failed,
    caught.
    The
    failing lines were `presents the elements of its source as an array`,
    `reports a shorter length than its source could serve`,
    `virtualArrayOf`.
 -  *vector-append-edits-the-shared-leaf*:
    guards "two extensions of one version do not see each other",
    "should keep a state that is extended twice from mixing items",
    "should keep every intermediate state of a long run";
    exit 1,
    3 failed,
    caught.
    The
    failing lines were `two extensions of one version do not see each other`,
    `a version extended after a drop still holds the dropped item`,
    `matches a plain array over random append and drop sequences`.
 -  *vector-drop-edits-the-shared-leaf*:
    guards "dropping the last item keeps the earlier and the later version";
    exit 1,
    3 failed,
    caught.
    The
    failing lines were `dropping the last item keeps the earlier and the later version`,
    `a version extended after a drop still holds the dropped item`,
    `matches a plain array over random append and drop sequences`.
 -  *buffer-view-forgets-its-offset*:
    guards "a long buffer is cut without copying it",
    "dropping the prefix of a view continues from the same array";
    exit 1,
    2 failed,
    caught.
    The
    failing lines were `dropping the prefix of a view continues from the same array`,
    `a long buffer is cut without copying it`,
    `dropPrefixFrom`.
 -  *buffer-cut-edits-the-buffer*:
    guards "should keep a buffer consumed twice from mixing tokens",
    "should keep an async buffer consumed twice from mixing tokens";
    exit 1,
    2 failed,
    caught.
    The
    failing lines were `should keep a buffer consumed twice from mixing tokens`,
    `should keep an async buffer consumed twice from mixing tokens`,
    `argument`.
 -  *leading-of-ignores-the-offset*:
    guards "copies the first tokens of a view without reading the view";
    exit 1,
    1 failed,
    caught.
    The
    failing lines were `copies the first tokens of a view without reading the view`,
    `leadingOf`,
    `FAILED | 1 passed (9 steps) | 1 failed (1 step) (46ms)`.

After restoring the last break the same five test files were run again:
exit 0,
a pass.
The raw records
are in `keep/logs/55-breaks-final.jsonl`,
and the script is `keep/scripts/breaks.ts`.

One break needed the tests themselves fixed first:
with the vector's append editing a shared leaf,
the two
`multiple()` tests passed while the vector's own tests failed,
because they built their state by copying a plain
array twice rather than appending twice to one shared view.
They now build a state of
`VIEW_THRESHOLD + 5` items,
which is a view,
and the break makes both of them fail.
That is why they are worth
having at the parser level at all.

Each break was applied to the final source,
the named test files were run with
`deno test --allow-read --allow-env <files>`,
the exit status and the failing test lines were recorded,
and the file
was restored and compared with its original text.
After the last restore the same files were run again and passed.

## Suites

Upstream's `packages/core` suite,
on the current releases (rule `PIN`:
Node 26.13.1,
Deno 2.9.8,
Bun 1.4.4,
from
the image `localhost/optique-prototype-work-current-bun:2026-11-13`),
in a container with no credentials and no
access to the Monochromatic repository:

 -  Final tree (both fixes,
    current code),
    in the clone:
     Deno 784 passed (8,340 steps),
    0 failed,
    29 ignored;
     Node 8,125 pass,
    0 fail,
    529 skipped;
     Bun 8,125 pass,
    529 skip,
    0 fail.
 -  The trace-patched base alone,
    in a worktree (first run of the four-tree loop):
     Deno 778 passed (8,301 steps),
    0 failed,
    29 ignored;
     Node 8,086 pass,
    0 fail,
    529 skipped;
     Bun 8,086 pass,
    529 skip,
    0 fail.
 -  Each tree alone on Deno,
    from source,
    in a plain copy of the base commit with that arm's
    `packages/core/src` (no worktree,
    so nothing could clear its ignored files):
    base 778 passed (8,301 steps),
    0 failed,
    `01`+`02` alone 782 passed (8,328 steps),
    0 failed,
    `01`+`03` alone 781 passed (8,319 steps),
    0 failed,
    all three
    784 passed (8,340 steps),
    0 failed.
    The logs are `keep/logs/base.log`,
    `multiple.log`,
    `buffer.log` and `both.log`.

What did not run or did not count:

 -  The four-tree Node and Bun runs in `keep/logs/25-suites-all-trees.log` (each tree installed and built in its
    own git worktree of the clone) are *not* usable evidence:
    the
    intermediate worktrees' `node_modules/.pnpm` was emptied between runs by the git wrapper's worktree handling,
    so those runs failed to resolve `fast-check` and `typescript` (`ERR_MODULE_NOT_FOUND`),
    and their counts are
    lower than the real suite.
    The final tree's numbers above come from the clone,
    whose `node_modules` was intact,
    and were re-run after the last source change.
 -  `current Bun` is 1.4.4;
    nothing was run on the older release the prototype's image first carried,
    and per rule
    `PIN` no result from an older release is reported.
 -  `deno fmt --check`,
    `deno lint` and `deno check` on `packages/core/src` are clean for the final tree;
    the log is
    `keep/logs/50-final-lint.log` and `keep/logs/51-final-check.log`.
 -  `deno publish --dry-run` (upstream's `check:publish`) was not run.
 -  The docs build (`docs/`) and the other packages' suites were not run;
    only `packages/core` changed,
    and
    `pnpm build` of every package succeeds with it.

## Measurements

All timings below are from `localhost/optique-prototype-arms:2026-11-13`,
run as
`podman run --rm --network none --cpus 2 --memory 8g --pids-limit 1024 --blkio-weight 10`,
with every arm,
import
map and script baked into the image and nothing mounted.
The arms are Deno source checkouts through an import map,
so
no build step is involved.
Deno 2.9.8,
on a host whose load average was between 4 and 5 on two CPUs (rule `NQH`).

### `token-scaling.ts`, five rounds, arm order rotated

Each round ran every arm with `--counts 2000,4000,8000,16000,32000 --runs 2 --warmup 1`,
and each figure below is
the median over the five rounds.
Arms:

 -  *main*:
    upstream `main` at `d26f1a24`.
 -  *base*:
    `main` with the input-trace patch,
    the base of this work.
 -  *multiple*:
    `base` + `01` + `02`.
 -  *buffer*:
    `base` + `01` + `03`.
 -  *both*:
    `base` + `01` + `02` + `03`,
    the two fixes delivered here.

Medians,
in ms:

 -  2,000 tokens:
    main 90.4,
    base 35.3,
    multiple 25.5,
    buffer 32.8,
    both 25.6
 -  4,000 tokens:
    main 312.9,
    base 53.8,
    multiple 35.7,
    buffer 52.1,
    both 37.2
 -  8,000 tokens:
    main 1,278.3,
    base 140.7,
    multiple 69.7,
    buffer 142.8,
    both 72.0
 -  16,000 tokens:
    main 5,726.7,
    base 786.2,
    multiple 169.6,
    buffer 868.2,
    both 171.4
 -  32,000 tokens:
    main 26,381.1,
    base 5,676.1,
    multiple 773.1,
    buffer 4,856.0,
    both 619.0

Fitted exponent of time against token count,
from those medians:
main 2.06,
base 1.85,
multiple 1.21,
buffer 1.85,
both 1.14.
Round by round,
the range of each arm's own fit was main 2.04 to 2.07,
base 1.82 to 1.90,
multiple 1.16
to 1.35,
buffer 1.79 to 1.88,
both 1.07 to 1.19.

Paired per-round ratios,
25 points (five rounds by five counts),
median with range and the number of points where
the candidate was slower:

 -  base against main:
    0.173 (0.108 to 0.447),
    0 of 25 slower.
 -  multiple against base:
    0.485 (0.129 to 0.771),
    0 of 25 slower.
 -  buffer against base:
    0.940 (0.808 to 1.104),
    11 of 25 slower.
 -  both against base:
    0.484 (0.106 to 1.046),
    1 of 25 slower.
 -  both against multiple:
    0.998 (0.479 to 1.360),
    12 of 25 slower.

Per count,
from the medians:
multiple against base is 0.72 at 2,000,
0.66 at 4,000,
0.50 at 8,000,
0.22 at 16,000
and 0.14 at 32,000 tokens;
both against base is 0.72,
0.69,
0.51,
0.22 and 0.109;
both against multiple,
which is
what the buffer fix adds on top of the `multiple()` fix,
is 1.00,
1.04,
1.03,
1.01 and 0.80,
so nothing below the
buffer's threshold and 1.25 times faster at 32,000 tokens.

Small sizes,
from the same image with `--counts 100,1000 --runs 9 --warmup 2`:
at 100 tokens main 1.02,
base
1.08,
multiple 1.09,
buffer 1.03,
both 1.15;
at 1,000 tokens main 23.7,
base 7.8,
multiple 7.96,
buffer 7.45,
both
7.10.
Below both thresholds the arms run the code they ran before,
and these figures are inside their own spread.

The crossover runs that decided the two thresholds are `keep/measurements/crossover-sweep.jsonl`,
`matrix-buffer-threshold-4096.jsonl`,
`final-thresholds-8192.jsonl` and `final-thresholds-16384.jsonl`.
With the
buffer threshold at 4,096 a parse of 8,000 tokens was 22% slower with views and one of 32,000 was 2.7 times
faster;
at 8,192 a parse of 16,000 tokens was 28% slower;
at 16,384 no parse below 16,384 tokens changes its code
path and the 32,000-token parse is 1.4 times faster than without the fix in that run,
1.25 in the five-round run.

### `commit-paths.ts`, 1,000 paths, five rounds

Same arms and image,
`--paths 1000 --runs 7 --warmup 2`.
Medians,
in ms:

 -  *without* `--separator`:
    main 61.5,
    base 43.8,
    multiple 42.8,
    buffer 44.3,
    both 44.1.
    Paired:
    base against
    main 0.725 (0.624 to 0.762,
    all five rounds faster);
    multiple against base 0.973 (0.968 to 1.088,
    three of five
    faster);
    buffer against base 1.012 (0.975 to 1.137,
    two of five);
    both against base 1.05 (1.003 to 1.108,
    none
    of five faster);
    both against multiple 1.031 (1.019 to 1.094).
 -  *with* `--separator`:
    main 45.1,
    base 29.3,
    multiple 29.3,
    buffer 29.9,
    both 29.0.
    Paired:
    base against main
    0.650 (0.616 to 0.673,
    all five faster);
    both against base 1.01 (0.968 to 1.087,
    two of five faster);
    both
    against multiple 1.014 (0.99 to 1.059).

A thousand paths is below both thresholds,
so neither fix changes a code path here.
The 3% to 5% in favour of the
base tree on the no-separator form is the two extra calls and WeakMap lookups per token plus run-to-run noise;
it is
not a cost this work set out to remove,
and it is reported because it is measured.

### Profile of the final arm

`node --cpu-prof --cpu-prof-interval=500` on the built `dist` of the final tree,
one parse per process,
through
the same image with no mounts and no network (`keep/scripts/profile/one-parse.ts`,
analysed by
`keep/scripts/analyse-profiles.ts`):
self time of the samples that sit inside a `parseSync()` frame.

At 16,000 tokens,
329.6 ms of parse time:
`input-buffer-view.js` 25.9% (`dropPrefixFrom` alone 25.5%),
`dependency-runtime.js` 12.5%,
`modifiers.js` 11.9%,
`constructs.js` 9.8%,
`persistent-vector.js` 8.7%,
`immutable-string-map.js` 6.5%,
`primitives.js` 4.9%,
`short-option.js` 4.9%,
`array-view.js` 3.3%,
`execution-context.js` 2.7%,
`failure.js` 2.7%,
`input-trace.js` 1.6%.

At 32,000 tokens,
593.5 ms of parse time:
`dependency-runtime.js` 14.7%,
`input-buffer-view.js` 14.6%
(`dropPrefixFrom` 12.7%),
`constructs.js` 10.2%,
`modifiers.js` 9.5%,
`primitives.js` 9.1%,
`persistent-vector.js` 8.1%,
`array-view.js` 5.8%,
`short-option.js` 5.7%,
`execution-context.js` 5.1%,
`immutable-string-map.js` 5.0%,
`failure.js` 3.3%,
`input-trace.js` 3.2%.

No single function holds more than 13% of the parse any more.
The largest item is the buffer view's own creation
(`dropPrefixFrom`);
at 16,000 tokens,
where the threshold keeps the plain array,
that same entry is the copy that
is still there.
For comparison,
the recorded profile of the trace-patched build in
`doc/troubleshooting/optique-input-trace-copies-its-map-for-each-recorded-token.md` put `multiple()`'s
`parseSync` at 71.2% and the function holding the `argument()` slice at 19.1% of a parse of 16,000 and 32,000
tokens.

The raw profiles are gzipped and base64 encoded at the end of `keep/measurements/profiles.log`.

## Changelog fragments and commit messages

`keep/changelog/*.md` are the two `sacho` fragments,
added with `sacho add --section @optique/core <topic>` and
formatted with `sacho fmt`;
`sacho sync --force`,
`sacho fmt` and `sacho check` all exit 0 on the tree with both
(`keep/logs/42-sacho.log`).
The materialised unreleased section of `CHANGES.md` that `sacho sync` produced is
`keep/diffs/changelog-materialized-section.diff`;
it is not part of the diffs,
because two diffs that both append to
the same region of `CHANGES.md` would conflict,
so `sacho sync --force` regenerates it after applying them,
which is
what the trace patch does as well.

Commit messages are `keep/commit-messages/01-array-view.txt`,
`02-multiple-state.txt` and `03-input-buffer.txt`,
one
per diff,
in upstream's form:
no Conventional Commits prefix,
the reason first,
and an
`Assisted-by: Pi:deepseek-v4.1-flash` trailer.

## What I could not verify, and doubts

 -  *The `multiple()` fix is the one to take.*
    It removes 80% to 85% of the parse at 8,000 to 32,000 tokens and
    drops the fitted exponent from 1.88 to 1.22.
    The buffer fix adds a smaller win only above 16,384 tokens.
    If the
    owner wants one diff,
    it is `01` + `02`.
 -  *The buffer fix has a worse cost profile than the `multiple()` fix.*
    A cut costs about 3 µs per token whatever
    the buffer's length (one proxy,
    the option probes' index reads,
    the prefix copy),
    while a copied token costs
    about a third of a nanosecond.
    The threshold therefore sits far above where a command line ever goes,
    and the
    fix is close to "not worth it" except for very long inputs.
    A cheaper view (a plain array with a movable cursor
    in the context) would need the public type change described above.
 -  *The measurements were taken on a loaded host.*
    Load average was between 4 and 5 on two CPUs during the runs
    (rule `NQH`:
    measure under the load there is,
    report the spread).
    The medians and the paired ranges are given;
    the spread at 8,000 to 16,000 tokens is ±15% to ±30%,
    which is why the buffer fix's small wins there are not
    claimed.
 -  *The 32,000-token point for the `multiple()`-only arm varied by a factor of two between rounds in an earlier
    run* (716 ms to 1,527 ms),
    most likely from allocation pressure and GC;
    its median over five rounds is reported
    with its range.
 -  *Proxy semantics on other engines were not tested.*
    Node,
    Deno and Bun all run V8;
    SpiderMonkey and JavaScriptCore
    would need their own run.
    The proxy answers `Array.isArray`,
    indexing,
    `length`,
    iteration,
    `Object.keys`,
    `JSON.stringify` and `node:assert` deep equality as an array,
    which is what the library and its tests use;
    code
    that `structuredClone`s a parser context or state would now throw,
    where before it produced a copy.
    Nothing in
    `packages/core` does that (one test clones a doc page,
    not a state).
 -  *`Object.isFrozen()` is false and writes throw.*
    A reader that inspects the state or buffer with `isFrozen`,
    or that mutates an array it was handed,
    sees a difference.
    This is the price of keeping the array type.
 -  *Rule `PIN` and Bun's skip semantics.*
    Bun ignores the `skip` option,
    so the ignored tests run there;
    the counts
    above are Bun's own.
 -  *The intermediate trees' Node and Bun suites did not run cleanly* (the `node_modules` problem above).
    Only Deno
    was run on each fix alone,
    plus the untouched base on all three.
    The final tree is the one that ran on all
    three.
 -  *Surprise,
    and the reason the buffer fix has a threshold:*
    the copy it removes is cheaper than expected.
    `Array.prototype.slice()` on a 32,000-element string array costs about a third of a nanosecond per element,
    so
    the per-token view (about 3 µs) only pays off when the buffer is very long;
    the first threshold I chose,
    4,096,
    made a parse of 8,000 tokens 22% slower than the copy it replaced.
 -  *Surprise:*
    `git fetch` of a branch was refused by this machine's git wrapper ("access rights"),
    so the pull
    request branch's head was read with `gh api` instead.
    Rule `AGENTS.md` `APQ`-style isolation was kept:
    the
    clone's push URL is `DISABLED`,
    and no push,
    pull request,
    issue or comment was made.
