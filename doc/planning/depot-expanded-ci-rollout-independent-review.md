# Independent review of the expanded CI rollout plan

## Purpose, revision, and review boundary

This review tests whether [the rollout plan][rollout] can be confirmed as the repository's
concrete operating model before implementation starts.
The reviewed revision is `a08edd938a` (2026-10-18),
the revision the [handover][handover] says it published at line 69.
The plan file has not changed between that commit and the `HEAD` at review time (`2966b94eed`).

The review read the plan in full,
the [evaluation][evaluation] in full,
the [handover][handover] answers and work records,
the [CI provider decision][ci-provider],
the [CI caching decision][ci-caching],
the [routing research][routing],
the [coverage proposal][coverage],
the [allowance reconciliation][allowance],
the [cost model][cost-model],
the [reuse research][reuse],
and the [cache capability review][cache-review].
It also read the current workflow inventory and live branch protection through read-only GitHub API calls.

This is a read-only review.
It did not dispatch provider work,
change workflows,
change branch protection,
or edit the plan.
It is not the independent whole-report sufficiency review the evaluation says is still missing
([evaluation][evaluation] line 986);
it checks the plan against the evidence,
not the full evidence corpus against itself.

Four independent read-only audit passes were run over separate surfaces:
evidence fidelity,
repository feasibility and sequencing,
cost and trust,
and governance and decision value.
Their findings were consolidated here,
and contested facts were rechecked by the reviewer.
Findings rest on primary files read by at least one reviewer,
and contested facts were rechecked against the cited lines.

## Verdict

The plan cannot be confirmed as written.
It is faithful on several accepted policy answers,
but it is not a confirmable operating model.

The blocking defects are:

1.  There is no enforceable validation gate.
    The plan says failures stop a gate,
    but `main` has no required checks,
    direct commits remain allowed,
    and the repository's development model is
    auto-push to the default branch.
2.  The spend authority has no monthly estimate,
    no aggregate ceiling check,
    and no defined headroom source.
    It also assumes a Developer plan while the recorded account is the Startup trial,
    and the plan says it does not change the subscription.
3.  The plan contradicts the standing provider decision and the scope of the caching decision,
    and no work item updates either record.
4.  There is no rollback,
    disable,
    abort,
    or incident path,
    and no work item has an owner,
    acceptance evidence,
    or definition of done.
5.  The untrusted-PR,
    fork,
    cache-write,
    and publishing trust boundaries are
    contradictory or incomplete.
6.  Selection and qualification are circular.
    The plan claims independently verifiable work items while leaving
    what is selected,
    what qualifies,
    and what becomes required undefined.

The plan also reverses the evaluation's central Node workload allocation without saying so,
softens three adverse cache measurements,
adds default-branch push validation that the accepted cadence does not contain,
omits native-promotion prerequisites,
omits separately priced meters and retention limits,
and presents research evidence that is 858 commits behind the current repository.

## What the plan gets right

The review should not obscure the plan's sound parts.
The following are accurate and useful:

- The regression ratchet is stated correctly:
  known-good checks block,
  new or changed failures block,
  setup,
  discovery and artifact-integrity failures always block,
  and only explicitly baselined existing source failures can remain advisory
  ([plan][rollout] lines 79 to 93).
- The recovery reserve matches the accepted answer:
  one automatic attempt for a classified provider or startup failure,
  exposure reserved for both attempts,
  original diagnostics retained,
  source failures not erased by a passing provider,
  and unclassified failures blocked
  ([plan][rollout] lines 103 to 108).
- Source,
  provider,
  machine,
  case and artifact identities are required in results
  ([plan][rollout] lines 43 to 46).
- The plan refuses a whole-CI speed multiplier,
  a universal test-splitting benefit,
  signing trust from a matching attestation,
  and complete VM coverage from selected controls
  ([plan][rollout] lines 47,
  124 to 125).
- The paused-session attachment stays on GitHub until it qualifies
  ([plan][rollout] lines 114 to 116).
- Cache-key prefixes are correctly rejected as an isolation mechanism
  ([plan][rollout] line 144).
- Unbounded autoscaling is correctly excluded from automatic paid admission
  ([plan][rollout] line 150).
- The plan does not trigger a release or impose a PR-only policy
  ([plan][rollout] lines 100 to 101).
- The internal document links resolve,
  the prose has no em dash,
  en dash,
  table,
  or overlong line,
  and the scoped `mise run lint:monochromatic-lint -- doc/planning/depot-expanded-ci-rollout.md`
  exits zero.
  Style is not the problem.

