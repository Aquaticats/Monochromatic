# Preventing mistakes: test cases that hold a seam or a count

Part of [Preventing this package's mistakes](mistake-prevention.md),
which lists every file of this record.
Two lessons of round w407 about what a case must be able to see:
a seam that one line of the production wiring provides,
and an absence that holds both when the thing was refused and when it was never asked.

## A seam is held by a case at the level that wires it

What happened:
`readDry: readDryness` in `run-config.ts` hands the run's reading of the provider meters to the re-ask,
and removing it silently returned the re-ask to the rule that ignores the reading,
which leaves Qwen3.8-27B out of every editor and refiner re-ask.
The cases of the provider and the stage packages passed the reading by hand to the function that needs it,
so none of them could fail when the run stopped passing it (ledger B1345).

The rule:
a value that one line of the production wiring hands to the code that uses it has a case at the level of that wiring:
the real client of the package that builds it,
with a fake transport,
asserting what the wiring changes.
The case derives its expectation from the two rules the seam chooses between,
and asserts that they differ,
so a change that made the rules agree fails the case and does not hide the seam.
The case is shown red by removing the line,
rebuilding the package that holds it and running the case,
and the log is kept.

What enforces it:
`run-config.unit.test.ts` ("HANDS THE RUN'S READING OF THE METERS TO THE RE-ASK OF A RUNAWAY EDITOR")
and the red log of ledger B1345.
The same shape holds for any other single-line seam of the wiring;
habit finds them.

## An absence is asserted beside the count of what was asked

What happened:
a case returned whether the re-ask of a runaway went to Qwen3.8-27B and expected false under the reading that has Hyper
serving it.
False is also the answer when no re-ask was made at all,
so a client that gave up passed (ledger B1347).

The rule:
a case that asserts a model was not asked asserts how many asks were made and which model the one it expects went to,
so the absence cannot be an ask that never happened;
a re-ask of the same model elsewhere is a second ask too,
and the case excludes it by name.
The old assertion is run against a copy that makes the ask impossible,
to show it passes there,
and the new one is run against the same copy,
to show it fails.

What enforces it:
`prompt-uniqueness-client.unit.test.ts` ("RE-ASKS A RUNAWAY EDITOR OF QWEN3.8-27B ONLY WHERE THE DRYNESS READ HANDED IN
HAS SYNTHETIC SERVING IT") and the two logs of ledger B1347.
