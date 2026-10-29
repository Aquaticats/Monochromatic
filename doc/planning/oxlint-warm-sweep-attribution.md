# Whole-repo oxlint timing, first attribution

Opened against issue #374,
 which asks for a warm whole-repo `mise run lint:oxlint` under sixty
seconds.
Measured 2026-08-08 on the machine running this session.
No optimization proposed yet:
 this records where the time goes,
 because every number quoted
for this before was a cold run.

## Current state, 2026-08-08

This document is long and append-only.
What follows is where things stand;
 everything below it is the record of getting here,
including four claims that were withdrawn.

### Landed

- `4fedeac65` derives the project inclusion scope once rather than per file.
- `f2eea0182` shares settled type classifications per project.
- `843e267de` recognises declaration files by name instead of decoding them to ask.
- `2b86858fe` withholds classifications that stood on an unresolved assumption,
 correcting a
defect `f2eea0182` introduced.
- `f84c5f487` refuses a configured project farther than the nearest declared one,
 correcting a
pre-existing wrong-offer path.
- `05b2c415e` keeps the decoded sources TypeScript already retains across snapshot replacement.
- `f583d6110` holds final indexes weakly and builds the file list where it is read.
- `1f9aa095d` fingerprints analysed sources from the snapshot and declarations from disk.

Three of those are correctness fixes rather than optimisations,
 and all three were found by
reading the code while measuring it rather than by timing it.
That is the argument for reading:
 the timing said where to look,
 and every defect was visible only
once something was actually opened.

Warm whole-repo `mise run lint:oxlint`:
 **3m04.7s to about 59s,
 a sixty-eight per cent
reduction**,
 with 2893 errors and 1555 rule findings throughout,
 and the final sweeps
byte-identical to the pinned digests.

### The two holes left open, then closed

Both were filed rather than fixed,
 then fixed.
Recording the intermediate state because the reasoning for deferring them was wrong in the same
way twice:
 each was deferred on a cost that had never been measured.

**#420,
 the outgoing overlay.**
`openSemanticFile` cleared its overlay map to the incoming source and reported only that source
through `fileChanges`,
 so the native server kept holding the text it was last given for a source
the overlay had stopped claiming.
This is the assumption the decoded-source cache rests on:
 `getRetained` never rechecks the content
hash,
 so retention is only as complete as the server's change report,
 and the report was missing
the outgoing source.
Unreachable under Oxlint,
 which hands over what it read from disk.
Reachable by anything supplying an unsaved buffer.

Closed in `4add69903` by retaining every text handed to the bridge instead of clearing.
Reporting the outgoing source would have meant a snapshot update per file,
 including on the reuse
path that exists to avoid one,
 and a snapshot update replaces every `Project` in the process.
Retaining reaches the same invariant for nothing.

The assertion that was supposed to cover this reopened one path twice,
 so it could not tell
retention from clearing and passed either way.
It now opens a second source of the same project,
 and was shown to fail when retention is removed.

**#421,
 declaration files in the fingerprint.**
Analysed sources were hashed from the snapshot and declarations still read from disk,
 so a
workspace `.d.ts` rebuilt underneath a running lint could key summaries to text nothing analysed.
A declaration file decides types,
 so a stale one poisons an entry as effectively as a stale
implementation.

Closed in `41aab4ab9`.
It had been left open on a 6.2s cost that came from one run against one run,
 inside a band wider
than itself,
 which is to say on nothing.

Measured properly,
 five warm sweeps of each configuration run back to back:

```text
declarations from snapshot   61.2  61.5  62.4  60.5  61.0     mean 61.32s
declarations from disk       60.4  61.5  62.8  61.5  60.5     mean 61.34s
```

**Two hundredths of a second apart,
 in favour of the correct one.**
Diagnostics byte-identical to the pinned digests on both,
 35 offers.

Two further things fall out of that table.

The spread within each configuration is 1.9s and 2.4s,
 against the 4.6s measured earlier.
The band is not a constant;
 the earlier figure was sampled while probes competed for the same
cores.
Treat it as a property of the conditions,
 not of the sweep.

Both configurations now measure about 61.3s,
 where the same code measured 57.3s to 61.9s earlier
in the day.
So a comparison is only meaningful between runs taken close together on an otherwise quiet
machine,
 and a number quoted from one session should not be compared against a number from
another.
Neither fix caused the shift:
 the second configuration is the older code and it moved too.

### What these timings can and cannot support

Read this before quoting any number above as the effect of any one change.

Four warm sweeps of one unchanged build measured **57.3s,
 58.9s,
 59.3s and 61.9s**.
The spread is 4.6s,
 so a single run resolves nothing smaller than about five seconds,
 and every
step in this document before this section was a single run.

What survives that:

- 3m04.7s to 1m50.7s,
 from the earlier session.
- 1m50.7s to 70.7s,
 the declaration-file decode skip net of the classification guard.
 Forty
seconds,
   far outside the band.
- The decode-cache change,
 67.3s to 58.3s.
 Nine seconds is only twice the band,
 but it does not
rest on the sweep:
 a project revisit after two discoveries costs 98.6ms with the clear and
0.5ms without it,
 measured directly with a control.

What does not survive it,
 and is withdrawn:

- The project-selection fix "saving 3.4s".
 Noise.
- The snapshot fingerprint "costing 6.2s".
 Noise.
 That number was quoted in a decision put to
the user,
 and it should not have been.
- Weak retention plus the file-list move "saving 3.1s".
 Noise.
 The change is justified by what
it retains,
   not by that figure.

Any future comparison of two configurations here needs repeated runs of each,
 not one of each.
The band was never measured until after five changes had been attributed to differences inside
it.

The one step that clears it easily,
 1m50.7s to 70.7s,
 is the declaration-file decode skip net of
the classification guard,
 which withholds publications the store used to make and so gives back
some of what `f2eea0182` won.
Both are in that measurement and the correctness change is not optional,
 so they are not worth
separating.

Diagnostics were compared as text rather than as counts:
 6795 diagnostic lines extracted and
sorted from two consecutive runs,
 `diff` empty.

### The diagnostic baseline, and how to regenerate it

Any change to caching or scope is verified against this,
 never against a count and never against
a faster run.

```sh
mise run lint:oxlint > sweep.txt 2>&1
rg --no-line-number '^\s+[x!]\s' sweep.txt | sort > diag.txt
sha256sum diag.txt
rg 'prefer-readonly-parameter-type' diag.txt | sha256sum
rg --multiline --count 'Parameter "(\{[^"]*)?[^"]*" should be readonly' sweep.txt
```

At `2883e6b30`:

- all diagnostics,
 sorted:
 `388defaeca1c99e7ef711d8e4048ab879a15f7e1c374d7b383c2b86896e6c8ea`,
 6795 lines
- rule diagnostics,
 sorted:
 `852b54ef0097dfb8f2e3a3858beac93574ce0f5e601e807fa64da304725a23ed`,
 1555 lines
- offers:
 35,
 being 27 whose parameter name is on one line and 8 whose destructured name is not

Count offers with the multiline form.
A plain `rg 'should be readonly'` answers 27,
 because a destructured parameter's name spans
lines,
 and 27 was very nearly recorded here as a change from 35 that had not happened.

### Reverted

`433135885`,
 reusing the all-disk project fingerprint,
 reverted in `144b97d51`.
Correct and byte-identical,
 but measured at 0.3 per cent reuse:
 the fingerprint is derived once
per worker-project pair,
 so a per-project memo is consulted exactly as often as it is filled.

### The decode cache TypeScript already keeps, and the call that emptied it

Landed in `05b2c415e`.
Warm 67.3s to 58.3s and cold 247.0s to 207.5s,
 both single runs,
 diagnostics byte-identical to
