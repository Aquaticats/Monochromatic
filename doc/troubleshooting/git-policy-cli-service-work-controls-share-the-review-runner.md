# The service work controls reviewed in the one runner of the process, so one of two reviews was refused

`package/git-policy/cli/src/native/landing/service_work_tests.rs` holds two controls that review through the
service's real handlers
(`GitHandlers::review` and `GitHandlers::review_awaiting`),
and those handlers ran every review in `standards_review_run::RUNNER`,
the one runner the whole service process shares.
A runner admits one job and answers every other caller `Contained::Busy`,
and `cargo test --lib` runs a crate's tests as threads of one process,
so two of those controls reviewing at one moment refused each other.
This is debt ledger item 3
(`package/git-policy/cli/doc/debt-ledger-2026-11-25.md`)
and issue 6500,
beside issue 6499's round,
which gave the `service::service_review` controls runners of their own.

## Symptom

A threaded run of the module
(`landing::service_work::`,
guest and arm tests skipped,
default threads)
did not show it:
it answered
`test result: ok. 117 passed; 0 failed; 0 ignored; 0 measured; 10567 filtered out; finished in 20.92s`
on 2026-11-26,
and it did after the change too,
because only two of those controls review and their reviews rarely meet in one moment.

Two controls in one process do show it.
Libtest takes more than one filter,
so these two run together at two threads and nothing else runs:

```console
$ cargo test --locked --offline --lib --features test-only-switches -- \
    landing::service_work::tests::a_review_runs_through_the_real_handlers_in_a_service_context \
    landing::service_work::tests::the_awaited_review_of_the_handlers_answers_what_the_synchronous_one_answers \
    --test-threads=2 --skip guest --skip _arm_tests
test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 10682 filtered out; finished in 20.03s
test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 10682 filtered out; finished in 20.04s
test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 10682 filtered out; finished in 20.04s
```

Which control failed varied:
the first run failed `a_review_runs_through_the_real_handlers_in_a_service_context`,
and the other two failed `the_awaited_review_of_the_handlers_answers_what_the_synchronous_one_answers`.
Each of the two passed alone (`--exact`),
`test result: ok. 1 passed; 0 failed` in 0.05 and 0.10 seconds.

## Root cause

`GitHandlers::review` called `review_item`,
and `GitHandlers::review_awaiting` called `review_item_awaiting`,
and both of those run the job in `crate::standards_review_run::RUNNER`
(`service/service_review.rs`).
`RUNNER` admits one job at a time for the whole process,
and a second caller is answered `Contained::Busy`,
which the item's verdict turns into a failed review that names the command to review it again
(`service_review.rs`'s `busy_reason`).

Issue 6499's round banned a control from calling those functions
(`source_guard/no_control_takes_the_production_runner_tests.rs`),
but these controls name none of them:
the runner is reached two calls deeper,
inside the handlers,
where a text scan of the controls' own source cannot see it.

## The evidence

The review the shared runner refused,
from the run's own service line:

```text
monochromatic-git-policy-cli-service: the review of <oid> failed: the standards review of <oid> was not started
because an earlier review, stopped at its ceiling, is still running and may hold the review log; review it again,
once that review has ended, with `SHADOW_REVIEW_SHA=<oid> mise run shadow-review`
```

and the assertion that failed because of it,
the mock gateway the control serves having been asked nothing:

```text
panicked at src/native/standards_review/standards_review_mock.rs:401:13:
no call reached the mock server within 20s, so the control it serves proved nothing
```

## The change

The handlers carry the runner a control hands them,
as they already carry the review's endpoint and the push's environment:
`GitHandlers::with_review_runner(&OWN)` is `#[cfg(test)]`,
`for_round` carries it into the round's copy,
and `review` and `review_awaiting` run their job in `review_runner_of()`,
which is that runner when a control gave one and `RUNNER` otherwise.
Each of the two controls declares `static OWN: Containment` and hands it to every pair of handlers it builds,
so a control's reviews meet only its own jobs.

No shipped behaviour changed:
every seam and every default is `#[cfg(test)]`,
the release hands no runner and therefore takes `RUNNER`,
and what `package/git-policy/cli/SPEC.md` states is what it does
(`:10218`,
"The whole review of a commit is waited for `REVIEW_CEILING` (2 minutes) by the service (`run_contained`);
a review it stopped waiting for is counted as outstanding until its thread ends and no other review starts
meanwhile").

The same two controls in one process after the change:

```console
$ cargo test --locked --offline --lib --features test-only-switches -- \
    landing::service_work::tests::a_review_runs_through_the_real_handlers_in_a_service_context \
    landing::service_work::tests::the_awaited_review_of_the_handlers_answers_what_the_synchronous_one_answers \
    --test-threads=2 --skip guest --skip _arm_tests
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 10682 filtered out; finished in 0.10s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 10682 filtered out; finished in 0.10s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 10682 filtered out; finished in 0.10s
```

and the module answered
`test result: ok. 117 passed; 0 failed; 0 ignored; 0 measured; 10567 filtered out; finished in 19.05s`.

## What the guard is worth

`source_guard/no_control_takes_the_production_runner_tests.rs` now sees this route:
a test-only source that points the real handlers' review at a gateway
(`with_review_endpoint(`)
must also hand them a runner
(`with_review_runner(`),
and its fixture plants one file of each shape,
so a scan that stopped reading the route fails on the planted file.

With the two controls' `.with_review_runner(&OWN)` calls taken out and the guard unchanged,
the guard's control answered `test result: FAILED. 2 passed; 1 failed` and named the four sites:

```text
landing/service_work_tests.rs:1616 names `with_review_endpoint(`, ...
landing/service_work_tests.rs:2955 names `with_review_endpoint(`, ...
landing/service_work_tests.rs:2958 names `with_review_endpoint(`, ...
landing/service_work_tests.rs:2994 names `with_review_endpoint(`, ...
```

With the route check itself taken out of the scan,
the planted-route control failed and reported only the planted call,
`left: ["helper_tests.rs:6 names `review_item(`, which runs a work item's review in `RUNNER`"]`,
which is what says the fixture is not passing while reading nothing.
Both were restored.

## Not covered

- The check is per file:
  a file that hands one pair of handlers a runner and points a second pair at a gateway passes,
  and a control that reviews through the real handlers without pointing their call at a gateway is not seen.
- The container gate was not run for this change:
  the round ran the modules it touched on this host,
  and a whole-library threaded run is not run here
  (`package/git-policy/cli/doc/how-to-write-this-packages-code-and-tests.md`).
- The rest of item 3's sites stay as the ledger measures them.
