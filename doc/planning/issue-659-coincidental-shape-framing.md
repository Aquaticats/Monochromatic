# Issue 659 framing review: coincidental-shape dismissals

Review of `Aquaticat/Monochromatic#659`,
titled `chore(slopo): dismiss 22 coincidental-shape duplicate clusters`.
This is a review only.
No issue body was edited,
no suppression was added,
and no source file changed.

## Evidence base

Every claim here was measured in this session.

Issue body:

```bash
# Fetch the body under review.
gh issue view 659 --repo Aquaticat/Monochromatic --json body
```

Report on disk:

- `.slopo.local.dir/index.md`,
   generated 2026-10-07 22:36:47.
- `.slopo.local.dir/cluster-0001.md` through `cluster-1008.md`.
- The tree is gitignored by `.gitignore:11` (`*.local.*`),
   so it is not a durable record.
- `index.md` records only the timestamp:
   no version,
   no config,
   no source revision.

Ignore inventory:
 `slopo.ignore.txt` holds 55 unique hash lines under five category headings.

Tool behaviour,
 read from installed slopo `0.8.0` under
`~/.local/share/mise/installs/pipx-slopo/0.8.0/slopo/lib/python3.13/site-packages/slopo`:

- `config.py:120-121` defaults `ignore_file` to `slopo.ignore.txt`.
- `result/analysis/ignore.py:14-22` strips everything after `#`,
   so comment lines are inert.
- `result/analysis/command.py:65-73` filters after clustering.
   It logs `Ignored {count} previously reviewed clusters.` only when the count is non-zero.
- `result/identity.py:24-30` builds the hash as sha256 over the sorted
   `(file_path, body_hash)` pairs of the cluster's units,
   truncated to 12 hex characters.
- `result/report/markdown/analyze.py:22-23` numbers clusters with `enumerate(clusters, 1)`.
   Cluster ordinals are therefore per-run and are reassigned on every regeneration.

Review ledger:
 `~/temp/agent/slopo-issues/review/ledger.jsonl`,
 1008 rows.
Fields used here:
 `cluster`,
 `hash`,
 `verdict`,
 `concept`,
 `mode`,
 `evidence`,
 `issue`.

Sibling issue bodies,
 all OPEN at review time:
`#613`,
 `#619`,
 `#620`,
 `#626`,
 `#627`,
 `#633`,
 `#641`,
 `#644`,
 `#646`,
 `#653`,
`#656`,
 `#657`,
 `#658`,
 `#660`.

Source spot-checks:

- `package/git-policy/cli/src/trust/typescript-source-capture.ts:54-63`
- `package/rolldown-plugin/import-attributes/src/patterns.ts:82-93`
- `package/cli/markdown-lint/src/fix.ts:55-66`
- `package/fs-path/core/src/root-filesystem.neutral.ts:47-53`
- `package/fs-path/core/src/memory-root-filesystem.ts:245-250`
- `package/oxlint-plugin/test-support/src/index.ts:188-272`
- `package/ssg/aquati.cat/src/lib/jsx-to-html.ts:59-77`
- `package/git-policy/cli/src/hook-dispatch/hook-dispatch-plan.ts:131-145`
- `package/git-policy/cli/src/shadow-repository/shadow-config.ts:63-78`
- `package/hyperscript/core/src/css/values.constructors.ts:297-314,346-365,462-464,482-484`

## What the issue gets right

- All 22 hashes resolve to exactly the cluster ordinals the body cites.
   `cba9d0b07950` is in `cluster-0187.md` and `597c076ea09d` is in `cluster-0989.md`;
   22 of 22.
- None of the 22 appears in `slopo.ignore.txt`,
   so "all hashes here are new" holds for keys.
- The hashes are live today.
   `git log --since='2026-10-07 20:00'` over the 35 files the clusters draw from returns nothing,
   and `git status --porcelain` over the same paths is empty.