the pinned digests.
The sweep is corroboration rather than the evidence;
 the evidence is the probe below.

`SourceFileCache` in `typescript/dist/api/sourceFileCache.js` is keyed by path,
 parse options and
**content hash**,
 and `API.updateSnapshot` calls `retainForSnapshot` to carry entries from the
previous snapshot forward for every file the server did not report as changed.
So TypeScript already keeps decoded sources across snapshot replacement,
 which is precisely what
churn would otherwise cost.

`openSemanticFile` calls `api.clearSourceFileCache()` before every snapshot update it performs,
and that method is `this.sourceFileCache.clear()`:
 it empties the whole store,
 for every project,
on every discovery.
Measured cost of refilling it for one project is 154.6ms against 0.6ms for a pass that finds it
already there.

Measured both ways with the same probe,
 the first pass acting as control:

```text
                                    without the clear   with it
first visit (control, must be slow)      158.4ms        167.2ms
immediate revisit, same Project             0.5ms          0.5ms
revisit after two discoveries               0.5ms         98.6ms
```

The control stays slow in both,
 so the probe is seeing decodes rather than a warm process.

**The safety argument is not that the store is hashed.**
That was the first reading and it is wrong:
 `getRetained` matches on the retained reference and
never rechecks the content hash.
What makes it safe is that the only text this bridge changes is the active file's overlay,
 and it
reports that file through `fileChanges`,
 so retention excludes it and it alone is refetched.

The one text the server is never told about is the previously active file,
 whose overlay is
dropped and whose content reverts to disk.
Oxlint reads from disk and hands the bridge what it read,
 so the two agree.
An editor integration handing an unsaved buffer would break that agreement,
 and would break it
with or without the clear,
 since clearing the client store only refetches the text the server
still holds.
That case wants the outgoing file reported as changed,
 not a cache emptied.

### Where the time went at 1m50.7s, before any of the changes above

Superseded.
Kept because it is what pointed at the derivation,
 and because three of its five lines have since
been changed out from under it.
Warm,
 inside the rule,
 summed across parallel workers,
 measured when the sweep was 1m50.7s:

- scope derivation 58.8s,
 82.8ms for each of 710 first touches
- project fingerprint 15.0s,
 20.7ms each,
 which is `readFileSync` over the whole project
- verification 13.3s,
 6.4ms per file
- memo fast path 1.7s,
 1.21ms per call
- demand index and propagation 0.4s

Those summed to the independently measured index and verification totals,
 so nothing was
unaccounted at that point.

What has changed since,
 and why none of these numbers should be quoted as current:

- the scope derivation no longer decodes the two thirds of each project it discards
- it no longer re-decodes anything on a return to a project,
 so the "first touch" cost is a first
touch per process rather than per snapshot generation
- the fingerprint reads analysed sources from the snapshot rather than from disk,
 leaving only
declarations on `readFileSync`

Re-attributing warm time inside the rule has not been redone since.
Anyone doing it should read "What these timings can and cannot support" first.

### The open question

Whether anything here is worth optimising further,
 and the test to apply first is **how many
times an answer is asked for,
 not what it costs to compute**.
The two memos that worked cached values consulted far more often than derived.

### What the derivation actually costs, and why the memo key died

Both halves of the earlier answer here were wrong,
 and each was wrong in a way the measurement
that produced it could not show.

**The cost is decoding sources,
 not reading four thousand files.**
A configured project holds several hundred files rather than four thousand:
 573 for this plugin,
 371 for `logger/core`,
 874 for `config/oxlint`.
`indexedSourceFileMap` asked `program.getSourceFile` for every one of them so it could read
`isDeclarationFile`,
 and that call decodes an abstract syntax tree.
Measured at 270 microseconds each,
 154.6ms over 573 files,
 against 0.6ms for a second pass once
they are decoded.
Two thirds of what it decoded it discarded on the next line.

**The memo key died;
 the answer was not asked once.**
`openSemanticFile` replaces the whole snapshot whenever a file needs project discovery,
 and takes
`Project` objects from the new snapshot.
Returning to a project after any later discovery therefore yields a different object,
 and
`inclusionScopeByProject`,
 `finalIndexesByProject`,
 `settledClassificationsByProject` and the
reverted fingerprint memo are all keyed on that object.

Measured directly,
 with the control first:

```text
0. array-or-throw.ts        -> or-throw/core  first visit
1. async-iterable-or-throw  -> or-throw/core  SAME object     <- control
2. create-logger.ts         -> logger/core    first visit
3. array-or-throw.ts        -> or-throw/core  NEW object
4. ascii.ts                 -> const/core     first visit
5. array-or-throw.ts        -> or-throw/core  NEW object
6. create-logger.ts         -> logger/core    NEW object
```

Two files of one project in a row share the object,
 so the probe can see identity.
After any new discovery,
 every earlier project comes back as a new object.
One discovery invalidates every cache for every project,
 not only for the project discovered.

So the sentence this section used to end with,
 that reducing the derivation meant fewer
worker-project pairs and was Oxlint's file distribution rather than this rule,
 is withdrawn.
It is snapshot generations,
 and this rule's own bridge creates them.

### Claims withdrawn, with their causes

1. A 47.3 per cent cache miss rate:
 the probe edited the plugin,
 which is in the linted
workspace and feeds the cache key.
2. `source` at ninety-five per cent of phases:
 single instrumented run,
 so it measured
repopulation.
3. No waste left inside the rule:
 verification was classified by its position in the call tree
rather than opened.
4. The fingerprint memo being worth fifteen seconds:
 cost measured,
 reuse ratio never
measured.
5. Five separate timing attributions of three to six seconds each,
 made from one run per
configuration before the run-to-run band was ever measured.
It is 4.6s.
Listed under "What these timings can and cannot support".
6. Reducing the scope derivation meaning fewer worker-project pairs,
 and being Oxlint's file
distribution rather than this rule:
 the 0.3 per cent reuse was read as a property of the value,
when it was a property of the key.
`Project` objects are replaced on every project discovery,
 so the memo was not consulted once
per fill,
 it was emptied.

Each is explained where it appears rather than merely replaced.

The fourth and sixth share one cause.
A reuse ratio was measured and believed without asking what the key was,
 and the key was the
thing that moved.
"How many times is the answer asked for" is only half the test.
The other half is **whether the store you would ask still exists when you ask**.

### Why the churn itself was never fixed

`Project` object churn is real and was never removed.
What was removed is its cost,
 which is the part worth removing.

A return to an earlier project used to mean re-deriving the inclusion scope,
 re-reading the whole
project from disk to fingerprint it,
 and rebuilding the demand index.
The first is now about three times cheaper and no longer decodes what it discards;
 the second no
longer re-decodes anything,
 since the decoded sources survive snapshot replacement;
 the third was
measured at 0.4s in total across the whole sweep.

So the standing proposal to remove churn by opening every configured project at bridge startup is
withdrawn rather than pending.
It carries a real memory risk,
 one native TypeScript child per worker holding every project,
 and it
would now be buying back something that costs little.
Anyone reopening it should first measure what a return to a project still costs,
 rather than
assuming it costs what it did when this document was opened.

### Two correctness defects found while measuring

Neither is a performance question,
 and one of them was introduced by a performance change here.

**A shared classification could be decided by lint order.**
`f2eea0182` published every finished classification to a per-project store,
 including results
computed while an enclosing type was still being walked.
`classify` answers `HONEST_READONLY` for a type already active above it,
 which only the walk that
assumed it resolves.
On a two-type cycle whose head carries the writable slot,
 so that both types reach a write:

