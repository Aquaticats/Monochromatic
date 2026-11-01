# Audit ledger: code no unit test runs, final census and what stays cold

The final whole-suite census of 2026-10-10,
the two clusters cut from it,
the entry tails round,
the last census,
and the stretches that stay cold with the reason for each.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## Final whole-suite census at `9d558da59`

Final whole-suite census and its last cold stretches (2026-10-10 UTC):
a whole-suite census was taken at `9d558da592162f9765fb03a2280af9477ada147d` (`census-2W1rHH`),
in a worktree holding no key,
as the user unit `tr-census-final`
(memory capped at 10G,
peak 10G,
5 min 25 s),
with the unit suite at 2,032 passes.
Its report reads:
library source,
22 files,
59 stretches over 110 lines,
0 functions never called;
entry files,
5 files,
5 stretches over 13 lines,
0 functions never called;
invariant throws counted apart,
91 stretches,
all in library source;
and 0 bundles no test loaded.
Other packages' sources add 62 files,
322 stretches over 3,501 lines,
which are theirs to test.
The passage on the census at `65bd3a645` expected the final census
to list only the `translate-slice.ts` stretch in library source;
this one listed 59.

## The 64 stretches given to two clusters

The 64 stretches went in two clusters,
cut by file,
to agents in side checkouts,
with the `translate-slice.ts` stretch held back:
the markup readers,
37 stretches in 12 library files,
and the runner and git stretches,
26 stretches in 14 files,
9 of them library source and 5 entry files.
The lead merged each patch with its cases committed apart from its production change,
and the passages "The markup readers' cluster of the final census"
and "The runner and git cluster of the final census" give what each closed.
Together they closed 50:
the markup readers 28 of their 37,
all library source,
and the runner and git cluster 22 of its 26,
21 in library source and one in an entry file.

## Census at `f5ea0872b` after both clusters

The census after both,
at `f5ea0872bf9f4e8d551b92c64aeba925cf1933e2` (`census-LFRrQs`),
with the unit suite at 2,033 passes,
reads:
library source,
4 files,
10 stretches over 30 lines,
0 functions never called;
entry files,
4 files,
4 stretches over 5 lines,
0 functions never called;
invariant throws counted apart,
98 stretches,
all in library source,
seven more than before,
one for each `unreachable:` throw the two clusters added;
and 0 bundles no test loaded.
The lead's gate on `f5ea0872bf9f4e8d551b92c64aeba925cf1933e2` passed:
the suite at 2,033 passing and 0 failing,
the lint clean,
and the 49 source scans passing.
These pass counts are counts of named suites,
not of cases.

## What stays cold at the end, and why

What stays cold,
and why:

- `footnote-unpositioned-runs.ts` 242,
  245,
  665,
  673 and 688,
  and `footnote-protected-ranges.ts` 75:
  `FootnoteRewriteError` refusals of the `position` kind,
  which the relabel pass turns into a withheld relabel that keeps the archive;
  the markup readers' agent made them `unreachable:` throws,
  and the lead kept them as they were (M143);
- `footnote-unpositioned-runs.ts` 552 and 561:
  the bracket-count refusals of `zoneByRawOffset`,
  for which no input was found and no proof stands;
- `parse-mdx.ts` 731:
  `parseMarkdownBody`'s rethrow of a fault that is no stack exhaustion,
  a propagation B100 decided;
- `translate-slice.ts` 366 to 386:
  the quote-loss refusal,
  held by the lead's ruling of 2026-10-06 in the passage on the census at `65bd3a645`,
  which still awaits the owner's word and was given to no agent;
- the entry tails of `corpus-run/damage-sample.ts` 44,
  `corpus-run/model-health.ts` 30,
  `corpus-run/roster-bench.ts` 66 and `corpus-run/editor-width-probe.ts` 47 to 48,
  each the rest of `main` after a run that needs a provider's answer,
  which no keyless child gets.

## The markup readers' cluster of the final census

