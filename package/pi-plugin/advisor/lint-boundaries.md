# Intentional sequential operation waits

## Rule and configuration

`package/config/oxlint/src/rule/correctness.ts:79` enables `eslint/no-await-in-loop` as a warning.
The inspected [Oxlint rule source][] defines a zero-field `NoAwaitInLoop` type
 and implements `Rule::run` without a rule-specific configuration parser or allowlist.
It reports an await nested in a loop and recommends parallelizing independent operations.

There is no rule option for a state-dependent scheduler.
Disabling the rule for this package would also exempt unrelated loops,
 so the operation uses a statement-scoped suppression instead.

## Operation scheduler

`src/operation.ts` waits for the next provider outcome,
 cancellation,
 launch boundary,
 or collection deadline.
Only after that event may it decide whether another dispatch is permitted.
The user's accepted policy forbids replacements after the first usable result
 and excludes every model on a provider as soon as exhausted credits are observed.

These waits are causally dependent.
Starting them together with `Promise.all` would either schedule from obsolete state
 or wait for every provider before enforcing collection cutoffs.
Actual provider work is already concurrent when overlap is enabled,
 with at most two running reviewers.
Only scheduling decisions remain sequential.

## Deterministic fixture drain

`src/operation-test-fixture.ts` advances a fixture-local clock through queued events.
After delivering an event,
 its promise continuations must settle before the clock advances again.
Otherwise a response that completed before the cutoff could be observed only after the fixture moved past it.
The drain therefore awaits each event's microtasks sequentially.
Its queue comes only from finite test scripts and bounded provider attempts.

## Disposable host sequence

`src/verify-host.ts` runs its fixed scenario list sequentially.
Only one child Pi process is live at a time,
 and each child has a separate disposable home and a parent watchdog.
Parallelizing the scenario loop is unnecessary for the integration contract
 and would change the chosen child-process resource bound.

## Timer input bounds

The [Node timer contract][] sets delays greater than `2147483647`,
 below `1`,
 or `NaN` to `1`.
Advisor therefore validates configured operation timeout,
 hedge delay,
 and collection grace as positive finite integers within that range.
An invalid large timeout must not turn repeated scheduler waits into one-millisecond wakeups.

No suppression changes the rule's severity or conceals unbounded provider waits.
Operation waits still have independent local cancellation and deadline boundaries.

[Node timer contract]: https://nodejs.org/api/timers.html#settimeoutcallback-delay-args
[Oxlint rule source]:
  https://raw.githubusercontent.com/oxc-project/oxc/main/crates/oxc_linter/src/rules/eslint/no_await_in_loop.rs
