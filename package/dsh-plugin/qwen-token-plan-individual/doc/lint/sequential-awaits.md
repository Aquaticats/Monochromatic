# `no-await-in-loop` suppressions in the Qwen Token Plan Individual DSH plugin

## Rule and configuration

`package/config/oxlint/src/rule/correctness.ts` enables
`eslint/no-await-in-loop` as a warning,
and this package's lint exits 1 on any warning.

The rule has no options
and no allow-list,
so a loop that must wait for one step before the next carries one
statement-scoped `oxlint-disable-next-line` comment on the `await` line.

Turning the rule off for this package would loosen a lint rule,
which needs prior approval.

## Where the order is the design

`src/providers.ts` `writeQwenTokenPlanIndividualRoute` retries one settings
write at a time
(`await settings.update(...)`),
because the retry exists for exactly the case where a competing writer holds
the same `package.json` writer lock across the attempt
(the plugin-manager's boot reload,
measured at 11151 ms).
Issuing the next write before the previous one settles would only queue both
behind the same hold instead of landing the second after it.

The same loop's `await sleep(...)` between attempts is the pause the retry is
made of:
an attempt is worth making only after the previous one has failed and the
hold it measured has had time to release,
so firing the attempts together would measure nothing.

`src/providers.ts` `lookupQwenTokenPlanIndividualCredentials` waits between
credentials lookups for the same reason:
the loader registers the credentials service at some point after this
plugin's `apply` runs,
and the interval is what lets that registration land instead of every
attempt repeating an unchanged lookup at once.

## Where the wait is for something outside the process

All three waits are for state another part of the same host process
(the writer lock holder, the loader's service registry) changes while the
loop runs.
Each read is of the whole current state,
so a later attempt supersedes an earlier one and attempting them together
would report one instant's state for every attempt instead of waiting for
the change.
The intervals are the design:
both are bounded by `QWEN_TOKEN_PLAN_INDIVIDUAL_MOUNT_RACE_RETRY_ATTEMPTS`
attempts
(`src/constants.ts`),
and attempting sooner would only repeat an unchanged observation.
