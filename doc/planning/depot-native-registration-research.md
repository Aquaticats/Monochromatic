# Depot CI native registration and the direct-to-main probe

Research date:
2026-10-19.
Repository state inspected at commit `4b1071f2f7`.
Installed CLI:
`depot version 2.102.23 (2026-10-15)`.
No repository files were changed and no run was dispatched for this report.
The run and workflow records quoted here were read with `depot ci run list`,
`depot ci run show`,
`depot ci status`,
and
`depot ci workflow list`,
which are read-only queries.

## Verdict

Depot's public CLI and API do not expose a registration command,
a registration
API,
or a registration listing.
Under the documented model,
registration happens
server side for workflow files under `.depot/workflows/`.
The docs describe
default-branch activation,
and a native `pull_request` gate in this repository
shows the rule also covers a non-default PR base.
The Depot Code Access GitHub
App is the event source and the check writer.
`depot ci migrate` and `depot ci
migrate preflight` do not register anything:
the first copies and transforms
files in the worktree,
and the second only asks the Depot API whether the app is
installed,
has repository access,
and has current permissions.

Automatic registration is not generally absent in this repository.
Native
`pull_request` runs already exist for a workflow on a non-default base branch,
with Depot checks used as required status checks.
The unresolved path is `push`
on `main`,
not event registration as a whole.

The direct-to-main probe was not a valid negative.
Two independent problems
invalidate it.
The only push that could have matched the probe workflow arrived
ten seconds after the commit that introduced the workflow,
and five seconds
after that commit's native run started;
the run stayed open for another 28
seconds,
so any asynchronous registration could not yet have been in effect.
In addition,
the query used as the oracle,
`depot ci workflow list`,
lists workflow executions,
not registered
triggers,
so an empty result cannot distinguish "not registered" from
"registered but never executed".
The probe was deleted about two minutes after
it was added,
which removed the only workflow that a later registration could
have run.

The repository's evaluation also overstates one point:
the deployed docs do not
state a pull-request merge requirement.
They require the workflow files to be on
the default branch and the workflow to contain a trigger matching the event.
A
direct push puts the same files on the default branch,
so it meets the condition
as written;
whether Depot's server treats the two landing paths differently is
not documented.
The probe failed for timing and observation reasons,
not because
it provably bypassed a required merge event.

## What the primary sources document

### The registration statement

The deployed overview says:

> When you add a workflow file to `.depot/workflows/` in your repository and
> merge it to your default branch,
> Depot registers automatic triggers for that
> workflow.

Source: <https://depot.dev/docs/ci/overview#how-depot-ci-works>,
retrieved
2026-10-19.
The same page describes Switchyard as the orchestrator and compute
layer,
and says the orchestrator schedules jobs when a trigger fires.

The deployed quickstart says:

> Depot CI uses the Depot Code Access app to read your repository and register
> workflow triggers.

and:

> Push and merge the changes so that automatic workflow triggers (like `push`
> and `pull_request`) are registered with Depot CI.

and:

> If your workflows include a `push` trigger on the default branch,
> merging
> starts your first run on Depot CI.

Source: <https://depot.dev/docs/ci/quickstart#connect-to-github> and
<https://depot.dev/docs/ci/quickstart#merge-the-migrated-workflows>,
retrieved
2026-10-19.

The clearest operational troubleshooting statement I found is on the Origin
page,
under "No run starts after migration":

> Confirm that the migrated files are under `.depot/workflows/` on the default
> branch and that the workflow contains a trigger matching the event you sent.
> Also confirm that Depot is connected through Origin for an Origin-hosted
> repository or through GitHub for a mirrored repository.

Source:
<https://depot.dev/docs/ci/integrations/origin#no-run-starts-after-migration>,
retrieved 2026-10-19.

Three conditions follow from these passages:

- The files must be under `.depot/workflows/` on the default branch.
- The workflow must contain a trigger matching the event that was sent.
- The repository must be connected through the forge (GitHub or Origin).

The deployed wording leaves one case unstated:
the repository's native PR gate
ran a `pull_request` workflow from a non-default base branch (see the
observations).
The docs do not describe that case,
so the default-branch
condition is not the whole rule.

No deployed page says a pull-request merge event is the registrar,
no page says
a direct push to the default branch is ignored,
and no page gives a latency
bound for registration.
The words "push and merge" and "merge it to your default
branch" describe how content reaches the default branch,
not a distinct GitHub
event that Depot consumes.

### The scheduled-workflow statement is the clearest activation wording

The compatibility reference says:

> Use `on.schedule` to run a workflow on a cron schedule.
> Add the workflow to
> `.depot/workflows/` and merge it to your repository's default branch to
> activate the schedule.
> Scheduled workflows run from the default branch.

Source: <https://depot.dev/docs/ci/compatibility#scheduled-workflows>,
retrieved
2026-10-19.

This is the same default-branch condition,
stated for a trigger that has no
GitHub event at all.
Depot evaluates the schedule itself,
which shows that
"activation" is a Depot-side state derived from default-branch content,
not
something that only a GitHub merge event can create.

### What `depot ci migrate` and `preflight` actually do

`depot ci migrate preflight` is documented as validating authentication,
detecting the repository,
and checking that the Depot Code Access app is
installed with the correct permissions and repository access:

<https://depot.dev/docs/cli/reference/depot-ci#depot-ci-migrate-preflight>

The shipped source matches that description.
`githubPreflight` detects the repo
from the git remote,
calls `GetInstallation` with the repo name,
matches an
installation by owner,
and prints one of four outcomes:
not installed,
installed
without repository access,
permissions need updating,
or "Depot Code Access app
is installed and configured for <repo>".