- The mechanism is real and native to the tool,
   so the proposal is not fictional.
   slopo reads `slopo.ignore.txt` by default,
   tolerates the comment lines,
   and reports a count.
   Acceptance criterion 3 names a command that exists (`mise.toml:648-649`).
- The spot-checked code claims are accurate:
  - `modulePath` strips at the first `?` unconditionally;
     `stripAttrQuery` strips only at the import-attribute marker.
  - `byOffsetDescending` is descending because fixes apply back to front.
  - One `pathSegments` resolves before splitting;
     the other does not.
  - `fixtureSourceRoot` delegates to `fixturePackageRoot`.
  - `isSafeHtml` tests `{ html }`;
     `isDangerousHtml` tests `{ __html: string }`.
  - `quoteConfigParameter` escapes for shell single quotes;
     `quoteGitConfigValue` escapes for git-config double quotes.
- The out-of-scope list matches the tracker.
   `#656`,
   `#657` and `#658` own the other three dismissal categories,
   and `#619` and `#627` own the path-relation and byte-order comparators.
- Format compliance is exact for this repo's issue conventions:
   the generated-by-AI banner,
   the `## Agent Brief` sections,
   checkbox criteria,
   zero em-dashes and en-dashes in the body,
   labels matching the three siblings,
   and `needs-triage` correct per `doc/agent/triage-labels.md` because criterion 1 is a choice.
- Hash-targeted dismissal is the repo's documented mechanism,
   and the alternative was measured.
   `doc/troubleshooting/slopo-threshold-tuning.md` reports that raising `similarity_threshold`
   to 0.97 cut 356 clusters to 225,
   hiding substantive findings with the incidental one.
   Choosing per-hash entries over a global threshold is the evidence-backed option.
- Caveat 2 is the strongest part of the framing.
   It states that "unrelated work" is a judgement about intent,
   and that each reason is recorded so a reviewer can dispute one reason
   instead of re-deriving the comparison.

## What is wrong, ranked

### 1. Two of the 22 are not coincidental-shape by the review's own ledger

The ledger records C866 (`cssOklch` and `cssOklchFrom`) as `STRUCTURAL-IDIOM`
and C989 (`cssMin` and `cssMax`) as `BOILERPLATE-TRIVIAL`.
Both carry the concept `css-value-constructors`,
and the generator assigned concepts to issues wholesale,
 so both landed here.
The title's count,
 the Summary's "Twenty-two clusters are coincidental shape matches",
and criterion 2's "under the existing `COINCIDENTAL-SHAPE` heading"
all file them under a category their own recorded verdict contradicts.
The body papers over the mismatch with an invented bucket,
 "Constructor families",
whose rationale ("per-function branding is the point") is a structural-idiom argument.
A third category is arguable too:
 `slopo.ignore.txt` lists "plain-vs-from constructors"
inside its `INTENTIONAL-VARIANT` definition,
 which is literally what C866 pairs.
slopo reads only the hash and ignores comments,
so the cost is a human record filed under the wrong heading,
 not a suppression that misbehaves.

### 2. The prose describes 23 clusters while the block proposes 22

C293,
 two offset-carrying Rust `Display` impls,
 appears in Current behavior
under "Different enums or vocabularies" with no hash line and no details group.
The ledger assigns it to `#656` with verdict `BOILERPLATE-TRIVIAL`.
Its stated rationale,
 that the crates already share the JSONC dependency,
removes an objection to extraction rather than supporting dismissal,
and neither impl maps an enum to a vocabulary.
A maintainer reconciling "Twenty-two" against the prose cannot.

### 3. Every proposed comment is truncated before it names both sides

The generator shortens evidence to 130 characters and appends a literal ellipsis.
All 22 comment lines measure 139 or 140 characters and all 22 end with ` ...`.
C187's line stops at `while stripAttr ...`,
so the distinction that justifies the entry is the part that is missing.
Criterion 2 requires "a one-line comment naming the two sides and why they are unrelated",
which the paste-ready block cannot satisfy.
This matters more here than elsewhere:
 `.slopo.local.dir/` is gitignored
