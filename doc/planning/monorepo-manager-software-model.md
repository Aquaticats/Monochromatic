# meow software model exploration

## Status and active scope

Design only.
No implementation is authorized.
The user accepted the resource-aware non-task model and requested continued grilling,
with an implementation-ready 0.x plan as the session's deliverable.
The remaining discussion makes that model's contracts concrete.
The [from-scratch design](monorepo-manager-from-scratch-design.md)
remains the requirements record.

The user rejected Claude's framing:

> Do not accept Claude's framing of what a "task" even is.
> I believe better UX can be had by not thinking of stages that software must go through
> in development as "build task" and so on.

The subsequent state-versus-stages questionnaire was also unhelpful.
The user tentatively preferred state first but emphasized a coherent model of automatically keeping everything
up to date across build,
correctness,
and publication.
That answer did not adopt a new glossary or configuration schema.

The agent then pursued publication-policy questions instead of resolving the non-task model.
The user corrected that detour:

> Do not get off-tracked.
> The publishing subsystem won't exist until 1.x.
> We're still talking about the model of "non-tasks".
>
> Answer:
> push to main.

Publication is absent from 0.x.
Its deferral to 1.x explicitly overrides the earlier session-wide 0.x scope for that subsystem only.
The recorded future trigger is push to `main`.
No publication question blocks the current design.

## Working proposal: package maintenance

Meow continuously maintains the repository rather than waiting for the user to orchestrate recipes.
Its intrinsic responsibilities describe software and what should stay current about it.
They also need eligibility rules:
maintaining the repository does not mean eagerly executing every concern despite blocking failures.
They are not generic commands with privileged names.

For 0.x,
that model must explain the already-required behavior:

- Maintain affected build outputs.
- Maintain current results for checks.
  The accepted default test set is already required;
  the clarified model places linting and type checking alongside tests as intrinsic maintained checks,
  not lesser arbitrary operations.
- Maintain managed-file requirements through the file-enforcer replacement.

This is a coherent responsibility,
not a choice between state and stages.
It can have necessary ordering without making authors specify a universal sequence of tasks.
A check that consumes a bundle needs that bundle;
a source consumer need not wait for an unrelated bundle.
The mechanism for expressing those relationships remains undecided.

### Where linting fits

The user found the package-maintenance explanation almost right and asked where linting fits.
The term correctness left that responsibility implicit.
In this proposal,
checks cover repository policy and style as well as functional correctness:

- Tests establish their runtime assertions.
- Type checking establishes whether the relevant type constraints hold.
- Linting establishes whether the applicable static-analysis and repository-policy rules hold.

All are intrinsic maintained checks.
Meow keeps their results current for affected inputs;
linting is not an explicit one-off operation merely because it produces no build artifact.
A package can have current build outputs and passing tests while still having current lint findings.
Those findings belong in its attention state,
without pretending its outputs were never built.
This is a statement about representing results,
not a recommendation to build eagerly after a blocking lint failure.

Lint checking and lint autofixing have different purposes.
Checking maintains findings;
autofixing changes source.
Including lint in automatically maintained checks does not itself authorize automatic source rewrites.
This distinction does not select an autofix policy.
Managed-file enforcement already owns its declared writes;
it is not blanket permission to rewrite all source files.

Repository evidence:
`mise.toml:590-612` separately defines `lint:oxlint`,
`format:oxlint` with `--fix`,
and `lint:types`.
This confirms distinct consumed responsibilities,
not a requirement to preserve task-shaped definitions in meow.

### Resource-aware progression

The user asked whether a package should proceed to building when linting fails,
from a resource-saving standpoint.
The previous explanation separated result states but left execution eligibility unspecified.

Accepted automatic-upkeep policy,
after the user answered "Okay,
I'm satisfied with what we have here" and requested continued grilling:

- Applicable blocking checks that can examine source run before downstream production.
- A current failure stops that progression for the affected inputs.
  Meow does not start downstream builds merely to make every result current.
- Work genuinely needed to perform the check still runs.
  This is not a global rule that all lint must finish before any build can happen.
- The package reports the failing check and the work blocked by it.
  Blocked downstream tests are not reported as failed.
  Earlier outputs and results retain their actual freshness;
  skipping work does not magically make outdated results current.