```text
head first:    cycleHead mutable, cycleMember deep-readonly
member first:  cycleMember mutable, cycleHead mutable
```

That is the wrong-offer direction rather than a withheld one.
`effect-outward-handoff.ts:181` returns early on `deep-readonly`,
 skipping the opaque effect it
would otherwise charge the handed slot,
 so the parameter reads as unmutated and the rule offers
`readonly` on something a constructor can write through.
Fixed in `2b86858fe`,
 pinned by a test shown to fail without the guard in `eb1714905`.

Isolated against this repository by removing only the publication condition,
 rebuilding,
 and
sweeping:
 6795 diagnostic lines,
 `diff` empty.
The conclusion that carries is **no type cycle in this repository currently reaches the defect**,
not that the defect never mattered.
The defect is established by the disposable-project probe;
 the sweep measures its blast radius
here and nothing more.

**A file's project can depend on what the worker linted before it.**
`cachedProjectForFile` returns the deepest project root already discovered that contains a file,
which is not the project TypeScript would choose for it.
The repository root `tsconfig.json` declares no `include`.
Measured,
 with the control first:

```text
package file alone:  package/logger/core/tsconfig.json (371 files)
root file first:     tsconfig.json (646 files)
```

The same source is analysed under a different project,
 with a different inclusion scope,
 decided
by whether that worker happened to lint a root-level file earlier.

Whether it can hide a caller is set containment rather than set size,
 and it does.
Restricted to sources the rule would analyse,
 the package project holds 117 and the root project
196,
 and **62 of the 117 are absent from the root**:
 `async-time/core`,
 `const/core`,
`numeric-format/core` and others the logger imports.
Those 62 are callers,
 and a caller that is not read is a mutation that is not charged.

Fixed in `f84c5f487`.
The upward walk now stops at the first ancestor that either names a discovered project or declares
an undiscovered one,
 so the answer is a function of the filesystem rather than of lint order.
`openSemanticFile` already checks that the project it receives contains the source,
 which is why
this went unnoticed:
 that check catches a wrong answer for every project except the one whose
`include` is absent.

Swept after the fix:
 6795 diagnostic lines,
 1555 rule findings,
 35 offers,
 `diff` against the
baseline empty.
So the substitution reached no diagnostic in this run,
 the same shape of result as the
classification defect:
 the path is real and reachable on demand,
 and this repository's file order
did not take it.
Both fixes remove a way for the answer to depend on something that is not the code being linted.

### How to measure this safely

1. Instrumenting the plugin changes the cache key,
 so run twice and read the second run only.
2. Register any probe's writer where every call reaches,
 never on the path being measured.
3. Recorded spans are summed across workers and exceed wall time;
 only proportions compare.
4. The rule's own phase labels cover a fraction of its warm work.
5. Verify a byte-identical sweep for any change to caching,
 since the failure mode is wrong
diagnostics rather than a slow run.

## What was measured

Warm run,
 with `node_modules/.cache/prefer-readonly-parameter-type` populated by the previous
sweep:
 3m04.7s wall,
 2m36.8s user,
 30.0s system.
The cache is 45 MB.

Cold runs of the same command through this session,
 with that cache deleted first:
 10m24s to
11m10s across eight measurements,
 clustering near 10m45s.

So the persistent summary cache removes roughly seventy per cent of the work,
 and the warm
figure is about three times the target rather than the eleven times a cold number implies.

Findings are identical between the warm and cold runs,
 1555 in both,
 with the same error and
warning totals.
That matters before trusting any timing:
 a cache that changed results would make the fast number
meaningless,
 and this one does not.

## Which three minutes

Measured by turning the one rule off in `oxlint.config.ts`,
 keeping the same task,
 invocation
and warm cache,
 and putting the file back afterwards.

Warm run with `prefer-readonly-parameter-type/prefer-readonly-parameter-types` disabled:
**13.0s** wall,
 against 3m04.7s with it on,
 and zero findings from it confirming it really was
off.

So this rule is about 171 of the 184 seconds,
 roughly ninety-three per cent of warm whole-repo
lint time.
Everything else oxlint does across the repository,
 core passes and every other plugin together,
is thirteen seconds.

That reframes issue #374 entirely.
The target is not a broad performance problem to chase across the linter;
 it is one rule,
 and
the rest of the run already sits comfortably inside sixty seconds with two thirds of the budget
to spare.
Reaching the target means taking this rule from 171 seconds to about 47,
 a factor of three and
a half,
 with no other component needing to change.

## What this does not yet say

### How to measure the next split, and what tripped

`EffectAnalysisBudget` in
`package/oxlint-plugin/prefer-readonly-parameter-type/src/prefer-readonly-parameter-types/effect-analysis-budget.ts`
already receives every timed span through `record({ startedAt, phase, },)`,
 and already knows
the phase label.
It accumulates only a single total,
 so the breakdown needs a temporary map keyed by the first
word of the label plus a dump on process exit.
That instrumentation type-checks and is the shortest path to the answer.

What tripped is the harness rather than the method:
 the run takes about three minutes and a
foreground command is cut off at two,
 which killed the workers before the exit handler wrote
anything and left the instrumented file in place until it was restored deliberately.
Run it in the background.

Only five sites report a phase at present,
 so the breakdown will account for part of the 171
seconds rather than all of it,
 and the size of the unaccounted remainder is itself the useful
number:
 it says how much sits outside every span the rule currently times.

### Which phase, measured

Warm run with per-phase accumulation,
 summed across every worker:

- `source` 583.3s
- `project` 23.4s
- `fixed-point` 4.5s
- `foreign` 0.8s
- 611.9s recorded in total

Two cautions before reading anything into those,
 both of which change what the numbers mean.

The total exceeds the rule's 171 seconds of wall time because the sweep runs workers in
parallel and each contributes its own spans,
 so these are worker-seconds and only their
*proportions* are meaningful.
And only five sites report a phase,
 so 611.9 worker-seconds is the total of instrumented spans
rather than of the rule's work;
 what falls outside them cannot be derived from this,
 because
wall time and summed time are not comparable under parallelism.

Within what is instrumented,
 `source` is 95.3 per cent and nothing else is close.

That is the phase to look at next,
 and the sharper question it raises is why it costs so much on
a *warm* cache:
 the cache exists to avoid re-deriving summaries,
 and a source phase still
dominating afterwards suggests either that it covers work the cache does not cover,
 or that
reuse is missing more often than intended.
Answering that needs the phase broken down further rather than another total.

### What the earlier suspects were


The suspects are the whole-program summary construction,
 the checker queries each summary
makes,
 and the per-file diagnostic pass,
 and nothing here separates them.

Nor does it say what the cache is worth per phase.
Cold costs 645 seconds against 184 warm,
 so 461 seconds are avoided by reuse,
 but whether the
remaining 171 is mostly unavoidable analysis or mostly cache misses is unmeasured.

## Why the cold number was the one being quoted

Every sweep in `doc/planning/prefer-readonly-return-substitution.md` deletes the cache first,
deliberately:
 a warm cache masked an analysis-internal failure earlier in that work,
 and the
guarded failure there is a wrong offer rather than a slow run.

Correct for that purpose and misleading for this one.
Anyone reading those eleven-minute figures as the cost of linting this repository would
conclude the target is an order of magnitude away.
It is closer to three times,
 and the first question is which three minutes.

### Why `source` costs so much warm, measured and then doubted

Instrumented the hit and miss branches inside `loadSource` and ran the warm sweep:
 4899 calls,
2584 hits and 2315 misses,
 a 47.3 per cent miss rate on an unchanged repository.

That would answer the question cleanly.
Nearly half of all source loads redo semantic analysis despite a populated cache,
 which is
where the seconds would be.

