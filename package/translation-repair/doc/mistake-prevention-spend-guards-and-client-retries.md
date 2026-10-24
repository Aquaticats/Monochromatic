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