## Blocking findings

### B1. The plan assumes a validation gate that does not exist

The plan preserves direct commits and auto-push
([plan][rollout] line 32),
then says qualified checks and new or changed failures "block their validation gate"
([plan][rollout] lines 79 to 81),
and requires gate controls before reliance
([plan][rollout] lines 95 to 101).

The repository does not have such a gate.
A read-only branch protection read returned
`required_status_checks.contexts: []`,
`checks: []`,
and `enforce_admins: false`.
The only active ruleset is `Copilot review`,
with no required checks.
The [routing research][routing] records the same empty list and no merge queue
(lines 110 to 116).
The same research measured 5,202 automatic default-branch arrivals across 2,526 head SHAs
against 24 PR arrivals across 6 head SHAs in one week
([routing][routing] lines 141 to 148).
The handover records the same empty required-check state
([handover][handover] line 3997).

With no required checks and direct pushes allowed,
a failing check is after-the-fact telemetry.
It cannot block a default-branch commit,
and the plan explicitly refuses to impose a PR-only model.
The plan never defines whether a gate blocks commits,
merges,
releases,
artifact promotion,
or only later work.

The event model also omits the merge queue.
Four existing workflows carry `merge_group` triggers
(`forbidden-strings.yml`,
`toml-edit-fuzz.yml`,
`readonly-semantic-bridge.yml`,
`logger-fuzz.yml`).
The plan mentions neither merge queue nor merge-group association,
while item 7 promises to verify the "real PR" boundary
([plan][rollout] line 178).

The revision must choose the enforcement model before adding profiles.
Either require PRs and required status checks on `main`,
which contradicts the plan's no-PR-only statement and needs the user's decision,
or define the gate as blocking release and artifact promotion,
with default-branch push runs as the primary signal and a post-hoc revert rule.
Name the check identities,
the merge-group path,
and the control that proves a failing check blocks the chosen boundary on both
a direct push and a PR.

### B2. The spend plan cannot be audited or enforced

The plan states a USD 20 Developer baseline and a USD 200 combined ceiling
([plan][rollout] lines 19 to 20),
standing authority for bounded recurring overage
([plan][rollout] line 21),
and an allocation ledger that does not guarantee an aggregate invoice
([plan][rollout] lines 22 to 23).
Those are the only dollar figures in the plan.

The plan contains no monthly usage estimate,
event count,
reservation sum,
or per-meter budget.
The [cost model][cost-model] requires explicit monthly scenarios with event counts
(line 54).
The [evaluation][evaluation] says the ceiling requires an explicit workload allocation
(line 39) and supplies the counterexample:
20,000 native two-CPU base minutes plus 20,000 Linux GitHub-runner two-CPU base minutes
costs USD 236 on Developer versus USD 200 on Startup
([evaluation][evaluation] lines 32 to 39).
The pricing evidence confirms the same totals.
The plan adopts neither that scenario nor any other,
so a reader cannot tell whether the proposed allocation fits the ceiling.

Per-profile reservations are required at
[plan][rollout] lines 154 to 155,
but nothing sums concurrent and monthly reservations plus actuals against USD 200.
There is no vendor aggregate dollar cap to fall back on:
the [allowance reconciliation][allowance] records separate product meters
and no single aggregate cap (lines 211 to 221).
The evaluation says production admission needs authorized headroom and expiry
([evaluation][evaluation] lines 941 to 944).

The plan never defines what "headroom" means.
If headroom means the vendor's remaining included allowances,
the accepted Q4 answer forbids account-wide inspection
([handover][handover] lines 89 to 95),
and no usage or settings endpoint was inspected
([allowance][allowance] lines 5 to 13).
If headroom means the repository ledger priced at list rates,
the plan should say so and sum that ledger,
which would be executable without account inspection.
As written,
the rule "do not assume unused credits" and the rule
"insufficient headroom selects GitHub" cannot both be applied to a defined quantity.

The account state is also unresolved.
The handover records that the current seven-day trial is on Startup
([handover][handover] lines 3747 to 3751),
and that no subscription change is authorized.
A USD 200 Startup fee leaves no overage room under the USD 200 combined ceiling,
while a Developer fee leaves USD 180.
The plan assumes Developer and says it does not change the subscription
([plan][rollout] line 24),
without naming who would change the plan,
when,
or which plan admission uses at implementation start.