It is not safe to report yet,
 and the reason is the instrumentation itself.
Measuring required editing `effect-demand-index.ts`,
 which is plugin source.
If the layered summary cache keys on plugin identity or content in any way,
 then editing the
plugin invalidated every entry and the miss rate is an artifact of the act of measuring rather
than a property of the rule.

A quick search for a plugin version or hash in the cache key found nothing,
 which is weak
evidence against the confound and not enough:
 absence of a match for the names guessed is not
absence of the mechanism.

To settle it,
 either read `layered-summary-cache` and establish exactly what the key contains,
or measure without touching plugin source,
 for instance by counting from the cache directory's
own contents across two consecutive unmodified runs.
Until one of those is done the 47.3 per cent should be treated as unmeasured.

This is the same failure this repository's `QPC` and `QIV` rules were written for,
 reached
from a new direction:
 not an instrument that could not see a difference,
 but one that created
the difference it then reported.

#### Settled: the miss rate was an artifact

Read the key rather than guessing at its field names.

`effect-summaries.ts` builds
`projectDigest = contentDigest(`${projectFingerprint.digest}\0${analysisRoot ??
 ''}`,)`,
 and
`effect-summary-cache.ts` stores each entry under that digest together with the file's own
`sourceText`.

Both halves are reached by editing a plugin file.
The plugin's own sources sit in the workspace this command lints,
 so changing
`effect-demand-index.ts` changes that file's `sourceText` directly and feeds a project-level
fingerprint that is not per-file.

So the 47.3 per cent miss rate measured the cost of having just edited the analyzer,
 not the
cost of running it.
Withdrawn.

The earlier search for a plugin version or hash found nothing and was read as weak evidence
*against* the confound.
It was evidence of nothing:
 the mechanism is there under names that describe the project
rather than the plugin,
 and the plugin is part of the project.
Guessing field names is not reading a key.

What remains true and unaffected:
 the warm run is 3m04.7s,
 the rule is ninety-three per cent
of it,
 and `source` is ninety-five per cent of instrumented spans.
Those came from runs with no plugin edit in place.

Measuring the warm miss rate needs a method that touches no plugin source,
 such as reading the
cache directory's own contents across two consecutive unmodified runs.

#### Measured without touching the plugin: the cache is not the problem

Two consecutive `mise run lint:oxlint` runs with the working tree clean,
 comparing the cache
directory's own contents by path,
 modification time and size.

- 4104 entries after the first run,
 4124 after the second
- 4095 identical across both
- 29 rewritten or added

A hit does not rewrite its entry,
 so 29 of 4124 is an upper bound on misses:
 about 0.7 per
cent,
 and lower still if any of those rewrites are ordinary bookkeeping rather than
recomputation.

That refutes the withdrawn 47.3 per cent outright,
 by a method with no plugin edit anywhere
near it.
Cache reuse on an unchanged repository is essentially complete.

So the 171 seconds are not cache misses,
 and improving the hit rate is the wrong target.
The time is spent on work that happens *despite* hits:
 candidates are the cache read and
validation path itself,
 which runs for every one of those 4000-odd entries,
 and whatever
per-source work sits inside the `source` span outside `loadSource`'s cached branch.

That is the next thing to separate,
 and the method is now established:
 measure from outside the
plugin,
 or accept that any edit to it invalidates what is being measured.

#### The cache read is not it either

Bounded from outside the plugin,
 by a standalone script reading and parsing every file in the
cache directory:
 4124 files,
 71.1 MB,
 **0.26s** single-threaded,
 with all 4124 parsing as JSON.

So deserialising the entire cache costs a quarter of a second against 171 seconds of rule time.
The read and validate path is eliminated as the explanation,
 and with cache misses already
eliminated at 0.7 per cent,
 both candidates named in the previous section are gone.

One number worth carrying to whoever continues:
 the confounded run counted 4899 `loadSource`
calls against 4124 cache files on disk.
If that call count survives re-measurement by an uncontaminated method,
 then roughly 775 loads,
about sixteen per cent,
 have no cache entry to hit at all,
 and full analysis of those would be
a candidate the phase labels do not currently distinguish from cached ones.
It is a lead rather than a finding,
 since its source is the run that was withdrawn.

What is established for issue #374,
 all of it from runs with no plugin edit in place:
 warm is
3m04.7s,
 the rule is ninety-three per cent of it,
 `source` is ninety-five per cent of
instrumented spans,
 cache reuse is essentially complete,
 and reading the cache is free.
The remaining time is per-source work done despite hits,
 and separating it needs instrumentation
inside the plugin,
 which is precisely what the cache key makes self-defeating.
Resolving that tension is the first problem for whoever takes this further,
 not the profiling.

#### The tension resolves by running the instrumented build twice

The obstacle was that instrumenting the plugin changes the cache key,
 so a probe measures its
own edit.
The fix needs no cleverness:
 run the instrumented build twice.
The first run repopulates the cache under the new fingerprint and the second is genuinely warm
relative to that code,
 with the counters differing from production only by two integers.

- Run A,
 repopulating:
 4778 calls,
 51.5 per cent hits,
 48.5 per cent misses
- Run B,
 warm:
 4764 calls,
 **100.0 per cent hits,
 zero misses**

Run A reproduces the withdrawn 47.3 per cent almost exactly,
 which confirms what that number
was:
 the cost of repopulating after an edit,
 measured once and mistaken for steady state.

Run B is the answer.
Every single one of 4764 source loads hits,
 and the run still takes 2m49s.

So the rule's warm cost is entirely work done *despite* a perfect hit rate.
With deserialising the whole cache measured at 0.26s,
 it is not reading the entries either.
Whatever `loadSource` does with a hit beyond reading it,
 validation and reconstruction being
the obvious candidates,
 or work in the `source` span outside `loadSource` altogether,
 is where
the 171 seconds are.

That is a much smaller haystack than this section started with,
 and the method for searching it
is now established rather than blocked.

#### Withdrawn too: `source` is not ninety-five per cent

The phase split recorded earlier came from a single instrumented run,
 which is the same
mistake as the miss rate and was not noticed at the time.
Re-measured with the two-run method:

- Run A,
 repopulating:
 `source` 595.0s (95.7 per cent),
 `project` 21.2s,
 `fixed-point` 4.6s,
`foreign` 0.8s
- Run B,
 warm:
 `project` 21.8s (87.3 per cent),
 `source` **2.0s** (8.0 per cent),
 `foreign`
0.8s,
 `fixed-point` 0.4s

Run A reproduces the withdrawn 95.3 per cent,
 confirming its origin.
Warm,
 `source` is two worker-seconds,
 not five hundred and eighty-three,
 and the phase that
led every earlier conclusion in this document leads nothing.

The larger fact is the total.
Warm,
 every instrumented span in the rule sums to **25.0 worker-seconds** while the rule takes
about 169 seconds of wall time,
 across workers that together have far more than 25 seconds of
work to do.

So the rule's own phase instrumentation accounts for almost none of its warm cost.
The time is outside every span the analyzer currently times,
 which points at the per-file
diagnostic pass or setup that no `record` call wraps,
 rather than at anything measured so far.

That is where the search should start,
 and it is not where any earlier section of this document
pointed.

#### What survives, and what does not

Sound:
 warm total 3m04.7s,
 ninety-three per cent of it attributable to this rule by disabling
it,
 cache hit rate 100 per cent warm,
 cache deserialisation 0.26s.
Each came from runs whose instrumentation could not affect what they measured.

Withdrawn:
 the 47.3 per cent miss rate,
 and the ninety-five per cent `source` share.
Both came from single instrumented runs and both are now explained rather than merely
replaced,
 which is the only reason to trust the corrections more than the originals.

