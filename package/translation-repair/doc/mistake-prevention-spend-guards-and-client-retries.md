# Preventing mistakes: spend guards and the attempts a client makes

Rules for a guard that holds a run to an amount of money:
what it admits,
what it counts,
and the retries a client makes inside the call it admits.

Part of [Preventing this package's mistakes](mistake-prevention.md),
which lists every file of this record.
It is a new file of 2026-10-24,
because [Preventing mistakes: guards, searches and claims](mistake-prevention-guards-searches-and-claims.md)
stood four lines short of the 1,000-line limit.

## A guard admits a call where the client sends several attempts

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
`run/src/corpus-run/claude-attempt-admit.unit.test.ts` fails when the route the probe builds does not hand the admit down,
and `probe/src/corpus-run/editor-fidelity-spend-guard.unit.test.ts` drives the whole route with a transport that answers
a retryable status four times at a spent sum where a guard of calls admits the call and the fifth attempt would cross the
cap.
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