Finally,
the admission paragraph omits the separately priced meters the evaluation requires:
storage,
passing Tests results,
SDK sandboxes,
BuildKit builder compute,
and optional products
([evaluation][evaluation] lines 938 to 940).
Registry images also default to unlimited retention,
and Fast transfer costs USD 0.10 per GB
([evaluation][evaluation] lines 909 to 917),
so "retention exposure" is not a cap.

The revision must state the active plan and allowance source at implementation start;
produce one monthly scenario with event counts,
per-meter reservations and a list-rate subtotal;
define headroom and its source;
add an aggregate ceiling check and a stop rule;
name the subscription transition owner,
or state that admission runs on Startup allowances;
and add storage,
Tests,
SDK,
BuildKit and optional-product quantities to every reservation.

### B3. The plan contradicts the standing provider and cache decisions

The [CI provider decision][ci-provider] says:
"stay on GitHub Actions standard hosted runners.
No migration is scheduled or anticipated;
the only option kept open is adding GitHub self-hosted runners for ubuntu x86"
(lines 4 to 9).
It also says non-GitHub providers are rejected so they are not re-proposed,
and that any reconsideration runs the `choosing-technology` vetting layers
([ci-provider][ci-provider] lines 9 and 122).
Its revisit triggers are sustained queueing or a deliberate per-push mutation decision
([ci-provider][ci-provider] lines 112 to 114).

The plan introduces Depot-managed GitHub runners and native Depot CI
([plan][rollout] lines 10 to 17,
41 to 42,
112 to 117),
but never cites the provider decision and never supersedes or amends it.
The routing research already says the old decision must be reconciled in place on adoption
([routing][routing] lines 123 to 125).

The [CI caching decision][ci-caching] also scopes itself to GitHub Actions `actions/cache`
and says so where it measures pnpm (lines 422 to 423).
Its "Who may save" rules are GitHub-cache rules:
PR and merge-group runs restore and never save
([ci-caching][ci-caching] lines 179 to 192).
The plan selects caches by owner and cache system and adopts Depot Cache,
BuildKit project cache and organization-global Disk
([plan][rollout] lines 127 to 137).
"Respect the existing CI caching policy"
([plan][rollout] line 139) does not say which rules extend to systems
the policy explicitly does not cover.

The plan is a `doc/planning/` proposal,
but it declares "accepted constraints",
a "permanent design",
"standing authority" and an "accepted regression ratchet"
([plan][rollout] lines 3,
10,
21 and 79).
Per the repository's planning rule,
accepted decisions belong in `doc/decision/`,
and only explicit acceptance unlocks that move.
No work item creates or amends a decision record,
and the plan has no date,
status,
owner,
or acceptance record.

The revision must add an item before the implementation sequence that
amends the provider decision in place with a dated supersession note,
records the `choosing-technology` vetting outcome,
extends or replaces the caching decision for Depot Cache,
BuildKit and Disk,
carries the PR restore-only rule into the new systems,
and records the accepted decision set separately from the implementation plan.

### B4. There is no rollback, abort, incident, or definition of done

The plan's sequence has no rollback,
disable,
abort,
incident,
monitoring,
or alert step.
A text search of the plan for those terms returns nothing.
The only recovery content is the single retry
([plan][rollout] lines 103 to 108)
and the headroom fallback to GitHub
([plan][rollout] lines 160 to 161).

"Unclassified failure blocks" is not an incident remedy.
If the failure belongs to a required check,
blocking means the repository cannot merge.
The plan has no answer for repeated Depot runner failures,
a qualified check that later flakes,
a regression that appears only after gating,
a cache-poisoning incident,
or a secret-exposure incident.

The ratchet makes demotion impossible by construction:
"Only explicitly identified existing source failures can remain advisory"
([plan][rollout] line 81).
A newly promoted check that misbehaves is not an existing source failure,
so it cannot become advisory,
and the plan gives no quarantine path.

The work items are called "independently verifiable"
([plan][rollout] line 163),
but none has an owner,
evidence artifact,
pass or fail threshold,
exit condition,
or rollback action.
Item 5 defers its pass condition to "after their consumer controls pass"
([plan][rollout] line 175),
item 6 defers promotion to "qualify"
([plan][rollout] line 176),
and item 7 defers reliance to a measurement with no threshold
([plan][rollout] lines 178 to 180).