- Unrelated packages and independent work are not blocked merely because this package failed.
- Priority and foreground attachment do not imply permission to bypass a blocking check.
  This proposal selects no manual-bypass policy.

Repository prerequisite example:
`mise.toml:350-420` defines `ensureOxlintConfig`,
which builds `package/config/oxlint` when its outputs are absent or outdated;
`lint:oxlint` calls it before invoking the lint wrapper.
Gating that prerequisite on the lint invocation it enables would create a cycle.
This does not authorize speculative builds unrelated to the check's needs.

Rationale for the accepted automatic-upkeep policy:

- Gated progression (accepted):
  pro,
  avoids starting downstream work for inputs already rejected by a blocking check;
  con,
  downstream build or test diagnostics wait until the gate passes.
- Independent concurrent upkeep:
  pro,
  downstream results can arrive without waiting for an independent lint result;
  con,
  starts work that a failing gate would have skipped.

Ranking:
gated progression > independent concurrent upkeep for the user's resource-saving concern.
This does not claim that lint is cheaper than a build;
no comparative timing was measured.
It clarifies the accepted interpretation of automatically keeping everything current:
advance as far as the current inputs permit,
rather than eagerly execute every concern.
The first follow-up interview settled source-check continuation,
foreground gate behavior,
supersession,
foreground input identity,
ordinary-source autofixing,
and the prohibition of `meow end` for automatic work.
Their accepted behavior is recorded in "Maintenance and foreground lifecycle" in the from-scratch design.

The model therefore needs intrinsic progression as well as meaningful result states.
Neither part requires authors to express generic task edges or an unapproved `depends_on` field.

### Accepted lifecycle refinements

- Finish independent source checks after one fails,
  while downstream production remains blocked.
- Foreground requests honor the same gates.
- Cancel superseded ordinary background builds and read-only checks.
- A waiting foreground maintenance request follows the latest relevant inputs.
- `meow end` cannot end automatic work in 0.x.
  The user rejected both proposed restart-lifetime policies rather than selecting one.
- Check ordinary source automatically;
  lint autofixing requires an explicit request.
  Managed-file enforcement retains its declared write authority.

Internal cancellation of obsolete work is distinct from a user asking to end maintained work.
The next answers settle Ctrl+C as detaching a foreground request from automatic maintenance,
and a currently blocked foreground request as reporting its blocker and returning nonzero.
The user rejected a special scheduling lane for a target label:
distinct invocations join the ordinary priority queue,
which decides concurrency and pausing.
Equivalent reusable computations still share work;
label equality alone is insufficient.

The user clarified the execution contract:
every execution has defined inputs and outputs.
An operation intended to re-execute can include `currentDateTime` as an input.
Its resolved value participates in the ordinary cache key.
The proposed per-definition reuse toggle is rejected;
there is no cache-off policy and no unconditional freshness rule for explicit operations.
Input binding must eventually specify volatile-value sampling,
without turning mere cache lookup into a new activation.

### Why these are not just tasks with new names

An arbitrary task primarily tells meow how to execute an operation.
An intrinsic responsibility tells meow what that operation means for the package.

- For a build output,
  meow knows which product is expected and whether it describes current inputs.
  A successful old build does not make a missing or changed output current.
- For a check,
  meow knows what is being examined and whether the result still applies.
  A current failing result is meaningful evidence,
  not an instruction to rerun unchanged inputs until success.
- For a managed file,
  meow knows the required content or structure,
  rather than merely remembering that a generator command once exited successfully.

Executions still exist and remain inspectable and controllable.
They are how these responsibilities are maintained,
not their definition.
A custom command does not become an intrinsic responsibility merely because its name is `build` or `test`.

Different responsibilities retain different meanings.
A failed production attempt may leave an old product stale.
A completed check may yield current failing evidence.
Cancellation does not establish a passing or failing assertion result.
There need not be one package-wide success state that erases these distinctions.

### Concrete repository case

Read directly from the repository:

- `package/jsonc-edit/core-ts/package.json` exposes source through `/ts`,
  plus Node and neutral bundles under `dist/final`.
- `package/jsonc-edit/core-ts/src/parse.unit.test.ts:14-22` consumes the neutral bundle.
- `AGENTS.md` rule `ST3` directs cross-package workspace consumers to source through `/ts`.
- `package/jsonc-edit/core-ts/mise.toml` currently expresses `buildAndTest` as a sequence of commands.
  That is incumbent orchestration,
  not meow's required domain model.

