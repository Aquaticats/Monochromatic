# `gh run watch --exit-status` exits 1 for its own API errors, as it does for a failed run

`gh` `2.103.3` (`2026-10-26`,
tag `v2.103.3`,
commit `63837fb778d6a39f02be0ce76ff89f2bc8d74bdc`) exits 1 from
`gh run watch <id> --exit-status` in three different situations:
the watched run concluded otherwise than success,
the run had already concluded otherwise than success when the watch started,
and the watcher itself failed while the run was still going,
for example one API call that answered 500.
The exit status therefore says nothing about the watched run,
and a script that treats 1 as "the run failed" reports a passing run as failed.

This repository shipped that mistake:
`mise run //package/git-policy/cli:native:release:wait -- 42935168319 0.6.1-alpha.9` printed
`watch exit 1` and `RELEASE RUN FAILED: no publish was started` for a release run whose recorded conclusion is
`success`,
3.6 seconds before its last job finished.

## Symptom

A waiter built on `gh run watch --exit-status` fails a run that passed:

```text
[//package/git-policy/cli:native:release:wait] $ node ../../crate-release/cli/bin/await-release.ts --crate monochromatic-git-policy-cli 42935168319 0.6.1-alpha.9
crate-release.yml run 42935168319
watch exit 1
RELEASE RUN FAILED: no publish was started. Read its log with: gh run view 42935168319 --log-failed
[//package/git-policy/cli:native:release:wait] ERROR task failed
```

The run's own record disagrees:

```sh
gh run view 42935168319 --json=status,conclusion,jobs \
  --jq '{status, conclusion, jobs: [.jobs[] | {name, status, conclusion}]}'
```

```json
{"status":"completed","conclusion":"success",
 "jobs":[{"name":"release","status":"completed","conclusion":"success"},
         {"name":"publish","status":"completed","conclusion":"success"}]}
```

The same command exits 0 whenever it is run against that run now:

```text
$ gh run watch 42935168319 --exit-status; echo "watch exit=$?"
Run crate-release (42935168319) has already completed with 'success'
watch exit=0
```

The saved log of the failed wait was last written at `2026-11-11T01:00:27.441Z`
(`${HOME}/temp/agent/disk-usage-2026-11-10/release-alpha9-wait-2.log`),
while the run's `publish` job completed at `2026-11-11T01:00:31Z` and the run's `updated_at` is `01:00:32Z`.
So the watcher exited 3.6 seconds before the run ended,
and its 1 could not have been the run's conclusion.

What the watcher printed on stderr was thrown away:
the waiter captured `gh`'s output and printed only its exit status,
so the message that would have named the real reason was lost.

## Root cause

`gh run watch` has three ways to exit 1,
and only one of them is the run's conclusion.