and `index.md` records only a timestamp,
so the ignore-file comment and the issue narrative are the only durable rationale.
Existing entries are complete short phrases,
 for example
`# isPrefix/isStrictPrefix complementary predicates, differing only by <= vs <`.
The fix is a concise complete sentence per entry,
 not a wider truncation budget.

### 4. The comments embed per-run cluster ordinals

Each proposed line begins `# C187`,
 `# C295`,
 and so on.
Cluster numbers come from `enumerate(clusters, 1)` at report time,
so after the next regeneration C187 denotes a different cluster.
Pasting the block writes 22 ordinals that silently become wrong.
No existing entry in `slopo.ignore.txt` carries one.
The durable identifier is the hash plus the two sides named by path and symbol.

### 5. No provenance stamp

The body never states the report timestamp,
 the slopo version,
 the embedding model,
the thresholds,
 the index root,
 or the source revision,
and `#659` is one of three dismissal issues that never cite
`doc/troubleshooting/slopo-threshold-tuning.md`.
The hash is a function of unit paths,
 unit bodies,
 and cluster membership,
so its validity is tied to a specific index.
`#656` shows the practice this issue omits:
 it cites the threshold-tuning write-up for C959
and explains that the cluster re-surfaced with a third unit,
 which is why its hash changed.

### 6. Churn is real but narrower than the file list suggests

Measured at unit level,
 0 of the 22 clusters share a code unit
(same path,
 overlapping line range) with a cluster owned by another issue,
and no sibling issue body names any of the 22 symbols.
`pathSegments`,
 `packageJsonLicenseExpression`,
 `longestFirst`,
 `createLine`,
 `packedLine`,
`fixturePackageRoot`,
 `sourceLine`,
 `wireSearch` and `byOffsetDescending`
all return no hit across the fetched bodies.
Two clusters are still exposed:

- C358 has a unit at `package/upstream-fork/deepmerge-ts.fuzz/src/coverage-v8.ts:94-99`.
- C342 has two at `package/upstream-fork/deepmerge-ts.fuzz/src/declared-type-sample.ts:181-184,399-401`.
- `#613` proposes one shared fuzz coverage harness replacing 13 sidecar copies,
   which would move those paths and invalidate both hashes.

The general exposure stands regardless of the sibling queue.
Any edit to a unit body changes the hash,
 and slopo has no dead-entry detection:
`load_ignored` returns a set,
 and the only signal is an aggregate count of matching clusters.
The repo already lived this once,
 when the `packages/` to `package/` rename staled 116 entries,
and `#656` and `#657` carry explicit "refresh superseded hashes" work.
`#659` has no equivalent criterion,
 no ordering statement relative to `#613`,
and no prune or staleness check.

### 7. Criterion 3 is not identity-checkable and the subset case is undefined

`Ignored {count} previously reviewed clusters.` aggregates every matching entry,
so it cannot show that these 22 were the ones ignored,
and it is not printed at all when the count is zero.
A count decrease alone also hides unrelated additions and removals.
A checkable form:
 capture the report's hash set before and after on an otherwise identical index,
and require the removed set to equal exactly the accepted hashes.
Criterion 2 is correctly conditioned on "For an accepted batch",
 criterion 3 is not conditioned,
and neither says what "accept a subset" produces.

### 8. Criterion 2 omits the file's own conventions

The category heading carries a count,
 `# ── COINCIDENTAL-SHAPE (3) ──`,
which becomes wrong once entries are added,
 and the issue never mentions updating it.
The block also introduces 20 per-concept headings where the file uses one heading per category,
and emits `# ── css-value-constructors ──` twice
because C866 and C989 are separated by four other concepts.

### 9. The decision is offered without a recommendation or a decision surface

Criterion 1 lists three outcomes with no pros,
 no cons,
 no ranking,
 and no recommendation.