For every item,
add owner,
evidence artifact,
threshold,
exit condition and rollback action.
Add plan-level abort triggers for artifact-control failure,
check-name or PR-association failure,
reservation or invoice exposure above the conservative reserve,
and a required check that cannot be enforced.
Make rollback concrete:
restore GitHub-hosted labels,
disable schedules,
remove required checks back to the measured empty baseline,
stop paid dispatch,
and retain evidence.

### B5. The untrusted-PR, fork, cache, and publishing boundaries are incomplete

The plan says untrusted PR source uses a "secret-free GitHub-hosted profile"
([plan][rollout] line 141),
while also saying Depot-managed GitHub runners receive selected shared-build and Rust-heavy profiles
([plan][rollout] lines 41 to 42).
"GitHub-hosted" is not defined.
It could mean public GitHub compute,
in which case no Depot-managed profile runs for fork PRs
and the benefit case changes.
It could mean Depot-managed workers under GitHub orchestration,
which is the distinction the handover's correction turns on
([handover][handover] lines 50 to 56).

The routing research states the narrower rule the plan drops:
untrusted source remains on a secret-free GitHub-hosted profile
"until Depot cache isolation is proven"
([routing][routing] lines 542 to 548).
Depot's Actions cache has repository scope and no branch isolation
([cache-review][cache-review] lines 20 to 35).

The existing provider decision also records a precondition the plan omits:
a public repository running fork PRs on self-hosted runners
needs fork-PR approval settings locked down
([ci-provider][ci-provider] lines 108 to 110).
The live approval policy is `first_time_contributors`.
Depot-managed runners are self-hosted from GitHub's perspective
([routing][routing] line 634).
The plan names no fork approval setting,
allowlist,
or statement that fork PRs stay on public GitHub compute.

The publishing boundary is also incomplete.
npm trusted publishing requires GitHub-hosted runners and excludes self-hosted runners
([routing][routing] lines 408 to 409).
The custom pnpr publisher binds the issuer,
audience,
main-branch subject,
repository identities and the exact `pnpr-publish.yml` workflow reference
([routing][routing] lines 411 to 419;
`package/config/pnpr/config.yaml` line 27).
The plan says it retains existing publishing issuer and workflow bindings
([plan][rollout] line 145)
and says publication boundaries stay GitHub-hosted
([plan][rollout] line 38),
but its native-promotion criteria do not exclude publication workflows.

The revision must define trust classes,
including fork PRs,
Dependabot,
and agent-requested work;
state whether any Depot-managed runner executes untrusted code and under which approval policy;
carry the restore-only cache rule and the cache-isolation qualifier into the plan;
and explicitly exclude issuer-bound publication workflows from native promotion,
with a rebinding step if that ever changes.

### B6. Selection and qualification are circular, and the admission set is absent

The plan uses "qualified" six times
([plan][rollout] lines 30,
58,
80,
100,
123 and 160),
"selected" seven times,
and "admitted" twice.
None is defined.
"Other selected workflows qualify against their own needs"
([plan][rollout] line 117) makes the criterion identical to the promotion.

The evaluation supplies the missing admission set:
the final selector must retain every admitted task from the coverage manifest,
including configuration,
generated-file and producer influences
([evaluation][evaluation] lines 739 to 742).
The plan never names the manifest,
the admitted task set,
or the selector.
It replaces them with "a conservative complete eligible profile when affected scope is unqualified"
([plan][rollout] line 50),
where "complete",
"eligible",
and the profile's size and cost are undefined.

Because the Mise workspace graph has a cycle,
affected selection is not available,
so this fallback is the normal path
([evaluation][evaluation] lines 744 to 753).
It is also the plan's largest unbudgeted execution driver.

The revision must name the admitted owners and the coverage manifest,
define the qualification controls,
evidence,
authority and reversal for each promotion,
define the required and advisory sets,
and give each work item a verification command or artifact.

## Major gaps

### G1. The central workload allocation silently reverses the evaluation

The plan assigns ordinary type checking,
linting and Node unit validation to GitHub-hosted Linux
as "permanent workloads"
([plan][rollout] lines 35 to 36).
The evaluation proposes the opposite:
"Use the native Linux pool for admitted Node static/unit/process owners"
([evaluation][evaluation] lines 723 to 725).
The routing research gives the same assignment:
trusted Node validation runs on Depot-managed Linux workers
([routing][routing] lines 542 to 547).
The coverage proposal supports the plan's choice:
"routine work without a measured Depot benefit retain GitHub placement"
([coverage][coverage] line 21).

