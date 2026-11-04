# Commit protocol models and components: open work

State of branch `feat/git-policy-cli-commit-protocol-models` after round 12 (the P1 findings of the three round-11 reviews).
Nothing here is wired into the live wrapper; the reviewers' line stands that non-reclaiming component and model work can continue and the
checkpoint reclamation and quarantine APIs are not yet a safe foundation for live integration.

## Closed in round 12 (P1)

- Components review 8, 2.1 to 2.3: a checkpoint is a replacement-root authority with a closure of content-identified entries, validated by reading
  and decoding; recovery's selection validates the closure before a candidate is a root; completion forces each entry with its name chain through
  the lane directory and removes only the segments the authority names (`commit/commit_state_log.rs`).
- Components review 8, 2.4: `Departure::new(Disposition, ...)` refuses a disposition whose promise has no file, forces the stop first, and returns an
  acknowledgment of the state prefix it forced (`commit/commit_inbox.rs`).
- Components review 8, 2.5: repair is owner-only (`InboxRepairer`), moves only the object it inspected, keeps unsupported versions, and forces the archive.
- Models review 6a, 1, 3, 4 and models review 6b (P1): the register no longer lets an ordinary wait end stand for the final save, a failed mark is an
  outside-fault exit, a planned exit is a drain, and the discharge model keeps every live request in a checkpoint (`model/`).

## Queued for round 13 (P2 and P3)

Components review 8 (`commit-components-sol-8.txt`):

- 3.1 P2: conflict detection depends on scan order.
- 3.2 P2: `StatusUnknown` does not cover initial-read ambiguity.
- 3.3 P2: quarantine accepts paths outside the inbox (the identity check is in; the review's remaining cases are not).
- 3.4 P2: the departure anchor does not cover inbox or state ancestry.
- 3.5 P2: selector and reclaimer, and prefix-lifetime, races.
- 3.6 P2 and P3: remaining namespace and cost problems.
- 3.7: departure ordering, decision 14 and foreground forcing.
- Section 4: the missing behavioural reds, and the working-directory test that does not reliably test the fixed bug.

Models review 6a (`commit-models-sol-6a.txt`):

- 2 P2: `BARRIER_ENTRIES` exemption's execution assumptions are too broad.
- 5 P2: timeout execution, pipe submission and saved closure remain separate conditional lemmas.
- 6 P2: the acceptance manifest checks a configuration prerequisite, not acceptance.
- Vacuity audit P2: the no-timeout test is narrower than its description.
- P3: explorer and reporting defects.
- Section 4 P2: required fidelity and regression controls; the capture variant trap.

Models review 6b (`commit-models-sol-6b.txt`):

- P2: recovery still assumes the validation work at issue.
- P2: the completion-guard replay does not isolate the guard.
- P2: the watch excludes unauthorized-overwrite schedules.
- P2: the reminder invariant is partly implementation-maintained.
- P2: the bound model's departure barrier is asserted.
- P2: submission Fifteen is a prohibition lemma, not an E1 implementation.
- Section 3 unchanged P2 gates.
- P3: reporting needs narrower claims.

## Revision 17's eighteen new controls

Covered: 13 (closure entry with a torn envelope, an unsynced name chain, a background-synced candidate), 14 (a departure without its stop,
an acknowledgment that does not cover the naming record), 15 in part (owner-only repair, unsupported version, an identity such as `../notes`,
the archive forced; not the three-file conflict example), 17 (ordinary completion versus final save, a failed mark is not a normal return,
a planned exit is not a parent-death termination), 18 in part (a checkpoint with two live requests keeps both; not the invalid suffix that
claims disposal).

Not covered: 1 to 12 and 16.