Two of the five substantive numbers in this document were wrong in the same way,
 and the second
survived a reading pass that was looking for exactly that fault.

#### Where to instrument next, and the shape the edit must take

The target is the rule's per-file entry:
 `Program` in
`package/oxlint-plugin/prefer-readonly-parameter-type/src/prefer-readonly-parameter-types.ts`,
which opens a semantic file session inside a `try` and is wrapped by no `record` call.
Timing entry to exit there,
 summed and compared against the rule's 169 warm seconds,
 says
directly whether the unaccounted time is inside the rule body or in setup around it.

An attempt at that edit failed and is worth describing so the next one does not repeat it.
The visitor's body is a single `try` whose `catch` uses a logger bound above it,
 so splicing a
timing `try` around the outside by text substitution detaches that binding and the file stops
compiling with `Cannot find name 'rl'`.
The edit needs to add accounting *inside* the existing `try` and a `finally`,
 not wrap it.

Run it with the two-run method like everything else here,
 and read run B only.

The three cautions this document has earned,
 in the order they bit:

1. Instrumenting the plugin changes the cache key,
 because the key derives from a
project-level fingerprint and the plugin's sources are in the linted workspace.
Run twice and read the second run.
2. Recorded spans are summed across parallel workers,
 so they exceed wall time and only
proportions are comparable.
3. The rule's own phase labels cover a small fraction of its warm work,
 so a breakdown of them
is not a breakdown of the rule.

#### Found: the time is inside `Program`, outside every phase span

Instrumented the per-file entry as described,
 accounting inside the existing `try` with a
`finally`,
 and run twice.

- Run A,
 repopulating:
 636.7 worker-seconds across 2080 file visits,
 306.1ms per file
- Run B,
 warm:
 **172.3 worker-seconds across 2080 file visits,
 82.8ms per file**

The rule contributes about 171 seconds to the warm wall time,
 and `Program` accounts for 172.3
worker-seconds of it,
 so the per-file visitor is essentially the whole cost rather than a part
of it.

Set that against the 25.0 worker-seconds all instrumented phases sum to,
 and roughly **147
worker-seconds,
 about eighty-five per cent,
 sits inside `Program` and outside every span the
analyzer times.**

That is the answer this section has been circling.
The warm cost is not summary construction,
 not cache misses,
 not reading the cache,
 and not
any labelled phase.
It is the per-file work that happens after `buildEffectSummaryIndex` returns:
 opening the
semantic file session,
 and verifying each callable to decide what to report.

Eighty-three milliseconds per file across 2080 files is the shape of the target,
 and the
repopulating figure of 306ms per file gives the contrast:
 even with every summary already
built,
 each file still costs a quarter of what a full analysis costs.

Whoever optimizes this should start by splitting those 82.8ms between session opening and
per-callable verification,
 which is one more `finally` in the same visitor and the same two-run
method.

#### The split: index construction dominates even at a perfect hit rate

Same visitor,
 accounting around `openSemanticFile` and `buildEffectSummaryIndex` with the
remainder attributed to verification,
 run twice.

Run B,
 warm,
 172.1 worker-seconds across 2080 files:

- `buildEffectSummaryIndex` **104.0s,
 60.4 per cent,
 50.0ms per file**
- verification of callables 56.5s,
 32.8 per cent,
 27.2ms per file
- `openSemanticFile` 11.5s,
 6.7 per cent,
 5.5ms per file

Run A,
 repopulating,
 for contrast:
 546.6s index,
 54.3s verification,
 11.0s session.

Two things follow.

The index build is the target.
It costs 104 warm seconds *with every summary a cache hit*,
 down from 546 when they are not,
so the cache already removes eighty-one per cent of it and fifty milliseconds per file remain
regardless.
That residue is per-file index construction rather than analysis:
 assembling and wiring an
index from summaries already in hand.

And it reconciles the earlier finding.
Instrumented phases sum to 25.0 warm worker-seconds while `buildEffectSummaryIndex` alone takes
104.0,
 so roughly seventy-nine seconds are spent inside that call in code no `record` wraps.
The untimed eighty-five per cent is not scattered:
 it is nearly all inside index construction.

Verification is second at 27.2ms per file and worth knowing about,
 since it barely moves
between the two runs,
 11.0s against 11.5s for the session and 54.3s against 56.5s for
verification.
Both are insensitive to the cache,
 which is expected:
 they work from the index rather than
building it.

#### Reaching sixty seconds

The rule must go from about 171 seconds to about 47 for the whole command to fit the target.
Removing index construction entirely would leave roughly 68 warm seconds of rule time,
 which is
still over.
So the target is not reachable by fixing one of these three alone,
 and any plan for #374 has to
say which combination it intends.

#### The index is reused; deciding whether to reuse it is what costs

`buildEffectSummaryIndex` does hold a whole-index cache,
 `cachedFinalEffectIndex`,
 with an
early return.
So the index is not simply rebuilt per file.
Measured warm,
 across 2080 calls:

- final-index hits **64.4 per cent**,
 1340 of 2080
- work performed *before* the cache check:
 **63.8 worker-seconds,
 30.7ms per call**

That is 61 per cent of the 104 warm seconds spent deciding whether the cached index applies,
paid on every file whether it hits or not.

The pre-check work is the key computation itself:
 collecting file names,
 building
`indexedSourceFileMap`,
 then `contentDigest` over every indexed file name sorted and joined.
The digest is over the whole file list,
 so its cost scales with project size and is repeated
once per linted file:
 2080 files each hashing a list of some four thousand names.

So the answer to "is the index rebuilt per file" is no,
 and the useful finding is the one
underneath it:
 a cache whose lookup costs 30.7ms cannot pay for itself at this call frequency,
and roughly 64 of the 171 warm seconds are spent on cache administration rather than on
analysis or on reporting.

The remaining 35.6 per cent of calls miss and build the index fully,
 which is the other half of
the 104 seconds.

Two independent directions follow,
 and both are now grounded rather than guessed:
 make the
key cheaper,
 since the file list does not change within a run and its digest could be computed
once per worker;
 or raise the hit rate above 64 per cent,
 since a third of calls still rebuild.
The first looks like the larger and safer win,
 and neither has been attempted.

#### What a cheaper key has to respect

The obvious fix,
 computing the file-list digest once per worker,
 is not sound as stated,
 and
the reason is worth recording before anyone writes it.

`fileNames` is the program's source file names *plus* `activeSourceFile.fileName`,
 sorted.
The digest is then taken over `indexedSourceFiles.keys()`,
 which is that list after
`indexedSourceFileMap` applies the ownership policy,
 not over `fileNames` directly.

So the digest is genuinely per-active-file in general,
 and memoising it on the project key
alone would reuse an index built for a different inclusion scope.
That is a wrong-index bug rather than a slow one,
 in a rule whose failure mode is a wrong
read-only offer.

What makes the optimisation plausible anyway is that the active file is almost always already
in `project.program.getSourceFileNames()`,
 so adding it changes nothing and the sorted list is
identical across every file of that project.
That is also why the final-index cache hits 64.4 per cent rather than never.

A sound version therefore needs a condition,
 something like:
 reuse the memoised digest only
when the active file was already among the program's names *and* the ownership filter's result
does not depend on which file is active.
The second half is the part to establish rather than assume,
 by reading `indexedSourceFileMap`.

Not implemented here.
The finding is sound and the fix is not yet safe to write,
 and the gap between those two is
where this session's errors have consistently lived.

#### The filter does not depend on the active file, so the memo is sound

Read `indexedSourceFileMap` rather than assuming,
 since this was the blocking unknown.

For every file *other* than the active one the result is independent of which file is active.
`activeSourceFile` appears twice and both uses concern that file alone:
 once to substitute the