The plan follows neither record explicitly.
It presents the GitHub assignment as settled,
uses "permanent",
which forecloses later measured movement,
and does not reconcile the two research recommendations.
The accepted Q17 answer says deeper Depot dependence is acceptable when measured benefits justify it
([handover][handover] lines 3811 to 3815).

The plan's line 16 makes the same error in stronger form:
"Ordinary Linux workloads remain on GitHub's own compute."
The evaluation says ordinary Linux work remains "part of its workload allocation"
([evaluation][evaluation] line 43),
not that all ordinary Linux work stays on GitHub.
The plan's own lines 41 to 42 and 112 contradict line 16.

The revision must state that this is a deliberate allocation choice,
cite the coverage rationale and the user's heavy-GitHub-hosted requirement,
replace "permanent" with "heavily used" unless the user intends to foreclose measured movement,
and rewrite line 16 to match the evaluation's "part of" wording.

### G2. Three adverse cache measurements are softened

The plan's cache bullets
([plan][rollout] lines 129 to 134)
omit the direction of the measurements the evaluation records.

The pnpm store archive was slower warm on both providers:
GitHub restore plus install took 11 to 12 seconds warm versus 9 to 10 seconds disabled,
and Depot took 4 to 5 seconds warm versus 3 seconds disabled
([evaluation][evaluation] lines 662 to 669).
The handover calls it a regression
([handover][handover] lines 1637 to 1638 and 1691 to 1692).
The plan says only that it does "not justify a universal store-cache default".

The Gradle task-output cache reduced the command by about 19.9 percent,
but whole warm jobs were slower:
162 and 177 seconds warm versus 146 and 153 seconds cache-off
([evaluation][evaluation] lines 674 to 682).
The plan says it has a "command-phase benefit" and asks to distinguish it from whole-job setup,
without stating that whole jobs were slower.

The archived target reduced the CLI build to 0.070 seconds from 52.228 seconds,
but the reader's full Nextest command was slower,
45.200 seconds versus 39.233 seconds
([evaluation][evaluation] lines 685 to 693).
The plan says only that the result does "not establish a Nextest execution speedup".

The revision must state the measured direction and numbers for all three mechanisms,
keep the pnpm archive disabled by default,
admit the Gradle cache only for the command boundary,
and treat the target archive as a reuse proof,
not a test-speed proof.

### G3. The performance comparisons are stated more strongly than the evidence

The plan says the retained comparisons "establish completed-phase benefits"
([plan][rollout] line 42).
The evaluation supports completed-phase time reductions,
but the comparison has two repetitions per side,
public GitHub left 28 commands unstarted and terminated six at their budgets,
and the ranges are observed spreads,
not confidence intervals
([evaluation][evaluation] lines 606 to 636).
The evidence file marks both ratios as not whole-pipeline multipliers.
The plan keeps the whole-CI caveat at line 47,
but states the benefit first and omits the two-sample and unequal-completion limits.

The revision must cite the phase means and the limits together,
or say "two completed-phase samples" rather than "benefits".

### G4. Default-branch push validation is an unrecorded cadence addition

The plan runs core profiles on PRs and default-branch pushes
([plan][rollout] lines 30 to 31).
The accepted Q2 answer is "core checks on PRs and the wider matrix weekly"
([handover][handover] lines 37 to 41;
[evaluation][evaluation] lines 1003 to 1004).
No accepted answer adds default-branch push checks.

The repository's direct-commit model may require them,
but the plan presents them as part of the accepted cadence without naming the workflows,
recording the addition,
or costing the event volume.
The routing research measured 5,202 automatic default-branch arrivals in one week
([routing][routing] lines 141 to 148).

The revision must decide and record whether every default-branch push runs core validation,
or only agent-requested and release-relevant pushes,
and cost the choice in the monthly scenario.

### G5. Native-promotion prerequisites are missing

The plan promotes selected Linux workflows to native Depot CI after qualification of
"actual features,
case identities,
GitHub result association and producer/consumer boundaries"
([plan][rollout] lines 112 to 114).
The evaluation records prerequisites the plan does not name:
native retries reuse filesystem and environment state,
native parallel branches share mutable directories,
and the measured single-job graph does not qualify every parallel-step merge rule
([evaluation][evaluation] lines 246 to 251).
The evaluation also says native check publication
"does not qualify branch-protection configuration or a stable production check-name mapping"
([evaluation][evaluation] line 507),
so item 6 cannot safely precede item 7's PR boundary check.

