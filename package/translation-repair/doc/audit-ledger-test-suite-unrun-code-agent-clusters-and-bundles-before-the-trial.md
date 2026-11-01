# Audit ledger: code no unit test runs, agent clusters and bundles before the trial

The clusters of the census cut and given to agents reading one commit,
what review changed in each patch,
and the nineteen bundles whose agents stopped at a usage limit,
up to the hand-over to the trial model.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## Six clusters given to agents reading one commit

Four clusters of `census-3G9C58` whose sources no commit since `d0cca211d` touched went to four agents at once,
each reading `7f529468e`:
`corpus-run/archive`,
`stream`,
`critic`
and `absolute`,
47 stretches over 16 files.
The whitespace normalization in `7f529468e` (B119) moved no line,
so the census's line numbers hold.

The four landed as `23fc6144b` (`critic`),
`265249d09` (`absolute`),
`37bab0e34` (`stream`)
and `9d6cde01a` (`corpus-run/archive`),
and two more clusters run the same way landed as `a87311e65` (`corpus-run/runs`)
and `de3a2f7b3` (`reference`).
Each agent's files were checked unchanged between the commit it read and the merge's parent,
and each patch was reviewed before it was applied.

## What review changed in each agent's patch

The review changed every patch.
In `critic`,
the agent made two comparators answer 1 for equal keys,
which breaks the comparator contract;
all four comparators and `heardCriticIds` now order through `compareCodePoints` (B95).
In `absolute`,
the refusal cases lacked a case accepting their shared valid reply,
without which each refusal could pass on a base that was invalid all along.
In `stream`,
a case pinned `readPayload` passing a frame that parses to an array or a number on as a frame naming nothing;
it now counts such a frame as unreadable,
as `anthropic-delta-scan.ts` does.
The same patch copied the Anthropic frame builders into `stream-drain.unit.test.ts`;
they moved into `anthropic-frames.test-fixture.ts`,
and the constant thinking-runaway callbacks the duplicate-body scan grouped became `String.prototype.repeat` over one frame.
In `reference`,
`mergedAttestations` filtered out an item it could not place,
while its own comment said verification guarantees placement.
The comment held:
the stage runs `quoteIsIn` on every archive quote against the same archive text under the same fold,
so the filter could only drop an unverified item unseen,
and it now throws an `unreachable:` error instead.
In `corpus-run/archive`,
one case pinned `phraseOccurrences` reading past a row whose lower case changes length,
the defect B121 then queued;
the case was held back and the defect fixed.
In `corpus-run/runs`,
a rethrow lacked its `cause`.
The unrecorded-identity arms there stay uncased,
since they need this host's own `/proc` reads to fail,
and so does the race-loss arm with an unreadable winner,
which needs the loser to read between the winner's create and write.
That arm may run in some runs of the two-takeovers case and not others
(inference from the code,
not measured).
The one unrecorded-identity arm `census-2W1rHH` still listed,
`corpus-run/runs-lock.ts` 351 to 352,
is cased since `5ba14c371fdc57996fb2aba3fdffb32758f62b33`,
through a stub refusing every read under `/proc/`
(the passage "The runner and git cluster of the final census").

## Nineteen bundles, and the agents that stopped at a usage limit

With the twenty-fourth batch's `page`,
those six closed 81 of the census's 590 stretches.
The other 509,
with 14 of the 15 functions never called,
went into 19 file-disjoint bundles of about 30 stretches each
(`make-bundles.mjs` in the session's scratch);
none of their files changed beyond whitespace since `d0cca211d`
(`git diff --ignore-all-space --ignore-blank-lines --name-only`,
which lists the changed `stream-delta-scan.ts` as a control).
The fifteenth,
`toKind` in `critic-wire.ts`,
sits in no bundle,
since `critic` closed before the bundles were cut.
The first five bundles went to five agents reading `9d6cde01a`,
each told to case only in test files of its own modules,
so no two agents write one file.

All five agents stopped part way at a usage limit.
Bundle 1 (31 lines:
29 stretches and 2 functions never called,
in 12 source files)
landed as `23090a91d`,
and bundle 2 (29 stretches over 12 files) as `ba6da4640`,
each after its agent's partial copies were reviewed against the source
and the rest was done by hand;
the two commit messages list what was taken,
what was rejected and why.
Reviewing them found B123 (open),
B124 and B125 (both fixed).
Of bundles 3 to 5,
only two test files reached the branch,
committed by the owner as they stood in `98e94e584` on 2026-10-03:
four cases in `contributor-name-authority.unit.test.ts`
and one in `corpus-run/canadian-spelling.unit.test.ts`,
the second written by the bundle 4 agent
(per the messages of `ca242a2c4` and `14e4e2cea`).
One of the contributor cases fails on that commit:
an empty-label link,
`[](https://example.test/whisker)`,
reads as the contributor name `[](https://example.test/whisker)`
where the case expects no name
(measured 2026-10-03,
the case's own FAIL line);
the fix is the trial model's first step under its handover
(`doc/handover/translation-repair-mimo-trial-2026-10-03.md` at the repository root).
The cut of the 19 bundles (`make-bundles.mjs`) and the agents' other partial copies lived in session scratch,
not in the repository,
so the batches after this one read against a fresh whole-suite census rather than the cut.
