# pnpr rebuild repeats shared task dependencies

## Static graph finding at 2026-11-04T20:15:12-05:00

The build graph at `8a897f3007593bf82ccbc24eabfae55962266366`
contains no cycle reachable from the pnpr publishing set.
It nevertheless expands shared dependency paths repeatedly:
the complete successful rebuild would dispatch `1,396,751,160` task executions
and `1,396,750,853` build command steps.
These are static call counts,
not completed CI work or a completion-time forecast.

The scope is the `169` names in `package/config/pnpr/config.yaml`:
`154` have build tasks and the other `15` pack without building.
The analyzer read `165` package task files,
resolved every reachable task reference,
and counted `475` distinct reachable tasks with `771` dependency and fanout edges.
Of those tasks,
`168` execute build commands.

Source files and line numbers in this report refer to that immutable commit,
exported with `git archive` before analysis.
The same results were obtained from the upgrade worktree before exporting the snapshot.

## Where repeated execution comes from

`package/monorepo-task/cli/src/load.ts:74`
merges template and package dependency lists.
`package/monorepo-task/cli/src/run.ts:587`
then recursively dispatches every dependency occurrence before its task body.
There is no completed-task cache or active-task cycle guard in that implementation.

`task.ts:234` defines `build` and `build:js` as child-task fanout templates.
The corresponding prelude helper at
`package/monorepo-task/cli/src/run.ts:245`
starts a fresh task CLI subprocess for each immediate child.
Caching imported task modules in `load.ts` does not cache task execution.

The publisher calls a separate CLI for each package build
at `package/config/pnpr/src/publish-steps.ts:75`.
Its package loop at `package/config/pnpr/src/publish-missing-versions.ts:287`
does not make completed task dependencies available to later package builds.

The translation tasks expose the multiplication directly.
For `build:js:node`:

- `package/translation-repair/markup/task.ts:14` has no task dependencies,
  so its expansion has `1` build command.
- `package/translation-repair/document-structure/task.ts:16` depends on `markup`,
  giving `1 + 1 = 2` build commands.
- `package/translation-repair/shared-contract/task.ts:16`
  depends on both `document-structure` and `markup`,
  giving `1 + 2 + 1 = 4` build commands.
- `package/translation-repair/typography/task.ts:16`
  depends on those shared prerequisites again,
  giving `1 + 2 + 1 + 4 = 8` build commands.

`package/translation-repair/coverage-census/task.ts:16`
lists overlapping prerequisites such as `bench`,
`probe`,
`run`,
and their shared dependencies.
Its build alone expands to `685,063,314` task executions
and `685,063,312` build command steps.
Across the complete pnpr rebuild,
the `markup:build:js:node` body occurs `698,375,360` times in the expanded graph.

## Cache scope changes the count

These are conditional graph models of the same source snapshot,
not measurements of a patched runner.
They assume every required command succeeds,
dependencies retain their ordering,
and all visits to a task in one model share the same effective call inputs.

- Current recursive execution:
  `1,396,751,160` task executions and `1,396,750,853` build command steps.
- One shared cache per top-level package build,
  with built-in fanout remaining in that invocation:
  `1,025` task executions and `718` build command steps across the `154` separate CLI invocations.
  The `coverage-census` build accounts for `36` tasks and `34` build commands in this model.
- One cache shared across the entire publisher:
  `475` task executions and `168` build command steps.
  The publisher does not currently provide this shared invocation scope.

The global count must not be promised for a repair that only caches within each package's CLI invocation.
The main upgrade work proposes repairing the existing runner's execution semantics
with an invocation-owned promise cache,
cycle detection,
and in-process built-in fanout.
That implementation is being verified separately.
It is not an adoption of a new task engine or a change to task ownership.

## Calculation and evidence limits

Task declarations were read as data;
no task body,
build tool,
package installation,
or synthetic workload was run for this analysis.
The analyzer expanded `extends`,
explicit `depends`,
and the root templates' direct-child fanout rule.
Every explicit dependency resolved,
and no additional task invocation was found in the inspected reached task bodies.
This is not an analysis of subprocesses that build tools or package lifecycle hooks may start internally.

Kahn's topological traversal consumed the entire reachable graph,
establishing that this graph is acyclic.
For a task `t`,
the task count is `1 + sum(taskCount(child))`;
the build-command count is `ownBuildSteps(t) + sum(buildSteps(child))`.
Fanout wrapper bodies do not count as build commands.
Reverse-topological recurrence and forward path-multiplicity propagation produced identical totals.
The manually checked `markup`,
`document-structure`,
`shared-contract`,
and `typography` expansions matched the computed `1`,
`2`,
`4`,
and `8` build-command counts.
The per-invocation model sums each top-level root's distinct reachable tasks;
the global model counts the union once.

Scratch evidence is retained under `${HOME}/temp/agent/pnpr-alpha17-review/`:
`analyze-task-graph.ts`,
`task-graph-evidence.json`,
and the `task-graph-8a897f3` source snapshot.
The analysis ran under `CPUQuota=200%`,
`MemoryMax=1G`,
`TasksMax=64`,
and `IOWeight=10`.

GitHub job metadata for rebuild run `41944061291`,
job `126971769694`,
showed the build-and-pack step starting at `2026-11-05T00:49:32Z`
and still in progress when inspected.
That observation does not measure completed build calls or establish which task was active.
The main upgrade work cancelled that run after receiving the static finding
and owns the disposable runtime reproduction and repaired CI verification.