The markup readers' cluster of the final census (2026-10-10 UTC):
cases in `f4df3d6f1923886c63d218d794d0efb96340dddc`,
production code in `40665630f2c096260d987736521970b828e89a0b`.
The cluster held 37 stretches in 12 files:
`mdx-tag-name.ts`,
`corpus-run/tag-attributes.ts`,
`inline-container-tags.ts`,
`mask-container-tags.ts`,
`nesting-html-block.ts`,
`nesting-inline-count.ts`,
`nesting-line-lexing.ts`,
`parse-mdx.ts`,
`footnote-graph-findings.ts`,
`footnote-parsed-spans.ts`,
`footnote-unpositioned-runs.ts` and `footnote-protected-ranges.ts`.
Of those,
28 are closed and 9 stay cold.

Fifteen are cased through each module's exported surface:
`mdx-tag-name.ts` 449 to 450,
556,
627 to 630 and 644 to 645;
`corpus-run/tag-attributes.ts` 137 to 139 and 317 to 318;
`inline-container-tags.ts` 139;
`mask-container-tags.ts` 362 and 509;
`nesting-html-block.ts` 80;
`nesting-line-lexing.ts` 299 to 300;
and `parse-mdx.ts` 62,
68 to 69,
75 to 76 and 156.
The inputs:
a prefixed or local tag name followed by a character the grammar refuses,
an attribute run the text ends inside,
a lone inline opener or closer whose tag never ends,
a line that is an angle bracket and an exclamation mark alone,
a footnote label the line ends inside,
and `MdxParseError` built with a cause that is no error,
an error naming a rule without a source,
and one naming neither.
By the agent's mutation rounds,
each case failed under a named mutation of its stretch.
All 43 named test files pass against the unchanged production code,
since each case reaches a stretch that already ran correctly.

Nine are dead code,
gone with code that has no such branch:
seven index reads of the form `x[i] ?? ''`,
at `nesting-line-lexing.ts` 132,
168,
261,
294 and 341,
`nesting-html-block.ts` 98 and `nesting-inline-count.ts` 375,
each in range by its loop or the check before it,
became `charAt`,
which returns the same unit and an empty text out of range;
the `?? 0` of `definedTwice` in `footnote-graph-findings.ts` 121,
which read a count the same function had just set,
became a set of repeated keys read off the counts;
and the push of an attribute's expression value in `membersOf` (`parse-mdx.ts` 467) is removed,
since `mdast-util-mdx-jsx` 3.2.0 builds that value with its type,
its text and its `data` alone and no position,
and a case pins those three keys,
so a later version that gives the value a position fails it.

Four are `unreachable:` throws,
written as three guards:
a member of `countDropped`'s walk that is no object (`parse-mdx.ts` 425 and 445),
one guard in the walk's loop,
`positionOfNode` gone and `membersOf` taking a record;
a link label child with no position in `outsideLabelSpans` (`footnote-parsed-spans.ts` 122),
which had answered `[]` without a word,
though only the autolink-literal transform builds a node without a position
and it skips text under a link or a link reference
(`mdast-util-gfm-autolink-literal` 2.0.1 and `mdast-util-find-and-replace` 3.0.2);
and an ordered marker read where no digit stands (`nesting-line-lexing.ts` 249),
which had answered zero,
though its one caller,
`markerEndOf`,
reads it only at a digit.
The census at `f5ea0872bf9f4e8d551b92c64aeba925cf1933e2` lists them among the invariant throws,
at `parse-mdx.ts` 502 to 505,
`footnote-parsed-spans.ts` 123 to 126 and `nesting-line-lexing.ts` 253.
`countDropped`'s TSDoc had said it throws for any node without a position;
it passes over a node the transform built,
and the TSDoc now says so.