Under the proposal,
meow understands that the bundle and the checks consuming it must be kept current.
The person should not need to discover and arrange the command sequence to achieve that.
Meow should also avoid inventing a bundle prerequisite for a consumer that reads source instead.

A source edit makes the affected products or results outdated.
Meow rebuilds the products and refreshes the relevant checks.
If a check fails,
that is what meow currently knows about the package;
it does not mean the maintenance loop should retry until green.

### How the accepted command surface fits

For maintained concerns,
`meow run //package/cow:test` brings their current result to the foreground:
it attaches,
raises priority,
waits when necessary,
and replays or streams output under the accepted contract.
The spelling `run` does not require a task-shaped domain model.

One-off operations can remain available without defining the development lifecycle.
Their precise relationship to maintained concerns is not yet settled.
The current discussion should establish that conceptual boundary before asking for block names,
namespace rules,
or override mechanics.

Configuration should augment native package metadata with meow-specific intent,
not duplicate native manifests merely to feed a generic task graph.
No new HCL block schema is adopted here.
The recorded tag,
inheritance,
and explicit-override preferences remain constraints,
not proof that every concern must be represented by a `task` block.

## Preserved contracts

- The watch daemon keeps affected builds and the default test set current.
- Native manifests provide cross-package relationships.
- Content hashes determine freshness.
- Failures are cached;
  unchanged failing inputs do not trigger retry-until-green.
- Output replay,
  forwarding,
  priority,
  attachment,
  and process controls retain their accepted behavior.
- `meow status` remains attention-only in 0.x,
  not a new success dashboard.
- No unapproved `depends_on` mechanism or universal linear lifecycle is introduced.

Correct applicability is necessary under any eventual model:
a result finishing after an edit must not certify inputs it never consumed.
Completion order is not freshness order.
The consistency mechanism is separate implementation design,
not the current user question.

## Precedents and their limits

Source-verified conceptual precedents,
not technology selections or runtime probes:

- [OpenTofu's language documentation](https://opentofu.org/docs/v1.12/language/)
  describes intended goals rather than steps.
  This supports questioning recipe-shaped configuration,
  not importing its dependency syntax or plan/apply workflow.
- Kubernetes' [`metav1.Condition` definition][condition-source]
  distinguishes a result from the generation it observes.
  This supports separating freshness from outcome without replacing meow's accepted content hashes.
- Kubernetes' [Pod phase documentation][pod-phase]
  calls phases high-level summaries rather than comprehensive state machines.
  Stage summaries can coexist with the coherent maintenance model.

The analogy stops where creating a product differs from discovering a defect.
Re-executing a failing check is not equivalent to repairing an absent artifact.

[condition-source]: https://raw.githubusercontent.com/kubernetes/apimachinery/master/pkg/apis/meta/v1/types.go
[pod-phase]: https://kubernetes.io/docs/concepts/workloads/pods/pod-lifecycle/#pod-phase

## Deferred publication record

For 1.x only,
not current design work:

- Publication is automatic on a version bump pushed to `main`.
- The target registry must already contain the package.
- An already-present version needs no publication.
- A never-published package does not receive automatic first publication.

Do not ask further publication-policy questions during the non-task discussion.

## Process-rule proposals

Not applied to `AGENTS.md`.
Merge the lessons into existing rules rather than adding overlapping rules.

For `QPM`,
question inherited nouns as well as mechanisms:

> Every option set asserts a premise,
> including inherited nouns.
> Treat incumbent labels as hypotheses:
> test whether the domain needs the concept before offering mechanisms.
> Dissolving beats choosing.

For `RCO`,
apply its responsibility coverage requirement to model redesign as well as incumbent removal:

> Removal or model redesign:
> map responsibilities to identity,
> owner,
> eligibility,
> selection status,
> parity tests,
> and retired behavior.
> Recommend replacement only after all have viable owners.

This covers omitted responsibilities,
execution eligibility,
and the distinction between a target label and a computation's identity
without inventing another overlapping rule.

For `VR2`,
keep the active design layer when a feature is mentioned:

> Verb or scope ambiguous:
> keep narrower reading.
> Mentioning a feature does not authorize its subsystem design;
> record it and return to active question.
> Propose expansion explicitly.