The current default branch has no `.depot` directory,
so "selected native Linux workflows" has no in-repository source yet.
The platform ledger records that merging `.depot/workflows` to the default branch
automatically registers events,
while the original GitHub workflows continue independently
([platform ledger][platform-ledger],
finding at the native CI source review).
The plan says it "retains" a visible routing override
([plan][rollout] line 177),
but the routing research only recommends adding one
([routing][routing] line 733).

The revision must name the selected files and their source of record,
merge them disabled or on a non-default branch first,
record retry and parallel shared-filesystem semantics,
record the check-name mapping,
exclude issuer-bound publication workflows,
and say "add" a routing override rather than "retain" one.

### G6. Admission and retention omit separately priced meters and deletion duties

The admission paragraph names resource bounds,
execution bounds,
retention exposure and a reservation
([plan][rollout] lines 154 to 155).
The evaluation requires finite workload reservations and separately priced
storage,
Tests,
SDK and optional-product quantities
([evaluation][evaluation] lines 938 to 940),
and says retention and deletion must name exported artifacts,
registry manifests and blobs,
snapshots,
Disk and logs separately
([evaluation][evaluation] lines 945 to 948).
The plan enables BuildKit,
Tests and VM profiles
([plan][rollout] lines 119 to 124)
but does not reserve their meters.

Concrete omissions:
storage above 25 GB at USD 0.20 per GB per month,
passing Tests results above one million at USD 0.000001 each,
SDK sandbox vCPU-seconds,
BuildKit builder minutes at USD 0.04 per minute,
unlimited default image retention,
and Fast CDN transfer at USD 0.10 per GB.
The [allowance reconciliation][allowance] and the pricing evidence record each rate.

The revision must add each meter to the admission record,
add a deletion policy per artifact class,
and make the aggregate ceiling check sum them.

### G7. Optional products and trust-adjacent surfaces have no exclusion list

The plan authorizes "applicable Depot build,
cache,
registry,
test and VM capabilities"
([plan][rollout] lines 14 to 15).
That open phrase,
combined with standing spend authority,
can be read to authorize any capability an implementer judges applicable.

The evaluation records reviewed and unactivated surfaces:
Depot Code,
Agents,
Dagger,
GPU,
managed and customer AWS,
custom runner images,
egress filtering and Tailscale,
private connectivity,
external observability sinks,
registry pull-through,
Fast CDN and server-side promotion
([evaluation][evaluation] lines 928 to 932 and 952 to 963).
The plan mentions none as excluded.

Dependabot is a current edge case,
not a live job source:
the repository has no Dependabot configuration,
security updates are disabled,
and a pull request query found no bot PRs.
Its runner and OIDC behavior still differs
([evaluation][evaluation] lines 931 to 932),
so the plan should say whether it is in scope.

The revision must add an explicit in-scope and out-of-scope product list,
and replace "applicable" with named services.

### G8. The evidence is older than the repository and the independent review is pending

The evaluation's source cohort is `e02be6c8b8` (2026,
October 18)
([evaluation][evaluation] line 59).
The current `HEAD` `2966b94eed` is 858 commits ahead,
with 1,484 files changed and 589,880 insertions.
The plan says it will refresh owners and baselines
([plan][rollout] lines 165 to 166),
which is the right work item,
but it does not state the drift,
name the implementation candidate,
or warn that the refreshed owner map may change the allocation and reservations.

The evaluation also says a fresh independent whole-report sufficiency review has not run
because the agent service quota is exhausted,
and that source,
hash,
pointer and arithmetic checks are not a substitute
([evaluation][evaluation] lines 986 to 988).
The handover repeats this limit
([handover][handover] line 26).
The plan asks for confirmation and implementation without surfacing it.

The revision must pin the implementation candidate revision,
state the evidence drift and pending review,
and treat the refreshed owner map as a re-baselining gate before paid profiles run.

### G9. "Broad native-platform validation" overstates the measured cohort

The plan says GitHub-hosted Windows and Mac runners retain broad native-platform validation
([plan][rollout] lines 37 to 38).
The evaluation measured selected Rust owner cohorts:
1,495 cases on the selected Mac contexts and 1,494 on the selected Windows contexts,
and says those standalone-workspace owners do not exercise IDE,
terminal or file-manager tests
([evaluation][evaluation] lines 193 to 205).
The measurements do not establish a fair native-platform performance ratio
([evaluation][evaluation] lines 216 to 220).