It does not say where the decision is recorded:
an issue comment,
 `doc/decision/`,
 or the ignore file itself.
For finding 1 the answer is close to determined by the existing taxonomy,
so the issue could have adopted and recorded it rather than leaving it open.

### 10. Caveat 1's deferred ideas point at a record no reviewer can reach

"Both stay dismissed here and are recorded in the ledger" refers to
`~/temp/agent/slopo-issues/review/ledger.jsonl`,
 which lives outside the repository.
The ideas themselves (`compareBy([...keyFns])` and `splitSegments(path)`) survive in the caveat
text,
 so nothing is lost outright,
 but the pointer is unreachable.
The caveat also contradicts the Summary's absolute "there is no common logic to extract"
for at least two of the 22.
`#657` handles the same situation better:
 its caveat names the three clusters carrying an open
question,
 and its criterion 3 requires them to be cross-referenced from their own issues
so the dismissal does not hide them.

### 11. The Summary's inference is stronger than its evidence

"Merging any of these would change behaviour for one side" shows that the current functions
differ,
 not that no behaviour-preserving abstraction exists.
`SYB` requires encoding for the destination grammar at the interpolation point;
it does not require independent implementations.
For C588 and C599 the shared part is `Array.from(value).map(perCharacter).join('')`
and the differing part is the entire escape table,
which is a shallow-abstraction argument rather than a grammar argument.
Both sides live in the same package,
 so the coupling objection does not apply.
For C833 the issue itself proposes the behaviour-preserving split in caveat 1.
The question that decides these entries is whether a shared abstraction would carry policy
or only indirection,
 and the body never asks it.

### 12. "None re-surfaces an existing entry" is proved only for keys

A dismissed cluster can re-surface under a new key when a path,
 a body,
 or the membership
changes.
 That is the failure mode the threshold-tuning doc documents,
and the handover lists roughly 20 such re-surfacings in this same report.
Comparing the 22 against the 55 existing entry comments produced no counterexample,
so the claim is probably true,
 but hash absence cannot establish it.
One supporting fact does hold:
 because none of the 55 existing hashes appears in the current
report,
 ignore filtering removed nothing from it,
so the report is effectively a no-ignore report and the 22 keys are genuinely unreviewed.
That last step was an inference when this review was first written and is now measured:
the section "Actions taken on 2026-10-09" records two probe runs on copies of the index which
produced identical 1008-hash sets with an empty ignore file and with the real one.

### 13. Smaller framing costs

- Current behavior holds a taxonomy dump instead of the current state,
   which is that 22 reviewed clusters still surface in every report
   and are absent from the ignore file.
- Bold-label bullets conflict with `MD6`'s prose convention,
   though all four siblings do it.
- `difficulty:easy` describes pasting,
   not deciding.
- The risk framing is one-sided.
   A wrong dismissal costs detection coverage until the code changes,
   and then becomes a silent dead entry.
   Leaving a non-actionable match visible costs reviewer attention.
   Dismissal is reversible by deleting one line and changes no runtime behaviour.
   Stating both sides would make "leave visible" a real option rather than a residual one.

## Systemic pattern across the four dismissal issues

Concept-level grouping put clusters under headings their ledger verdict contradicts in all four:

- `#656` carries one `TEST-OR-GENERATED` cluster,
   C284.
- `#657` carries three `UNCERTAIN` and two `STRUCTURAL-IDIOM` clusters.
- `#658` carries one `INTENTIONAL-VARIANT` cluster,
   C748.
- `#659` carries the two in finding 1.

Nine clusters in total.
`#657` is the only one that reconciles them in its body,
and its criterion 3 turns the reconciliation into a verifiable obligation.
The other three,
 including `#659`,
 present a category in the title
that their own evidence partly contradicts.

## Verification limits

