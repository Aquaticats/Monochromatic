# Preventing mistakes: spend guards and the attempts a client makes

Rules for a guard that holds a run to an amount of money:
what it admits,
what it counts,
the retries a client makes inside the call it admits,
and a wait taken again for a provider that has just refused.

Part of [Preventing this package's mistakes](mistake-prevention.md),
which lists every file of this record.
It is a new file of 2026-10-24,
because [Preventing mistakes: guards, searches and claims](mistake-prevention-guards-searches-and-claims.md)
stood four lines short of the 1,000-line limit.

## Attempts a guard cannot see

What happened:
the editor fidelity probe's spend guard asked,
before each call,
whether the spend so far plus that call's ceiling fitted within `--max-usd`.
The Claude client sends up to five attempts for one call,
and Anthropic bills an attempt that delivered a stream whether or not it was read.
So a call admitted at one ceiling could be billed at up to five,
and an arm could pass its amount by the retries of its last call,
with the guard saying it had held (ledger B885's follow-up).
The same shape had stood one layer down in the round before:
the guard counted a call once while the client wrote one line for the last attempt and none for the abandoned ones
(ledger B885).
Neither the guard's tests nor the client's tests were wrong on their own terms.
Each looked at one call,
and the multiplier lived in the layer between them.

The rule:
a guard admits what is sent,
and what is sent is an attempt.
When a client retries inside a call,
the guard is told before each attempt,
the first and every retry,
with the most that attempt can be billed,
and it refuses the attempt before it is sent.
A guard that can only see calls is a runaway stop on the replies it sees,
and says so.
The same rule holds for any other limit a retrying layer can multiply:
a count of asks,
a quota,
a rate.
The multiplier is read off the client's source (`TRANSIENT_RETRY_LIMIT` in `provider/src/transient-retry.ts`
and what retries),
not off the call the test makes.

What a guard charges follows from the same reading.
A call that never left,
rejected by a layer above the client,
was never billed and counts nothing.
A call that threw counts each attempt once,
by the line the client wrote for it,
and counts the call's ceiling only when its last attempt wrote no line.
A reply with usage but no line on the meter counts at its ceiling,
since an unknown cost may not be assumed small.

What enforces it:
`provider/src/transient-retry.unit.test.ts` fails when an attempt is sent before the caller is told of it,
`run/src/corpus-run/claude-attempt-admit.unit.test.ts` fails when the route the probe builds does not hand the admit
down,
and `probe/src/corpus-run/editor-fidelity-spend-guard.unit.test.ts` drives the whole route with a transport that
answers a retryable status four times at a spent sum where a guard of calls admits the call and the fifth attempt
would cross the cap.
Three mutants of the guard each fail a case:
the refusal of an attempt removed,
a call that never left charged a ceiling,
and a last attempt that wrote a line charged again.
No scan finds a guard built over a retrying client,
so a new guard is read against the client it wraps by hand,
and its first case is the retry that crosses the cap.

How to find the next one:
list every place a limit is checked and the layers between the check and the wire,
and for each layer ask how many sends one admitted unit can become.
A positive control comes first:
a scripted transport that fails the first attempts with a retryable status,
at a spent sum chosen so that the unit fits and its retries do not.

## A guard's assumption is asserted

What happened:
the editor fidelity probe's spend guard keeps the attempts the client told it of for the call in flight,
and a comment said this holds because the probe asks one row at a time.
Nothing checked it.
A change that overlapped two calls would have read one call's attempts as the other's,
and counted a throw at the wrong call's ceiling or not at all,
with every test still green (ledger B897).
Two edges of the same guard had no case of their own:
the attempt whose bound brings the spend exactly to the amount,
and the other methods of the client,
which a reader might fear billed outside the guard.

The rule:
an assumption a guard's accounting rests on is asserted in the guard,
with a case that breaks it,
and a comment that says "holds because" is a missing assertion.
The guard throws when a second call is asked while one is in flight.
Each boundary of the comparison that refuses (`<=`) has a case on its own edge,
and a mutant of the comparison is built and run once to see the case fail.
A method of the guarded client that can bill is guarded,
and one that cannot is shown not to by a case over the real client with a scripted transport:
the case asks the method and sees no exchange,
and asks a chat call on the same client and sees one,
so an empty transport cannot pass it by being empty.

What enforces it:
`probe/src/corpus-run/editor-fidelity-spend-guard.unit.test.ts` holds the one-call-at-a-time case,
the attempt-only boundary (0.5 USD spent,
an attempt bound of 0.5 admitted and one of 0.51 refused under 1)
and the zero-exchange case for `decide` and `quotas`.
No scan finds a comment that states an assumption,
so a new guard's author lists the assumptions its comments make and asserts each.

How to find the next one:
read the guard's comments for "because",
"holds",
"only" and "one at a time",
and for each,
find the line of code that would fail if it stopped being true.

## A repeat measurement proves each repeat made its own call

What happened:
the editor fidelity bench walks the pool of trials in three passes so that the money buys more units of evidence.
The run client has a prompt reuse layer,
which answers a prompt it has seen from the reply it kept,
and the second and third passes asked the identical prompts.
So 62 of the 150 rows of the billed high arm were answered by the layer in 0 ms,
a repeat compared a reply with itself,
and the spend guard booked each of them at its ceiling because a reply with usage and no line on the meter looks like
an unmetered provider.
The guard then reported 23.94 USD of 24 spent where 0.62 had been,
and the arm ended early.
Nothing in the arm's checks noticed:
the rows were logged,
each once (ledger B899 of round 11).

The rule:
a repeat measurement proves that each repeat made its own call.
The repeat's identity includes what makes it a repeat (here the pass),
so that a layer which reuses by identity cannot reuse across it,
and the code that scores a repeat refuses a reply that no request produced and says so aloud.
A guard books what was sent,
so a reply with no attempt told to the guard books nothing.
Any count of trials,
units or rows that stands for evidence counts the ones that made a call.

What enforces it:
`probe/src/corpus-run/editor-fidelity-probe-run.unit.test.ts` runs two passes through the real reuse layer and its
payload store and sees the model asked in each,
and fails with a client that reuses across passes (`RepliedWithoutCallError`).
`probe/src/corpus-run/editor-fidelity-spend-guard.unit.test.ts` holds the guard that books nothing for such a reply.
`probe/src/corpus-run/editor-fidelity-arm-passes.unit.test.ts` holds the read step that fails an arm with such rows.
No scan finds a measurement that repeats a prompt over a layer that reuses by identity.

What the guard cannot tell:
it ends an arm on a reply with no attempt told to it only for a client built over its admit,
since only that client tells it of each attempt.
A client given as itself tells it nothing,
and a reply of the reuse layer then looks like a reply whose provider wrote no line on the meter,
which the guard counts at its ceiling as an unreported cost.
Every construction in production passes a builder,
so the case does not arise in a run,
and a case in `probe/src/corpus-run/editor-fidelity-spend-guard.unit.test.ts` pins the booked ceiling.
A new caller of `spendGuardedClient` passes a builder,
or the reuse check is off.

The scan that fails a test file importing a fixture writer:
a test that imports `infoLinesDuring` or `warnLinesDuring` from `typography/console-warn-lines.test-fixture.ts`
diverts a process global,
so the process globals scan (ledger B79) fails it unless every suite around its cases runs one case at a time.
Two rounds of this work broke the scan that way,
the second for five commits in a row.
Run `mise run //package/translation-repair/source-scan:source-scans` before the first commit of a test
that imports such a writer,
and set `concurrency: 1` on its suite when it does.

How to find the next one:
for each measurement that asks the same thing more than once,
list the layers between the asker and the wire and ask which of them can answer without sending.
A positive control comes first:
run two repeats through the real layers with a scripted transport and count the exchanges.

## A wait that buys the same refusal is not repeated

What happened:
XingZ6020 seated Synthetic while its meter read "unreadable,
treating as spendable".
Every call was refused,
each refusal set a 300 s cooldown,
and the seat reader waited the cooldown out for every slice
and asked again,
only to be refused at once:
a wait of 240 to 300 s in each of 58 distinct minutes across five and a half hours (ledger B965).
The wait was written for a provider that comes back,
and nothing in the loop asked whether this one had.
Each wait looked reasonable when it started,
and the bound the entry carried did not reach it,
because that bound filters one provider's return and a refusal cooldown is waited for whatever the bound is.

The rule:
a wait that is taken to see whether a provider comes back records what it bought.
A provider that refused right after a wait it was given,
with nothing served in between,
has answered the question,
and the next wait for it is not taken in that entry.
The record is the layer's own evidence (the hold ended,
the router reported nothing served,
the provider refused),
not a time window,
since a window must outlast the retry ladder of the layer below it.
A hold the refusal itself states,
a payment refusal and a NeuralWatt day start are not guesses and are never given up.
The record lives for the entry,
so the next entry meets the provider fresh.
A loop that waits and asks again is bounded by what its waits bought,
and by its entry's abort only as the last resort.