1. The run had already ended before the watch started
   ([watch.go `L139-L146`](https://github.com/cli/cli/blob/63837fb778d6a39f02be0ce76ff89f2bc8d74bdc/pkg/cmd/run/watch/watch.go#L139-L146)):

   ```go
   	if run.Status == shared.Completed {
   		fmt.Fprintf(opts.IO.Out, "Run %s (%s) has already completed with '%s'\n", ...)
   		if opts.ExitStatus && run.Conclusion != shared.Success {
   			return cmdutil.SilentError
   		}
   		return nil
   	}
   ```

2. The watch loop broke on its own error and returned it
   ([watch.go `L162-L192`](https://github.com/cli/cli/blob/63837fb778d6a39f02be0ce76ff89f2bc8d74bdc/pkg/cmd/run/watch/watch.go#L162-L192)):

   ```go
   	for run.Status != shared.Completed {
   		run, err = renderRun(out, *opts, client, repo, run, prNumber, annotationCache)
   		if err != nil {
   			break
   		}
   ```

   ```go
   	if err != nil {
   		return err
   	}
   ```

   `renderRun` wraps the three reads of one poll:
   the run ([watch.go `L222`](https://github.com/cli/cli/blob/63837fb778d6a39f02be0ce76ff89f2bc8d74bdc/pkg/cmd/run/watch/watch.go#L222)),
   its jobs ([watch.go `L227`](https://github.com/cli/cli/blob/63837fb778d6a39f02be0ce76ff89f2bc8d74bdc/pkg/cmd/run/watch/watch.go#L227)),
   and the annotations of each job
   ([watch.go `L243`](https://github.com/cli/cli/blob/63837fb778d6a39f02be0ce76ff89f2bc8d74bdc/pkg/cmd/run/watch/watch.go#L243)).
   A 403 on annotations is handled as missing permissions;
   every other error,
   such as a 500 on the jobs URL,
   ends the watch mid-run.

3. The run ended other than successfully
   ([watch.go `L208-L210`](https://github.com/cli/cli/blob/63837fb778d6a39f02be0ce76ff89f2bc8d74bdc/pkg/cmd/run/watch/watch.go#L208-L210)):

   ```go
   	if opts.ExitStatus && run.Conclusion != shared.Success {
   		return cmdutil.SilentError
   	}
   ```

`cmdutil.SilentError` is "an error that triggers exit code 1 without any error messaging"
([errors.go `L34-L35`](https://github.com/cli/cli/blob/63837fb778d6a39f02be0ce76ff89f2bc8d74bdc/pkg/cmdutil/errors.go#L34-L35)),
and `gh` documents exit code 1 as "if a command fails for any reason" (`gh help exit-codes`).
So paths 2 and 3 are indistinguishable by exit status,
and path 2 can only be told apart by reading the error text that path 3 never prints.

An earlier reading of this incident was wrong and is worth naming:
the plausible cause was "the watcher read the run while the API still said `completed` with an empty conclusion".
That was disproved by the two job records above:
at `01:00:27.441Z` the `publish` job had not completed,
so no reading of the run could have called it ended.
The run's `status` field is not a reliable progress signal either:
GitHub computes it from the least advanced job and it can move backwards
([cli/cli#13972](https://github.com/cli/cli/issues/13972)),
while each job carries its own `status`.

## Verification

Version under test:
`gh` `2.103.3` (`2026-10-26`),
the toolchain package this host runs
(`gh --version` prints `gh version 2.103.3 (2026-10-26)`).

The harness drives the real `gh` binary against a stand-in GitHub API over HTTPS on `localhost`:

- a self-signed certificate for `localhost` is trusted through `SSL_CERT_FILE`,
  which Go's certificate verification honours;
- `GH_HOST=localhost:8443` and `GH_ENTERPRISE_TOKEN=probe-token` select the stand-in host,
  and `GH_CONFIG_DIR` points at a scratch directory;
- the stand-in serves `/api/v3/repos/acme/widget/actions/runs/4242`,
  the workflow (`/actions/workflows/403381398`),
  the jobs URL,
  and `/check-runs/<id>/annotations`;
- a phase file makes the jobs endpoint fail,
  so the watch meets an error exactly where a real transient API failure would.

Script (`${HOME}/temp/agent/release-wait-2026-11-11/probe/driver.ts` and `server.ts`,
both scratch and not tracked in this repository,
so the section above is what a later session rebuilds them from):
start the watch against a run whose status is `in_progress` with its last job running,
wait 3.5 seconds,
switch the phase so the jobs endpoint answers `500`,
then run the same command against the run recorded as `completed` with `success`.

Output:

```json
{"first":{"code":1,
          "stderr":"failed to get jobs: HTTP 500: Server Error (https://localhost:8443/api/v3/repos/acme/widget/actions/runs/4242/jobs?per_page=100)",
          "label":"api error while the run is still going"},
 "second":{"code":0,
           "stdout":"Run crate-release (4242) has already completed with 'success'",
           "label":"run already completed with success"}}
```

The run was still going when the first command exited 1.
`gh`'s own requests in that run,
in order,
end with
`probe error /api/v3/repos/acme/widget/actions/runs/4242/jobs` and then nothing:
the watcher stopped instead of polling again.

### Catalog: exits 0 with `--exit-status`

- A run recorded as `completed` with `success`,
  watched after it ended:
  exit 0,
  `Run <workflow> (<id>) has already completed with 'success'`.
  Measured twice:
  against the stand-in API,
  and against the real run `42935168319`.

A live watch whose every poll answers cleanly ends in the same final check as case 3 of the root cause,
which returns 0 for the `success` conclusion;
that passing variant was read from the source rather than run,
because the already-completed case exercises the same check.

### Catalog: exits 1 with `--exit-status`

- A run that concluded `failure`,
  watched after it ended:
  exit 1,
  `Run crate-release (42934268993) has already completed with 'failure'` on standard output and nothing on standard
  error (path 1,
  `SilentError`),
  measured against that real run.
- A run that concluded `failure` while being watched:
  exit 1,
  with nothing on standard error,
  which is what `SilentError` does;
  that variant was read from the source and not run,
  because it needs a run that fails during the watch.
- A run still going whose jobs URL answers `500` once:
  exit 1,
  `failed to get jobs: HTTP 500: Server Error` (path 2,
  the one that cost this repository a false failure).
- A run still going whose run URL answers `404` on the first read:
  exit 1,
  `failed to get run: HTTP 404: Not Found (...)`.

Paths 1 and 3 report the run's own conclusion;
both of those failures print nothing on standard error.
Path 2 reports the watcher,
the network or the API,
and is the only one that prints a message.

## Verified workarounds

Read the run's own record and decide from that,
instead of from the watcher's exit status:

```sh
gh run view <id> --json=createdAt,status,conclusion,jobs
```

A run passed only when `status` is `completed` and `conclusion` is `success`;
a job that did not pass is named by its own `conclusion`,
with the first step whose `conclusion` is not `success`,
`skipped` or `neutral`;
a run that is not `completed` is followed again after a pause.

Tradeoffs:

- The poll loop is the caller's,
  so a caller that never stops polling never returns.
  Bound the number of looks and name the bound in the output.
- One look per interval instead of one every three seconds costs responsiveness on a fast run
  and saves API budget on a slow one.
- `status` alone would be a poor test of "still going"
  ([cli/cli#13972](https://github.com/cli/cli/issues/13972)),
  because it is computed from the least advanced job and can regress;
  the jobs list is what shows progress.
- A run that ended with no conclusion at all reads as neither pass nor fail,
  so the caller must decide what an empty conclusion means,
  and saying "completed with no conclusion" is safer than calling it a pass.

This repository's implementation is `package/crate-release/cli/src/run-outcome.ts`
and its tests read answers recorded from four real runs under
`package/crate-release/cli/src/fixture/`.

## What does not work

- Treating `--exit-status` 1 as "the run failed":
  it also covers the watcher's own errors,
  as the incident above shows.
- Parsing the watcher's stderr to tell the two apart:
  path 3 prints nothing at all,
  so the absence of a message cannot be read as a failed run either.
- Watching the run a second time after a 1:
  a completed run answers 0 and a failed one answers 1,
  but a run that is still going can fail the watch again,
  so the second watch only reduces the odds instead of removing the ambiguity.
- `gh run view <id>` without `--json`:
  the human report prints the same fields,
  but its text is a display format,
  not a contract.
- Reading `gh run view --json=conclusion` alone:
  it is empty for a run that has not ended,
  and it does not name the job or the step that did not pass.

## Upstream filing decision

Nothing is filed.
The audit below records why,
so a later session can re-run it if upstream signal changes.

Duplicate and prior-art search,
run 2026-11-11 with `gh search issues --repo cli/cli`:

- `run watch exit status` returned five issues,
  none of them this conflation:
  [#13972](https://github.com/cli/cli/issues/13972) (run status tracks the least advanced job),
  [#2835](https://github.com/cli/cli/issues/2835) (get action status),
  [#3577](https://github.com/cli/cli/issues/3577) (`failed to get annotations` on skipped runs),
  [#3962](https://github.com/cli/cli/issues/3962) (`--exit-status` once exited 0 for an already failed run),
  and one unrelated thread.
- `failed to get jobs run watch` returned nothing.
- `watch exits 1 api error run still running` returned nothing.
- `watch run failed exit code` returned nothing.

Nothing to add to an existing thread:
the threads found are about the `status` fields and about the annotations read,
not about one exit status covering both verdicts.

The 6-constraint check:

1. Upstream's fault?
   Partly a wording and contract issue rather than a bug:
   `gh help exit-codes` says exit 1 means "a command fails for any reason",
   so both meanings are consistent with the documentation,
   and `--exit-status` only promises "non-zero status if run fails".
2. Can upstream fix it?
   Yes.
   Any of three changes would:
   a distinct exit code for a watcher error,
   a `--json` verdict on `run watch`,
   or
   `--exit-status` refusing to exit on its own errors.
3. Do they support the use case?
   Yes:
   `--exit-status` exists so that
   `gh run watch && notify-send` scripts can decide,
   and the front matter of the help text is about exactly that.
4. Would the repo welcome a contribution?
   Not checked in this session:
   `CONTRIBUTING.md`,
   the issue forms and the maintainer responses were not read,
   so this constraint is recorded as unknown rather than passed.
5. Would they likely fix it?
   Unknown:
   no issue for this behaviour exists,
   and no maintainer statement about the exit-code contract was found.
6. Has a minimal fix been prototyped?
   No.
   Each candidate fix in constraint 2 changes the documented exit-code contract for every script that reads
   `gh run watch --exit-status`,
   and choosing which one is upstream's decision,
   not ours.
   The consumer-side fix removes this repository's need for the change.

Because constraints 4,
5 and 6 are not met,
there is no fileable draft.
If a later session prototypes one exit-code shape and reads the contribution policy,
the draft below is the starting point.

```text
do not file as-is

Title: gh run watch --exit-status exits 1 for its own API errors, as it does for a failed run

`gh run watch <id> --exit-status` exits 1 both when the run concluded otherwise than success and when the watcher's
own API call failed while the run was still going. A script that reads the exit status cannot tell a failed run from a
failed watch, and a passing run gets reported as failed.

Reproduction: watch a run in progress while its jobs URL answers 500 once.
The command exits 1 with `failed to get jobs: HTTP 500 ...`, and the same run later answers
`has already completed with 'success'` and exits 0.

Suggested fix: give the watcher's own failure a different exit code from a run verdict, or refuse to apply
--exit-status when the poll loop broke on an error.
```

Risk to the poster,
should a later session decide to file:
filing puts a person's GitHub account in front of maintainers we do not know,
some repositories run trust or blocklist systems that can follow an account to unrelated projects,
and an agent-written report judged low-effort is exactly what such systems are for.
Filing is the account owner's decision,
not something this document or an agent settles.

## References

- `package/crate-release/cli/src/run-outcome.ts`,
  the reader of a run's own answer.
- `package/crate-release/cli/src/wait-run.ts`,
  `followRunOutcome`,
  the bounded follow loop.
- `package/crate-release/cli/src/fixture/recorded-run-answers.ts`,
  the recorded answers the tests read.
- `${HOME}/temp/agent/disk-usage-2026-11-10/release-alpha9-wait-2.log`,
  the saved wait that reported the false failure.