- The ledger records `mode: full-read` for all 22 clusters.
   That means the reported bodies were read in full,
   not that surrounding callers,
   ownership,
   or existing utilities were inspected.
   Only C293,
   which belongs to `#656`,
   is recorded as `full-read+file-inspected`.
   This audit added independent source inspection for the clusters named in the evidence base,
   not for all 22.
- Whether sharing would be worthwhile for a given pair depends on callers and contracts
   not enumerated here,
   so finding 11 challenges the argument's shape,
   not each verdict.
- The drift check is a git-history and working-tree check over 35 paths.
   It is not a byte comparison of indexed bodies against current source,
   and it cannot detect a membership change that leaves every unit untouched.
- `slopo analyze` was not run.
   Verifying criterion 3 by execution would regenerate `.slopo.local.dir/`,
   which is the evidence under review,
   so it needs a copied database
   and a separate report directory first.

## Options for acting on this

1.  Option A,
     comment the corrections on `#659`.
    Retitle to 20 clusters.
    Reconcile C866 and C989 with their recorded verdicts,
     either moving them to `#658` and `#656`
     or keeping the concept together under separate category headings and saying so.
    Drop or re-point the C293 paragraph.
    Regenerate the 22 comments as complete sentences without cluster ordinals.
    Add a provenance line,
     a set-diff acceptance criterion,
     and the heading-count update.
    Pros:
     it fixes the record where it will be used,
     it keeps one decision surface,
     and the work is bounded.
    Cons:
     it mutates another agent's issue,
     and it leaves the same defects in the three siblings.
2.  Option B,
     correct all four dismissal issues and record the dismissal convention once.
    The convention covers a complete comment,
     no ordinal,
     provenance,
     category reconciliation,
     and set-diff verification.
    It belongs in the `slopo.ignore.txt` header
     or in `doc/troubleshooting/slopo-threshold-tuning.md`.
    Pros:
     it fixes the shared generator defect once,
     and it leaves a consistent record across roughly 150 entries.
    Cons:
     it means four issue edits,
     and the generator lives in scratch outside the repo,
     so its truncation and grouping behaviour must be re-derived or the script adopted.
3.  Option C,
     record this audit only and leave the issues untouched.
    Pros:
     no external mutation,
     and the maintainer decides with the evidence in hand.
    Cons:
     the paste-ready block stays wrong at the point of use,
     and the next agent to work `#659` pastes truncated comments carrying stale ordinals.

Ranking:
 A over B over C.
A beats B because `#659` is the issue under review,
 the correction is bounded,
 and B's extra scope depends on a scratch generator that is not in the repo.
B beats C because the defect sits at the point of use:
 a planning doc does not intercept a maintainer pasting the block,
 and one generator produced all four issues.

## Related repository findings surfaced by this review

- `doc/troubleshooting/slopo-threshold-tuning.md` cites slopo `0.4.0` source paths,
   including `src/slopo/analysis/ignore.py:18-24` for the hash construction
   and `src/slopo/analysis/command.py:63-69` for the ignore filter.
   In installed `0.8.0` these are `slopo/result/identity.py:24-30`
   and `slopo/result/analysis/command.py:65-73`.
   Per `WR6` the moved identifiers need an in-place note giving the current values
   and the move date.
- That doc's title pins `0.4.0` while `mise.toml:213` installs `pipx:slopo` at `latest`,
   so the verified version and the running version can diverge without any signal.

## Actions taken on 2026-10-09

The user asked for all the work needed,
 so option B was executed:
 all four dismissal issues were
corrected,
 the convention was recorded once,
 and both related repository findings were fixed.
Nothing was written to `slopo.ignore.txt`;
 the accept decision in each issue's criterion 1 is still
the maintainer's.

Issues corrected in place,
 each with a comment recording the change and its measurement basis:

- `#659`,
   retitled to `chore(slopo): dismiss 22 clusters (20 coincidental-shape, 1 structural-idiom,
   1 boilerplate-trivial)`.
