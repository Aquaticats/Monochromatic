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

### With no configuration file

Recorded 2026-11-22T14:27:54-05:00,
after step 8 merged as `00d857a435` and its recheck ran the built sweep with no file
(`~/temp/agent/integration-recheck-zfs-step-8-attaching-sweep-2026-11-22/keep/review.md`,
"Owner item 1"):
with no file,
with an empty object,
or with only a `startProbe` object,
the sweep exited 2 and attached nothing,
because `cloneMounts.checkout` was required,
and the start probe needed its own `startProbe.dataset` and `startProbe.checkout`.

The project answered "1:
no file needed (Recommended)".
With no file,
the root handler finds the checkout as the one mounted ZFS dataset whose last name component is
`monochromatic-git-checkout`,
takes the container beside it,
and the start probe finds its dataset by its design name too.
Only a dataset that is mounted counts.
When the name matches none or several,
the handler refuses and lists them.
The setup still writes the file,
and a file that is present wins.

Not taken:
"2:
setup writes whole file",
which pins every entry in the root-owned file and stops the route when the file is missing
(the recheck ranked it first,
because under the chosen answer a root program picks its target by name);
and "3:
keep as built".

The start probe's side of this answer is recorded in `cli-git-zfs-start-probe.md`,
section "Answer of 2026-11-22".

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

## The bound of 4,096 children stays

Accepted 2026-11-22 in the question tool,
recorded 2026-11-22T20:21:19-05:00,
on the integration pass of the step 8 follow-up
(`~/temp/agent/integration-pass-zfs-step-8-recheck-findings-and-sweep-cost-2026-11-22/keep/review.md`,
section "Owner item 2: the bound of 4,096 children and the figures beside it").

Asked with that pass's figures:
the only count on record is the owner's "Up to 64" worktrees at once;
with 1,024 mounted clones one steady sweep took about 3.4 s (2.8 to 4.5 s) in the pass's guest;
the fix round's single reading at 4,096 was 20 to 22 s,
two thirds of the provisional 30 s a waiting command is to allow,
and no command waits on a receipt yet.

The project chose "A:
keep 4,096 (Recommended)":
the bound does not change,
and the comment beside `MAXIMUM_CHILDREN` is corrected to cite only measurements whose raw lines are kept.

Not chosen:
lowering the bound to 1,024,
and keeping the count while adding a time limit per sweep.

## Adopted with the project's veto open

These were not asked;
the integration pass ranked one option first for each and the coordinator adopted it:

- the 4096-child bound is measured in the guest and its numbers are written beside the constant
  (the bound itself has since been decided:
  section "The bound of 4,096 children stays");
- the claim's window between its directory proof and its path-based calls stays documented and open;
- readiness is published `running` before the sweep and `finished` or `failed` after it,
  and the plan's paragraph is reworded to say so.