Oxlint overlay for its own entry,
 and once to include it unconditionally,
 ahead of the
declaration-file test,
 the project checks,
 `isWorkspaceSourceFileName` and the `analysisRoot`
prefix.

So the key set is exactly:
 the ownership-filtered set,
 which is a property of the project,
union the active file.

That yields a sound design rather than a hopeful one.
Memoise the *filtered map* per project key,
 not the digest,
 because building the map is the
expensive part:
 a `flatMap` over some four thousand names,
 repeated for each of 2080 files.
Then per call,
 test whether the active file is already in the memoised set,
 which is O(1).
When it is,
 and it usually is for workspace sources since `isWorkspaceSourceFileName` admits
them,
 the digest is the memoised one and nothing is recomputed.
When it is not,
 digest the union,
 which is the rare path.

That removes most of the 63.8 warm worker-seconds now spent before the cache check,
 without
changing which index is selected for any file:
 the set it keys on is identical either way,
 by
the argument above rather than by measurement.

Still not implemented,
 and the remaining risk is no longer soundness but placement:
 the memo
must be keyed on something that cannot outlive the project it describes,
 and picking that key
wrongly is the same wrong-index bug by another route.

#### Placement is settled by precedent, so the fix is fully specified

The last stated risk was where to keep the memo without it outliving the project it describes.
`effect-final-index-cache.ts` already answers that:
 `finalIndexesByProject` is a process-level
`Map` keyed on the project *object*,
 then partitioned by `projectKey`.
Project identity bounds the entry,
 since a new semantic snapshot is a new project and reaches
none of the old entries.

The filtered-map memo belongs in exactly that shape,
 keyed the same way and living beside it.
Nothing new has to be decided about lifetime,
 and following the existing structure is also what
keeps a fourth caching layer from acquiring a fourth invalidation story.

So the change is now fully specified:

1. Memoise the ownership-filtered map per project object and `projectKey`,
 beside
`finalIndexesByProject`.
2. Per call,
 test whether the active file is in the memoised set;
 reuse its digest when it is
and digest the union when it is not.
3. Sound because the filter is active-file independent,
 which
"The filter does not depend on the active file" establishes by reading it.
4. Expected to remove most of 63.8 warm worker-seconds,
 which is the measured cost of the work
before the cache check.

What remains is writing and verifying it,
 which means the unit suite,
 the two-run timing to
confirm the saving,
 and a full sweep to confirm the diagnostic set is byte-identical,
 since a
wrong-index bug would show there as changed findings rather than as a slow run.

### Landed: deriving the inclusion scope once per project

Implemented as specified,
 and measured.

Warm whole-repo lint **2m22.4s against 3m04.7s**,
 saving 42 seconds,
 twenty-three per cent of
the command.
That tracks the prediction:
 30.7ms per call across 2080 calls is about 64 worker-seconds of key
derivation,
 most of which is skipped once the scope is reused.

Findings and offers are **byte-identical**,
 1555 and 35,
 matching the sweep before the change.
That is the test that mattered.
The failure mode for this optimisation is selecting an index built for a different inclusion
scope,
 which produces wrong diagnostics rather than a slow run,
 so the unit suite passing was
never sufficient on its own.

Two details decided whether it was safe,
 and both are in the code's own comments rather than
only here.
Only the key set and digest are stored,
 never the map,
 because the map's values hold the overlay
`SourceFile` of whichever run built it.
And the store is keyed on the project object,
 as `effect-final-index-cache.ts` keys its own,
 so
a new semantic snapshot reaches none of the entries.

#### What is left for #374

Warm is 2m22s against a sixty-second target,
 so 2.4 times over rather than 3.
The attribution says the rest is verification,
 around 56.5 warm seconds,
 and the 35.6 per cent
of index calls that still miss and rebuild.
Neither has been attempted,
 and the arithmetic recorded in "Reaching sixty seconds" still
holds:
 no single remaining component closes the gap alone.

### The remaining derivations are first-touch, not scope mismatches

Instrumented the memo after it landed,
 warm,
 2080 calls:

- scope memo present:
 1358,
 65.3 per cent
- active file already in that scope:
 **1358**,
 the same number
- final index hit on it:
 1356
- full derivation still needed:
 724,
 34.8 per cent

The middle two being identical is the result.
Whenever a scope has been derived for a project partition,
 the active file is in it every
single time,
 so the case the memo declines to cover,
 an active file outside the project's
filtered set,
 does not arise in this repository at all.

The 724 remaining derivations are therefore first touches:
 calls where no scope exists yet for
that partition.
Their number is a property of how work is distributed rather than of the cache,
 since each
worker holds its own module state and must derive once per project it is handed.
Two thousand and eighty files across some hundreds of package partitions and several workers
gives roughly that count.

So the memo has reached its ceiling as designed.
Sixty-five per cent is not a hit rate to be improved by better keying;
 it is one derivation per
worker per project,
 and the only ways below it are fewer partitions per worker or state shared
between workers,
 which are scheduling questions rather than caching ones.

That also revises what "the 35.6 per cent of index calls that still miss" meant in
"Reaching sixty seconds".
Those misses are not repeated work on the same scope.
They are the irreducible first derivation for each partition a worker sees,
 and closing them
means changing how oxlint distributes files,
 which is outside this rule.

### The last component is productive work, not overhead

Split the verification pass,
 warm,
 2076 files and 667608 collected AST nodes:

- `collectAstNodes` **0.0s**,
 0.0ms per file
- `verifyReadonlyCallable` **40.3s**,
 19.4ms per file

The walk is free.
Collecting two thirds of a million nodes costs nothing measurable,
 because they are already
materialised by the time this asks for them.
That eliminates the one candidate here that would have been cheap to fix.

What remains is the rule deciding what to report,
 nineteen milliseconds per file,
 and it is
the first component measured in this document that is not overhead.
The index key derivation was pure waste and was removed.
The remaining index derivations are a scheduling artefact.
The walk is free.
Verification is the analysis itself:
 building each parameter's facts and demanding the proofs
before emitting anything.

So the attribution is complete,
 and it ends somewhere different from where it started.
There is no further waste to remove inside this rule.
Going faster from here means computing less,
 which means deciding what the rule may stop
proving,
 and that is a correctness conversation rather than a performance one.

Warm now stands at 2m22s against a sixty-second target.
Verification at roughly 40 seconds and the per-worker first derivations are what remain,
 and
neither yields to a cache.

### Correction: "productive work" was an inference, and it reopened

The previous section concluded there was no waste left inside this rule.
That was reached from where verification sits in the call tree,
 not from measuring inside it,
which is the same fault as the two withdrawn figures wearing different clothes:
 a conclusion
about a component nobody had opened.

Split it,
 warm,
 8337 callables:

- `readonlyParameterFacts` **35.0s,
 4.2ms per callable**,
 ninety-three per cent of verification
- reporting decisions 2.7s,
 0.33ms per callable

Four point two milliseconds inside one function,
 per callable,
 is not self-evidently
irreducible.
It is where parameter types are classified,
 and type classification across a codebase repeats
enormously:
 the same `Row`,
 the same `readonly string[]`,
 the same `AbortSignal`,
 resolved
again for every callable that mentions them.

So the conclusion that reaching sixty seconds requires computing less is withdrawn as premature.
It may still be true.
It is not established,
 and the obvious thing to establish first is whether classification
results are being recomputed for types already seen,
 which would be waste of exactly the kind
already removed once from the index key.

The lever to try is memoisation keyed on type identity,
 and the measurement to take first is
how often the same type is classified more than once.
Neither has been attempted.

That this section exists is the finding worth keeping.
The attribution was declared complete three times before it was,
 each time because a component