- `#656`,
   retitled to name 32 boilerplate-trivial entries plus 1 test-or-generated and 6 dead keys.
   Its double claim on C866 and C989 was removed,
   since `#659` proposes those two.
- `#657`,
   retitled to name 47 proposed entries,
   3 held back and 15 dead keys.
- `#658`,
   retitled to name 44 structural-idiom entries plus 1 intentional-variant and 2 dead keys.
- `#647` received a comment recording that its pointer to "the remaining-uncertain issue" does not
   resolve for cluster 562,
   because `#660` covers clusters 218,
   291,
   294,
   352,
   600,
   656 and 912.

What the regenerated blocks changed:

- Comments are complete sentences built from the review ledger's evidence,
   never truncated,
   and none
   ends with an ellipsis.
   All 147 proposed entries pass a check that each comment names two distinct
   units.
- No comment carries a cluster ordinal or a line number.
- Entries are grouped by the category the ledger recorded for that cluster,
   so the 9 clusters that
   concept-based grouping had filed under a contradicting heading now sit under the right one.
- Three clusters recorded `UNCERTAIN` (562,
   622, 721) are held back from `#657`'s proposal entirely
   and listed with their evidence,
   because "competing actions are equally valid" is not grounds for
   suppression.
- One comment quoted a literal triple backtick,
   which closed `#656`'s fenced block early and broke
   the rendering of everything after it.
   The text is spelled out and the builder now rejects any
   block containing a fence.
- Every non-hash line of a block is now a comment,
   including the entry-count trailer,
   which the first regeneration emitted bare.
   A paste test found it:
   appending all four blocks to a copy of `slopo.ignore.txt` and calling
   `load_ignored` on the result loaded 148 tokens instead of 147,
   the extra one being the trailer text.
   After the fix the same test loads exactly the 147 proposed hashes and no other token.
- Each body carries the provenance the report itself lacks,
   and the acceptance criteria verify by
   hash-set diff against a measured baseline instead of the aggregate `Ignored N` log line.

Repository changes committed:

- `doc/troubleshooting/slopo-threshold-tuning.md` gained a `0.8.0` re-verification section,
   the
   in-place path notes `WR6` requires,
   the dead-key measurement,
   the cleanup decision for the 55
   keys,
   and a "Recording a dismissal" convention section that all four issues now cite.
- `doc/handover/slopo-cluster-issue-triage.md` gained a corrections section and retitled entries for
   the four issues,
   kept in the file's existing wrapping style so the additions are not buried in a
   reformat of its 629 pre-existing violations.

New measurements that were not available when the findings were written:

- All 55 keys in `slopo.ignore.txt` are dead against the 2026-10-07 report.
   Two `slopo analyze` runs
   on copies of that index,
   one with an empty ignore file and one with the real file,
   both reported
   `1008 clusters with 4377 units`,
   and the second printed no `Ignored` line.
   Both produced the
   identical 1008-hash set,
   which also proves the report on disk was not ignore-filtered and that it
   is reproducible from that index on a later date.
- The dead keys split into 32 with a ledger-evidenced successor (23 into the four dismissal issues,
   9 into clusters owned by code-change issues where deletion rather than refresh is the right
   action) and 23 with no recorded successor,
   12 of which are the i18n renderer keys `#658`
   discusses.

Still open,
 and not this review's to close:

- The accept decision in each of the four issues.
- A home for the three held-back clusters:
   562's dangling pointer from `#647`,
   and 622 and 721,
   which
   no issue tracks.
- The cleanup decision for the 23 dead keys with no evidenced successor.

A follow-up investigation asked whether these dismissals are right at all,
or whether they normalise duplication and miss small shared utilities.
It is recorded in `doc/planning/slopo-dismissed-cluster-actionability.md`.
Its bearing on this review:
 eight of the eleven judgement-heavy clusters in `#659` stand,
three point at an extraction that belongs to an existing issue or to one file's internal helper,
and the review's method,
 not the tool's threshold,
is what kept a 212-unit idiom family from ever being counted.