The rule for a give-up:
giving a provider up narrows where a call may go,
and waiting for it buys something only when its return would make the entry finishable.
The give-up is set aside,
and the cooldown waited out and the provider asked again,
in two cases only:
(a) the given-up provider is the only writing provider not dry,
since the other way ends the run for a provider that serves again after its cooldown;
or (b) with that provider back no bench floor the entry's stages need would be left unmet,
while without it at least one is
(the writing floors and the judging quorums,
read from the seat reader's own helpers over every bench any phase leans on).
Otherwise the give-up holds.
The wording of round 3 was superseded in round 4,
and [the entry B965 of the ledger][b965-entry] keeps the account of it.
The decision is taken for each provider given up,
and the floors are read twice,
with the providers left and with that provider added.
Where the give-up holds,
a call for a model only the given-up provider serves must end as that seat unreachable,
with no request sent,
and never as an error of the entry or the end of the run.

The rule for a return that served nothing
(the owner's answer of 2026-10-29,
"Remember futile returns"):
the give-up reads a refusal cooldown alone,
and a wait for any other hold repeated the same loop.
A provider whose hold ended and which refused again with nothing it answered in between
has refused its last return,
whatever kind of hold it was,
and a stage short of a bench does not wait for the next hold of that provider until it answers a call.
The mark is the budget layer's own evidence,
as the give-up's is,
and it yields to the same two exceptions through the same decision.
It takes nothing out of routing,
and a served call clears it.
The record is [ledger B1010][b1010-entry].

What enforces it:
`run/src/corpus-run/run-seats-cooldown-repeat.unit.test.ts` runs the real budgets and the real seat reader on one fake
clock with a provider that refuses at once after each cooldown,
and fails when the reader waits for more than the first beside a live provider,
or ends the entry when the refusing provider is alone.
Its pins fail for three mutants:
the served check removed,
the entry-start check removed,
and a stated return counted as a cooldown.
`provider/src/provider-budget-refusal-holds.unit.test.ts` holds the cases of the marks themselves,
`provider/src/provider-budget-give-up-aside.unit.test.ts` those of the set-aside,
and `provider/src/provider-budget-cooldown-set-aside.unit.test.ts` the budgets and the router's wait over it.
`run/src/corpus-run/run-client-create.unit.test.ts` builds the run client itself (`runClientFrom`) over a
scripted transport on a fake clock,
and fails when the entry waits for Synthetic more than once in the state of XingZ6020
(NeuralWatt,
DeepSeek and Xiaomi wet),
or beside a wet DeepSeek or NeuralWatt alone,
or waits fewer cooldowns than it is refused when Hyper refuses beside a wet NeuralWatt,
or ends the run for a Synthetic that is the only writing provider not dry beside a wet Xiaomi;
`run/src/corpus-run/run-seats-give-up-floors.unit.test.ts` holds the floors,
and `run/src/corpus-run/run-client-writing-roster.unit.test.ts` the roster the router is handed.
`stage/src/stage-call-given-up-provider.unit.test.ts` runs the real router over the real budgets and fails when the
seat only the given-up provider serves is anything but unreachable.
`provider/src/provider-budget-futile-return.unit.test.ts` and `provider-budget-futile-returns.unit.test.ts` hold the
cases of the mark of a return that served nothing,
and `run/src/corpus-run/run-seats-futile-return.unit.test.ts` runs the real budgets and the real reader on a fake clock
and fails when the reader waits for a provider after a return that served nothing,
or stops waiting for one that has served since.
No scan finds a loop that waits and asks again.

How to find the next one:
list every place the code waits for a provider,
a hold or a window to end and then asks the same party again,
and for each ask what the code knows after the second refusal that it did not know before the first wait.
A positive control comes first:
script the party to refuse at once after every wait and count the waits on a fake clock.

[b965-entry]:
  audit-ledger-providers-and-seats-a-run-holds-out.md#b965-the-seat-reader-waited-out-the-refusal-cooldown-of-a-provider-that-refused-at-once-after-it-for-every-slice
[b1010-entry]:
  audit-ledger-short-bench-floors-and-the-waits-that-cross-them.md#b1010-a-stage-waited-for-a-provider-whose-return-had-served-nothing