The agent's patch held 14 files,
8 of production code and 6 of tests;
the lead's merge took 12 and restored `footnote-unpositioned-runs.ts` and `footnote-protected-ranges.ts` from the base,
so both stand as before the cluster.
The agent had turned six `FootnoteRewriteError({ kind: 'position' })` refusals into `unreachable:` errors:
`runLeaves` twice (242 and 245),
`runEdges` three times (665,
673 and 688)
and `footnoteProtectedRanges` once (75).
The only production catch of `FootnoteRewriteError` is `relabelArchiveFootnotes`
(`corpus-run/pass-footnote-relabel.ts`),
through `requireFootnoteRewriteRefusal` (`footnote-rewrite-error.ts`),
which returns the archive unchanged with `withheld: 'rewrite-validation'`;
every other caller of `activeFootnoteMarkers`,
`unpositionedRuns`,
`fragmentReadingOf` and `footnoteProtectedRanges` lets both classes propagate,
by the lead's search.
So the change would have altered nothing off the relabel path,
and on it would have turned a withheld relabel into a stopped entry,
against the owner's rule that a run always ships (`doc/design-commitments.md` (moved on 2026-10-18 to
`package/translation-repair/doc/design-commitments.md`));
the lead kept the refusals (M143).
`footnote-parsed-spans.ts` is not on the relabel path
(its only production importer beside the footnote barrel is `footnote-mentions.ts`),
so its new throw changes no outcome.
The agent's proofs for the six rest on the sources of `mdast-util-gfm-autolink-literal` 2.0.1,
`mdast-util-find-and-replace` 3.0.2 and `mdast-util-mdx-jsx` 3.2.0,
and for the parent of a type the transform never splits text in,
on mdast's phrasing content and a probe:
inference,
not a named refusal.

Left open:
`footnote-unpositioned-runs.ts` 552 and 561,
the bracket-count refusals in `zoneByRawOffset`:
the agent ran 4,026 texts through `fragmentReadingOf` and `parseDocument` and none threw,
though no positive control showed the probe reaching the runs,
and `character-entities` 2.0.2 decodes only `lbrack`,
`lsqb`,
`rbrack` and `rsqb` to a square bracket;
no proof either way.
And `parse-mdx.ts` 731,
the rethrow of a fault that is no stack exhaustion (B100).

Run on the merged tree before the production commit:
lint clean,
49 scans passing,
and the suite at 2,032 passing with no failure.
No defect was found in this cluster.

Open to the owner's veto:
the six `FootnoteRewriteError` refusals stay,
leaving their six stretches cold,
where the agent's version,
a crash on a shape no input builds,
is the alternative that closes them;
the `parse-mdx.ts` cases build `MdxParseError` with causes its `unknown` type admits but the grammar never throws;
`x[i] ?? ''` became `charAt` rather than `nonNullishOrThrow`;
the duplicate count became a set of repeated keys rather than an explicit throw;
and the mutation that proves the case of `corpus-run/tag-attributes.ts` 139 throws,
since every value mutation there still ends in `NO_TAG`.

## The runner and git cluster of the final census

The runner and git cluster of the final census (2026-10-10 UTC):
red in `5ba14c371fdc57996fb2aba3fdffb32758f62b33`,
fixed in `f5ea0872bf9f4e8d551b92c64aeba925cf1933e2`.
The cluster held 26 stretches in 14 files,
9 of them library source
(`corpus-run/artifact-pool.ts`,
`corpus-run/cli-refusal.ts`,
`task-runner-guard.ts`,
`local-program-failure.ts`,
`corpus-git-context.ts`,
`corpus-commit-probe.ts`,
`corpus-run/runs-lock.ts`,
`credential-search.ts` and `corpus-run/judge-fidelity-probe-run.ts`)
and 5 entry files
(`corpus-run/editor-standing-read.ts`,
`corpus-run/damage-sample.ts`,
`corpus-run/model-health.ts`,
`corpus-run/roster-bench.ts` and `corpus-run/editor-width-probe.ts`).
Of those,
22 are closed and the 4 entry tails stay cold.

Seventeen are cased through each module's exported surface or the command as built,
each,
by the agent's mutation rounds,
failing under a named mutation of its stretch:

- `corpus-run/artifact-pool.ts` 171,
  184 to 185,
  185 to 187,
  188 and 195:
  `resolvePool` called with no `names` under a policy requiring a commit named by a revision;
  the case reads this checkout's `HEAD` and `HEAD~1` read-only through `fixtureGit`,
  since `resolvePool` has no repository seam,
  so it needs a checkout of two commits or more;
- `corpus-run/cli-refusal.ts` 131 and 139:
  a thrown value that is no error,
  and an error with no stack;
