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

From lines 43424 to 43433 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  exists on the attach path only), lows L-1 to L-5, and two owner items. OWNER ITEM 1, ASKED 2026-11-22: with no
  configuration file, with `{}`, or with only a `startProbe` object, the sweep exits 2 and attaches nothing;
  `cloneMounts.checkout` is required, the start probe needs `startProbe.dataset` and `startProbe.checkout`, and the
  accepted start-probe design has `doctor` print the probe's field for the owner to apply, against the owner's "Everything
  must work by default w/o any configuration". OWNER ITEM 2, the sweep's cost with every clone mounted: 64 children about
  0.3 s steady and 1 s cold (one child about 0.15 s); 1,024 about 8.3 s and 25.5 s; 2,048 about 26 s and 66 s. A
  `git worktree add` waits on a sweep's receipt, so the growth is treated as a defect on a waiting path and fixed without a
  question; the bound's measured figures go beside the constant, veto open. GOING:
  `pi-zfs-step-8-recheck-findings-and-sweep-cost` on `fix/git-policy-cli-zfs-step-8-recheck-findings-and-sweep-cost` (M-1 to
  M-4, the lows, the sweep's cost; brief under `~/temp/agent/zfs-step-8-recheck-findings-and-sweep-cost-2026-11-22/`; the

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
the section whose heading starts "Owner item 2").

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

## Four further answers on the look-up with no configuration file

