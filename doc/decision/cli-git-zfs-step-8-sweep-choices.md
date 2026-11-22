# cli-git ZFS step 8: the sweep's open choices

## Status

Accepted 2026-11-22 in the question tool,
recorded 2026-11-22T01:55:12-05:00.
It settles choices that ZFS plan step 8's build made where the design and decisions were silent
(`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
"Step 8:
The attaching sweep for a clone that carries a mount request"),
as its integration pass put them
(`~/temp/agent/integration-pass-zfs-step-8-attaching-sweep-2026-11-22/keep/review.md`,
section "The twelve choices").

## What was decided

### The datasets the sweep acts on

The project first answered that everything must work by default with no configuration,
then chose "C:
setup writes,
handler falls back":
step 19's one-time setup,
which creates the checkout dataset `monochromatic-git-checkout` and its sibling container
`monochromatic-git-worktree-clone-datasets`
(the design's "Dataset and path layout"),
writes both names into the root-owned configuration file itself,
and when the file holds no names the handler finds the two datasets by those design names.
No person edits the file by hand.

### Receipts for refusals

"C:
receipt for unloaded key":
a refusal files no receipt,
except an unavailable key,
which files a `refused` receipt naming that step so the waiting command learns of it at once.

### A slot whose claim is held

"A:
holder re-reads after":
a sweep that finds the slot's claim held refuses,
and the holder reads the intent again after it releases the claim,
so a request that arrives during a sweep is not lost.

### The `first-attach` writer

"A:
build it in step 8 now":
step 8 builds the `first-attach` writer and the call the start-probe design gives it,
in the claim order that design states.

## Adopted with the project's veto open

These were not asked;
the integration pass ranked one option first for each and the coordinator adopted it:

- the 4096-child bound is measured in the guest and its numbers are written beside the constant;
- the claim's window between its directory proof and its path-based calls stays documented and open;
- readiness is published `running` before the sweep and `finished` or `failed` after it,
  and the plan's paragraph is reworded to say so.