- `task-runner-guard.ts` 81 and 105:
  a marker shaped as a command name;
- `local-program-failure.ts` 48:
  a code's length;
- `corpus-git-context.ts` 153:
  a clone directory named too long to resolve,
  whose `ENAMETOOLONG` is not among `UNRESOLVED_CLONE_CODES`;
- `corpus-commit-probe.ts` 199 to 204 and 306 to 310:
  both probes failing,
  through a new mode of the file's fake git
  that dies on the commit and then ends with a status the probe does not read;
- `corpus-run/runs-lock.ts` 181 to 183,
  a filesystem that refuses the link,
  through a `link` stub rejecting with `EPERM` and `syncBuiltinESMExports`;
  351 to 352,
  a host without `/proc`,
  through a stub refusing every `readFile` under `/proc/`;
  and 214 to 217,
  a real runs directory of 4,060 units,
  where the case found B390;
- `corpus-run/editor-standing-read.ts` 355 to 362:
  as built,
  rounds of a seat's digest that drew no ballot,
  one in the singular and two in the plural.

Five stretches are four `unreachable:` throws:
`linkFallbacks` (`credential-search.ts` 270),
whose silent `? 0` arm read a state falling back to itself,
which would keep `stepFrom` in its loop for ever;
`requestedSpecs` (`corpus-run/judge-fidelity-probe-run.ts` 104 to 107),
whose bare rethrow of anything but a `FidelityReferenceError`,
left by B382,
answered a class nothing it calls throws;
`gitStopsAtTheClone` (`corpus-git-context.ts` 106 and 108),
which answered `false`,
so that its caller rethrew,
for a rejection of `realpath` that is no error carrying a code,
where the agent's probe on Node 26.10.0 got an error with a string code for `ENOENT`,
`ENOTDIR`,
`ELOOP`,
`ENAMETOOLONG`,
a NUL byte and an empty path;
and `probeGitDir` (`corpus-commit-probe.ts` 197),
whose rethrow met a raw throw of nano-spawn's preparation of the call,
which reads the working directory Node keeps until a `process.chdir` no production code makes,
and names the command parts the commit probe had just named without a throw.
The last three keep the caught value as their cause,
as the throw of B292 does;
`linkFallbacks` catches nothing,
so its throw has none,
where the fix commit's message says all four keep their cause.
None of the three rethrows had a caller that matched the original class:
each handed an unclassified value on as a fault,
and the wrapped error is a fault just the same,
by the lead's reading.
The census at `f5ea0872bf9f4e8d551b92c64aeba925cf1933e2` lists the four among the invariant throws,
at `credential-search.ts` 275,
`corpus-run/judge-fidelity-probe-run.ts` 104 to 114,
`corpus-git-context.ts` 112 to 115 and `corpus-commit-probe.ts` 204 to 207.

Left open:
the entry tails of `corpus-run/damage-sample.ts` 44,
`corpus-run/model-health.ts` 30,
`corpus-run/roster-bench.ts` 66 and `corpus-run/editor-width-probe.ts` 47 to 48,
each the rest of `main` after an awaited run that needs a provider's answer.
`editor-width-probe` also builds its client with `createRunClient()` before `resolveRunsDir()`,
so a child holding no key never reaches `runEditorWidthProbe`;
handing it a factory,
as `roster-bench` hands `newClient`,
would change that.
Other open items:
`editor-standing-read.ts` still holds its whole printer in the entry file,
where the end state the launch wants is a library module with its own cases;
an existing case of `runs-lock.unit.test.ts`,
whose name began "PROPAGATES a filesystem refusal that is not EEXIST",
reaches the `writeFile` refusal with `EACCES`,
not the link refusal that name reads as,
and `a13df268ec1189400f0579cd809df8a2a6b1cee0` has since named it for the write refusal it reaches;
and the artifact pool case's read of this checkout.

Red against the unchanged code:
`runs-lock.unit.test.ts` alone fails,
with 3 FAIL lines (B390),
and the other nine named test files pass.
Run on the merged tree before the fix commit:
lint clean,
49 scans passing,
and the suite at 2,033 passing with no failure,
one more than before for the suite the runs lock cases added.