Accepted 2026-11-22 in the question tool,
recorded 2026-11-22T22:22:43-05:00,
on the integration pass of the branch that builds the section "With no configuration file"
(`~/temp/agent/integration-pass-zfs-sweep-and-start-probe-need-no-configuration-file-2026-11-22/keep/review.md`,
"Owner's to decide",
decisions D1 to D4).
They bind the sweep and the start probe alike
(`doc/decision/cli-git-zfs-start-probe.md` holds the probe's own answers).
Each answer is given as the option's label and its own description,
as the project was shown them.

### A dataset mounted in the clone container is never the checkout

Asked:
with no file,
the sweep and the probe find the checkout as the one mounted dataset whose last name is `monochromatic-git-checkout`,
and none or several refuse.
The pass showed in a guest that an ordinary account,
with only the grants the route gives it,
can make a clone with that last name inside the clone container and have the route mount it,
which makes "several" and stops the route for every user until the clone is destroyed.
The branch added a rule this record did not hold,
that a dataset mounted directly inside the clone container is never the checkout,
and knew the container by its designed name only.

The project chose "A:
the container this run uses (Recommended)":
"Ignore a dataset mounted directly inside whichever clone container this run uses:
the designed one,
or the one root's file names."
The costs stated with it:
the probe reads one more key of the file than it does today,
and a little more code.

Not chosen:
the designed name only,
as built;
any clone anywhere,
which makes the look-up open the ZFS device;
and no exception.

### The probe lists what it found

Asked:
when the sweep finds none or several it refuses and prints every row it found,
while the probe prints only that it did not run and writes one word in its record.

The project chose "A:
list the rows,
like the sweep (Recommended)":
"When the probe meets none or several,
it prints each mounted dataset it found with its path."
The cost stated with it:
one more read of the mount table in that case.

Not chosen:
the word only;
and the rows as extra lines in the record.

### Snapshot directories are hidden on the checkout

Asked:
the look-up refuses when the machine's mount table is larger than 4 MiB.
Listing a snapshot's directory makes the kernel mount that snapshot,
and each such mount adds a row,
so an account could grow the table until the look-up refuses for everyone
(the pass computed about 25,500 such mounts and did not try it).
`main` has the same bound.

The project chose "C:
hide snapshot directories",
not the option recommended:
"Turn off snapshot-directory visibility on the checkout so accounts cannot trigger those mounts."
The cost stated with it:
browsing snapshots under `.zfs/snapshot` in the checkout is lost.

Not chosen:
leaving the bound and documenting it,
which was recommended;
and reading the table without a bound for the look-up.

What the question did not establish,
and the build must before it relies on the answer
(the coordinator's note,
not the project's words):
which setting of the installed OpenZFS release stops those mounts,
since a directory that is merely not listed can still be entered by name;
who applies it and when;
and whether any part of the route reads a snapshot through that directory.

#### What hiding comes to, answered 2026-11-23

Asked in the question tool on 2026-11-23,
after the branch's second integration pass measured the setting on OpenZFS 2.4.4
(`~/temp/agent/integration-pass-zfs-sweep-and-start-probe-need-no-configuration-file-2026-11-22/keep/review-second-pass.md`).
What the project was shown:
only `snapdir=disabled` stops those mounts;
`hidden`,
the default,
still lets anyone enter a snapshot by name.
`disabled` exists from OpenZFS 2.3.0.
Accounts cannot undo it.
It does not stop an account that had already listed the directory before it was set.
Browsing `.zfs/snapshot` and `zfs diff` on that dataset are lost.
Each clone keeps its own snapshot directory unless the clone container is set too,
and clones get snapshots during removal.
As built,
nothing sets or checks it
(plan steps 19 and 18 are unbuilt).

The project chose "A:
checkout + clone container (Recommended)":
"Step 19's setup sets `snapdir=disabled` on the checkout and on the clone container (clones inherit it);
step 18's `doctor` reports either one where it isn't disabled (the route can read the setting);
setup says so plainly where the ZFS module is older than 2.3.0 and refuses the value.
Pros:
closes the growth through both.
Cons:
no `.zfs/snapshot` browsing and no `zfs diff` on the checkout or the clones."

Not chosen:
"B:
checkout only";
and "C:
back to bound + document",
which would have undone the answer of 2026-11-22.

### An empty configuration file is refused

Asked:
a file that exists and is empty is refused as "not one JSONC document",
where a missing file is fine.

The project chose "A:
keep refusing,
say why (Recommended)":
"An empty file is refused,
and the message says an empty file is not the same as no file."
The cost stated with it:
an accidentally emptied file stops the route until fixed or deleted.

Not chosen:
treating an empty file as no file.

## A standing mount's source is read by its mount number

Answered in the question tool on 2026-11-23,
recorded 2026-11-23T18:03:17-05:00,
on the second Sonnet pass of the step 8 follow-up
(`fix/git-policy-cli-zfs-step-8-recheck-findings-and-sweep-cost` at `c87a50a09b`;
`~/temp/agent/integration-second-pass-zfs-step-8-recheck-findings-and-sweep-cost-2026-11-22/keep/review.md`,
"Owner item 1",
finding P2-H1).

Asked:
to get faster,
the branch keeps a copy of the kernel's mount list and re-reads it only when the kernel signals a change.
One change sends no signal:
renaming a mounted `legacy` dataset changes the dataset name the mount shows.
With 1,024 clones mounted,
root renames clone `c108` to `c108-moved`,
makes a new clone and renames it to `c108`;
`main` refuses,
and the branch prints `attached .../c108` and files a `done` receipt for a clone that is not mounted there.
Steady sweeps on the branch are 0.20 to 0.79 of `main`'s time from 8 to 1,024 clones.

The project chose "C:
statmount(2) per mount":
"Ask the kernel for that one mount's source by its unique mount number (Linux 6.8 and later;
host is 7.3.5),
falling back to B on older kernels.
Pros:
exact,
constant cost per clone,
keeps the whole gain."
B,
the fallback,
is "Whenever the copy says a mount is this clone's own,
check it on the live mount table,
the call `main` makes,
and keep the copy for everything else."

The question ranked B first,
on the cost of building C,
and the project answered:
"C.
And this question was framed wrong.
Just because something is difficult doesn't mean it's less desirable."

Not chosen:
"B:
confirm 'own' live",
except as C's fallback;
"D:
trust the clone's receipt";
and "A:
state the limit".

## Adopted with the project's veto open

These were not asked;
the integration pass ranked one option first for each and the coordinator adopted it:

- the 4096-child bound is measured in the guest and its numbers are written beside the constant
  (the bound itself has since been decided:
  section "The bound of 4,096 children stays");
- the claim's window between its directory proof and its path-based calls stays documented and open;
- readiness is published `running` before the sweep and `finished` or `failed` after it,
  and the plan's paragraph is reworded to say so.