was classified by its position rather than by opening it.

### The repeat is visible in the source: the classifier memo is per-call

No measurement needed for the first half of the question.
`classifyReadonlyType` in
`package/oxlint-plugin/prefer-readonly-parameter-type/src/prefer-readonly-parameter-types/readonly-classifier.ts`
opens with:

```ts
const memo = new Map<number, ReadonlyClassification | typeof CLASSIFICATION_ACTIVE>();
```

A fresh map per call.
It is keyed by semantic type ID and does its job within one traversal,
 which is cycle handling
for a recursive walk,
 and it retains nothing between calls.

So every parameter re-classifies its whole type graph from scratch.
A `Row` mentioned by two hundred callables is walked two hundred times,
 and each walk redoes
every nested property,
 element and signature beneath it.
That is consistent with 4.2ms per callable in `readonlyParameterFacts`,
 and it is waste of the
same kind already removed from the index key:
 a memo whose scope is narrower than the
repetition it faces.

The lever is therefore to widen that memo's lifetime rather than to compute less.

What has to hold for it to be safe,
 and what to check before writing it:

1. Type IDs are stable only within one checker or project instance,
 so the shared store must be
keyed on that,
    as `effect-final-index-cache.ts` and the inclusion-scope memo already are.
2. `CLASSIFICATION_ACTIVE` is a traversal marker,
 not a result,
 and must never be published to
a later call.
3. Whatever the classification depends on besides the type itself must be part of the key.
`classifyReadonlyType` takes `checker` and `project` alongside `type`,
 and whether the result
can differ for one type across those is the thing to establish by reading rather than assuming,
because getting it wrong returns a classification from a different semantic world.

Point three is the one that decides it,
 and it is exactly the shape of the question that made
the inclusion-scope memo safe:
 does the result depend on anything but the key.

#### Point three holds: the classification depends on nothing but the type

Read rather than assumed,
 since this was the deciding condition.

Every use of `checker` and `project` inside `classifyReadonlyType` is derived from the type
being classified:
 `handle.resolve(project,)` on that type's own declaration handle,
`checker.getBaseConstraintOfType(current,)`,
 `checker.isArrayType(current,)`,
`checker.getTypeArguments(current,)`.
None consults anything outside the type.

So the classification is a pure function of the type,
 computed through the checker that owns
it.
Type IDs are meaningful only within one checker instance,
 and that is the same boundary the
result is valid within,
 so keying a shared store on the project and the type ID is exactly
right:
 within that scope the answer depends on nothing else,
 and outside it the key cannot
collide because the store does not span instances.

All three conditions are therefore satisfied,
 and widening the memo is sound.
`CLASSIFICATION_ACTIVE` still must not escape:
 it means "this traversal is currently below
this type",
 which is true of a traversal rather than of a type,
 so only settled results may be
published.

### Landed: sharing settled classifications per project

Warm whole-repo lint **1m50.7s against 2m22.4s**,
 another 32 seconds.
Findings and offers byte-identical at 1555 and 35.

Cumulative across both changes:
 **3m04.7s to 1m50.7s,
 a forty per cent reduction**,
 with the
diagnostic set unchanged throughout.

The byte-identical sweep is what confirms it rather than the unit suite.
A classification leaking across semantic worlds would surface as changed diagnostics,
 not as a
fast run,
 which is the same reason the inclusion-scope memo was gated on a sweep.

#### What this cost to find

Two sections of this document concluded there was no waste left,
 and both were wrong the same
way:
 a component classified by its position in the call tree rather than by opening it.
The second of those was written after the first had been withdrawn for exactly that fault.

The finding needed no measurement in the end.
`classifyReadonlyType` builds its memo per call,
 which is visible on the line that declares it,
and the repetition it faces is a property of any codebase that names a type more than once.
What the measurements bought was knowing which function to open,
 which was worth having;
 what
they could not do was substitute for opening it.

### Re-measured after both changes

Warm,
 2076 files:

- `buildEffectSummaryIndex` **76.0s**,
 36.6ms per file,
 from 104.0s
- verification **13.3s**,
 6.4ms per file,
 from 56.5s

Verification fell by seventy-six per cent,
 which is the shared classification memo doing what it
was written for and confirms the mechanism rather than merely the wall clock.

The rule now spends about 89 warm seconds against roughly 171 before,
 and index construction is
once again the dominant component at eighty-five per cent of it.

That returns to the 724 first-touch derivations.
At 76 seconds across 724 of them the arithmetic is about 105ms each,
 which accounts for
essentially the whole remaining index cost:
 what is left is not repeated work but the first
derivation for each project a worker is handed.

So the two levers already established are the only ones the measurements point at,
 and neither
is a cache:
 make a single derivation cheaper,
 or reduce how many worker-project pairs exist.
The second is oxlint's file distribution rather than this rule.

Warm now stands at 1m50.7s against a sixty-second target.

### A probe inside the derivation produced nothing, cause unknown

Attempted the split inside a single derivation,
 timing the scope and digest work,
 the project
fingerprint,
 and the demand index with its propagation.
It type-checked,
 both runs completed,
 and **no probe output was written at all**.

That is not a null result about the code.
It is a broken instrument,
 and the difference matters:
 724 derivations occur per run,
 so the
counters should have fired hundreds of times.

Not diagnosed.
The likely candidates are the closing timer being anchored on a `return index;` that is not the
path taken,
 or the exit hook never registering on the branch that runs.
Recorded rather than retried because guessing at a second instrument to check the first is how
this document acquired four withdrawn figures.

The tree was restored and verified clean,
 and the unit suite passes,
 so nothing is left behind.

What stands unmeasured as a result:
 the composition of a single derivation,
 currently
estimated at about 105ms from 76 seconds across 724 of them.
That estimate is arithmetic over a total,
 not an observation of one derivation,
 and by the
standard this document has had to adopt twice it should not be trusted until something opens
one successfully.

#### Diagnosed: the probe registered its writer on the path it was measuring

Tested by writing unconditionally at the miss path instead of on exit.
On a single-file lint the miss path executed **zero** times,
 the final index being already
cached.

That is the fault.
The failed probe registered its `process.on('exit')` writer inside the miss path,
 so any
process that never misses never registers a writer and never writes,
 whatever it accumulated.
The three probes that worked in this same file all registered at a path every call reaches.

The instrument's registration was conditional on the thing it was measuring,
 which is the same
shape as everything else this document has had to withdraw,
 and it produced silence rather than
a wrong number.
Silence was the lucky outcome:
 a partial registration across workers would have written a
plausible total from whichever processes happened to miss.

The fix is to register the writer at function entry,
 unconditionally,
 and accumulate only on
the miss path.

Whether that alone explains it is not established.
`process.on('exit')` in a worker thread is not the same as in a process,
 and how oxlint hosts
JS plugins was never checked here.
The three successful probes prove writers can fire at all,
 so at minimum the registration
placement is necessary;
 whether it is sufficient wants the retry rather than an argument.

### Inside a derivation: the project fingerprint, recomputed per derivation

The corrected probe fired.
Warm,
 727 derivations,
 15.5s between the two steps it times:

- project fingerprint **15.0s,
 20.7ms each**
- demand index and propagation 0.4s,
 0.6ms each

Repopulating,
 for contrast:
 15.3s fingerprint and 186.2s demand index,
 245.4ms each.

So once summaries are cached the index build is essentially free,
 and what a warm derivation
costs is the fingerprint.

That is the same defect a third time.
`effectProjectFingerprint` describes the project,
 not the file being linted,
 and it is
recomputed for every derivation of that project.
The inclusion-scope memo and the classification memo were both this:
 a value whose scope is the