Open to the owner's veto:
the four `unreachable:` throws,
where keeping the silent arm and the bare rethrows is the alternative;
the two stubs of `corpus-run/runs-lock.ts`,
where leaving 181 to 183 and 351 to 352 open is the alternative;
the artifact pool case reading this checkout,
where a required `repository` parameter on `resolvePool`,
handed down through the five pooling entries,
or leaving 184 to 195 open is the alternative;
casing the arm with no `names` while `names` stays optional,
where making it required is the alternative,
since all four production callers pass it;
the `/proc` stub's body differing from the one in `corpus-run/process-identity.unit.test.ts`,
where a shared fixture is the alternative;
and pinning only the first printed line in the artifact pool case,
where pinning the whole printed list is the alternative.
The wording of the runs lock's warning is under B390.

## Runner entry tails round

Closed on 2026-10-10 (UTC) by the runner entry tails round,
red in `cb553634188266b72019e60b6bd285d5746b0d32`,
fixed in `752be22cf8f4d875333bf1d54efdb90955d619a1`:
`editor-width-probe` builds its client from a factory once the draw holds a slice (B391);
`corpus-run/editor-standing-read.ts` is 43 lines of wiring over four role modules,
each with cases of its own (B392);
and the artifact pool case builds a throwaway repository of two commits,
since `resolvePool` takes a required `repository`,
handed down through the five pooling entries (B393),
the alternative the veto on that case named.
The four entry tails stay cold,
each now the closing `},);` of its entry's one awaited call,
after a run that needs a provider's answer:
wiring,
and left as such.
By that round's census over its 24 named test files,
the entry files show 4 files with 4 stretches over 4 lines,
those four closing lines,
and the four new standing modules,
`corpus-run/editor-width-probe-run.ts`,
`corpus-run/artifact-pool.ts` and `corpus-run/artifact-eligible.ts` show no cold stretch;
it read only the suites of its named files,
so its counts are not comparable with the census at `f5ea0872bf9f4e8d551b92c64aeba925cf1933e2`.
Still open to the owner's veto:
the four `unreachable:` throws,
the two stubs of `corpus-run/runs-lock.ts`,
casing the arm with no `names` while `names` stays optional,
the `/proc` stub's body,
and pinning only the first printed line in the artifact pool case,
which the rewritten case still does.
`corpus-run/artifact-eligible.unit.test.ts` still reads this checkout's root commit and `HEAD`,
now open under B393.

## Last whole-suite census of 2026-10-10

The whole-suite census of 2026-10-10 (UTC) at `116bf6f4bba63f7375181dfddfef4e251eb10d34`,
the head after the follow-up round of B391 to B400 and its docs,
ran twice.
The first run (`census-F4BjiD`) stopped on three failing test files:
nine as-built cases,
three in each of the suites `draw-sample as built`,
`displacement-probe as built` and `editor-calibrate as built`,
failed because the logger's own line
`logger internal error: sink verification failed for entry 3: Timed out after 5000ms: sink 3 verify`
reached the stderr of the child each case started,
and each case pins that stderr whole,
by the run's suite log (T9).
The lead's gate on the same code had passed,
the suite at 2,047 passing and 0 failing,
so the census's own load,
an instrumented build with the whole suite under V8 coverage,
brought the line on.
The second run (`census-VnXJNm`) passed,
with the unit suite at 2,047 passes,
and reads:
library source,
4 files,
10 stretches over 30 lines,
0 functions never called;
entry files,
4 files,
4 stretches over 4 lines;
invariant throws counted apart,
98 stretches;
and 0 bundles no test loaded.
Its library files are the four of the census at `f5ea0872bf9f4e8d551b92c64aeba925cf1933e2`,
`translate-slice.ts` with 1 stretch,
`footnote-unpositioned-runs.ts` with 7,
`footnote-protected-ranges.ts` with 1 and `parse-mdx.ts` with 1,
and its entry stretches are the same four entry tails,
now at `corpus-run/damage-sample.ts` 50,
`corpus-run/model-health.ts` 30,
`corpus-run/roster-bench.ts` 60 and `corpus-run/editor-width-probe.ts` 56.
