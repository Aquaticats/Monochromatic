# Task-slice contract question

Purpose:
 two decisions are waiting on the user,
 after the scoring-axis correction
 ("Parity with the current status quo is poor scoring axis.
Assume we're fine with huge huge refactors and everything.").
The question UI was cancelled,
 so the questions live here.

What changed:
 [`doc/planning/task-slice-comparison.md`](../planning/task-slice-comparison.md)
 is rescored on intrinsic capability for the slice job
 (commits `49ef3fcac9`,
 `d9fea09f5c`,
 `2338d2b297`).
Migration and shape cost score nothing.
Documentation trust is a scored deduction of `4 - trust` points.

What to inspect:
 the "Scoring axis",
 "Ratings",
 "Ranking for the task slice",
 and "Weight sensitivity" sections of that document.
The full per-adjacent-pair justifications live there.

How to respond:
 name a decision option
 (or write your own),
 and the queue resumes.

## Decision 1, task-slice owner

The slice is task discovery,
 per-package addressing,
 `depends` edges,
 task bodies,
 watch,
 failure tolerance,
 and machine-readable listing,
 at 222 packages and 1,811 tasks.

### Ranking

Capability total first,
 then the total after the trust deduction.

1.  Pants 2.33.1 (32, 29)
2.  Please 17.34.0 (26, 24)
3.  Buck2 2026-10-14 (26, 23)
4.  Nadle 0.6.1 (25, 22)
5.  Keep Mise (24,
    trust unaudited)
6.  Tilt 0.37.8 (21, 17)
7.  pnpm 12.16 (19, 16)
8.  projen 0.104.19 (13, 11)
9.  Poe the Poet 0.49.0 (12, 11)
10. meow as designed,
    not scorable

Pants is first under every weighting tested.
The band from Please through Keep Mise
 reorders with the weighting.

### Why each adjacent pair

- Pants over Please (32 to 26).
  Both cover every named duty.
  Pants grades higher on body safety
  (typed argv with no encoding class,
  against `escape_safe` plus a silent `\$` rewrite)
  and on watch cleanliness
  (`plz watch` serves stale `[cached]` test results).
  They tie on edges,
  on failure semantics (3 each),
  and on caching (4 each),
  though the caches differ:
   Pants' measured hits
  with `--changed-since` and `--changed-dependents`,
  Please's input-hash skip
  as the only cross-invocation cache measured.
  Flip condition:
  weight caching durability
  above body safety and watch cleanliness.
- Please and Buck2 tie at 26.
  The tie breaks to Please
  on documentation trust (2 of 4 against 1 of 4)
  and on two of the three weightings
  (duty only 15 to 14,
  duty dominant 41 to 40).
  Buck2 is the better engine where it reaches:
   best failure semantics measured
  (default tolerance,
  JSON `action_error` causes,
  aggregate exit 3)
  and the richest introspection
  (`targets --json`,
  `uquery`/`cquery`,
  `--build-report=-`,
  BXL).
  But Buck2 owns no watch command at all
  (only a protobuf `subscribe` RPC),
  which is a named duty,
  and its local cache measured `Cache hits: 0%`
  across every daemon restart.
  Flip condition:
  treat watch as wrapper-fillable
  and weight failure semantics plus introspection
  above watch and caching.
  This is the controlling preference for the pair.
- Buck2 over Nadle (26 to 25).
  Both duty weightings reverse it
  (Nadle 18 to 14,
  43 to 40).
  Nadle holds the two strongest single cells:
   typed in-process TypeScript bodies
  with zero escaping class,
  and clean watch plus input/output caching with `--why`.
  Nadle's gaps are structural:
   it blocks every dependent of a failure
  at the scheduler level,
  so tests-after-build-failure fails outright,
  and `TaskNotFoundError` exits 3 on any absent task.
  It is also the only fixture-scale,
  one-maintainer candidate.
  Flip condition:
  weight the dev loop
  above scale and maintenance.
- Nadle over Keep Mise (25 to 24).
  The incumbent's two measured holes
  are the ones the comparison set out to close:
   nothing is cached at all,
  and root `mise run test`
  runs only the three root suites
  and misses 15 packages' tests.
  Keep Mise is strong elsewhere
  (watch in daily use across 104 `watch:build` tasks,
  `mise tasks --json` at full corpus size,
  proven at 222 packages).
  Flip condition:
  fix the two holes in place
  and Keep Mise retakes the pair
  with no migration.
- Keep Mise over Tilt (24 to 21).
  Tilt cannot own failure tolerance
  in the mode that exits:
   `tilt ci` aborts at the first failure
  and abandons unstarted resources,
  and `tilt up` never exits with an aggregate status.
  Tilt's strengths are sharp where they exist
  (exact `resource_deps`,
  the best optional-task behavior measured,
  native watch).
  Trust is 0 of 4,
  the only zero,
  and both unacknowledged bugs
  land on surfaces Tilt would own.
  Flip condition:
  give `tilt ci` a keep-going mode
  and any caching at all.
- Tilt over pnpm (21 to 19).
  pnpm cannot express the corpus graph:
   476 of 603 cross-package `depends` edges
  become `^name` over-approximations,
  33 hold only transitively,
  5 are inexpressible.
  Over-approximation schedules wrong work
  and breaks failure semantics.
  pnpm's wins are adoption-side
  (proven at 222 packages,
  `missingScript: true` pass-through,
  large ecosystem).
  Flip condition:
  weight proven scale
  and missing-optional pass-through
  above edge exactness and watch.
- pnpm over projen (19 to 13).
  projen aborts the whole graph
  at the first failure
  and resolves typed cross-project `spawn`
  to the wrong task silently
  (the probe ran `APP build`
  where `LIB build` was the edge).
  No flip on the measured evidence.
- projen over Poe the Poet (13 to 12).
  The pair ties on duty weighting
  and the trust deduction closes the gap
  (both land 11).
  Poe's core is clean
  (exact dedup `deps`,
  shell-free `cmd`).
  But watch,
  caching,
  and listing are absent,
  optional-task robustness
  is the worst measured
  (one missing reference poisons every run),
  and failure semantics
  are the weakest measured.
  Poe holds the field's best trust (3 of 4).
  Flip condition:
  emit stubs for every absent task
  and add watch plus a listing shim.
- meow as designed cannot be ranked.
  Nothing exists to measure.
  The custom-implementation gate still applies:
   a hand-written owner is eligible
  only when every existing tool
  fails a named hard constraint,
  and Pants does not.

### Options

- Pants 2.33.1.
  Pros:
   typed argv bodies,
  exact edges in `peek` JSON,
  `--loop` watch,
  measured cache hits,
  affected selection,
  no named duty absent.
  Cons:
   dependents silently omitted from the summary,
  named missing address poisons `::`,
  30s default timeout.
  Ranked first under every weighting.
- Please 17.34.0.
  Pros:
   only measured cross-invocation cache,
  real `--keep_going`,
  monorepo-grade scale,
  exact `deps`.
  Cons:
   `escape_safe` discipline required,
  `\$` rewrite trap,
  `plz watch` stale `[cached]` results,
  `--since` mutates the live worktree.
  Ranked second.
- Buck2 2026-10-14.
  Pros:
   best failure semantics,
  richest introspection,
  Meta backing.
  Cons:
   no watch command,
  cache dies with the daemon,
  three escaping layers for bodies.
  Ranked third;
  second if watch is wrapper-fillable.
- Nadle 0.6.1.
  Pros:
   typed in-process bodies with no escaping class,
  clean watch,
  caching with `--why`,
  typed agent causes.
  Cons:
   dependents blocked on failure,
  `TaskNotFoundError` on absent tasks,
  fixture scale,
  one maintainer.
  Ranked fourth on equal weights;
  second on both duty weightings.
- Keep Mise.
  Pros:
   watch in daily use,
  `mise tasks --json` at full scale,
  proven at 222 packages,
  no migration.
  Cons:
   nothing cached,
  root `mise run test` misses 15 packages.
  Ranked fifth on capability.
- Tilt 0.37.8.
  Pros:
   exact `resource_deps`,
  best optional-task behavior,
  native watch.
  Cons:
   no failure tolerance in the exiting mode,
  no caching,
  trust 0 of 4.
  Ranked sixth.
- pnpm 12.16.
  Pros:
   proven at 222 packages,
  missing-script pass-through,
  large ecosystem.
  Cons:
   edge model cannot express the corpus graph,
  no watch,
  no measured byte-exact body form.
  Ranked seventh.
- projen 0.104.19.
  Pros:
   `execArgs` byte-exact,
  versioned `tasks.json` manifest.
  Cons:
   silent wrong-task spawn,
  aborts at first failure,
  no parallel step,
  no watch or caching.
  Ranked eighth.
- Poe the Poet 0.49.0.
  Pros:
   exact dedup `deps`,
  shell-free `cmd`,
  best trust.
  Cons:
   three named duties absent,
  one missing ref poisons every run.
  Ranked ninth.
- meow as designed.
  Pros:
   its design targets exactly
  the duties this axis rewards.
  Cons:
   nothing exists to measure,
  and the custom-implementation gate
  is not satisfied while Pants clears it.
  Not scorable.
- Hold and decide later.
  Nothing is adopted,
  no queue item fires.

## Decision 2, `AGENTS.md` tightening

Spotted failure mode:
 the comparison was scored
 on fidelity to the incumbent's shapes
 instead of capability for the target workload,
 and the axis was never asked.
`GAP` calls for a tightening of an existing rule.

Options,
 ranked:

- Tighten `OPT`.
  Append `Score capability for the target workload,
  never fidelity to the incumbent.`
  177 characters after normalization,
  inside the 50-word and 200-character cap.
  Ranks first:
   it names the scoring failure
  at the rule that produced the artifact.
- Tighten `ASK`.
  Append `refactor appetite vs shape fidelity`
  to the values list.
  186 characters,
  inside the cap.
  Ranks second:
   it fixes the unasked value at the root
  but leaves the scoring habit unaddressed.
- Both,
  `ASK` then `OPT`.
  Ranks third:
   complete,
  but two rule edits for one failure.
- Decline.
  Ranks last:
   `GAP` wants a tightening
  after a spotted failure mode.

## How to respond

Write the two answers,
 for example:

```text
1. Pants
2. Tighten OPT
```

Neither answer adopts anything.
Adoption still waits for explicit acceptance
 (`doc/decision/` record
 plus a reopened vet for the chosen candidate).