project,
 derived once per call.

The lever is the same and its safety question is the same:
 what does the fingerprint depend on
besides the project,
 and is that stable within a run.
It is named `effectProjectFingerprint` and takes the project,
 which is suggestive and not
evidence;
 the earlier two were only safe because reading established that the result depended
on nothing outside the key.

One caution against over-reading this.
The two timed steps sum to 15.5s while index construction measures about 76s warm,
 so roughly
sixty seconds sit in the scope and digest work ahead of them,
 which this probe does not time.
That is the larger share and the inclusion-scope memo already addresses part of it;
 what
remains there is the derivation path the memo does not cover.

#### The fingerprint does depend on the active file, unlike the previous two

Read it,
 and the answer differs from the earlier memos rather than repeating them.

`effect-project-fingerprint.ts` digests every project file's *text*,
 and for the active file it
substitutes the Oxlint overlay:

```ts
if (fileName === activeSourceFile.fileName)
  return activeSourceFile.text;
```

That is the whole purpose of the overlay,
 so the fingerprint is genuinely per-active-file.
A plain per-project memo would hand a later file a fingerprint computed from a different file's
unsaved text,
 which selects a stale index:
 wrong diagnostics rather than a slow run.

The condition that makes it sound is the same *shape* as the inclusion-scope one and a different
*test*.
Memoise the fingerprint computed from on-disk text,
 and use it only when the active file's
overlay text matches what the project already holds for it.
When they differ,
 which is what an editor overlay means and what a lint of unmodified files does
not produce,
 compute fresh.

That is worth stating because the obvious version of this optimisation is unsafe,
 and the
previous two were safe for reasons that do not carry over.
Three memos,
 three different safety arguments:
 the inclusion scope because the filter ignores
which file is active,
 the classification because it depends only on the type,
 and this one only
under a text-equality condition it must test at each call.

Not implemented.
The remaining unknown is where the other sixty seconds of index construction sit,
 which this
probe does not time and which is the larger share.

### The index accounts fully, and the scope derivation is what is left

Warm:

- memo fast path 1.7s over 1368 calls,
 **1.21ms each**
- entry to derivation 58.8s over 710 calls,
 **82.8ms each**
- project fingerprint 15.0s,
 20.7ms each
- demand index and propagation 0.4s,
 0.6ms each

Those sum to about 76 seconds,
 which is index construction as measured independently,
 so
nothing is unaccounted here any more.

Two readings follow.

The inclusion-scope memo works as intended.
A fast-path call costs 1.21ms against the 30.7ms that deriving the key used to cost,
 and run A
shows the same path at 33.98ms while the memo is still filling,
 which is the before-and-after
in one run.

And what remains is the scope derivation itself:
 58.8 warm seconds building
`indexedSourceFileMap` and digesting its keys,
 82.8ms for each of 710 first touches.
That is roughly twenty microseconds per file across some four thousand files,
 repeated once per
worker-project pair.

So the remaining index cost is not repetition a cache can remove.
It is the first construction of a project's scope,
 paid once per worker that touches that
project,
 and the levers are making that construction cheaper or reducing how many worker-project
pairs exist.
The second is oxlint's file distribution,
 as established earlier.

The fingerprint at 15.0s is the one piece with a known and different remedy,
 under the
text-equality condition recorded above,
 and it is a fifth of what the scope derivation costs.

#### Why the fingerprint costs 20.7ms: it reads the project from disk

`projectSourceText` in `effect-project-fingerprint.ts` returns the overlay text for the active
file and `readFileSync` for every other file in the project.

So a derivation reads some four thousand files from disk and hashes them,
 once per
worker-project pair.
That is what the 20.7ms and the 15.0 warm seconds are,
 and it is disk I/O rather than
computation.

It also makes the memo condition cheap in a way that was not obvious before reading it.
Establishing that the active file's overlay matches its own disk text costs **one** read,
 and
when it does,
 every text the fingerprint would read is the disk text,
 so a stored all-disk
fingerprint describes the call exactly.
One read to avoid four thousand.

An attempt at that implementation is not in the tree.
It broke on rewriting the return statement and referenced a text helper that does not exist,
was reverted,
 and types and the unit suite are clean.
Recorded rather than retried,
 because the edit needs the file open in front of it rather than
another blind substitution,
 which is the same discipline that produced the two changes that did
land.

The shape for whoever writes it:

1. Module-level `WeakMap<Project, EffectProjectFingerprint>`,
 keyed as the other three stores
are.
2. Compare `activeSourceFile.text` against that one file's disk text.
3. Reuse the stored fingerprint only when they match.
4. Store only from a call where they already matched,
 so what is kept is always the all-disk
fingerprint and never one carrying an overlay.

Worth about 15 warm seconds against the 58.8 the scope derivation costs,
 so it is the smaller
of the two remaining pieces and the only one with a known remedy.

### The fingerprint memo landed and delivered about a second, not fifteen

Warm 1m49.5s against 1m50.7s.
Findings and offers byte-identical,
 so it is correct;
 it is simply not worth what the
component measurement suggested.

The prediction was around fifteen seconds,
 from 20.7ms across 727 derivations,
 and the gap is
undiagnosed.
Two candidates,
 neither tested:

The condition may rarely hold.
`overlayMatchesDisk` compares Oxlint's overlay text against `readFileSync`,
 and those can
differ for reasons that have nothing to do with unsaved edits:
 a byte-order mark,
 line-ending
normalisation,
 or any decoding Oxlint applies before handing text to a plugin.
If it seldom matches,
 the memo is never consulted and the extra read is pure cost.

Or worker-seconds may not convert to wall time at anything near one to one.
Every component figure in this document is summed across parallel workers,
 and fifteen
worker-seconds spread over several of them is a much smaller wall saving.
That would apply to the two earlier changes as well,
 whose wall savings were larger than this
reasoning predicts,
 so the conversion is evidently not uniform and is not understood here.

The measurement to take is whether `overlayMatchesDisk` holds,
 counted the way the other
probes count,
 registering its writer where every call reaches.
Until that is known the change is unjustified rather than wrong:
 it costs one extra read per
derivation and returns about a second,
 and if the condition seldom holds it should be reverted
rather than kept.

Recorded rather than reverted immediately because reverting on an unmeasured guess is the same
error as keeping on one.

#### Answered: the condition always holds, and the memo has nothing to reuse

Measured,
 warm:
 720 fingerprint calls,
 **720 matched disk (100.0 per cent)**,
 and **2 reused
the memo (0.3 per cent)**.

Both candidates in the previous section were wrong.
The overlay never differed from disk,
 so byte-order marks and line endings were not the
problem,
 and the worker-second conversion was not either.

The real reason is structural,
 and it was already in this document.
The fingerprint is computed once per derivation,
 and derivations are one per worker-project
pair.
So a per-project memo is consulted exactly as often as it is filled,
 with nothing left over:
there is no *second* call for the same project in the same worker to hit it.

That makes the change correct,
 useless,
 and slightly costly,
 since it adds one read per
derivation to answer a question whose answer is never needed twice.
Reverted in `144b97d51`.

The lesson is not about fingerprints.
Two earlier memos worked because the value they cached was consulted far more often than it was
derived:
 the inclusion scope 2080 times against 710 derivations,
 the classification once per
parameter against once per type.
This one is consulted once per derivation *by construction*,
 and no measurement of its cost
could have revealed that,
 because the cost was real.
The question that separates a useful memo from a useless one is not what it costs to compute
but how many times the answer is asked for,
 and that ratio is what should be checked first.

`doc/planning/oxlint-warm-sweep-attribution.md` records the cost of not checking it:
 one
implementation,
 one sweep,
 one probe,
 and a revert.