The revision must name the measured cohorts and their limits,
and say Depot-managed Windows remains eligible for the demonstrated OS boundaries.

### G10. There are no success metrics, definition of done, or ownership plan

The plan's last item measures duration,
failures and allocation exposure
"before relying on the recurring schedule"
([plan][rollout] lines 178 to 180).
It names no target,
baseline,
threshold,
review point,
or abandonment condition.
It also creates a new control plane:
admission with concurrent reservations and supersession,
a workload-to-provider map,
exact-result collection,
a weekly matrix,
packaged-artifact consumers,
cache validation,
BuildKit,
Tests and VM profiles,
native promotion,
a routing override,
and a budgeting ledger.
No item names who builds or operates it,
or what maintenance the dual-provider model adds.

The revision must add per-item owners and done conditions,
a rollout success threshold,
a review point after the Depot-managed runner stage and before native promotion,
and an operating-cost statement.

## Ambiguities and wording

### A1. The load-bearing qualifiers are undefined

"Qualified" appears at lines 30,
58,
80,
100,
123 and 160.
"Selected" appears seven times.
"Admitted",
"eligible",
"bounded" and "conservative" are also used without criteria.
"Ordinary" and "heavy" carry the user's allocation constraint but have no boundary,
although the handover explicitly refuses a numeric interpretation
([handover][handover] line 118).

### A2. "Headroom" and "incomplete admission" have no referent

The plan says insufficient headroom selects GitHub and otherwise reports incomplete admission
([plan][rollout] lines 160 to 161).
It does not say which allowance,
whose ledger,
in what artifact,
or with what consequence.
The evaluation separates included allowances from the aggregate invoice
([evaluation][evaluation] lines 938 to 942).

### A3. The routing override is named once and undefined

The plan says the promotion retains "a visible routing override"
([plan][rollout] line 177).
Nothing says who may use it,
on what authority,
for how long,
whether it works during an incident,
whether it covers Depot-managed runners as well as native CI,
or whether it preserves required-check identity and the audit trail.
"Retain" is also wrong if no override exists yet.

### A4. The result schema is not specified

The plan requires gates to consume an "exact required artifact set and worker outcomes"
([plan][rollout] line 95),
and results to carry provider,
machine,
source,
case and artifact identities
([plan][rollout] lines 43 to 46).
No field list,
storage location,
retention rule,
or access rule exists,
so "exact" cannot be checked.

### A5. The registry and transfer sentence decides nothing

"Reviewed registry and transfer options receive no invented performance benefit"
([plan][rollout] line 149)
forbids an inference.
It does not say whether pull-through,
Fast CDN,
or server-side promotion are in or out,
or that tagged retention and unlimited default image retention were considered.
A reader can take it as a decision when it is not.

### A6. The final deferral sentence is unbounded

The plan says only runner labels,
cache encodings,
retry timers and "other engineering details"
are chosen during implementation
([plan][rollout] lines 192 to 194).
That list is false as a description of what is deferred.
Required and advisory membership,
paid allocation,
qualification criteria,
headroom,
rollback,
and the fork boundary are also undecided,
and none is a label,
encoding,
or timer.
"Other engineering details" can excuse any omitted decision.

### A7. The confirmation is circular

The plan asks the user to confirm that it "captures the agreed operating model"
([plan][rollout] line 189).
The only written statement of that model is the plan itself.
The user cannot check the mapping without reading the 4,692-line handover,
and the plan does not enumerate which handover answers it encodes or where it departs from them.

## Questions to resolve before Q7

A careful user would need these answers before confirming:

1.  Which exact checks become required,
    and what do they block:
    commits,
    merges,
    releases,
    artifact promotion,
    or only post-hoc review?
    If required checks are added to `main`,
    does that accept a PR-based path
    alongside auto-push,
    and how does the merge queue participate?
2.  What is the monthly workload estimate,
    per-meter reservation,
    and list-rate total?
    What is headroom measured against,
    and what stops paid dispatch at the ceiling?
3.  Is the account on Startup or Developer at implementation start,
    who changes it if needed,
    and which allowances does admission use?
4.  Does any Depot-managed runner execute fork or untrusted PR code?
    What fork approval policy applies,
    and which caches can those runs read or write?
5.  Which decision records are amended or superseded,
    with what date and owner,
    and which exact plan revision does a confirmation cover?