- `pkg/cmd/ci/migrate.go:326-402` at tag `v2.102.23`
  (<https://github.com/depot/cli/blob/v2.102.23/pkg/cmd/ci/migrate.go#L326-L402>).
- The installation request is at `pkg/cmd/ci/migrate.go:355-400`.
- The exact message the probe saw is printed at `pkg/cmd/ci/migrate.go:400`.

"Installed and configured" therefore means three things and nothing more:
an
installation exists for the repository owner,
`RepoAccessible` is true for the
repository,
and `RequiresNewPerms` is false.
It says nothing about triggers
being registered,
and the RPC it calls is named `GetInstallation`,
not anything
related to workflows.

`depot ci migrate` runs that preflight and then copies and transforms files:

- `runMigrate` calls `preflight` and then `workflowsWithContext`,
  with no
  registration call in between (`pkg/cmd/ci/migrate.go:405-419`).
- The copy is a local `os.WriteFile` into `.depot/workflows/`
  (`pkg/cmd/ci/migrate.go:700`),
  plus local action copying and path rewriting.
- The command's own last instruction is "Activate these workflows by pushing
  and merging them into <default branch>" (`pkg/cmd/ci/migrate.go:773-798`).

There is no registration RPC in the public CI service.
The v1 `CIService` has
run,
dispatch,
retry,
rerun,
cancel,
read,
log,
artifact,
`ListRuns`,
and
`ListWorkflows` methods,
and the migration service has installation and
migration methods
(`proto/depot/ci/v1/ci.proto:7-91`,
`proto/depot/ci/v1/ci.proto:96-98`,
<https://github.com/depot/cli/blob/v2.102.23/proto/depot/ci/v1/ci.proto#L7-L98>).
The v2 migration service adds `StartMigration`,
`RetryMigration`,
`ReportSecretImport`,
and `RegisterSecretMigrationIntent`
(`proto/depot/ci/v2/migration.proto:8-26`),
but the shipped CLI does not call
`StartMigration`;
its migration path for GitHub is local file copying.
The only
"intent" in the CLI is the five-minute authorization for the one-shot
secrets-and-variables import,
which is unrelated to workflow triggers
(`pkg/cmd/ci/migrate_secret_intent.go`).

Consequence:
copying `.depot/workflows` by hand is equivalent to `depot ci
migrate` for activation purposes.
No migration intent,
project,
or repository
link is created by the CLI.
The docs' "Commit .depot/workflows and merge it into
your default branch to activate the workflows" is the whole contract
(<https://depot.dev/docs/ci/integrations/origin#workflow-migration-details>).

### The GitHub App is the event path

The docs say the Code Access app "reads your repository and registers workflow
triggers".
The public GitHub App record shows what the app is allowed to do:

- Slug `depot-code-access`,
  owner `depot`,
  created 2022-07-14,
  updated
  2026-09-16.
- Permissions include `contents: write`,
  `workflows: write`,
  `actions: write`,
  `checks: write`,
  `pull_requests: write`,
  `statuses: write`,
  `merge_queues: read`,
  `metadata: read`.
- Subscribed events include `push`,
  `pull_request`,
  `merge_group`,
  `repository_dispatch`,
  `deployment_status`,
  `status`,
  `create`,
  and `delete`.

Source: <https://api.github.com/apps/depot-code-access>,
retrieved 2026-10-19.

This establishes that the app can receive push and pull-request events and can
write checks and workflow state.
It does not establish the server-side rule that
turns an event into a run,
which is not public.

The installation model is per GitHub account or organization,
with a repository
selection.
The quickstart tells organization owners to open Organization
Settings,
find "GitHub Code Access",
and install the app on "the personal
account or organization that owns your GitHub repository"
(<https://depot.dev/docs/ci/quickstart#connect-to-github>).
The CLI preflight
checks the owner-level installation and the specific repository's accessibility,
not a per-workflow or per-project link.
The public CI API identifies runs by
organization and repository (`repo` in `owner/name` form) and the API reference
lists "the organization has no GitHub Code Connection" as a cause of
`permission_denied`
(<https://depot.dev/docs/api/ci/reference>).

There is no documented per-project registration step for Depot CI.
The CI
commands and API address the organization plus repository.

### Trigger support

The compatibility matrix marks these as supported:

- `on`,
  `on.<event_name>`,
  `on.<event_name>.types`
- `on.push`,
  `on.push.branches`,
  `on.push.tags`
- `on.pull_request`,
  `on.pull_request.branches`
- `on.merge_group`
- `on.*.paths`
- `on.schedule`
- `on.workflow_dispatch`,
  `on.workflow_dispatch.inputs`
- `on.workflow_run`
- `on.repository_dispatch`,
  `on.repository_dispatch.types`

Source: <https://depot.dev/docs/ci/compatibility#workflow-level-configuration>,
retrieved 2026-10-19.

Unsupported triggers are GitHub-specific events such as `release`,
`issues`,
`check_run`,
and `status`
(<https://depot.dev/docs/ci/compatibility#github-only-event-triggers>).

So `push`,
`pull_request`,
`paths`,
`branches`,
and `workflow_dispatch` are all
documented as supported.
The probe's trigger block was within the supported
set.

The compatibility page also resolves some events from the default branch's
workflow state:
for `repository_dispatch`,
"GitHub only delivers
`repository_dispatch` events against your repository's default branch,
so the
workflow that runs is the one on the default branch"
(<https://depot.dev/docs/ci/compatibility#repository-dispatch-events>).
This is
consistent with a model where Depot resolves a workflow from a ref at event
time,
rather than maintaining a separate trigger database that only a merge can
populate.

### The closest explicit push model is on the Depot Code page

For Depot Code repositories the docs are unambiguous:

> Depot Code repository events trigger CI only on `push`.
> When you push to a
> Depot Code repository,
> Depot CI runs every workflow with an `on: push`
> trigger against the pushed commit.
> Each new commit included in a push starts
> its own set of runs.

Source: <https://depot.dev/docs/code/overview#running-depot-ci-workflows>,
retrieved 2026-10-19.

This is a per-push,
per-commit evaluation against the pushed tree.
It is the
clearest deployed statement that describes the matching rule at that level of
detail.
It applies to Depot Code repositories,
not GitHub repositories,
so it
cannot simply be transferred to Aquaticats/Monochromatic.
It does show that the
product's trigger model is not inherently "one merge registers a permanent
trigger list".

### Manual dispatch and local runs are separate paths

`depot ci dispatch` triggers a workflow that declares `on.workflow_dispatch` on
a named ref.
The CLI sends `DispatchWorkflowRequest` with `repo`,
`workflow`
basename,
`ref`,
and inputs
(`pkg/cmd/ci/dispatch.go:27-77`,
`pkg/cmd/ci/dispatch.go:63-70`).
The API reference says the workflow must have a
`workflow_dispatch` trigger and that inputs are validated against the schema
(<https://depot.dev/docs/api/ci/reference>).
It reads the workflow from the ref;
it does not require an earlier automatic registration.
This is why a successful
dispatch on a research branch says nothing about event registration.

`depot ci run` submits a workflow from the local working tree,
applies
uncommitted changes as a patch,
and starts a run without a GitHub event
(<https://depot.dev/docs/cli/reference/depot-ci#depot-ci-run>).
The CLI calls
`CIRun` (`pkg/api/ci.go:378-389`),
and the run is recorded with trigger `api`
in this repository's records.
Local runs are the correct way to prove a
workflow file is valid and that the workflow-list view can show a workflow,
but
they do not exercise event registration.

### Checks and duplicate execution

When a Depot CI job runs,
Depot reports a check per job on the corresponding
commit or pull request,
named after the workflow and job,
with the Depot icon;
reruns update the existing check rather than creating a duplicate
(<https://depot.dev/docs/ci/observability/github-checks>).

The quickstart warns that after the `.depot` directory is merged,
"the
workflows run in both GitHub and Depot CI" and that deploys,
artifact updates,
or any external side effect "will execute twice"
(<https://depot.dev/docs/ci/quickstart#merge-the-migrated-workflows>).
This is a
product-level duplicate-execution statement,
not a statement about two Depot
runs of the same workflow.

### Product boundaries

- Depot CI runs GitHub Actions YAML from `.depot/workflows/` on Depot compute.
  It is a separate CI engine built on Switchyard,
  not a wrapper around GitHub
  Actions
  (<https://depot.dev/docs/ci/overview#faq>).
- Depot-managed GitHub Actions runners keep `.github/workflows/` and replace
  only the runner label,
  for example `runs-on: depot-ubuntu-24.04`;
  GitHub still
  orchestrates the workflow
  (<https://depot.dev/docs/github-actions/overview>).
- Depot Code is Depot-hosted source control (a diskless Git server with GitHub
  mirrors) that feeds Depot CI;
  its repository events trigger CI on `push`
  (<https://depot.dev/docs/code/overview>).
- `.depot/` is the Depot CI input directory for migrated workflows and local
  actions.
  `.github/` remains untouched and keeps running on GitHub during
  migration.

## What this repository's live records show

These are read-only observations made on 2026-10-19 with the installed CLI and
the GitHub API.
Timestamps are UTC.

The probe commits:

- `a2feb0d0d545b6ad3c17a131e3170f10722ddabb` added
  `.depot/workflows/registration-probe.yml` and a matching `.github` copy at
  00:49:25.
- `6523bcaeddfc3648cb66543ea94ca9ad2d932a34` added
  `probe/registration-marker.txt` at 00:49:35.
- `f8bc2d6c67c8b5100425aa125baac2a28cf96333` removed both workflows and the
  marker at 00:51:30.

Native runs recorded for those commits:

- Push of `a2feb0...` created run `ps_hf4mk4q5l6`,
  trigger `push`,
  created
  00:49:30,
  finished 00:50:03.
  `depot ci status ps_hf4mk4q5l6` lists no
  workflows at all.
  `depot ci workflow list --repo Aquaticats/Monochromatic
  --sha a2feb0d0d545` returns an empty array.
- Push of `f8bc2d...` created run `ps_wsbhxt62x0`,
  trigger `push`,
  created
  00:51:35,
  finished 00:52:09.
  It also lists no workflows.
  `depot ci workflow
  list --sha f8bc2d6c67` returns an empty array.
- Push of `6523bca...` created no run record.
  `depot ci run list --repo
  Aquaticats/Monochromatic --sha 6523bcaedd` with all five statuses returns
  `null`.
- The docs-only commit `0941f588ca...` is timestamped 00:51:38,
  eight seconds
  after the cleanup push,
  and also has no run record.
  It was not part of the
  probe.

The GitHub side of the probe:

- GitHub Actions run `39182515195` exists with `event: push`,
  `head_sha:
  6523bcaeddfc3648cb66543ea94ca9ad2d932a34`,
  `created_at: 00:49:40`,
  path
  `.github/workflows/registration-probe.yml`,
  conclusion `success`.
- The check runs on `6523bca...` are only from `github-actions`;
  the probe check
  is named `probe`.
  There is no check from the Depot app on that commit.
- The check runs on `a2feb0...` and `f8bc2d...` are also GitHub Actions only;
  there are no Depot checks on either commit.

The workflow view:

- `depot ci workflow list --repo Aquaticats/Monochromatic --name
  registration-probe -n 200 --output json` returns an empty array.
- A full 200-record dump of `depot ci workflow list --repo
  Aquaticats/Monochromatic` contains no `registration-probe` entry.
  It does
  contain one push-triggered workflow from a research branch,
  `rust-wg-privileged.yml`,
  with run `ps_96gh1gkb1z` created 2026-10-18
  11:54:42,
  and six `pull_request`-triggered entries for `research native PR
  gate`.
  `depot ci status ps_96gh1gkb1z` shows that run failed with a YAML
  parse error,
  even though the workflow at that commit declares only
  `on: workflow_dispatch`.
- For the PR run `zptxtpxmn0`,
  `depot ci workflow list --sha
  69e64f40bf` (the merge sha) returns an empty array,
  while `--sha c61f59f686`
  (the head sha) returns the workflow.
  The workflow filter matches the parent
  run head SHA,
  not the merge SHA (`proto/depot/ci/v1/ci.proto:998-1035`).

### An existing positive control: native pull_request registration and checks

The repository already contains a successful native `pull_request` control.
The
workflow at `research/depot-runtime-base-20261018:.depot/workflows/research-runtime-pr.yml`
declares:

- `name: research native PR gate`
- `on.pull_request.branches: [research/depot-runtime-base-20261018]`
- `on.pull_request.paths` for the fixture paths,
  including
  `.depot/research/runtime/**` and the workflow files
- one job `research-native-gate` on `depot-ubuntu-24.04`

Source:
`git show research/depot-runtime-base-20261018:.depot/workflows/research-runtime-pr.yml`
at base commit `f7a6b290e9ac70c4ea0dbeaffb8837d8943a4bb9`.

The observed native records:

- Six workflow records with trigger `pull_request`,
  name `research native PR
  gate`,
  path `research-runtime-pr.yml`,
  created between 06:45:49 and 06:57:01
  on 2026-10-18,
  with run IDs `nhwk20hk2v`,
  `bvt87d90gp`,
  `g5dw0jfv6c`,
  `bm84mlc78m`,
  `bx47zbxvw1`,
  and `zptxtpxmn0`.
- Run `zptxtpxmn0` has trigger `pull_request`,
  ref `refs/pull/1766/merge`,
  sha
  `69e64f40bf8699cac1f36e5251a3ed5e9a587a87`,
  head sha
  `c61f59f6869d5f6c733d4024f32d8fca1e25cdde`,
  workflow
  `m83946f4d9`,
  and finished at 06:57:25.
- The commit `c61f59f6...` has a check named
  `research native PR gate / research-native-gate` from app
  `depot-code-access` (app id 219785),
  with a `depot.dev` details URL.
- That check was a required status check on the disposable base.
  The merge API
  rejected the PR with HTTP 405 while the check failed,
  and accepted it when the
  check passed.

Source:
`doc/audit/depot-runtime-pr-2026-10-18.evidence.json`,
and
`doc/planning/depot-runtime-orchestration-research.md:236-245`.

Implication:
Depot CI automatic `pull_request` registration,
`branches` and
`paths` filters,
and Depot check reporting all work on a non-default base
branch in this organization.
A strict "default branch only" reading of the docs
is therefore incomplete:
it describes push and schedule activation,
or at least
it is not the whole registration rule.

Other native push runs in the same window:

- `ps_38bl7d0p4m` for `e5c04390a5` at 00:34:14,
  finished,
  zero workflows.
- `ps_670wr8qhkd` for `5c1d1aef1b` at 00:44:14,
  finished,
  zero workflows.
- `ps_25jxlnlt9p` for `62ba3e10a61f` at 21:48:19,
  finished,
  zero workflows.
- `ps_bc8f86rs4q` for `73edb149b0a0` at 21:02:29,
  finished,
  zero workflows.

Zero-workflow runs are therefore normal in this repository.
A native run record
does not imply that a workflow executed,
and the absence of a run record does
not by itself imply that no trigger is registered.

## Why the direct-to-main probe produced no native run, ranked

### 1. The marker push raced the add push's native run (most likely)

The add push and the marker push were ten seconds apart.
The add push created
native run `ps_hf4mk4q5l6` at 00:49:30,
and that run did not finish until
00:50:03.
The marker push arrived at 00:49:35,
while that run was 28 seconds
from completion.
If Depot completes any registration or workflow sync as part of
the push handling that produced that run,
the marker push was evaluated against
the pre-add state,
where no `registration-probe` workflow existed.
The run
record then has no workflow to include,
and the push produces no matching run.

What would distinguish it:
land the workflow,
wait until its native run is
terminal plus a margin (five to ten minutes),
then push the marker.
If the
marker run appears,
the failure was latency.
If it does not,
registration did
not follow the add push even with time.

### 2. The marker push event was dropped or coalesced (possible, less specific)

There is no run record for `6523bca...` at all,
not even a zero-workflow run.
That is a stronger absence than "the workflow did not match".
The docs-only
commit `0941f588...` is timestamped 00:51:38,
eight seconds after the cleanup
push's run started at 00:51:35,
and it also has no run record.
That pattern is
consistent with a per-repository in-flight guard or debounce that drops pushes
arriving during an open run.
It is not proof,
because a docs-only push with no
matching workflow would also produce no run under a match-only rule.

What would distinguish it:
two marker pushes spaced well apart (five minutes),
with no other pushes in between.
If the first is dropped and the second runs,
event handling is the problem.
If both run,
there is no coalescing.
If neither
runs while a known-registered push workflow still fires,
the problem is
specific to the new workflow.

### 3. The oracle cannot observe registration (certain, independent of the cause)

`depot ci workflow list` lists "recent CI workflows" with `run_id`,
`status`,
`trigger`,
`sha`,
and job counts
(`pkg/cmd/ci/workflow.go:178-300`,
`proto/depot/ci/v1/ci.proto:998-1035`).
Each row is a workflow that belongs to a
run.
There is no field or command for a registered trigger that has not
executed,
and the public CI service has no such RPC
(`proto/depot/ci/v1/ci.proto:7-91`).

The `--name` filter is a substring match on the workflow name or path fallback
(`proto/depot/ci/v1/ci.proto:1001-1002`),
so the filter value itself was fine.
The problem is the noun:
the command lists executions,
not registrations.

What would distinguish it:
after landing the control workflow,
do not trigger
it,
then query `workflow list`.
It will be empty.
Then trigger it and query
again.
Only the second query can speak to execution,
and neither query can
speak to registration.

A second oracle trap is worth recording:
`depot ci run list` defaults to
queued and running when no status is given
(`pkg/cmd/ci/run.go:810-816`,
`proto/depot/ci/v1/ci.proto:961-964`).
A finished probe run is invisible to the
default command.
The evaluation avoided that trap by using `workflow list`,
but
replaced it with the execution-versus-registration trap.

### 4. The two-minute cleanup removed the only workflow that later registration could run (certain)

The probe workflow was deleted at 00:51:30,
about two minutes after it was
added.
Even if Depot registered it after the add run finished at 00:50:03,
the
only remaining push was the deletion.
A registration that completed after the
marker push had no future matching push to act on.
The observation window and
the workflow lifetime were both too short.

What would distinguish it:
keep the workflow in place for at least one full
observation cycle after a registration-completing push,
and push the marker
after that.

### 5. Push registration may be scoped differently from pull_request registration (possible)

The PR gate proves `pull_request` registration from a non-default base,
but it
does not prove push registration on `main`.
The docs' default-branch language
may apply specifically to push and schedule,
or push matching may require the
workflow to exist on the default branch before the event.
The probe cannot
distinguish either rule because the only matching push raced the add push.

What would distinguish it:
the push control in the positive-control section,
with the workflow present on `main` before a separate marker push.
If it runs,
push registration is not the difference.
If it does not,
while the PR gate still
works,
push matching is scoped differently.

### 6. Push filter handling could differ from pull_request handling (unlikely)

The compatibility matrix marks `on.push`,
`on.push.branches`,
and `on.*.paths`
as supported (<https://depot.dev/docs/ci/compatibility>).
The probe YAML used a
valid `name`,
a valid `on.push` block with `branches` and `paths`,
a supported
`permissions` value,
a valid Depot sandbox label (`depot-ubuntu-24.04`,
<https://depot.dev/docs/ci/overview#x86_64-sandboxes>),
and a valid
`timeout-minutes`.
GitHub parsed and ran the identical `.github` copy
successfully.
The native PR gate also uses `branches` and `paths` filters
successfully,
which proves that syntax works for `pull_request`.
Push-specific
path matching is not independently proven.

What would distinguish it:
run the same file with `depot ci run --workflow
.depot/workflows/registration-probe.yml` before landing it.
That proves the
file compiles and executes on Depot,
though it does not test triggers.
Then use
a control workflow with no `paths` filter to separate filter handling from
registration.

### 7. The push event was not delivered, or the app path failed for that one push (unlikely)

The preflight reported the app installed and configured,
the app subscribes to
`push`,
native push runs exist for this repository before and after the probe
(for example `ps_38bl7d0p4m` at 00:34:14 and `ps_670wr8qhkd` at 00:44:14),
and
the app writes checks for native `pull_request` runs.
A total app or webhook
outage is therefore inconsistent with the surrounding records.
A single dropped
push delivery is not excludable from outside.

What would distinguish it:
vendor-side webhook delivery logs for the
`6523bca...` push,
or a fresh control push that is known to arrive.
Without app
credentials,
this cannot be checked from this host.

## Was the probe a valid test of the documented path

No,
for four separate reasons.

1. It did not leave time for registration.
   The workflow landed at 00:49:25,
   its
   own native run started at 00:49:30 and ended at 00:50:03,
   and the only
   matching push was at 00:49:35.
   The probe tested a ten-second window with no
   documented latency bound,
   against a native run that stayed open for 33
   seconds.
2. It observed the wrong object.
   `depot ci workflow list` lists executions.
   An
   empty result cannot establish that a trigger is unregistered.
3. It deleted the workflow before any later push could exercise a registration
   that completed after the marker push.
4. The trigger push produced no run record at all.
   A missing run is a different
   observation from a run with no matching workflow,
   and the probe did not
   isolate which one occurred.

The probe also generalized from one push while the repository already contained
a successful native `pull_request` control.
It did not need to establish
registration from scratch,
and the existing gate shows the app path,
filter
syntax,
and check reporting can work.

The statement in the probe evidence that "a direct push is not the documented
merge step" is an inference,
not a quotation.
The deployed docs require the
files to be on the default branch and the workflow to contain a matching
trigger;
a direct push puts the files on that branch,
and no deployed page
excludes it.
The probe's invalidity does not depend on that distinction,
and using it as the
reason for invalidity obscures the real race and oracle problems.

## Correct minimal positive control

This experiment is not run here.
The existing `pull_request` gate is already a
positive control for PR registration and check reporting;
do not repeat it.
What remains untested is push registration on the default branch.
The experiment
separates three variables the probe conflated:
landing path (pull-request merge
versus direct push),
registration latency,
and event loss.

### Reference arm already observed: the pull_request gate

The `pull_request` gate described in the observations already demonstrates
native registration,
`branches` and `paths` filters,
and Depot check reporting
on a non-default base.
Repeating it would add cost without answering the push
question.
It also shows what a positive observation looks like:
a run with
trigger `pull_request`,
a workflow record with a non-empty path,
a finished job,
and a check from app `depot-code-access`.

### Arm 1: push trigger, workflow landed by pull-request merge

1. Confirm the app:
   run `depot ci migrate preflight --org l4zncjjbj5` and record
   the exact output.
   Confirm no native run is queued or running with `depot ci
   run list --status queued --status running`.
2. Create a branch.
   Add `.depot/workflows/registration-control.yml` with the
   content below.
   Do not add the marker file in this commit,
   so the merge push
   cannot match the paths filter.

   ```yaml
   name: registration-control
   on:
     push:
       branches: [main]
       paths: ['probe/registration-control.txt']
   permissions:
     contents: read
   jobs:
     probe:
       runs-on: depot-ubuntu-24.04
       timeout-minutes: 2
       steps:
         - run: echo "$GITHUB_EVENT_NAME $GITHUB_SHA"
   ```

3. Open a pull request and merge it.
   Record the merge commit SHA and merge time.
   Wait until any native run for the merge SHA is terminal.
   Do not push anything
   else.
4. Wait at least five additional minutes.
   The native run for the add push in the
   failed probe stayed open for 33 seconds;
   five minutes gives an
   order-of-magnitude margin without inventing an SLA.
5. Push one commit that changes only `probe/registration-control.txt` (for
   example,
   a timestamp line).
   Ensure no other push reaches the repository
   within the next five minutes.
6. Observe,
   in this order:
   - `depot ci run list --repo Aquaticats/Monochromatic --sha <marker sha>
     --status queued --status running --status finished --status failed
     --status cancelled` (all statuses explicitly;
     the default hides finished
     runs).
   - `depot ci workflow list --repo Aquaticats/Monochromatic --sha <marker sha>
     --output json`.
   - `depot ci status <run-id>` if a run exists.
   - `gh api repos/Aquaticats/Monochromatic/commits/<marker sha>/check-runs`
     and look for a Depot check named `registration-control / <job>`.
7. If nothing appears,
   wait ten minutes and push a second marker commit.
   This
   separates latency from a one-off dropped event.

Expected result if the documented path works:
a push-triggered run whose
workflow list contains `registration-control`,
a finished job,
and a Depot check
on the marker commit.

### Arm 2: push trigger, workflow landed by direct push (only if Arm 1 passes)

Repeat Arm 1 on a disposable repository,
or on the same repository with a
different workflow name,
but land the workflow with a direct push to `main`
instead of a pull-request merge.
Use the same five-minute wait and the same
separate marker push.
This is the arm that answers the actual probe question.
Do not run Arm 2 on a repository where a duplicate execution could publish or
deploy anything.

### Oracle control

Before the arms,
run the control workflow once locally with `depot ci run
--workflow .depot/workflows/registration-control.yml` and confirm that it
appears in `depot ci workflow list` with trigger `api`.
This proves the YAML is
valid and that the listing view can show a workflow.
It does not prove push
registration.
A `depot ci dispatch` control is not possible for a push-only
workflow.

### Cost bounds

The control job is a single `echo` on a 2x8 sandbox,
priced at $0.0001 per
second and billed by the second with no one-minute minimum
(<https://depot.dev/docs/ci/overview#pricing>,
<https://depot.dev/docs/ci/overview#x86_64-sandboxes>).
With
`timeout-minutes: 2`,
the worst case is 120 seconds,
or $0.012 per run.
Each
arm can produce up to three runs (merge or landing push,
first marker,
second
marker),
so both arms plus the local control are at most seven runs,
or about
$0.09.
Real cost should be a few cents because the job finishes in seconds.
No artifacts,
no
secrets,
no external writes,
and no deploy steps are involved.

### Falsifiers

- Arm 1 runs and Arm 2 does not:
  the landing path,
  or the direct-push case
  specifically,
  matters.
- Both arms run:
  the documented default-branch condition is sufficient.
- Neither arm runs,
  but the local control appears in `workflow list`:
  the
  observation path works and push registration is absent or broken for this
  repository.
- Neither arm runs and the local control also does not appear:
  the oracle or
  account path is broken,
  and no conclusion about registration is possible.
- A run appears for the first marker but not the second:
  event loss or
  coalescing,
  not registration.

## Where the repository's evaluation and capability review misstate the mechanism

### The probe evidence treats "direct push" as outside the documented path

`doc/audit/depot-registration-probe-2026-10-18.evidence.json`,
field
`documentedRegistrationPath`,
says:
"A direct push is not the documented merge
step."
The deployed docs do not say that.
The quickstart's step 2 is "Merge the
changes into your default branch",
but the operational troubleshooting
condition is "the migrated files are under `.depot/workflows/` on the default
branch and the workflow contains a trigger matching the event you sent".
A
direct push puts the same files on that branch,
so it meets the condition as
written;
no deployed page distinguishes the two landing paths.
The sentence
converts an ambiguous verb into a requirement,
and then uses that requirement to
explain a failure that is better explained by the ten-second race and the
execution-only oracle.

### The review reports registration where it observed execution

`doc/planning/depot-expanded-ci-evaluation-independent-review.md:370-376` says
"no Depot workflow registration within two minutes".
The measurement was a
`workflow list` query,
which cannot observe registration.
The accurate statement
is:
no run record for the marker commit and no workflow execution for it.
The
distinction matters because a registered trigger that never fired would produce
exactly the same empty list.

The same section treats automatic registration as unproven and pauses it until
research reports.
That is too broad.
The repository's `depot-runtime-pr`
evidence records native `pull_request` workflows with Depot checks on PR #1766
(`doc/audit/depot-runtime-pr-2026-10-18.evidence.json`),
so registration is not
absent in general.
The open question is push registration on `main`.

The same section says the direct-push observation "is not a valid negative for
the documented merge path".
The conclusion (not a valid negative) is right,
but
the reason is wrong.
The probe is invalid because it raced the registration run
and used an execution-only oracle,
not because a direct push is a different
documented path.

### The capability review compresses the conditions and overstates the default-branch rule

`doc/planning/depot-native-ci-capability-review.md:33` says "Default-branch
merging registers automatic native triggers."
The docs state three conditions:
files under `.depot/workflows/` on the default branch,
a trigger matching the
event,
and a GitHub connection through the Code Access app.
The sentence also
implies a merge event is the registrar,
which the docs do not state.

It is also contradicted as a general rule by the repository's own native PR
gate,
which ran `pull_request` workflows from
`research/depot-runtime-base-20261018`,
not `main`,
with `branches` and `paths`
filters,
and whose Depot checks were required checks
(`doc/planning/depot-runtime-orchestration-research.md:236-245`).
At best,
the
sentence is a statement about push and schedule activation.
It is a useful
summary for the adoption decision,
but it is not precise enough to test
against.

### The platform ledger's "require" is a synthesis

`doc/planning/depot-platform-surface-ledger.json:2063` says native automatic
triggers "require the Depot Code Access App and default-branch .depot
workflows".
Both conditions are documented,
but the docs also require a
matching trigger,
and the phrase does not mention the registration latency the
probe may have raced.
This is a minor compression,
not a contradiction.

### The dispatch precaution is prudent but not a mechanism claim

`doc/planning/depot-ci-adoption-proposal.md:97-100` pauses automatic native
triggers and requires explicit dispatch until a positive control passes.
That is
prudent for the push path,
which is the one the probe left untested.
The
existing PR gate is a positive control for pull_request registration,
not for
push.
It should not be read as evidence that automatic
triggers do not work;
the probe did not establish that either way.

## Open questions that only a live account or a vendor answer can settle

- Does the Code Access app deliver GitHub `push` webhooks to Depot in real time,
  and does Depot retry a failed delivery?
- Is there a per-repository debounce or in-flight guard that drops pushes
  arriving while a run is open?
  If so,
  is the first or the last push kept,
  and
  what is the window?
- When a push adds `.depot/workflows/`,
  when does registration complete relative
  to that push?
  Is the adding push itself evaluated against the new workflow?
- Is registration state per repository,
  per branch,
  or only per default branch?
  What happens to registrations when `.depot/workflows` is deleted?
- Is push registration scoped to the default branch while pull_request
  registration is scoped to the pull request's base branch?
  The PR gate worked
  from a non-default base,
  but that was a `pull_request` event.
- Must a push workflow already exist on the default branch before the event,
  or
  is the pushed commit's `.depot/workflows` evaluated?
  The probe cannot
  distinguish these because the marker push raced the add push.
- Are push triggers evaluated against the pushed commit or against the default
  branch's `.depot` tree?
  The Depot Code page says the pushed commit for Depot
  Code repositories;
  the GitHub-repository rule is not stated.
- Does `depot ci workflow list` ever include a registered workflow that has not
  executed?
  The schema and the CLI suggest no;
  a vendor answer would settle it.
- Is a pull-request merge event required,
  or is any push to the default branch
  sufficient?
  The docs do not distinguish them.
- Does the dashboard's server-side migration path (`StartMigration` in
  `depot.ci.v2`) create a repository or project link that the CLI's local
  migration does not?
- What does a 33-second zero-workflow push run represent?
  Is it a sync pass,
  a
  no-match run,
  or a run created for every push regardless of matches?
- Why did the commits timestamped 00:52:59 and 00:56:32 produce no run records
  after the cleanup,
  while earlier main pushes produced zero-workflow runs?
- Are GitHub checks emitted for a push run before,
  during,
  or only after the
  first job starts?
  The probe commit has no Depot check,
  which is consistent
  with no execution,
  but the emission rule is not documented.

## Sources

Deployed documentation,
retrieved 2026-10-19:

- Depot CI overview: <https://depot.dev/docs/ci/overview>
- Depot CI quickstart: <https://depot.dev/docs/ci/quickstart>
- Depot CI compatibility: <https://depot.dev/docs/ci/compatibility>
- Origin integration troubleshooting: <https://depot.dev/docs/ci/integrations/origin#no-run-starts-after-migration>
- Depot CI CLI reference: <https://depot.dev/docs/cli/reference/depot-ci>
- Depot CI API reference: <https://depot.dev/docs/api/ci/reference>
- Manage workflow runs: <https://depot.dev/docs/ci/how-to-guides/manage-workflow-runs>
- GitHub checks: <https://depot.dev/docs/ci/observability/github-checks>
- Depot Code overview: <https://depot.dev/docs/code/overview>
- Depot GitHub Actions runners: <https://depot.dev/docs/github-actions/overview>
- Markdown source index: <https://depot.dev/llms.txt> and
  <https://depot.dev/llms-all.txt>

Note on documentation provenance:
the public `depot/docs` repository at its
current HEAD (`e40bfe1683ae561156d415513f2a648f89e82f12`,
2026-09-11) contains
the same registration sentences in `content/ci/overview.mdx:125`,
`content/ci/quickstart.mdx:37` and `:84`,
and
`content/ci/integrations/origin.mdx:60`.
Its quickstart text lags the deployed
page elsewhere,
for example the app installation target and the secrets-import
flow.
The deployed pages above are treated as authoritative and were retrieved
on 2026-10-19.

CLI source at tag `v2.102.23`,
cloned from `https://github.com/depot/cli`:

- `pkg/cmd/ci/migrate.go:326-402` (GitHub preflight,
  installation check,
  final
  message)
- `pkg/cmd/ci/migrate.go:405-419` (migrate runs preflight,
  then local copy)
- `pkg/cmd/ci/migrate.go:700` (local `os.WriteFile` of migrated workflows)
- `pkg/cmd/ci/migrate.go:773-798` (next steps:
  push and merge to activate)
- `pkg/cmd/ci/migrate_secret_intent.go` (secret import intent,
  unrelated to
  triggers)
- `pkg/cmd/ci/dispatch.go:27-77` (dispatch command and request)
- `pkg/cmd/ci/workflow.go:178-300` (workflow list command)
- `pkg/cmd/ci/run.go:810-816` (run list defaults to queued and running)
- `pkg/api/ci.go:378-397` (`CIRun` and `CIDispatchWorkflow`)
- `proto/depot/ci/v1/ci.proto:7-91` (CI service methods,
  no registration RPC)
- `proto/depot/ci/v1/ci.proto:148-172` (`RunRequest`)
- `proto/depot/ci/v1/ci.proto:183-196` (`DispatchWorkflowRequest`)
- `proto/depot/ci/v1/ci.proto:961-964` (`ListRuns` status default)
- `proto/depot/ci/v1/ci.proto:998-1035` (`ListWorkflows` request and response)
- `proto/depot/ci/v2/migration.proto:8-26` (migration service methods)

GitHub records:

- App metadata: <https://api.github.com/apps/depot-code-access>
- Probe GitHub Actions run:
  <https://api.github.com/repos/Aquaticats/Monochromatic/actions/runs/39182515195>
- Check runs for the three probe commits:
  `https://api.github.com/repos/Aquaticats/Monochromatic/commits/<sha>/check-runs`

Local observation commands:

- `depot ci run list --repo Aquaticats/Monochromatic --status finished --status
  failed --status cancelled --status running --status queued -n 60 --output
  json`
- `depot ci run list --repo Aquaticats/Monochromatic --sha <sha> --status
  finished --status failed --status cancelled --status running --status queued
  --output json`
- `depot ci run show ps_hf4mk4q5l6` and `depot ci run show ps_wsbhxt62x0`
- `depot ci status ps_hf4mk4q5l6`,
  `depot ci status ps_wsbhxt62x0`,
  `depot ci status ps_38bl7d0p4m`,
  `depot ci status ps_670wr8qhkd`
- `depot ci workflow list --repo Aquaticats/Monochromatic -n 200 --output json`
- `depot ci workflow list --repo Aquaticats/Monochromatic --name
  registration-probe -n 200 --output json`
- `depot ci workflow list --repo Aquaticats/Monochromatic --sha <sha> --output
  json`

Repository records read:

- `doc/audit/depot-registration-probe-2026-10-18.evidence.json`
- `doc/audit/depot-runtime-pr-2026-10-18.evidence.json`
- `doc/planning/depot-runtime-orchestration-research.md:236-245`
- `git show research/depot-runtime-base-20261018:.depot/workflows/research-runtime-pr.yml`
- `doc/planning/depot-native-ci-capability-review.md:33`
- `doc/planning/depot-expanded-ci-evaluation-independent-review.md:370-376`
- `doc/planning/depot-ci-adoption-proposal.md:97-100`
- `doc/planning/depot-platform-surface-ledger.json:2063`
- `doc/planning/depot-developer-pool-capabilities-research.md:468-472`

## Follow-up control result, 2026-10-19

Both landing arms passed after this report was written.

- Arm 1,
  pull-request merge landing:
  PR #1775 merged as `d642e9f0`;
  the later marker push `5361ae16fe` produced native run `ps_4dzmpnfm09`,
  workflow `registration-control`,
  a finished `probe` job,
  the log line `push 5361ae16fe`,
  and the Depot check `registration-control / probe` from app `depot-code-access`.
- Arm 2,
  direct-push landing:
  workflow commit `9136cd78`;
  the later marker push `86499d3d9f` produced native run `ps_9hmw58cpsd`,
  workflow `registration-control-direct`,
  a finished `probe` job,
  the log line `push 86499d3d9f`,
  and the Depot check `registration-control-direct / probe` from app `depot-code-access`.
- Cleanup commit `e85603fc65` removed both workflows and markers;
  the cleanup push produced a zero-workflow sync run and no workflow execution.

Conclusion:
push registration on `main` works for both landing paths.
The earlier direct-to-main probe was invalid for the timing,
oracle,
and cleanup reasons above,
not because push registration was absent.
The corrected control is recorded in
`doc/audit/depot-registration-control-2026-10-18.evidence.json`.
