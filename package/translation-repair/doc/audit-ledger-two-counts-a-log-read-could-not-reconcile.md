# Audit ledger: two counts a log read could not reconcile

Two differences that the check of the `XingZ6022` read left unconfirmed:
466 panel decision lines against 412 issues in the tally,
and 3,690 spend lines against 3,689 stored payloads.
Both are by design,
and each entry says where the design is written so that the next read does not raise it again.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B1063: 466 panel decision lines against 412 issues, the 54 being accepted duplicates merged at emission

Status:
recorded on 2026-10-29 (UTC),
measured from the log and the artifact of `XingZ6022` (build `c6185d4086`),
by design,
no code changed.

What the run showed:
the log holds 466 panel cluster lines
(`adjudicated/<id> <verdict> at <severity>; issue/<id>: supported ...`),
which carry 605 claims,
and the `TALLY` line (log line 29722) says `repairIssues=412 repairAccepted=304`.
By verdict the lines and the tally agree except in one place:
accepted 358 lines against 304,
needs-human 58 against 58,
rejected 48 against 48,
source-defect 2 against 2.
The 54 are all accepted.

Cause:
the tally counts issues after `dedupeAcceptedIssues`,
and the panel lines are the clusters before it.
The artifact holds 54 findings
`duplicate-issue-merged (adjudicated/<incoming> into adjudicated/<survivor>)`
in `lanes.repair.result.findings`,
and the same 54 spread over the chunks' own findings.
The 54 incoming ids are exactly the 54 panel cluster ids that no `chunk N: issue adjudicated/<id>` line carries.
Each is an accepted cluster of one claim in the same slice as its survivor,
which is also accepted and carries the chunk line,
and there are 24 slices and 38 distinct survivors.
The 412 chunk lines carry all 605 claim ids of the panel,
so nothing was dropped.
Slice 2 shows it:
cluster lines 1793 (`f70aa4`) and 1797 (`f8d16f`),
and the chunk line 1923 that holds both claims under the first.
Haiku's reason on 1799 says the second is "the same mistranslation as claim 2".

The design:
`dedupeAcceptedIssues` (`shared-contract/src/dedupe-issues.ts`),
applied in `repair/src/repair-chunk.ts` before the envelopes are cut,
merges accepted issues of one chunk that share a category and the start of their spans.
It merges and does not drop,
so the survivor carries every claim.
It is written in `doc/decision/translation-repair-duplicate-issue-emission.md`
(ratified on 2026-08-13:
"Deduplication happens at EMISSION, not at scoring").
`settled-tally.ts` counts `repair.result.issues`,
which is after the merge.

What the log cannot show:
the merge findings reach the artifact (`repairFindings=5808` in the tally includes them) and never the log,
so the log alone cannot reconcile 466 with 412.
A reader reconciles them from the artifact:
466 less the `duplicate-issue-merged` findings is the tally's 412.
Printing one line per merge was not done.

Method:
`panelcount.ts`,
`chunkcount.ts`,
`merged.ts` and `mergejoin.ts` in `/var/home/user/temp/agent/tr-fan/w379-accounts/`.
Every one of the 54 merges was checked against the log for four things:
an accepted cluster line of one claim,
an accepted survivor line,
the same slice,
and the chunk line carrying the survivor and not the incoming cluster.
All 54 passed.

## B1064: 3,690 spend lines against 3,689 stored payloads, the one being an unfinished attempt booked at its bound

Status:
recorded on 2026-10-29 (UTC),
measured from the log and the runs directory of `XingZ6022`,
not a defect,
no code changed.

What the run showed:
3,690 `SPEND` lines and 3,689 files in `prompt-payloads`.
`XingZ6021` has 2,505 and 2,505.

Cause:
exactly one `SPEND` line carries `estimated=abandoned-bound`,
line 21050:
Haiku 5.5,
prompt 60,666,
completion 32,000,
cost 0.02358325 USD.
It is the select stream that was ended at its content bound (32,010 characters against 32,000,
stream line 21049,
voice-lost warning 21051).
That call returned no reply,
and `promptUniqueClient` (`stage/src/prompt-uniqueness-client.ts`)
stores a payload only after the wrapped call returns one
and releases the claim when it throws,
so there is a booking and no file.
The booking is the most an unfinished attempt could have been billed,
which the Claude ledger writes so that it never under-reads
(`provider/src/spend-line.ts`,
`SPEND_RECKONINGS`,
ledger P1 and P14).
The other 3,689 lines carry no estimate mark and equal the file count.
That is a counting argument:
a `SPEND` line carries no digest,
so each was not matched to its file.

An observation of a different kind:
the NeuralWatt call ended at its content bound (stream line 14462,
warning 14463) left no `SPEND` line at all,
since `settleNeuralWattCall` writes the line after a whole stream
(1,295 NeuralWatt streams of `deepseek-v4.1-flash` against 1,294 lines),
and its energy is carried as an abandoned estimate in the ledger instead (`neuralwattReckonedWh` 0.324,
the stuck figure of the read's D6).
The two providers differ here.
This was not examined further.