6.  What disables a profile,
    reverts routing,
    quarantines a required check,
    invalidates a cache,
    or stops spend after a bad qualification,
    cost overrun,
    or trust incident?
7.  Which owners and commands are in the admitted set,
    what makes a profile "qualified",
    and who signs a promotion?
8.  Does every default-branch push run core validation,
    or only PRs,
    dispatch,
    and releases?
9.  Which optional products,
    registry features,
    egress controls,
    and observability sinks are out of scope?
10. What is the definition of done for items 1 to 7,
    and what evidence would abandon the rollout?

## Recommended revision order

The following sequence would make the plan confirmable.
It is a recommendation only;
this review did not edit the plan.

1.  Add a prerequisite item before the implementation sequence that:
    records the active plan and allowance source;
    amends the provider and caching decisions in place with dated supersession notes;
    records the branch-protection and required-check enforcement decision;
    defines the trust classes and fork policy;
    lists optional products as in or out;
    and records the review revision and acceptance owner.
2.  Rewrite the allocation paragraph to state the deliberate choice between the evaluation's
    native Node allocation and the coverage proposal's GitHub placement,
    with the measured-benefit and heavy-GitHub-hosted rationale.
    Replace "permanent" with "heavily used" unless the user intends a permanent assignment.
3.  Add the monthly scenario,
    per-meter reservations,
    headroom definition,
    aggregate ceiling check,
    stop rule,
    and subscription transition owner.
4.  Add the enforcement model,
    check identities,
    merge-group path,
    and the direct-push and PR controls.
5.  Add the trust boundary:
    fork policy,
    cache save and read rules per system,
    cache-isolation qualifier,
    and publication-workflow exclusion.
6.  Add per-item owner,
    evidence artifact,
    threshold,
    exit condition,
    rollback action,
    and the plan-level abort triggers.
7.  Name the admitted set and coverage manifest,
    and define qualification,
    promotion,
    required and advisory membership.
8.  Correct the evidence statements:
    the adverse cache directions,
    the two-sample and incomplete-run limits,
    the native-platform cohort limits,
    the evidence drift and pending independent review.
9.  Add native-promotion prerequisites:
    selected files and source,
    disabled first merge,
    retry and parallel semantics,
    check-name mapping,
    and routing override definition.
10. Replace the final deferral sentence with two explicit lists:
    decisions the plan settles,
    with criteria and owners,
    and decisions deferred,
    with the artifact and signer for each.

## Review checks and limits

Read-only checks run:

- Full reads of the plan,
  evaluation,
  handover answers and work records,
  both CI decisions,
  routing research,
  coverage proposal,
  allowance reconciliation,
  cost model,
  reuse research and cache review.
- `gh api repos/Aquaticats/Monochromatic/branches/main/protection`
  and the rulesets endpoint,
  read-only.
- `gh api repos/Aquaticats/Monochromatic/actions/permissions/fork-pr-contributor-approval`,
  read-only.
- `git rev-list --count e02be6c8b8..HEAD` and `git diff --shortstat e02be6c8b8..HEAD`.
- `git show a08edd938a` and `git diff a08edd938a..HEAD -- doc/planning/depot-expanded-ci-rollout.md`
  to confirm the reviewed revision.
- Workflow trigger inventory and `merge_group` search.
- Scoped Markdown lint of the plan and manual style checks.

Limits:

- No provider work was dispatched and no source was executed.
- No billing,
  invoice,
  account usage,
  or subscription endpoint was read.
- This review checks the plan against the evidence.
  It does not replace the pending independent whole-report sufficiency review of the evaluation.
- The current repository is 858 commits ahead of the evaluation cohort,
  so refreshed owner and failure baselines may change the allocation and reservations.

## References

[rollout]: depot-expanded-ci-rollout.md
[evaluation]: depot-expanded-ci-evaluation.md
[handover]: ../handover/depot-trial-grilling.md
[ci-provider]: ../decision/ci-provider.md
[ci-caching]: ../decision/ci-caching.md
[routing]: depot-developer-pool-routing-research.md
[coverage]: depot-expanded-coverage.md
[allowance]: depot-included-allowance-reconciliation.md
[cost-model]: depot-expanded-ci-cost-model.md
[reuse]: depot-reuse-strategy-research.md
[cache-review]: depot-cache-capability-review.md
[platform-ledger]: depot-platform-surface-ledger.json
