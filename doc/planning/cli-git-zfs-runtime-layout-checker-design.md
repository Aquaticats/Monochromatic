# cli-git ZFS runtime layout checker design

Note of 2026-11-20:
the project accepted this design as written on 2026-11-20 in the question tool,
under "Accept as written (Recommended)",
with none of the four veto-open choices vetoed.
The decision record is
[`doc/decision/cli-git-zfs-runtime-layout-checker.md`](../decision/cli-git-zfs-runtime-layout-checker.md),
which also states the corrections the last gate check's lows asked for.
This body stands as accepted,
at `22703f23df`,
and the banner below records how the file was written before that acceptance;
the acceptance supersedes it as this file's standing.

## A non-final, unapproved proposal by an agent

This file designs one open item of the accepted ZFS worktree design,
"The runtime layout checker" in
[Open items the decisions leave][design-open-items],
which blocks step 6 of [the implementation plan][plan-file],
"The start-time compatibility gate and the start probe".
It was written on 2026-11-19T18:10:30-05:00 by a Pi coding agent session
on branch `docs/planning-cli-git-zfs-runtime-layout-checker-design`,
made from `main` at `73eaad0846`, and revised as
[Revisions](#revisions) records.
It read the sources named under [Sources read](#sources-read),
ran the read-only host probes named under
[What was run, and what was not](#what-was-run-and-what-was-not),
and changed no pool, dataset, mount or unit, and wrote no file in the repository except this one.
It ran no guest, so every behaviour outside the probes is a source reading,
and the measurements that would make the choices safe are
under [Owed measurements](#owed-measurements), unrun.
It decides nothing that belongs to the owner: the choices that are the owner's are
under [Questions for the owner](#questions-for-the-owner),
the parts a writer can settle are
under [What a writer can settle without asking](#what-a-writer-can-settle-without-asking),
and each rule marked there as open to veto,
with "veto none" as its default.
Nothing here is implemented, released or approved.

The owner answered three of this file's questions on 2026-11-19 and left the fourth held.
[Answers of 2026-11-19](#answers-of-2026-11-19) records each answer in its words,
and [the amendment these answers owe](#the-amendment-these-answers-owe-the-proposal-the-plan-and-the-binding-readme)
gives the proposal, the plan and the binding `README.md` the exact text each one needs.

## Answers of 2026-11-19

The project put this file's questions in the question tool between 2026-11-19T20:35
and 2026-11-19T20:43:51-05:00, and answered a follow-up on Q2's evidence at
2026-11-19T21:31:47-05:00.
The three first-round answers are recorded in the main checkout's
`doc/handover/cli-git-rust-implementation.md`, at lines 40398 onward of the file at `e4003a8563`,
inside one long paragraph that contains the sentence "THE ROUND WAS ASKED" rather than beginning
with it; the follow-up answer to Q2 landed after that commit, at line 40463 of the file at
`cb84b5769f`, inside the long list item that begins at line 39945 and contains "LAYOUT CHECKER
REVIEW ENDED" at lines 40451 and 40452 rather than beginning with it.
That handover is a working file (rule `H0P`), so this section is the durable copy of the answers
this design rests on.
Each answer is quoted in the project's words under its own heading, and every other section of
this design follows them: none asks a question answered here, and none of the answers leaves an
option open except Q1's own pick.

### The reader, Q1 in its narrower form

The first version asked Q1 more widely.
The owner was asked the narrower form, "is it a hard requirement that the parser is our own code
and the product starts no second program", and answered "No, not required (Recommended)".
So no hard constraint admits the in-house reader, the choosing-technology skill's existing-tools
rule keeps it ineligible, and the eligible order is the three candidates that pass every gate.
The pick is still open.
Finalist validation now lands under [Finalist validation](#finalist-validation),
and the pick waits for the `doc/audit/` vet report,
a discovery result that is no longer blocked,
and the pre-discovery check-in,
which [Q1](#q1-which-reader-parses-the-module-btf) lists as the items that condition its answer.
That section now records this answer and keeps its pick conditional on them.

### The release string, Q2

The answer: "Any version, layout decides (Recommended)".
So the gate compares no release string at all.
The layout comparison and the request numbers decide, and `doctor` prints the loaded module's
release string and says when it is not the tested `2.4.4`.

The question took three exchanges before it settled, and the record keeps the first two; the third is
[the evidence correction](#the-evidence-correction-q2-asked-again).

- the first answer asked back "Why not allow any?";
- told that no layout comparison can show a behaviour change, the owner asked
  "Does 'change request behavior w/o changing layout' really happen in practice?";
- the coordinator then compared OpenZFS `zfs-2.2.2`, `zfs-2.3.0` and `zfs-2.4.4` for the twelve
  requests the binding uses, the handlers and key tables of `module/zfs/zfs_ioctl.c`, fetching
  them under `/home/user/temp/agent/openzfs-ioctl-behaviour-across-releases-2026-11-19/`
  with the script `diff-handlers.ts` and keeping the `diff-*.diff` files there;
- with that evidence the owner answered "Any version, layout decides (Recommended)".

The comparison's artifacts are the script `diff-handlers.ts`, the three `zfs_ioctl-*.c`,
`zfs_h-*.h` and `zfs_ioctl_h-*.h` copies, and two diff files:
`diff-snapshot_list_next-handler-zfs-2.2.2-to-zfs-2.3.0.diff` and
`diff-clone-handler-zfs-2.3.0-to-zfs-2.4.4.diff`.

What that comparison read, and what it did not. It read the twelve handler bodies of
`module/zfs/zfs_ioctl.c` and their key tables. Five of the twelve requests
carry a key table and seven carry none in all three tags, so the run covers twelve handlers and
five key tables, not twelve of each. It read no request number, no registration line, no permission
policy and no function a handler or a policy calls, and those set a request's answer as much as its
own body does. This revision re-read the three copies, read the request numbers from the three
`zfs.h` copies and widened the handler read to registrations, policies and same-file callees, which
is what [What was run, and what was not](#what-was-run-and-what-was-not) records.

The request numbers. All twelve the binding sends hold the same value in the three tags, as do
`ZFS_IOC_FIRST` (23,040), `ZFS_IOC_PLATFORM` (23,168) and `ZFS_IOC_LAST` (23,177).
`zfs-2.2.2` has 102 enumerators where `zfs-2.3.0` and `zfs-2.4.4` have 104: the two added are
`ZFS_IOC_POOL_PREFETCH` (23,128) and `ZFS_IOC_DDT_PRUNE` (23,129), and they arrive at 2.3.0 inside
the core block's reserved numbers (`include/sys/fs/zfs.h:1537-1538` at 2.3.0). The binding sends
neither.

The handlers and key tables. No key table changed, and two handlers changed by one line each:
`zfs_ioc_snapshot_list_next`, where 2.3.0 turns
`issig(JUSTLOOKING) && issig(FORREAL)` into `issig()` between 2.2.2 and 2.3.0,
and `zfs_ioc_clone`, where 2.4.4 calls the relocated `dsl_dataset_clone` in place of
`dmu_objset_clone`, in the same synchronous task shape, between 2.3.0 and 2.4.4.
The clone's callee differs beyond its name: `dsl_dataset_clone_check` passes one argument fewer to
`dsl_fs_ss_limit_check` than `dmu_objset_clone_check` did, the `proc` argument gone, and
`dsl_dataset_clone` calls `zvol_create_minors` where `dmu_objset_clone` called `zvol_create_minor`.

The registrations, policies and callees. Of the 51 functions of `zfs_ioctl.c` reachable by name
from the twelve handlers and from the nine policy functions their registrations name, 10 have
different bodies in the three tags: the two handlers, and eight of the 28 functions `SET_PROP`
and its policy reach, whose `zfs_ioc_set_prop` text is identical in all three. That path has two
entry points. `SET_PROP` reaches the eight in three ways: five through `zfs_set_prop_nvlist`, one,
`props_skip`, through its own handler, and two, `zfs_secpolicy_recv` and `zfs_secpolicy_setprops`,
through the policy its 2.4.4 registration names. The third gate re-check's
`trace-call-paths-output.txt`, in its `keep/evidence/`, walks from each handler and reaches
`zfs_prop_set_special` through `zfs_set_prop_nvlist` from `zfs_ioc_set_prop` and from
`zfs_ioc_clone` alike. `zfs_ioc_clone` applies a clone's `props` through that function
(`module/zfs/zfs_ioctl.c:3634-3639`, 2.3.0), so a clone created with properties reaches those five
and none of the other three: with `SET_PROP`'s own roots cut, 7 of the 46 functions the other
eleven handlers reach still differ, five of the seven on the property-set path
(`compare-handlers-registrations-and-reachable-callees-output.txt`, in the same folder).
Of the eight, six are inert for the wrapper's call shapes: a `VERIFY` spelling in `props_skip`,
`zfs_prop_set_userquota` and `zfs_check_settable`, an empty
`switch` case `zfs_check_settable` loses, a permission path `zfs_secpolicy_recv` takes only when
`zc_cookie` is set, and the pool-config locking that moves inside `zfsvfs_hold` and `zfsvfs_rele`
at 2.4.4. The other two are the changes worth naming: `zfs_prop_set_special` for the interval that
kept the layout, and the new `zfs_secpolicy_setprops` for the interval that changed it alongside the
registration's move.
That count covers callees in `zfs_ioctl.c`; the callees that live in other files, the `dsl_*`,
`dmu_*`, `zvol_*` and `spa_*` functions, were read only for `zvol_set_ro`,
`zvol_set_volthreading`, `zvol_set_common` and the clone pair `dmu_objset_clone` and
`dsl_dataset_clone`.

A behaviour change that kept the layout, 2.2.2 to 2.3.0. That interval keeps the layout: the
declarations of `zfs_cmd_t`, `zinject_record_t` and `dmu_objset_stats_t` are identical across it,
and the header diff carries comments, defines and two new `zinject_type_t` values and changes no
struct member. Across it `zfs_prop_set_special` changed: it gains `ZFS_PROP_VOLTHREADING`,
`ZFS_PROP_READONLY` through `zvol_set_ro` and `ZFS_PROP_LONGNAME`, and it sends `SNAPDEV` and
`VOLMODE` through `zvol_set_common` (`module/zfs/zfs_ioctl.c:2556-2634`, 2.3.0, the `LONGNAME`
case ending at 2634).
`SNAPDEV` and `VOLMODE` are properties of a filesystem dataset as well as a volume
(`module/zcommon/zfs_prop.c:441-443` and `:486-489`, 2.3.0), and the routing those two names take
changed at 2.3.0: `zfs_prop_set_special` sends both through `zvol_set_common`
(`module/zfs/zfs_ioctl.c:2565-2568`), where 2.2.2 called `zvol_set_snapdev` and `zvol_set_volmode`
(`:2526-2531`).
2.2.2 has no `zvol_set_common`: its `module/zfs/zvol.c` holds `zvol_set_snapdev` at `:1621` and
`zvol_set_volmode` at `:1708`, and 2.3.0's `zvol_set_common` is at `:1963`.
So what that routing does when it meets a filesystem dataset is unread, because the body was
compared with neither 2.2.2 function's.
Taken together, a volume's property set answers differently at 2.3.0 with the layout kept, and no
comparison of layout can see it.
For the wrapper's call shapes it is no change, and that is a source reading, not a measurement.
The wrapper sends two user properties: the mount desired-state property
`org.monochromatic:git-worktree-clone-mount-desired-state`, named as `SET_PROP`'s own change under
[Requests and input ownership](cli-git-zfs-worktree-design-proposal.md#requests-and-input-ownership),
line 175, and quoted under
[Requests, results and ordering](cli-git-zfs-worktree-design-proposal.md#requests-results-and-ordering),
line 537, and the sweep nonce `org.monochromatic:git-worktree-clone-mount-sweep-nonce` (line 557).
The proposal sends the desired-state property in the clone's `props` only if the delegated creation
path is verified, and otherwise sets it with `SET_PROP` (lines 371 to 377), so both property paths
can carry it, and `zfs_ioc_clone` reaches the same `zfs_set_prop_nvlist` call for it.
In all three releases a user property returns at `zfs_prop_set_special`'s user-property check,
before its `switch`, at the same four lines (`module/zfs/zfs_ioctl.c:2456-2460` at 2.2.2,
`:2486-2490` at 2.3.0, `:2636-2640` at 2.4.4), and the value is set afterwards through the generic
property path; it reaches no release-specific case. On a filesystem dataset name `zvol_set_ro`
returns -1, because no volume carries that name (`module/zfs/zvol.c:386-401`, 2.3.0).
No caller names any of those cases, `LONGNAME` included: 2.2.2 has no
`ZFS_PROP_LONGNAME`, so `zprop_name_to_prop` returns `ZPROP_INVAL`, which is `ZPROP_USERPROP`,
`zfs_prop_user` needs a colon, and `zfs_set_prop_nvlist` refuses the name as an invalid property
with `EINVAL` (`module/zfs/zfs_ioctl.c:2627-2637`, 2.2.2); 2.3.0 holds the dataset and answers
`ENOTSUP` when the pool lacks `SPA_FEATURE_LONGNAME`, or sets the value when it has it.
That is a refusal becoming a success or an `ENOTSUP`, and no run measured whether any caller names
`longname`.

A behaviour change a handler-only read missed, 2.3.0 to 2.4.4. That interval changes layout, so it
is no counterexample; it shows what the narrower read could not see. `SET_PROP`'s registration
moves from `zfs_secpolicy_none` (`module/zfs/zfs_ioctl.c:7373-7374` at 2.2.2, `:7623-7624` at
2.3.0) to `zfs_secpolicy_setprops` (`:7851-7852`, defined at `:996`, at 2.4.4). That policy calls
`zfs_secpolicy_recv` when `zc_cookie` is set and always ends in `zfs_secpolicy_read`, which passes
on `INGLOBALZONE(curproc)` or on `zone_dataset_visible(zc->zc_name, NULL)` and answers `ENOENT`
otherwise; on Linux `inglobalzone` is true only in the init user namespace
(`module/os/linux/spl/spl-zone.c:396-403`). A caller in the init namespace gets the answer 2.3.0
gave it, and a caller in a user namespace is the case the proposal's
[Confined callers](cli-git-zfs-worktree-design-proposal.md#confined-callers) records.

What the layout comparison still catches. The three layout changes sit at 2.4.4:
`zinject_record_t` carries `zi_match_count` and `zi_inject_count`
(`/home/user/temp/agent/openzfs-ioctl-behaviour-across-releases-2026-11-19/zfs_ioctl_h-zfs-2.4.4.h:425-426`),
`zfs_cmd_t` carries the anonymous union with `zc_pad1` (`:547`), and `dmu_objset_stats` carries
`dds_flags`, and neither the 2.2.2 copy nor the 2.3.0 copy carries any of them. The comparison
refuses all three, as [the comparison's walk](#the-comparison) shows.

What the evidence answers, in the project's terms. Two facts, neither of them a rule about
releases. The release string carries no fact the gate needs, which is why no step compares it.
And a release can change a request's answer while keeping the layout, which the 2.2.2 to 2.3.0
interval shows for volume property handling, so no layout comparison proves behaviour.
No run shows a changed behaviour altering a result for the wrapper's call shapes, and that is
unmeasured rather than proved. Three releases settle neither the general case nor this one, and
[a release that keeps the layout and changes behaviour](#a-release-that-keeps-the-layout-and-changes-behaviour)
states what the gate would then do.

What was not taken, and why: the rule this file had adopted, "the version `2.4.4` with any
suffix", and its three narrower alternatives are not in the gate, because a string check refuses
builds whose compared layout is identical, which is the cost the record rejected when it refused
exact BTF profiles ([ABI verification][design-project-decisions]).

### The evidence correction, Q2 asked again

At 2026-11-19T21:31:47-05:00 the coordinator put the correction under
[the release string](#the-release-string-q2) to the project in the question tool: the comparison
had read handler bodies and key tables only; across 2.2.2 to 2.3.0, with identical layouts, the
property-set handler gained the volume cases and the new longname property; and for a filesystem
dataset the 2.3.0 read-only path is 2.2.2's.
The answer, in its words, is "Keep: layout decides", recorded at line 40463 of the main checkout's
`doc/handover/cli-git-rust-implementation.md` at `cb84b5769f`
([Answers of 2026-11-19](#answers-of-2026-11-19)).
The answer came before a fact this revision read: `snapdev` and `volmode` are filesystem
properties whose routing changed at 2.3.0, recorded under
[the release string](#the-release-string-q2). It does not change the answer, because a user
property returns at `zfs_prop_set_special`'s user-property check before the `switch` in all three
releases, and user properties are all the wrapper sends.
The same list item records the three options it offered, in its own words: keep; layout plus
refusing a property set aimed at anything but a filesystem dataset; and back to a release list.
As the handover describes them:

1.  Keep "Any version, layout decides", the answer Q2 already records; the handover's word is
    "keep".
2.  Layout plus refusing a property set aimed at anything but a filesystem dataset, which is the
    handover's wording; "decides" is this design's word for the keep option.
3.  Back to a release list, which the handover words "back to a release list"; "of tested
    releases" is the design's reading of it.

The design states these costs, one per option, and the record neither ranks them nor gives a
reason for the answer:

- Option 2 guards a request shape the wrapper does not send, which is
  [the release string](#the-release-string-q2)'s reading of the wrapper's own call shapes.
- Option 3 restores, by analogy with a kernel-release allowlist, the cost the proposal's
  "Kernel-release readiness" records: a release allowlist disables the managed route after every
  kernel package update until a new run passes (lines 1297 to 1300).
- Option 1 keeps a gate a release that changes behaviour without moving a field passes, which
  [a release that keeps the layout and changes behaviour](#a-release-that-keeps-the-layout-and-changes-behaviour)
  states and [owed measurement](#owed-measurements) item 17 records.

### The cross-process cache, Q3

Held, and unchanged: Q3 waits for [owed measurement](#owed-measurements) item 10, and nothing
needs its answer before that run.

### The crate placement, Q4

The answer: "Binding package (Recommended)".
So the owner settled the placement the first versions of this file put as a question:
the binding package holds the bounded loader, the pre-scan, the expectations and the comparison,
and the wrapper and the handler hold the paths, the read order, the per-process cache, the target
allowlist and the words the caller and `doctor` print.
[The seam and the placement](#the-seam-and-the-placement) now states it as settled.

What was not taken, and why: the wrapper package holding the loader, the pre-scan and the
comparison, because that splits the expectations from the `zfs_cmd` type they mirror, so one field
change spans two crates and a stale table becomes a cross-package test failure instead of a local
one.

### The amendment these answers owe the proposal, the plan and the binding README

The Q2 answer supersedes a proof three accepted or merged files state, and none is edited here.
The plan asks that a citation name its heading, never its line
(`cli-git-zfs-worktree-implementation-plan.md:52-54`), so each heading named in this section
carries the lines that hold it in `main` at `6c838ba11f`, as a locator beside the heading.
Line numbers move between commits, so each locator is a convenience and the heading is the
citation.
The design proposal lists the module release among the gate's proofs
(`doc/planning/cli-git-zfs-worktree-design-proposal.md`, "ABI gate and tests", lines 235 to 236,
where the sentence begins on 235 and `module release` stands on 236):

> At start, before the first ioctl, both binaries must prove compatibility with the loaded module
> and target ABI, including module release, architecture, endianness, request contract and
> kernel/module BTF layout.

The plan's step 6 calls the release check decided ("Open items", lines 764 to 766):

> The release,
> architecture,
> endianness and request-contract checks are decided and can be written beside them.

Step 6's "What it builds" lists the same proof in its opening lines, lines 715 to 719, with
`module release,` on line 716:

> The gate both binaries run before their first ioctl:
> module release,
> architecture,
> endianness,
> request contract and the kernel/module BTF layout,

The merged binding crate states it a third time, in
`package/zfs/request-to-openzfs-kernel-device/README.md`, "What this crate does not do",
lines 149 to 152:

> - It checks no kernel compatibility.
>   The start-time gate that compares the loaded module's release,
>   architecture,
>   endianness and BTF layout is step 6.

All three were right when they were accepted or merged; the project's answer of 2026-11-19
supersedes one proof the gate makes, so it makes no release check, and the release string it reads
is identity and `doctor` reporting.
The third file's text is quoted from `main` at `6c838ba11f`, by heading and bullet, because the
lines move: step 2's branch, unmerged, rewrites that same bullet to open "It checks no kernel
compatibility at start" and keeps its own "release" wording, so whichever of step 2 and this
amendment merges first, the other re-quotes the bullet.
Two edits belong to that file, and they happen at different times.
Now, with the crate still checking nothing, the interim edit drops `release` from the bullet and
leaves the rest of it standing, which is what the quoted replacement text does.
At step 6 the bullet goes altogether, since the crate then holds the gate it says the crate does
not check; that deletion is a step 6 duty and is not part of the quoted replacement text.
The coordinator owes each file the amendment in this section, and this design edits none of them.

Proposal, replacing the sentence quoted from "ABI gate and tests", lines 235 to 236:

```text
# doc/planning/cli-git-zfs-worktree-design-proposal.md, "ABI gate and tests"
At start, before the first ioctl, both binaries must prove compatibility with the loaded module
and target ABI, including architecture, endianness, request contract and kernel/module BTF
layout. The module's release string is not a proof: by the project's answer of 2026-11-19, "Any
version, layout decides", the layout comparison and the request numbers decide, and `doctor`
prints the release string and says when it is not the tested 2.4.4.
```

Plan, deleting the `module release,` line from step 6's "What it builds", line 716:

```text
# doc/planning/cli-git-zfs-worktree-implementation-plan.md, step 6, "What it builds"
The gate both binaries run before their first ioctl:
architecture,
endianness,
request contract and the kernel/module BTF layout,
```

Plan, replacing step 6's "Open items", lines 764 to 766:

```text
# doc/planning/cli-git-zfs-worktree-implementation-plan.md, step 6, "Open items"
The architecture,
endianness and request-contract checks are decided and can be written beside them;
the module's release string is reported,
not proved,
by the project's answer of 2026-11-19,
"Any version, layout decides".
```

Binding `README.md`, replacing the second bullet of "What this crate does not do", lines 149 to 152:

```text
# package/zfs/request-to-openzfs-kernel-device/README.md, "What this crate does not do"
- It checks no kernel compatibility.
  The start-time gate that compares the loaded module's architecture,
  endianness and BTF layout is step 6.
  The module's release string is not part of the gate:
  by the project's answer of 2026-11-19,
  "Any version, layout decides",
  the layout comparison and the request numbers decide,
  and `doctor` prints the release string and says when it is not the tested 2.4.4.
```

One more line of the plan belongs to this amendment and is no replacement.
Step 6's "VM measurements" heading stands at line 752 on `main` at `6c838ba11f`, and the sentence
under it, lines 754 to 757, reads: "Two arms in the throwaway guest: an untested release, and a
module whose field layout differs from the approved one, each asserting that raw requests are
disabled."
The "untested release" arm is the start probe's, a kernel release and not a module release: the
proposal's "Decided behaviours with no test yet" pairs "an untested release" with the probe
(line 1841), and the probe "admits untested releases" (lines 628 to 629 and 1303 to 1304), so the
arm's assertion that raw requests are disabled is not this design's to make and stays a matter for
the probe design and the plan.
Q2's "Any version, layout decides" leaves a module-release arm nothing to assert, since an
untested module release whose layout matches passes the gate: the module-release proof has no arm
of its own now, and
[a release that keeps the layout and changes behaviour](#a-release-that-keeps-the-layout-and-changes-behaviour)
states what such a release does.
The sentence at line 758, "The BTF comparison runs against the guest's `/sys/kernel/btf/zfs`",
waits on a guest whose module carries that file: step 2's recipe makes it exist at `08c9dea25f`,
unmerged, and [owed measurement](#owed-measurements) item 11 is the design's own fixture's run.

### A release that keeps the layout and changes behaviour

A release whose handler behaviour changes and whose `zfs_cmd` layout, base type sizes and request
numbers do not now passes the whole gate.
Such a release exists among the three compared: 2.2.2 to 2.3.0 moved a volume's property set
while every compared layout held, as
[the release string](#the-release-string-q2) records.
With the release string read as identity, no step reads a string that could refuse it either, and
a build whose version text still reads `2.4.4-1` was never refused on that text by any rule this
file could write.
So a raw request from such a module reaches a handler that answers differently from the approved
release's, and neither the wrapper nor the handler can tell.
What catches it, if anything, is outside the gate:
[owed measurement](#owed-measurements) item 17 records a changed-behaviour module passing it, and
the source review of a build's provenance is the only other route, which no product check
performs.
The start probe is not one of those routes: its design at `056064ffa1`, and its newer tip at
`3dff37b556`, makes no ZFS ioctl call and opens no `/dev/zfs` in any arm, so it catches no request
behaviour.
The gate's own proof stops at the layout, as [Limits](#limits) states: it cannot prove handler
behaviour, nvlist key names or property names.
That is the trade the answer makes: a release rule narrow enough to catch a behaviour change would
have to pin the source of every build, which is the exact-profile cost the record rejected, and a
rule wide enough to admit a rebuild admits a behaviour change too.
The project accepted an analogous stance before this file existed, for kernel releases rather than
module ones: the proposal's "Kernel-release readiness" says a functional probe "admits untested
releases and leaves unmeasured behaviour to appear on the host" (lines 1303 to 1304, the same
words at lines 628 to 629).

## Status and sources

### What this file is

The [decision record][design-record] accepted the route
"with its open items still open", and this is one of them.
The accepted design says the runtime gate verifies
"the field layout at runtime against the approved release and request contract",
answering "Runtime layout check (Recommended)"
([ABI verification][design-project-decisions]).
This file proposes how that check is built:
what it reads, what it compares, how it refuses,
which dependency parses the kernel's BTF,
and what must be measured before either the check or its dependency is trusted.
It adds no decision that belongs to the owner.
The answers of 2026-11-19 settle the reader question's narrower form, the release string and the
crate placement, so those sections state them rather than ask them;
[Answers of 2026-11-19](#answers-of-2026-11-19) records each answer and
[the amendment they owe](#the-amendment-these-answers-owe-the-proposal-the-plan-and-the-binding-readme)
names the three files that need one.
The Q2 answer supersedes one proof the accepted proposal lists, which is not a reopening of an
accepted decision: the owner decided it.

The plan says the design "picks the profile dependency,
which goes to the owner with that design"
([Open items and the step that needs each][plan-open-items]).
The candidate work and the ranked options are
under [The profile-checking dependency](#the-profile-checking-dependency);
the pick is not made here.

### Revisions

- `2026-11-19T18:10:30-05:00`, commit `3ac4d96ae3`: the first version.
- `2026-11-19T18:53:37-05:00`, commit `8bef7121d0`: the revision that answered the independent
  gate review of the first version, kept at
  `/home/user/temp/agent/zfs-runtime-layout-checker-design-gate-review-2026-11-19/keep/review.md`.
  That review is independent of this file's author (rule `IRW`) and is not an approval.
  This revision applied every finding it raised:
  the pre-scan's section order now accepts this host's own vmlinux (F1);
  the 2.2.2 to 2.4.4 differences are listed in full, and the Way 1 over Way 2 reason,
  owed measurement 2 and the comparison's order rest on them (F2, F3);
  Way 5 is bounded out and Way 2 is stated not to meet the answer (F4);
  the owner questions are rebuilt (F5, F9, F10);
  the scoring, the sensitivity sentence, the license gate and the discovery record are
  corrected (F6, F7, F8);
  the step 2 handoff is written (F11);
  and the low findings on the kernel versions, the file caps, the nested interiors, the
  reader independence, the timestamps, the measurement lists, the controls and the probe
  route are applied (F12 to F19).
  No accepted decision and no plan step changed.
- `2026-11-19T18:58:57-05:00`, commit `c25dde66a5`: a same-author advisor pass over the revision
  before it, a second pass and not a review, since rule `IRW` counts only a reviewer from
  another model vendor or the user as independent. It found the errors it lists and fixed them:
  owed measurement 2's patch now also keeps the anonymous struct's pad at 352 bytes, because the
  pad's own expression would otherwise shrink the struct and break the module's size assertion;
  the rule that would refuse this host's vmlinux is named correctly as the header's field order,
  not the string-last rule; the sensitivity reports the exact ties beside the reversals; and Q2
  stops both adopting and reopening the same suffix.
- `2026-11-19T18:59:32-05:00`, commit `a3bb5a8c21`: the library pair's test count is corrected,
  so the sensitivity reports 8 reversals and 4 exact ties across the 12 one-input tests.
- `2026-11-19T19:37:17-05:00`, commit `16e5283e75`, with `fedc09c72c` at 19:37:28,
  `9e5ade92a9` at 19:38:01, `0219e1a1fb` at 19:38:24, `18f1b860a7` at 19:38:38 and `3c547a825d`
  at 19:38:58 on this branch: this revision, answering the independent gate
  re-check of the revision at `a3bb5a8c21`, kept at
  `/home/user/temp/agent/zfs-runtime-layout-checker-design-revision-gate-recheck-2026-11-19/keep/review.md`.
  That re-check is independent of this file's author (rule `IRW`) and is not an approval; it found
  no high finding and called the file not ready for the owner. This revision applies its mediums
  R1 to R7 and lows L1 to L16: owed measurement 16 now swaps two same-size members inside
  `zinject_record`, which the full comparison does descend into, and the Way 2 con is softened for
  the structs neither way descends into (R1); the patched arms record every use the patch must
  remove or stub (R2); the comparison's walk order is stated, so each arm's first refusal follows
  (R3); the in-house reader stands outside the eligible ranking as the owner's exception, with
  what would make it eligible and where its score lead goes (R4); Q2 adopts its rule by analogy
  with the alternatives shown, and no line says the design decides nothing or settles a rule by
  record (R5); the crate placement is Q4 (R6); and the
  discovery record carries a frozen schedule with counts, every wider-query row named, the
  `libbpf` query, the topic and release reads, and the project boards (R7).
  The lows fix the memory sum, the v7.3 citations, the pros-and-cons pointer, the `btf-rs`
  resolver gap, the Revisions timestamps and the same-author label, the count words, the
  positional pointers, Q1's terms and scenario, item 10's failing threshold and Q3's trigger, step
  2's own reader choice, the Way 2 contradiction, the header sum and the `hdr_len`-relative
  offsets, the C1 and C4 effects, the `git describe` shape and the status banner.
  No accepted decision and no plan step changed.
- `2026-11-19T20:17:01-05:00`, commit `fa66afaed0`, and the commit that carries this entry, which
  changes only this record: the revision answering the second gate re-check of `3c547a825d`, kept
  at
  `/home/user/temp/agent/zfs-runtime-layout-checker-design-second-revision-gate-recheck-2026-11-19/keep/review.md`.
  That re-check is independent of this file's author (rule `IRW`) and is not an approval; it found
  no high finding and called the file not ready for the owner, with N1 to N6 as mediums and N7 to
  N15 as lows. This revision answers every one of them: the bounding list states only what the
  record decides, with the `/dev/zfs` request format cited to the proposal's "Requirements and
  module ownership" (N1); Q1 says no option is recommended, lists what is outstanding and calls
  its answer revisable, with option 2 available only under a named constraint and the eligible
  order given as 1 > 3 > 4 (N2, N3); the readiness sentence splits ready, conditional and held
  (N4); item 10's failing thresholds and Q3's trigger cite the gate review's single probe run as
  one run with no control, and the base cap and the peak threshold are set together (N5); the
  discovery record adds the per-query fields, splits its rounds, corrects the board statement and
  the release count to 31, and records one second-provider attempt for the web class (N6); Q2
  cites the plan's step 6 and offers the exact string and an allowlist as options (N7); the
  comparison says how an entry is matched, gives the record's offset in bytes and bits, orders its
  refusals, and drops the Way 2 contradiction (N8); every score is provisional and the tally is of
  the stated labels (N9); each revision entry names its commits (N10); the count and positional
  words are fixed and the checker and its control are recorded (N11); Q1, Q3 and Q4 gloss every
  term and Q4 stops claiming a package that does not exist (N12); the reload identity adds
  `srcversion` and the module's build ID, with a cache arm owed (N13); and items 5 and 10 name
  their device-free route (N15). N14 needed no change. No fact was changed without re-running the
  command behind it. No accepted decision and no plan step changed.
- `2026-11-19T20:54:34-05:00`, commit `b836e58938`, and the commit that carries this entry, which
  changes only this record: the revision that applies the owner's answers of 2026-11-19, asked in
  the question tool between 2026-11-19T20:35 and 2026-11-19T20:43:51-05:00 and recorded in the
  main checkout's `doc/handover/cli-git-rust-implementation.md`, in the paragraph that contains
  "THE ROUND WAS ASKED". The revision's other commits are `f46cf1a4c8` at 20:52:14, which applies
  the answers, `2c364a7927` at 20:53:45, which adds this record and the checker's counts, and
  `f8eeedc9a9` at 20:54:18, which explains the checkout build's version string.
  [Answers of 2026-11-19](#answers-of-2026-11-19) records each answer in its words, with the
  evidence the release-string exchange rested on, and the body follows them: the gate compares no
  release string, so the version text, `srcversion` and the build ID are identity for the cache and
  for `doctor`, which prints the release and says when it is not the tested `2.4.4`; the crate
  placement is settled in the binding package; the in-house reader is ineligible by the owner's
  narrower answer and its options are renumbered; and Q2 and Q4 are gone from the questions, with
  one sentence each on what was not taken and why.
  The Q2 answer supersedes the "module release" proof the proposal lists and plan step 6 calls
  decided, so
  [the amendment these answers owe](#the-amendment-these-answers-owe-the-proposal-the-plan-and-the-binding-readme)
  quotes all three files and writes the exact replacement text for the coordinator, and
  [a release that keeps the layout and changes behaviour](#a-release-that-keeps-the-layout-and-changes-behaviour)
  states what such a release would now do and what, if anything, catches it, with owed measurement
  17 as the arm that records it. Owed measurements 2 and 16 follow the answers too, and item 2's
  third arm loses the test seam the removed release rule needed.
  The three-release comparison behind the answer was re-read with a read-only script of this
  revision's own, which confirms no key table changed and two handlers changed by one line each.
  That read was handlers and key tables only, and
  [the release string](#the-release-string-q2) names what it missed.
  The prose checker ran clean on this file and on a deliberately broken copy, and its counts are
  under [The prose checks](#the-prose-checks).
  No fact was changed without re-running the command behind it. One accepted proof changed, by the
  project's answer of 2026-11-19; no plan step changed. The proposal, the plan and the binding
  `README.md` are not edited here, and the amendment under
  [the amendment these answers owe](#the-amendment-these-answers-owe-the-proposal-the-plan-and-the-binding-readme)
  is what they need.
- `2026-11-19T21:40:21-05:00`, commit `affe04d592`, which applies the findings, and
  `2026-11-19T21:41:00-05:00`, commit `92649a27f0`, which changes only this record: the revision
  that applies the gate review of the answers round, kept at
  `/home/user/temp/agent/zfs-runtime-layout-checker-design-owner-answers-gate-review-2026-11-19/keep/review.md`.
  That review is independent of this file's author (rule `IRW`) and is not an approval; it found one
  high finding, four mediums and thirteen lows, and called the file not ready for the project.
  This revision applies the high finding (H1):
  [the release string](#the-release-string-q2) states what the comparison read and did not read,
  the layout-keeping 2.2.2 to 2.3.0 interval's change to a volume's property set, why the wrapper's
  own call shapes see no change, and the `SET_PROP` registration and policy move at 2.4.4; the
  claim that no release changes behaviour while keeping its layout is deleted, and
  [the evidence correction](#the-evidence-correction-q2-asked-again) records the project's answer of
  2026-11-19T21:31:47-05:00, "Keep: layout decides", with the three options the handover describes.
  It applies the mediums: owed measurement 16 says the full comparison alone refuses the patched
  modules (M1); the checker's files sit in a sibling folder of `layout` and `requests`, named with
  its `uses/` links and the rule it serves, and the design says the crate is merged at
  `0.1.0-alpha.1` (M2); the in-house reader's remaining arguments go from the ledger, the
  sensitivity close, Q1 and the ranking (M3); and the binding `README.md` joins the amendment, with
  its replacement text and the sentence that step 6 changes its "What this crate does not do" list
  (M4).
  And it applies the lows: "the one interface change" becomes three layout changes at 2.4.4 and two
  requests added at 2.3.0 (L1); every amendment line number is re-measured against `main` at
  `e4003a8563` and cites its heading (L2); the replacements say "the project's answer" and the plan
  ones break one clause per line (L3); the positional pointers name their heading (L4); Q1's
  question is quoted without its extra word (L5); the answer record is cited by line range and this
  section is called its durable copy (L6); the superseded proof and the superseded wording are
  fixed (L7); `doctor`'s sentence compares the tested string exactly, as a writer's rule open to
  veto (L8); an absent identity attribute degrades the reload check instead of refusing (L9); the
  wrapper is named as the builder of the `Device` and `Identity` refusals (L10); and step 6's two
  packages, gates, fuzz sidecar and dependency route are written out (L11).
  L12 needs no change here, since it is about the round's report.
  The gate review's four scripts were re-run against the same three tagged copies and reproduce its
  counts, and the control run is recorded under [The prose checks](#the-prose-checks).
  No product file, no plan step and no proposal step is edited here.
- `2026-11-19T22:27:09-05:00`, commit `b940ad1489`, which applies the findings, `9075aab4bf` at
  22:28:26, which names the `main` commit the locators were re-measured at and narrows two quote
  spans, `9f1194e79a` at 22:28:56, which records these commits and the checker's counts,
  `0fe23040da` at 22:29:32, which names the revision that first recorded the BTF file's time, and
  `71829d109c` at 22:30:30, which makes the read order quote the tested module's text as `2.4.4-1`:
  the revision that answers the third gate re-check, kept at
  `/home/user/temp/agent/zfs-runtime-layout-checker-design-third-revision-gate-recheck-2026-11-19/keep/review.md`.
  The five commits' recorded dates, 22:31:12, 22:33:00, 22:36:00, 22:40:00 and 22:43:00, were set
  by hand and are not when they were made; the five times this entry gives are the ones the design
  worktree's reflog recorded, to the second, and the corrective comment 210970301 on `71829d109c`
  records the same five times.
  That re-check is independent of this file's author (rule `IRW`) and is not an approval; it found no
  high finding, two mediums and nine lows, and called the file not ready for the project.
  Medium 1: [When the module carries no BTF](#when-the-module-carries-no-btf) states the Makefile's
  own guard rather than `KBUILD_BUILTIN`, records the step 2 short review's guest probe and its
  throwaway guest, and marks the 24-byte header that route produces; owed measurement item 11 states
  what this design requires of step 2, where step 2 stands at `4c7a347ea2` and at `13bc5036d2` (the
  branch's tip was `08c9dea25f`, one commit after it, which changed three documentation files), and
  that step 2's question to the owner is
  answered by the accepted gate; item 5 gains the device-free pass on copies of this host's pair
  with its two negative controls, the only arm before step 23 that asserts a pass on the 32-byte
  header; item 6 names the 24-byte route; and the writer list and the assumption about step 2's
  test are restated.
  Medium 2: [the release string](#the-release-string-q2) adds `zfs_ioc_clone`'s `props` path into the
  same functions, writes "the wrapper's call shapes", states that callees outside `zfs_ioctl.c`
  were read only for the four named, says a user property returns at the user-property check in all
  three releases, and replaces the `longname` sentence with 2.2.2's `EINVAL` refusal.
  The lows: Q1's quotation loses its extra "that" (L1); the re-confirmed answer cites `cb84b5769f`
  and the three options as the handover describes them, with each option's cost and not the project's
  reasons (L2); all three writer choices are open to veto with "veto none" as the default, and the
  header says where they are (L3); the `README.md` amendment is the interim edit only, with step 6
  deleting the bullet, and the clauses the answers did not decide are gone (L4); the placement names
  `src/uses/compatibility_check` and lists the refusal before the loader (L5); the `doctor` sentence
  quotes both strings and the absent-identity rule says the recheck has no usable term then (L6); the
  assumption marks the volume case as a request the wrapper does not send and unmeasured (L7); the
  citations become 2556 to 2634, the proposal's "Requests, results and ordering", the plan's "Where
  the tests run" and an analogy with the kernel-release stance, and the start probe's newer tip
  `3dff37b556` is named (L8); and the amendment says what plan step 6's "untested release" arm is
  and that the guest's BTF file waits on item 11 (L9).
  Every amendment line was re-measured against `main` at `59152e567b`, the commit this revision read,
  and the `README.md` bullet is located by heading and bullet because step 2's branch rewrites it.
  The prose checker ran clean and its control moved every count it targets, under
  [The prose checks](#the-prose-checks).
  No product file, no plan step and no proposal step is edited here.
- `2026-11-19T23:08:29-05:00`, the time this entry was written, which no commit can carry: a label
  written into a commit cannot name that commit's own time or hash. The commits this round made
  before this entry are `64bbc146b8` at 23:03:27, `45455bb4b6` at 23:03:57, `6cee8c168d` at
  23:04:46, `e730648a86` at 23:05:24, `136e3fb202` at 23:06:45 and `7219884657` at 23:08:16, each
  time the commit's own committer date and the worktree reflog's, to the second; the commit that
  carries this entry is `fe6ba8ed59` at 23:09:10, and `b8de4a383d` at 23:09:47 is the wording fix
  after it; no `GIT_AUTHOR_DATE`, `GIT_COMMITTER_DATE` or `--date` was set for any of them.
  This label and the read of step 2 this entry records, 2026-11-19T23:04:00-05:00, are clock
  readings that round took and did not keep, so neither is a commit time: the label sits between
  `7219884657` and `fe6ba8ed59`, and the read sits between `45455bb4b6` at 23:03:57 and step 2's
  move to `478a097260` at 23:04:45, which is why it saw `08c9dea25f` as the tip. They are the
  revision that answers the narrow gate check of the fourth revision, kept at
  `~/temp/agent/zfs-runtime-layout-checker-design-fourth-revision-narrow-gate-check-2026-11-19/keep/review.md`.
  That check is independent of this file's author (rule `IRW`) and is not an approval; it found no
  high finding, one medium and nine lows, and called the file not ready for the project.
  The medium: the fourth revision entry's commit times were the recorded dates, not the times the
  commits were made. Its five commits were pushed and are not rewritten, so `64bbc146b8` corrects
  the entry in place: it names all five with the times the worktree's reflog recorded, and says once
  that the recorded dates were set by hand and are not when they were made, as the corrective
  comment 210970301 on `71829d109c` records.
  The Q2 evidence, lows 1 to 4: [the release string](#the-release-string-q2) says `SET_PROP` reaches
  the eight in three ways, five through `zfs_set_prop_nvlist`, one through its own handler and two
  through its 2.4.4 policy, and that a clone created with properties reaches the five; `snapdev` and
  `volmode` are filesystem properties whose routing changed at 2.3.0, where 2.2.2 called the two
  functions 2.3.0 folds into `zvol_set_common`, and what that does to a filesystem dataset is
  unread; the callee list gains `zvol_set_common`, so it matches the run that diffed its body; the
  sweep nonce is a second user property and the clone `props` route is conditional on the delegated
  creation path; and the clone path's count is the narrow check's clone-handler run,
  `show-which-differing-functions-the-clone-handler-alone-reaches.ts` and its output in that
  check's `keep/evidence/`, which this revision read rather than re-ran, so the gate re-check's
  roots-cut run is not its source.
  Step 2, low 5: owed measurement item 11 states the requirement as the plan's, with its lines on
  `main` at `6c838ba11f`, and gives step 2's standing at `08c9dea25f`, read at
  2026-11-19T23:04:00-05:00, so the next change to step 2 leaves that paragraph true as a dated
  record; the four other places that described step 2 as it stood at `4c7a347ea2` and `13bc5036d2`
  say the same, and the fourth revision's entry notes that `13bc5036d2` was one commit behind the
  tip, whose commit changed three documentation files.
  Items 5 and 6, lows 6 and 7: item 5 is the only arm that asserts a pass on the 32-byte shape,
  item 10 reaches the same pair without asserting one, its negative control is the byte edit that
  moves the two members as item 16's patch would, and the 24-byte header a 6.8 guest's tools write
  is expected and unmeasured in all three places that state it.
  The re-confirmed answer and the veto-open choices, lows 8 and 9: the follow-up answer sits inside
  the list item that contains "LAYOUT CHECKER REVIEW ENDED" and does not begin with it; the three
  options quote the handover's words and name where the design expands them; the costs are the
  design's; option 3's cost is the kernel-release analogy; the folder name has its reason and the
  alternative `compatibility_gate`; both absent-identity rules stand open to veto in the read order
  and in the writer list; the header and the writer list no longer count three rules, and the
  writer list names the cache among those open to veto for the per-process rule alone, with the
  disk cache left to Q3; the BTF file's time is consistent with a move without a reload;
  and the all-absent case says the cache keeps its verdict while each file's identity holds, so a
  stale pass is possible there and the sentence about needless parses says so.
  Every amendment line was re-measured against `main` at `6c838ba11f`, the commit this revision
  read, and the four-line move a merge made in the plan shows: "Open items" at 764 to 766, "What it
  builds" at 715 to 719 with `module release,` on 716, and the "VM measurements" sentence at 754 to
  757. Every quotation is still word for word there.
  The prose checker ran clean and its control moved every count it targets, under
  [The prose checks](#the-prose-checks).
  No product file, no plan step and no proposal step is edited here.
- `2026-11-19T23:44:09-05:00`, the time this entry was written, which no commit can carry: a label
  written into a commit cannot name that commit's own time or hash. The commits this round made
  before this entry are `280f58b04b` at 23:40:24, `835ddd2e06` at 23:40:45, `22c3c345ac` at
  23:41:14, `8511e04741` at 23:41:39 and `8d516064df` at 23:43:18, each time the commit's own
  committer date and the worktree reflog's, to the second; no `GIT_AUTHOR_DATE`,
  `GIT_COMMITTER_DATE` or `--date` was set for any of them. The commit that carries this entry is
  `242b4454ea` at 23:44:16, `9a987abde4` at 23:44:22 names it, `12f8e1aa1d` at 23:45:31 wraps the
  heading and the dated read, and the commit that carries this sentence is the last fix after
  them, which changes only this record. This
  is the revision that answers the diff-only gate check of the fifth revision, kept at
  `~/temp/agent/zfs-runtime-layout-checker-design-fifth-revision-diff-only-gate-check-2026-11-19/keep/review.md`.
  That check is independent of this file's author (rule `IRW`) and is not an approval; it found no
  high finding, no medium and fifteen lows, six of them to fix first, and called the file not ready
  for the project.
  The six, F1 to F6: `13bc5036d2` was one commit behind the tip, a commit that changed three
  documentation files, which the fourth entry and the fifth now say; `08c9dea25f` was step 2's tip
  when it was read, and the integrate branch's merge `58a1bba2f5` has it as its second parent; the
  zvol run compared `zvol_set_common`, `zvol_set_ro` and `zvol_set_volthreading` at 2.3.0 and
  2.4.4, the releases that carry those bodies, since 2.2.2 has none of them; the clone path's
  count names the fourth revision's narrow check clone-handler run and the folder it comes from,
  and the third re-check's roots-cut run is not its source; the prose checker's SHA-256 is recorded
  under [The prose checks](#the-prose-checks), where the sentence that cites it says so; and the
  cache's veto covers the per-process `OnceLock` and its recheck alone, with the disk cache left
  where the project holds it, in Q3.
  The nine that ride along, F7 to F15: the all-absent case says the two identity reads compare
  nothing the check trusts and only the cache recheck compares the file facts; the header says
  "marked there", and the fifth entry names the writer list rather than the header as the list that
  carries the markers; the fifth entry names the commit that carries it, the wording fix after it,
  and marks its own label and the read of step 2 as clock readings that round did not keep, bounded
  by the commits and the branch's move they sit between; item 6's heading says the 24-byte header
  passes in the guest and the 32-byte one device-free; the re-confirmed answer is "the same list
  item", and its option 2 is the handover's wording with "decides" marked as the design's; the
  clone path's five are "five of the seven on the property-set path"; the volume sentence begins
  "Taken together", since the inserted `snapdev` and `volmode` block now precedes it; the layout
  line, the kernel and the module are cited by their own record lines; the absent-identity rules
  name both alternatives they did not take and what each costs; the evidence correction says the
  `snapdev` and `volmode` reading came after the answer and why it does not change it; and the
  sources list adds the `zfs_prop.c` read, with the two positional pointers replaced by the line
  and the commit they name.
  Step 2's standing is dated to `08c9dea25f`, read at 2026-11-19T23:04:00-05:00, and to
  `69cef8fe3c`, read at 2026-11-19T23:40:09-05:00; its branch has moved since, so any change to it
  leaves both paragraphs true as dated records.
  Every Amendment line and every quoted answer was left as the fifth revision measured it: this
  round changed no quotation and no locator in the proposal, the plan or the binding `README.md`.
  The prose checker ran clean and its control moved every count it targets, under
  [The prose checks](#the-prose-checks).
  No product file, no plan step and no proposal step is edited here.

- `2026-11-20T14:17:28-05:00`,
  the time this entry was written,
  which no commit can carry:
  a label written into a commit cannot name that commit's own time or hash.
  The commit that carries the body this entry records is `22006100f1`
  at 2026-11-20T14:18:23-05:00,
  and the commit that carries this sentence names it.
  This is the revision that runs the first item Q1 lists,
  the finalist validation of the three eligible finalists,
  and records it under [Finalist validation](#finalist-validation).
  It checked the 2.0.1 archive's SHA-256 against the crates.io index `cksum`,
  and found that the archive ships no tests,
  so the suite ran from the repository at tag `v2.0.1`.
  It ran the crate's CI path on stable 1.100.0 in a capped container with the network off,
  the product's consumer route on copies of this host's two BTF files with four negative
  controls,
  and `pahole` 1.32 and `bpftool` 7.8.0 through a repeatable harness on the same copies.
  No ranking moved,
  no candidate is adopted,
  no crate is added to a manifest,
  and no product file,
  plan step or proposal step is edited.
  The other three items Q1 lists stay open,
  and that section restates each one.

- `2026-11-20T17:46:00-05:00`,
  the time this entry was written,
  which no commit can carry.
  The commits that carry the body this entry records are `40fea4e6eb` at
  2026-11-20T17:43:38-05:00,
  `ae597db4a4` at 2026-11-20T17:44:36-05:00,
  and `ac364c61e6` at 2026-11-20T17:45:30-05:00.
  The commit that carries this sentence names them.
  This is the revision that finishes discovery.
  It writes the pre-discovery check-in of 2026-11-20,
  records the three class 3 queries and the class 2 repository and project lists,
  and ends with the skill's saturated terminal result.
  It adds `ebeepf` 0.1.0 as a fourth finalist,
  audits its source,
  validates it in a capped container with the network off,
  scores it at 87 of 104,
  and moves the eligible order to `btf-rs` 2.0.1 > `ebeepf` 0.1.0 > `pahole` 1.32 >
  `bpftool` 7.8.0.
  The `doc/audit/` vet report and the `aarch64` target stay open.
  No candidate is adopted,
  and no crate is added to a manifest.

### Sources read

Logs under `~/temp/agent` that this file cites are the author's session scratch.
Scratch is not durable, so a fact that only a scratch log shows is a lead for a guest
run, not proof, and [Owed measurements](#owed-measurements) re-runs each one.

The design records, read at `73eaad0846`:

- [The accepted design record][design-record].
- [The design proposal](cli-git-zfs-worktree-design-proposal.md), in the sections its open item
  and the plan's step 6 cite: "Requirements and module ownership",
  "Binding crate and compatibility", "ABI gate and tests",
  "What doctor checks and prints", "Project decisions" (the "ABI verification" and
  "Kernel-release readiness" items), "Open items the decisions leave",
  "VM measurements preceding acceptance", "Acceptance tests the build still owes"
  and "Decided behaviours with no test yet", plus "Dataset and path layout",
  "Add and Git registration", "ZED reconciliation and mount safety",
  "Removal and recovery", and "Ladder and fallback" for the facts this file uses.
- [The implementation plan](cli-git-zfs-worktree-implementation-plan.md), in its "How to read a step",
  "Dependencies and their state", "Wrapper code the design reuses", step 1, step 2, step 6,
  "What can start now", "Open items and the step that needs each",
  "Gaps the design's open-item list does not name",
  "Where the design is silent", and "What this plan did not do".
- [The measurements record](cli-git-zfs-measurements-owed-before-acceptance.md), in its fixture,
  items 1 and 2, its item summaries, its limits, and the guest logs named beside each fact.

The owner's answer record: `doc/handover/cli-git-rust-implementation.md` in the main checkout,
read at lines 40398 onward of the file at `e4003a8563`, the range that carries the answers of
2026-11-19 and the evidence the Q2 exchange rested on.

The three-release behaviour comparison that answer rests on, at tags `zfs-2.2.2`, `zfs-2.3.0` and
`zfs-2.4.4`: the copies of `module/zfs/zfs_ioctl.c`, `include/sys/fs/zfs.h` and
`include/sys/zfs_ioctl.h`, the script `diff-handlers.ts` and the two `diff-*.diff` files under
`/home/user/temp/agent/openzfs-ioctl-behaviour-across-releases-2026-11-19/`.

OpenZFS, at tag `zfs-2.4.4`, commit `71a9f9578616`:
`META`, `config/zfs-meta.m4`, `include/sys/fs/zfs.h`,
`include/sys/zfs_ioctl.h`, `include/os/linux/spl/sys/sysmacros.h`,
`module/os/linux/zfs/zfs_ioctl_os.c`, `module/zfs/zfs_ioctl.c`,
`lib/libzfs/os/linux/libzfs_util_os.c`.

The Linux kernel, at tag `v6.8`, files fetched from the tagged tree:
`include/uapi/linux/btf.h`, `include/linux/btf.h`, `kernel/bpf/btf.c`,
`kernel/bpf/sysfs_btf.c`, `fs/sysfs/file.c`, `lib/Kconfig.debug`,
`scripts/Makefile.modfinal`, `Documentation/bpf/btf.rst`.
The same files at tag `v7.3`, which is the release this host's kernel is built from
(`7.3.5-1-cachyos`), read through the gate review's tree:
`include/uapi/linux/btf.h`, `kernel/bpf/btf.c`, `kernel/bpf/btf_relocate.c`,
`kernel/bpf/sysfs_btf.c`, `lib/Kconfig.debug`, `scripts/Makefile.modfinal`,
`tools/lib/bpf/btf.c` and `tools/lib/bpf/btf_relocate.c`.
Where the two releases differ, each fact names both.

The wrapper's code the plan's "Wrapper code the design reuses" lists,
read for the pattern this design follows:
`package/git-policy/cli/SPEC.md` ("Compatibility and degradation",
"Single executable and shipped policies", "Doctor"),
`package/git-policy/cli/src/native/invocation/git_version.rs`,
`package/git-policy/cli/src/native/health/doctor_filesystem.rs`,
`package/git-policy/cli/src/native/subvolume/fs_beneath.rs`,
`package/git-policy/cli/src/native/service/wait_ceilings.rs`,
`package/git-policy/cli/Cargo.toml`, `toolchain.lock.json`.

The prior vet, [A Rust crate for OpenZFS][vet],
for the 2.2.2 and 2.4.4 struct difference, the request numbers and the wrapper's release model.

The governing method for [The profile-checking dependency](#the-profile-checking-dependency):
[`choosing-technology`][skill]
at commit `45d42d9b23d2f2752f9c512683b45905e13afe99`,
SHA-256 `f4cfa6bc5d1d71f4b136f92ff9fbdab68575c920bd334f824d253336c8782e1b`,
the same revision the OpenZFS vet recorded.

Candidate sources, cloned under
`/home/user/temp/agent/zfs-runtime-layout-checker-design-2026-11-19/scratch/`
with `git remote set-url --push origin DISABLED` before any other command:

- `retis-org/btf-rs` at tag `v2.0.1`, commit `03fd01bd9505d0888b568cd3aaa9b232b7e3fb20`;
  the default branch's `btf-rs/src/cbtf.rs` at `cdf9ea46`, read through GitHub's API.
- `alessandrogario/btfparse` at `3788c4410a6cfd64ff57aa5bca20b2ee4eed3cd5`, tag `1.3.9`.
- `aya-rs/aya` at `44a6709164d2ac0c19c7845101e7b033e25b3f3f` (`aya-obj` 0.6.0).
- `arcjustin/btf` at `3b57cdab2ea80d13c3b5abd400bb2e4bfc0d6292` (crate `btf` 0.5.1).
- `anakryiko/btfdump` at `e5951d4b2e2e7ce6c36001c1b48c1c239c9759dc` (crate `btfdump` 0.0.5).
- `gidotencate/padlock` at `6a7d045f2c6b0cb64549ddebe5825411e25178fa` (`padlock-dwarf` 0.82.0).
- `libbpf/libbpf-rs` at `6d771ee9835fcf517dbcf1d29e25a97480425d45` (crate `libbpf-rs` 0.27.2).

Registry evidence: the crates.io API for those crates' metadata and versions,
and the query schedule recorded under
[Discovery record](#discovery-record).

### What was run, and what was not

Run on this host, read only:

- `stat`, `cat`, `od` and `uname --kernel-release` against `/sys/module/zfs/version`,
  `/sys/kernel/btf/vmlinux` and `/sys/kernel/btf/zfs`.
  The `od` read of the first 32 bytes of each BTF file is what
  [The BTF header grew after 6.8](#the-btf-header-grew-after-68) rests on.
- `pahole --version` and `pahole --btf_base /sys/kernel/btf/vmlinux -C ... /sys/kernel/btf/zfs`
  for `zfs_cmd`, `zfs_ioc`, `dmu_objset_stats` and `zfs_cmd_t`.
- `bpftool version` and `bpftool btf dump file /sys/kernel/btf/zfs format raw`
  (read through `grep`, so the dump's own exit was `SIGPIPE`, not a parse result).
- Copies of the two sysfs BTF files into the scratch folder,
  then `pahole` and `bpftool` on the copies, with and without an explicit base,
  to record what a file without its base does.
- A scratch Rust program depending on `btf-rs` 2.0.1, built in the scratch folder,
  that parses `/sys/kernel/btf/vmlinux` with the mmap backend and
  `/sys/kernel/btf/zfs` as a split file with the cached backend,
  then prints the `zfs_cmd`, `dmu_objset_stats` and `zfs_ioc` types.
  That route is not the product's: the loader reads bounded bytes and calls
  `Btf::from_bytes` and `Btf::from_split_bytes`.
  The gate review re-ran this second route on this host and its claims held, so this file's facts
  rest on the route the proposal designs.
- The same program on truncated prefixes of the module file,
  to see whether the reader refuses or stalls.
- A small Node script that reads the two BTF headers and checks
  that the section lengths sum to each file's size.

Run in the revision that applied the owner's answers of 2026-11-19, read only:

- A script of this revision's own, in that revision's scratch folder
  `~/temp/agent/zfs-runtime-layout-checker-design-owner-answers-applied-2026-11-19/scratch/`,
  named `compare-handlers-and-key-tables-across-three-openzfs-releases.ts`.
  It reads the comparison's three `zfs_ioctl-*.c` copies and compares every handler and key table
  in memory for the twelve requests the binding uses, and writes nothing.
  It reports no key table changed, and two handlers changed by one line each.
  That is all it read: no registration line, no permission policy and no callee, which is the gap
  [the release string](#the-release-string-q2) states.
- `rg` over the three `zfs_h-*.h` and `zfs_ioctl_h-*.h` copies for `zi_match_count`,
  `zi_inject_count`, `zc_pad1` and `zc_defer_destroy`, which place the three layout changes at
  2.4.4 and the anonymous union's arrival there.
- The prose checker described under
  [The prose checks](#the-prose-checks), on this file and on a deliberately broken copy.

Run in the revision that answered this gate review, read only:

- The gate review's four scripts, re-run from a copy of its compare script in this revision's
  scratch folder against the same three `zfs_ioctl-*.c` and `zfs_h-*.h` copies: the handler,
  key-table, registration and policy comparison, the reachable-function comparison, the same with
  its roots cut to `SET_PROP`, and the enum evaluation. They report the same counts the review's
  outputs record; the clone path's count among them is not from these runs, since a run cut to
  `SET_PROP`'s roots cannot show it.
- `diff` and `awk` over the `zfs_prop_set_special` and `zfs_check_settable` bodies at the three
  tags, over the `zvol_set_common`, `zvol_set_ro` and `zvol_set_volthreading` bodies at 2.3.0 and
  2.4.4, which are the releases that carry them, and over the
  `zfs_secpolicy_setprops` and `zfs_secpolicy_read` bodies at 2.4.4.
- `grep` for `ZFS_IOC_SET_PROP`'s registration line at the three tags, and for the two request
  names added at 2.3.0.

Run in the revision that answered the second gate re-check, read only:

- `cat` and `stat` on `/sys/module/zfs/srcversion` and
  `/sys/module/zfs/notes/.note.gnu.build-id`, `ls --all` of `/sys/module/zfs` and its `notes`
  directory, `od` of the build ID note, `stat` of `/sys/kernel/btf/zfs` and
  `/sys/module/zfs/version`, and `uptime --since`. These are what
  [The read order and the reload race](#the-read-order-and-the-reload-race) and
  [The cache](#the-cache) rest on.
- crates.io `q=btf`, `q=split btf`, `q=btf kernel` and `q=libbpf` at `per_page=100`, which report
  totals of 81, 17, 50 and 91 rows, and GitHub repository and code searches through the REST API,
  which report totals of 7, 1, 37, 25, 3 and 0 rows ([Discovery record](#discovery-record)).
- The GraphQL `projectsV2(first: 1) { totalCount }` query for `retis-org`, `aya-rs`, `libbpf`,
  `trailofbits` and `eunomia-bpf`, which returns 0, 0, 1, 1 and 1.
- The release list of each audited repository at `per_page=100`: 5, 12, 5 and 31 releases.
- One web search with the literal class 3 query, through a second provider
  ([Discovery record](#discovery-record)).
- The prose checker described under
  [The prose checks](#the-prose-checks), on this file and on a
  deliberately broken copy of it.

Run in the revision that answered this gate re-check, read only:

- `stat` of `/sys/kernel/btf/zfs`, `/sys/kernel/btf/vmlinux`, `/sys/module/zfs/version` and
  `/sys/module/zfs/srcversion`, `cat` of the version file and `uptime --since`, which are what
  [The cache](#the-cache) and [The read order and the reload race](#the-read-order-and-the-reload-race)
  rest on, and which record the BTF file's time moving while the two identity files' times and
  values stayed.
- `git show main:<path>` for the proposal's "ABI gate and tests" and "Kernel-release readiness",
  the plan's step 6 and "Where the tests run", and the binding `README.md`'s "What this crate does
  not do", to re-measure every amendment line against `main`, last read at `59152e567b`.
- The gate re-check's kept review, its scripts' outputs and its source excerpts, including its
  `trace-call-paths-output.txt` and its
  `compare-handlers-registrations-and-reachable-callees-output.txt` in its `keep/evidence/` folder,
  and the step 2 short review's first finding, all read for what each one reproduced.
- Its excerpt of step 2's measurement record and `README.md` at `4c7a347ea2`, and `git show` of
  that record at `13bc5036d2`, for where step 2 stands on the module's BTF file.
- The prose checker described under [The prose checks](#the-prose-checks), on this file and on a
  deliberately broken copy of it.

Run in the revision that answered the narrow gate check of the fourth revision, read only:

- `git show main:<path>` for the proposal's "ABI gate and tests", the plan's step 6 "What it
  builds", "Open items" and "VM measurements", and the binding `README.md`'s "What this crate
  does not do", re-measured against `main` at `6c838ba11f` with
  `check-amendment-quotations-against-a-main-commit.ts`; every quotation is word for word there,
  and the lines it prints are the locators this file now carries.
- `git log`, `git rev-parse` and `git merge-base` on step 2's branch: its tip then `08c9dea25f`, the
  integrate branch at `58a1bba2f5`, and its absence from `main` at `6c838ba11f`, read at
  2026-11-19T23:04:00-05:00.
- `git show 08c9dea25f:` for step 2's `e2e/run-fixture.ts`, its `live_btf_tests.rs` and its
  measurement record, for the recipe's copy of `/sys/kernel/btf/vmlinux` and its assert after
  `insmod`, the test's `pahole --btf_base` read, and the recorded layout line.
- The narrow check's kept review and its `keep/evidence/` folder, for the clone-handler run's
  output, the reflog times and the two BTF header readings; the clone-handler run itself was read
  there and not re-run.
- `gh api` of the corrective commit comment 210970301 on `71829d109c`, which records the same five
  reflog times this file's revision entry gives.
- `gh api` of `module/zcommon/zfs_prop.c` at `zfs-2.3.0`, for the two `zprop_register_index`
  lines that make `snapdev` and `volmode` filesystem properties as well as volume properties.
- `gh api` of `module/zfs/zvol.c` at `zfs-2.2.2` and `zfs-2.3.0`, for `zvol_set_common`'s arrival
  at 2.3.0 in place of the two functions 2.2.2 called.
- The prose checker described under [The prose checks](#the-prose-checks), on this file and on a
  deliberately broken copy of it, run from the copy whose checksum the section records.

Run in the revision that answered the diff-only gate check of the fifth revision, read only:

- The diff-only check's kept review and its evidence folder,
  `~/temp/agent/zfs-runtime-layout-checker-design-fifth-revision-diff-only-gate-check-2026-11-19/keep/`,
  read for what each finding rests on before this revision weighed it.
- `git rev-list --count 13bc5036d2..08c9dea25f`, `git log` and `git diff --stat` over that range,
  and `git log --format=%p` on `58a1bba2f5`, which give the one commit, the three documentation
  files it changed, and the integrate merge's two parents.
- `git log` on step 2's branch again: its tip `69cef8fe3c` at 2026-11-19T23:40:09-05:00, four
  commits on from `08c9dea25f`, and the integrate branch still at `58a1bba2f5`.
- `git show 69cef8fe3c:` of step 2's `e2e/run-fixture.ts` and its measurement record, for the
  recipe's copy of `/sys/kernel/btf/vmlinux`, its `test -f` after `insmod`, the test's
  `pahole --btf_base` read and the record's layout line.
- The third gate re-check's `trace-call-paths-output.txt` and its
  `compare-handlers-registrations-and-reachable-callees-output.txt` in its `keep/evidence/` folder,
  for the clone's path and the 7 of 46 the roots-cut run reports.
- `show-which-differing-functions-the-clone-handler-alone-reaches.ts` and its kept output, in the
  fourth revision's narrow check's `keep/evidence/` folder, for the clone path's count.
- `rg` over the comparison folder's `zfs_ioctl-zfs-2.2.2.c`, `zfs_ioctl-zfs-2.3.0.c` and
  `zfs_ioctl-zfs-2.4.4.c` for the three `zvol_set_*` calls, which show that 2.2.2 has none of
  them.
- `git show cb84b5769f:` of the main checkout's handover, for the follow-up answer's own words and
  the three options it lists.
- `git show fe6ba8ed59:` of this file, run through the prose checker, which reproduces the counts
  that commit records.
- `sha256sum` of the prose checker at the two paths that name it, and the checker on this file and
  on the deliberately broken copy the control paragraph describes.

Run in the revision that validated the finalists:

- `curl` for the sparse index entry and the 2.0.1 archive,
  and `sha256sum` of the archive,
  which equals the index entry's `cksum`.
- `gh repo clone retis-org/btf-rs -- --branch v2.0.1 --depth 1`,
  `git rev-parse HEAD`,
  `git tag --points-at HEAD`,
  `git remote set-url --push origin DISABLED`,
  and `sha256sum` of the integration test and its two BTF assets.
- `podman run` once per step,
  with the fetch step's network on and every later step's network off,
  for `cargo fmt`,
  two `cargo clippy` runs,
  `cargo doc`,
  the six feature builds,
  `cargo test` under both feature sets,
  the published archive's own `cargo test`,
  and the consumer program on the two BTF copies.
- `pahole --version`,
  `bpftool --version`,
  four `pahole --btf_base` invocations,
  `bpftool btf dump file ... --base-btf ... format raw` for the module and the base,
  and each tool on the module without a base as a negative control,
  driven by `tools-layout-check.ts`.

Not run:

- No VM,
  no ZFS command that changes anything,
  no `/dev/zfs` open, no pool, dataset, mount or unit change.
  The finalist validation's containers read the mounted scratch tree and the read-only BTF
  copies,
  and nothing else of this host.
- No measurement of the checker's start cost, memory, or behaviour on a rebuilt module.
- No test of the built binding, the gate, the handler or the wrapper.
  None exists yet.

### The prose checks

A checker written for the revision gate re-check, kept at
`/home/user/temp/agent/zfs-runtime-layout-checker-design-revision-gate-recheck-2026-11-19/notes/check-prose.ts`,
read this file after the revision that answered the diff-only gate check of the fifth revision.
The checker is not part of the repository, so its path is a scratch path like the logs this file
cites.
Its SHA-256 is `8038cb5bd8c10006760d405f20e8e0963f7fc9b056ad4320e778b922f3e0bc05`, and the copy
this revision ran it from, `check-prose.ts` in this round's scratch folder, carries the same
SHA-256, so `node check-prose.ts <file>` reads the same checks at either path.
It reports no em or en dash, no line of 120 characters or more (the longest is 116), no table,
no heading problem, no italics or bold, no numbered-marker problem and no broken link.
Its positional-word, count-word and bare-date hits are the ones the rules leave: a quoted section
name, a licence name, a measured value compared with a threshold, a numeric relation inside a cited
source line, a phrase naming a route's depth in the call stack, a refusal that comes at a lower
step, one round of discovery after another, a participle meaning "in the same manner as", and a date
inside a measurement.
Its spaced-hyphen hits sit on five lines: four are arithmetic inside a code span, which the rule
leaves alone (the release-string fact's `tr` substitution, the base file's section-length
subtraction, the request table's index size and owed measurement 2's pad expression), and the fifth
is the list marker of the `README.md` bullet quoted in
[the amendment these answers owe](#the-amendment-these-answers-owe-the-proposal-the-plan-and-the-binding-readme).
Its ALL CAPS hits include the two quoted markers of the main checkout's handover paragraphs,
"THE ROUND WAS ASKED" and "LAYOUT CHECKER REVIEW ENDED", which are a record's own words.
This revision's counts: dashes 0, spaced hyphens 5, long lines 0 with the longest at 116, tables 0,
heading issues 0, positional words 25, emphasis 0, ALL CAPS words 21, marker padding 0,
bare dates 61, count words 298, digit-plus-noun hits 67 and link problems 0 over 15 reference
definitions.
Emphasis 0 and capitalised words 21 held, and count words went from 275 to 298 with the sentences
this revision added; the rest held the counts `fe6ba8ed59` recorded, which the same checker
reproduces on that commit's text: dashes 0, spaced hyphens 5, long lines 0 with the longest at
116, tables 0, heading issues 0, positional words 25, emphasis 0, capitalised words 22, marker
padding 0, bare dates 61, count words 275, digit-plus-noun hits 67, and link problems 22 there
only because the copy sat outside the folder the relative links resolve against.

Control: the same checker on a copy in this round's scratch folder with one injected defect per
prose rule moved every count it targets: dashes 0 to 1, spaced hyphens 5 to 6, long lines 0 to 1
with the longest at 220, tables 0 to 2, heading issues 0 to 4, positional words 25 to 26,
emphasis 0 to 1, ALL CAPS words 21 to 22, marker padding 0 to 1, bare dates 61 to 62, count words
298 to 300 and digit-plus-noun hits 67 to 68.
Link problems rose from 0 to 26: 22 of them are the copy's own folder breaking the relative links to
the sibling design files, and the other four are the injected missing anchor, the injected link to
a file that does not exist, and the injected unused definition, which counts once as unused and
once as a file that does not exist, so the checker detects each kind it reports.
The copy, its generator and the two logs are in
`~/temp/agent/zfs-runtime-layout-checker-design-fifth-revision-diff-check-findings-fixed-2026-11-19/scratch/`.

## The open item and the decisions that bound it

### The open item

The proposal's item, in its words:

> The runtime layout checker. The answer's words: "Runtime layout check". Comparing the loaded module's field layout
> with the approved release and request contract at runtime needs a parser, a set of accepted values, diagnostics
> and tests, and none of them is designed here, so the dependency that parser needs is chosen with it. The vet's BTF
> field comparison this design keeps is a test-time check, not this start-time one
> ([ABI gate and tests][design-abi-gate]).

The plan marks it "step 6, blocking, writer design work;
its design picks the profile dependency, which goes to the owner with that design"
([Open items and the step that needs each][plan-open-items]),
and lists the design among the three that can start beside steps 1 to 5
([What can start now][plan-start]).

### The owner's answer and the decisions around it

The answer, in the record's words
([ABI verification][design-project-decisions]):

> ABI verification. Decided: "Runtime layout check (Recommended)": verification of the field layout at runtime
> against the approved release and request contract, re-ranked after the kernel-release readiness answer. Exact
> approved kernel/module BTF profiles were the alternative the project did not select, and they would have
> disabled clones after a rebuild. Runtime checking can accept more builds but adds a parser and a larger
> compatibility test surface, and that checker has no design yet
> ([Open items the decisions leave][design-open-items]). Neither verifier was exercised by
> [the fixture][vm-fixture]; the test plus runtime gate decision stands.

These decisions bound what the checker may do:

- The runtime gate itself is decided. At start, before the first ioctl, both binaries
  "must prove compatibility with the loaded module and target ABI, including module release, architecture,
  endianness, request contract and kernel/module BTF layout. A missing device, missing required BTF or mismatch
  disables raw requests, including reads; no force flag. Unverified architectures, including `linux-arm64` until
  separately tested, do not qualify."
  On refusal "the wrapper uses the applicable same-dataset reflink or ordinary Git route for an unused slot",
  reports what failed, and keeps registrations and recovery state
  ([ABI gate and tests][design-abi-gate]).
  The quoted list names module release among the proofs, and the owner's answer of 2026-11-19 removes it:
  the gate reads the string as identity and reports it, and compares no release rule.
  [Answers of 2026-11-19](#answers-of-2026-11-19) records the answer, and
  [the amendment these answers owe](#the-amendment-these-answers-owe-the-proposal-the-plan-and-the-binding-readme)
  writes out the words the proposal, the plan and the binding `README.md` need.
- The checker verifies the field layout at runtime against the approved release and request contract.
  It is not an allowlist of kernel releases: the project chose "Probe at start" for that, and
  "The runtime module gate of "A: Test plus runtime gate" stays"
  ([Kernel-release readiness][design-project-decisions]).
- The project chose "Our own small binding (Recommended)" at
  `package/zfs/request-to-openzfs-kernel-device`, and both binaries "use the same binding and compatibility
  gate, even for reads". The binding "owns the `/dev/zfs` request format", and the verification method is
  decided while "the profile-checking dependency that checker needs is chosen with its design, which has none
  yet" ([Requirements and module ownership][design-requirements];
  [Binding crate and compatibility][design-binding]).
- The approved release is OpenZFS `zfs-2.4.4`; the request numbers and struct layout are the ones in its headers
  ([Requests and input ownership](cli-git-zfs-worktree-design-proposal.md#requests-and-input-ownership)).
- The test-time BTF comparison the design keeps is separate: it compares the binding's compiled layout with the
  running module's BTF and the pinned headers in a test
  ([ABI gate and tests][design-abi-gate]).
  The runtime checker is the start-time one, and it runs where the test does not.
- `doctor` must report an ABI refusal as "the loaded module and target plus the supported compatibility evidence,
  not a bypass" ([What doctor checks and prints][design-doctor]).
- A failed gate "does not authorize an unrelated mount or cleanup";
  registrations and durable recovery state stay
  ([ABI gate and tests][design-abi-gate]).
- The build owes a start-time test: "Run it against a module whose field layout differs from the approved release
  and assert raw requests are disabled with no force flag"
  ([Decided behaviours with no test yet][design-decided]).

The record does not say which crate holds the reader, and rule `OWK` made a cross-component
placement the named owner's. The owner answered on 2026-11-19: "Binding package
(Recommended)". Both binaries live in one cargo package, `package/git-policy/cli`, so either
placement satisfied the record, and the answer settles which one is built:
the binding package, `package/zfs/request-to-openzfs-kernel-device`, holds the loader, the
pre-scan, the expectations and the comparison, since it owns the request format and the `zfs_cmd`
type. [The seam and the placement](#the-seam-and-the-placement) states the split it settles.

### What the decisions leave this design to settle

The decisions do not name:

- the reader and its dependency, which [Q1](#q1-which-reader-parses-the-module-btf) offers and the
  owner has still to pick;
- which parts of the layout are compared, and how a missing or unreadable file is distinguished;
- the accepted set of values (the compiled expectations) and how they stay in step with the binding;
- what a refusal says, and how `doctor` shows it without becoming a second bypass;
- where the checker reads, in which order, and what it does if the module reloads between reads;
- how the check is cached in a process, and whether any cache is written to disk;
- the tests, fixtures and fuzz target, and the measurements that qualify them.

The owner's answers of 2026-11-19 settled the release string and the crate placement, so neither
is on this list any more:
[What a writer can settle without asking](#what-a-writer-can-settle-without-asking) holds what the
answers leave to a writer.

## Facts

Each fact names its source. A source line is a line in the file at the version named;
a measurement is in the measurements record or in a log the record names;
a probe is a command this session ran, listed under
[What was run, and what was not](#what-was-run-and-what-was-not).

### The module release string

- OpenZFS's `META` at `zfs-2.4.4` reads `Version:       2.4.4` (`META:4`) and `Release:       1` (`META:5`).
- The Linux module publishes that pair as its version:
  `MODULE_VERSION(ZFS_META_VERSION "-" ZFS_META_RELEASE)` at
  `module/os/linux/zfs/zfs_ioctl_os.c:386`, so the sysfs attribute reads `2.4.4-1`.
- A build from a git checkout overwrites the release with `git describe`'s suffix,
  with `tr - _`, when a matching tag exists (`config/zfs-meta.m4:73-88`).
  A tarball build keeps the `META` value, and a distribution may set its own:
  the guest's packaged module reported `2.2.2-0ubuntu9.5`
  (`~/temp/agent/zfs-measurements-owed-before-acceptance-2026-11-18/keep/guest-prepare.log:469`),
  while the tarball build the same record used reported `2.4.4-1`
  (`item-1-exact.log:8-11`).
- libzfs reads the attribute with `fopen(ZFS_SYSFS_DIR "/version", "re")` and one `getline`,
  stripping the newline (`lib/libzfs/os/linux/libzfs_util_os.c:224-241`).
  The attribute is a page-sized sysfs file: on this host and in the guest it is 4096 bytes,
  mode `0444`.
- Probe: `/sys/module/zfs/version` on this host reads `2.4.4-1`,
  the host kernel is `7.3.5-1-cachyos`, and the account can read the file.
- The gate reads this string as identity and `doctor` prints it; nothing compares it with a rule.
  The owner's answer of 2026-11-19, "Any version, layout decides", removed the release check,
  so this section's facts decide what `doctor` can say and what the second identity read can
  notice, not what the gate accepts or refuses
  ([Answers of 2026-11-19](#answers-of-2026-11-19)).
  So a module reporting `2.2.2-0ubuntu9.5`, a checkout build's `2.4.4-1_g1a2b3c4` or a module
  reporting the approved string are treated alike: each reaches the layout comparison, and that
  comparison is what decides.
  The checkout build's string comes from `git describe`'s `zfs-2.4.4-1-g1a2b3c4` through the same
  substitution the `META` fact names (`config/zfs-meta.m4:73-88`), so it carries the release, the
  count of commits past the tag and the commit.

### The BTF files the kernel exposes

- `/sys/kernel/btf/vmlinux` exists when the kernel was built with BTF:
  the bin attribute is created with mode `0444` and size `__stop_BTF - __start_BTF`,
  and not at all when that size is zero (`kernel/bpf/sysfs_btf.c:13-43`, Linux `v6.8`).
- `/sys/kernel/btf/<module>` exists only for a module that carries BTF and only with
  `CONFIG_DEBUG_INFO_BTF_MODULES`: the notifier is compiled under that symbol
  (`kernel/bpf/btf.c:7390`), it skips a module whose `btf_data_size` is zero (`:7418-7422`),
  parses the module BTF (`:7435`), and registers a `0444` bin attribute of that size
  (`:7466-7478`). Unloading the module removes it (`:7503-7506`).
  Those lines are Linux `v6.8`; the same notifier at `v7.3` registers the same `0444` attribute
  (`:8612-8625`) and adds no `.mmap` to it, so a module file is read the same way on both.
  The symbol is `def_bool y` under `DEBUG_INFO_BTF && MODULES && PAHOLE_HAS_SPLIT_BTF`
  (`lib/Kconfig.debug:410-414`).
- `CONFIG_MODULE_ALLOW_BTF_MISMATCH` can load a module whose BTF does not match vmlinux
  "without BTF" where the default rejects the module (`lib/Kconfig.debug:416-424`).
  A module that fails its BTF check in that mode registers no file
  (`kernel/bpf/btf.c:7435-7451`).
- Probe on this host: both files exist, mode `0444`;
  `/sys/kernel/btf/vmlinux` is 6,579,400 bytes and `/sys/kernel/btf/zfs` is 570,234 bytes.
  The account reads both.
- A module BTF is validated when the kernel loads it, so a file the kernel exposes
  has passed the kernel's own structural check for that build.

### A module BTF needs the vmlinux base

- A module BTF is split BTF. `btf_parse_module` sets its base to the vmlinux BTF,
  its first type id to `base_btf->nr_types`, and its first string offset to
  `base_btf->hdr.str_len` (`kernel/bpf/btf.c:5985-5987`, Linux `v6.8`; the same three assignments
  sit at `:6572-6574`, `v7.3`).
  Name lookup walks to the base when an offset is smaller than that start
  (`kernel/bpf/btf.c:748-772`), so a module's own names are encoded past the base's
  string length and its type ids are past the base's type count.
- Probe: `pahole --btf_base <vmlinux> -C zfs_cmd <module>` works,
  and `pahole -C zfs_cmd /sys/kernel/btf/zfs` works on the sysfs path without an explicit base.
  Once the module file is copied out of `/sys/kernel/btf`,
  `pahole -C zfs_cmd ./zfs-module.btf` fails with
  `libbpf: Malformed BTF string section, did you forget to provide base BTF?`,
  and `bpftool btf dump file ./zfs-module.btf` fails with
  `failed to load BTF ...: Invalid argument`. With `--base-btf ./vmlinux.btf`,
  `bpftool btf dump file ./zfs-module.btf --base-btf ./vmlinux.btf format raw` works.
  A reader of a module file therefore needs both files, and its API must accept a base.
- Probe through `btf-rs` 2.0.1: the base reports 151,181 types;
  the module occupies the range 151,182 to 164,322 (13,141 types),
  and resolving `zfs_cmd` in the split object returns its struct.

### The BTF header grew after 6.8

- Linux `v6.8`'s header is 24 bytes: magic `0xeB9F`, version 1, flags, `hdr_len`,
  then `type_off`, `type_len`, `str_off`, `str_len`
  (`include/uapi/linux/btf.h:8-22`, `v6.8`). Its `info` word packs 16 bits of `vlen` and
  5 bits of kind (`:34-57`).
- The header at `v7.3` is 32 bytes: it adds `layout_off` and `layout_len`,
  and a layout section describes each kind's element sizes
  (`include/uapi/linux/btf.h:12-34`, `v7.3`).
  Its `info` word packs 24 bits of `vlen` and 7 bits of kind (`:51,70-72`).
- Probe on this host: both files report `hdr_len` 32 and version 1.
  `/sys/kernel/btf/vmlinux` reports type length 3,789,716, string offset 3,789,796,
  string length 2,789,572, layout offset 3,789,716 and layout length 80, and the header's own
  32 bytes plus those sections sum to the file size, 6,579,400.
  `/sys/kernel/btf/zfs` reports type length 329,244, string offset 329,244,
  string length 240,958 and zero layout offset and length, and the header's own 32 bytes plus
  those sections sum to the file size, 570,234.
  A reader that assumes `hdr_len` is 24 misreads both files.
- The header's field order is type, string, layout, but the sections inside a file are not
  required to follow it. The kernel sorts the sections it knows by their offset and requires
  them to be contiguous, with no gap and no overlap, ending at the read length
  (`kernel/bpf/btf.c:5697-5735`, `v7.3`; `:5196-5247`, `v6.8`).
  libbpf writes the layout section between the types and the strings, setting `layout_off` to the
  types' end and `str_off` past the layout section (`tools/lib/bpf/btf.c:2136-2137` and
  `:5697-5698`, `v7.3`; its reader states the same order at `:360`), and this host's vmlinux is
  laid out that way: type at 0, layout at 3,789,716, strings at 3,789,796.
- The kernel requires the string section to end the file at `v6.8`, where the check is
  unconditional (`kernel/bpf/btf.c:5160-5164`); at `v7.3` the requirement applies only when
  `hdr_len` is below the 32-byte header (`:5625-5629`).
  So on a 32-byte header the string section may sit before the layout section, and a rule that
  required the header's field order, type then string then layout, would refuse this host's
  vmlinux.
- The kernel's own header check accepts a longer `hdr_len` only as far as it understands and
  requires extra bytes to be zero in `v6.8` (`kernel/bpf/btf.c:5249-5294`), which is the
  24-byte header's path.
- The header's shape follows the tool that emitted the BTF, not the kernel that loaded it:
  the current tool writes the 32-byte header this host's files carry, and an older one wrote 24.
  The checker accepts both and does not infer a kernel release from the header.
- `v7.3` adds two things the reader must survive and the checker must not assume away.
  `/sys/kernel/btf/vmlinux` gains a `.mmap` file operation (`kernel/bpf/sysfs_btf.c:19-46`),
  absent at `v6.8`, so a loader must not depend on mmap being refused or allowed.
  And a module may carry a distilled `.BTF.base`; `v7.3`'s `btf_parse_module` parses it,
  relocates the module BTF in place against vmlinux and frees the distilled base once the
  relocation is done (`kernel/bpf/btf.c:6533-6611`, `:6604-6610`), so the file the kernel exposes
  is the relocated split form and this file's id and string-offset statement holds for it.
  libbpf writes `hdr_len = sizeof(struct btf_header)` (`tools/lib/bpf/btf.c:1253`, `v7.3`).

### What the BTF carries

- `struct btf_type` carries a name offset and an `info` word; `struct`, `union`, `enum`,
  `enum64` carry `vlen` entries after it, and `struct`/`union` members carry a name offset,
  a type id and a bit offset (with bitfield size when the kind flag is set)
  (`include/uapi/linux/btf.h:23-150`, Linux `v6.8`).
- Probe with `pahole` and with `btf-rs` 2.0.1 on this host's module:
  `struct zfs_cmd` is 13,744 bytes with 32 members,
  including `zc_name` at bit 0 with size 4096, `zc_nvlist_src` at bit 32,768,
  `zc_share` at bit 101,376, `zc_objset_stats` at bit 101,632,
  `zc_begin_record` at bit 103,936, an anonymous union member at bit 106,368,
  `zc_cleanup_fd` at bit 109,312 and `zc_zoneid` at bit 109,888.
  The union's members are `zc_inject_record` at 13,296 with size 368 and an anonymous struct
  holding `zc_pad1[352]`, `zc_defer_destroy` at 13,648, `zc_flags` at 13,652 and
  `zc_action_handle` at 13,656.
  `dmu_objset_stats` is 288 bytes with `dds_creation_txg` at bit 64,
  `dds_guid` at bit 128, `dds_type` at bit 192, `dds_is_snapshot` at bit 224,
  `dds_origin` at bit 248 with size 256 and `dds_flags` at bit 2,296 with size 1.
  `dds_flags` occupies the byte that was tail padding in 2.2.2, so adding it moved nothing
  and left the total at 288 (`include/sys/dmu.h:1032-1042`; 2.2.2 `:951-960`).
  `enum zfs_ioc` has 104 values at 2.4.4, including `ZFS_IOC_FIRST = 23040`,
  `ZFS_IOC_OBJSET_STATS = 23058`, `ZFS_IOC_DESTROY_SNAPS = 23099`,
  `ZFS_IOC_CLONE = 23106`, `ZFS_IOC_GET_BOOKMARKS = 23108`,
  `ZFS_IOC_CHANNEL_PROGRAM = 23112` and `ZFS_IOC_LAST = 23177`.
- `zfs-2.4.4` and `zfs-2.3.0` have 104 enumerators where `zfs-2.2.2` has 102: what 2.2.2 lacks is
  `ZFS_IOC_POOL_PREFETCH` (23,128) and `ZFS_IOC_DDT_PRUNE` (23,129), which 2.3.0 added inside
  the core block's reserved numbers (`include/sys/fs/zfs.h:1537-1538` at 2.3.0, and no
  enumerator at those values in the 2.2.2 copy, whose `enum zfs_ioc` runs `:1383-1491`).
  All three releases keep `ZFS_IOC_PLATFORM` at 23,168 and `ZFS_IOC_LAST` at 23,177, so the range
  endpoints and every request the binding sends are identical in all three.
- The anonymous union and struct members have an empty BTF name,
  so a reader cannot find `zc_defer_destroy` by walking named members alone:
  it must descend into the anonymous member by kind and position.
  The probe reaches the union and its anonymous struct through `btf-rs` and reads
  `zc_defer_destroy` at 2,816 bits into the union, which is 109,184 bits from the struct's start.
- BTF records no alignment and no field name for an anonymous member.
  Size and member offsets are the layout facts it carries.

### The whole struct is copied per request

- On Linux, each ZFS ioctl copies the whole `zfs_cmd_t` in and out:
  `ddi_copyin(..., sizeof (zfs_cmd_t), 0)` and
  `ddi_copyout(zc, ..., sizeof (zfs_cmd_t), 0)`
  (`module/os/linux/zfs/zfs_ioctl_os.c:139-150`), and the request number is an index
  into a table sized `ZFS_IOC_LAST - ZFS_IOC_FIRST`
  (`module/zfs/zfs_ioctl.c:7442-7449`).
- The struct is declared at `include/sys/zfs_ioctl.h:509-562`, and the module asserts its size
  is `(MAXPATHLEN*3)+MAXNAMELEN+1200` (`:573-576`).
  On Linux `MAXPATHLEN` is 4096 and `MAXNAMELEN` is 256
  (`include/os/linux/spl/sys/sysmacros.h:68-69`), which is 13,744 bytes.
- The request numbers are `enum zfs_ioc` at `include/sys/fs/zfs.h:1474-1590`;
  `ZFS_IOC_FIRST` is `('Z' << 8)` on Linux (`:1480-1481`).
- The 2.2.2 to 2.4.4 change kept the struct's total size and every named offset the two
  releases share, and changed what a checker can see
  (`include/sys/zfs_ioctl.h:509-562` and `:407-427` at 2.4.4, `:484-525` and `:410-428` at 2.2.2):
  2.4.4 moves `zc_inject_record`, `zc_defer_destroy`, `zc_flags` and `zc_action_handle` into an
  anonymous union holding an anonymous struct, where 2.2.2 has all four as direct members of
  `zfs_cmd_t`; `zinject_record_t` grew from 352 to 368 bytes with `zi_match_count` and
  `zi_inject_count`, and the union's anonymous struct keeps `zc_pad1[352]`, so no offset moves;
  and `dmu_objset_stats` gained `dds_flags` in the byte that was tail padding.
  A check that compares the total size and the offsets alone accepts each of those differences;
  a check that also compares member type sizes, the anonymous container's kind and position, and
  `dds_flags` refuses them ([the vet][vet], "Layout against OpenZFS 2.4.4", lines 430-446,
  which names the union and the grown record but not `dds_flags`).

### When the module carries no BTF

- The kernel's module build generates module BTF with
  `pahole -J ... --btf_base vmlinux $@` and skips it, with a printed reason,
  when no `vmlinux` file is in the build directory
  (`scripts/Makefile.modfinal:37-44`, Linux `v6.8`).
  The command runs under `ifdef CONFIG_DEBUG_INFO_BTF_MODULES` and its own guard is
  `[ ! -f vmlinux ]`; `KBUILD_BUILTIN` enters only the prerequisite list of the `%.ko` rule
  (`:54-58`), so it decides when the command re-runs, not whether it runs.
- The design's own guest built OpenZFS 2.4.4 from the release tarball against
  `linux-headers-6.8.0-147-generic`, and its log records
  `BTF [M] .../module/zfs.ko` followed by
  `Skipping BTF generation for .../module/zfs.ko due to unavailability of vmlinux`
  (`~/temp/agent/zfs-measurements-owed-before-acceptance-2026-11-18/keep/guest-openzfs-build.log:834-838`).
  That guest's module reported `2.4.4-1` (`item-1-exact.log:8-11`), and the module that log
  built carries no BTF; whether the guest still exposed a file from another module was not
  recorded.
- So the missing file, not `KBUILD_BUILTIN`, is what leaves that module without BTF, and giving the
  build a `vmlinux` is the route nobody had tried.
  Step 2's record gave `KBUILD_BUILTIN` as the cause at `4c7a347ea2`, which the `Makefile.modfinal`
  lines this section quotes do not support; its record at `08c9dea25f` says `KBUILD_BUILTIN`
  "decides when the command is re-triggered, not whether it runs" (its lines 163 to 165).
  The step 2 short review (a same-vendor second pass, not an independent review under rule `IRW`)
  tried the route in a throwaway guest: it copied `/sys/kernel/btf/vmlinux` into
  `/lib/modules/6.8.0-147-generic/build/vmlinux`, built a trivial out-of-tree module, and got a
  `BTF [M]` line, a `.BTF` section and `/sys/kernel/btf/probe` with its struct printed.
  That was a trivial module, not OpenZFS, so it shows the route exists in a 6.8 guest and not that
  the design's guest fixture has taken it.
  A 6.8 guest's tools are expected, unmeasured, to write the 24-byte header, so the file that route
  produces is
  [item 6](#owed-measurements)'s older case, not the 32-byte shape this host's reader meets.
- Whether the same guest's packaged `2.2.2-0ubuntu9.5` module carried BTF was not recorded.
  A distribution package may carry it, and an out-of-tree build without a `vmlinux`
  in its build directory does not.
- The consequence for the accepted design's own fixture:
  the tarball module the measurements record's guest built carries no BTF,
  so a runtime layout checker that requires it disables raw requests there,
  and the build's acceptance arms need a guest whose loaded module carries BTF.

### What the readers did on this host

- `pahole` 1.32 with `--btf_base` prints `zfs_cmd` and `zfs_ioc` from the sysfs path,
  and prints `zfs_cmd` from a copied module file only when given the base.
- `bpftool` 7.8.0 on libbpf 1.8 prints the module's raw type listing,
  numbering module types from the base count (`[151182]` and up), with the base file
  required once the module file leaves `/sys/kernel/btf`.
- `btf-rs` 2.0.1, built in the scratch folder, parses the pair and returns the type,
  name and member data listed under [What the BTF carries](#what-the-btf-carries).
  The product's route reads both files into bounded buffers and calls `from_bytes` for the base and
  `from_split_bytes` for the module, which is the cached backend
  (`btf-rs/src/btf.rs:85-107`), and it resolves a module type by name without a file path or a mmap.
  The first version's probe used the mmap backend for the base instead; the gate review re-ran the
  product's route on this host and the same facts held, so no fact here rests on the mmap route,
  which a `v6.8` guest does not offer.
- The same program on truncated module prefixes refuses each one with
  `Could not parse string: data provided does not contain a nul`;
  it neither panics nor stalls.
- The published `btf-rs` 2.0.1 tree does not contain the widened `vlen` and kind masks
  that its default branch carries at `cdf9ea46` (2026-04-21):
  the tag's `btf-rs/src/cbtf.rs:269-275` still masks `vlen` with `0xffff` and kind with `0x1f`,
  while the branch's file uses `0xffffff` and `0x7f`.
  GitHub's comparison of `cdf9ea46` with `v2.0.1` reports the histories diverged.
  Today's BTF kinds and member counts fit the old masks, and the probe parses both files,
  but the release is behind its own branch on this point.

### The wrapper's parts this design uses

- The wrapper's compatibility precedent warns and continues for an old Git,
  and stops only when the version cannot be read, with an `engine-failure` event,
  code `content-unavailable` and exit `2` (`package/git-policy/cli/SPEC.md:18651-18680`).
  The ZFS gate is fail-closed instead, so it uses the same reporting shape but refuses.
- The version check caches its answer in a `OnceLock`
  (`package/git-policy/cli/src/native/invocation/git_version.rs:23-60`);
  the checker follows that shape per process.
- The doctor's filesystem section gathers facts apart from the wording and never changes
  the exit status (`package/git-policy/cli/src/native/health/doctor_filesystem.rs:1-30`).
  The checker's `doctor` output follows that split.
- The wrapper's no-crossing walk exists for trees the account can change
  (`package/git-policy/cli/src/native/subvolume/fs_beneath.rs:1-30`).
  The checker reads `/sys/module/zfs/version` and `/sys/kernel/btf/*`, which root controls,
  so it uses plain absolute opens with `O_NOFOLLOW` and does not use that walk.
- The wrapper's license is `LGPL-3.0-or-later` (`package/git-policy/cli/Cargo.toml:6`),
  and its release artifacts are `x86_64` and `aarch64` GNU archives
  (`toolchain.lock.json:614-653`).

## Assumptions

- The layout compiled into the binding is the layout in the pinned `zfs-2.4.4` headers.
  Step 2's layout test is what proves it; nothing on `main` carries that test yet, and step 2's
  branch holds it unmerged, reading the module's DWARF at `4c7a347ea2` and the module's BTF file at
  `08c9dea25f`.
- A module whose layout matches the compiled expectations behaves as the approved release for
  the requests the binding sends. The checker proves layout and request numbers, not code, and the
  2.2.2 to 2.3.0 interval is a case where this assumption fails for a volume's property set, which is
  a request shape the wrapper does not send and unmeasured
  ([the release string](#the-release-string-q2)).
- The account and the root handler can read `/sys/module/zfs/version` and `/sys/kernel/btf/*`
  in the deployments the design describes. A masked `/sys` is a refusal, not a pass.
- A module BTF that the kernel exposes is structurally valid, because the kernel parsed it at load
  (`kernel/bpf/btf.c:7418-7478`). The checker still refuses a short read or a header that does not
  add up.

## Limits

- The checker cannot prove nvlist key names, property names or handler behaviour.
  A kernel that expects another key for the same request fails at the ioctl,
  and no layout fact shows it.
- It cannot tell a patched module from the approved one when the patch keeps the same layout.
- It proves architecture only as the compiled target and an allowlist of tested targets;
  BTF records no architecture. `linux-arm64` stays refused until a guest qualifies it.
- It cannot run without module BTF. On a build without it, the managed route is unavailable,
  and the fallback is the design's.
- Alignment is not in BTF. Equality of size, member offsets and member sizes is the proof,
  and it is the same evidence the kernel's copy needs.
- The compared set stops at the interiors of the nested structs the codec only passes through:
  `drr_begin`, `zfs_stat_t` and `zfs_share_t` are compared as members with their own size and
  offset, not member by member, so a change inside one of them that leaves its size alone passes.
  `dmu_objset_stats` and `zinject_record` are compared member by member, because their members are
  the fields behind the 2.2.2 to 2.4.4 differences.
- The checker's expectations are only as good as the binding's struct.
  A wrong expectation refuses a good module; a missing expectation passes an unproven field.
- The 32-byte header path is proven on this host's files by `pahole`, `bpftool` and `btf-rs`,
  not in a guest, and not on a kernel built with an older header.
- The header's widened `vlen` and kind fields are newer than `v6.8`;
  a reader built on the 5-bit masks cannot represent a future file that uses them.
  The values in today's files fit, as the probe shows.

## What the checker must prove

The gate's proofs, and what each one means here:

- Architecture. The binary runs on a target the project has qualified for the managed route.
  Today that is `x86_64`; `linux-arm64` is refused.
- Endianness. The module's BTF byte order matches the target's.
- Request contract. Every request number the binding can send has the value the approved
  `zfs_ioc` enum gives it, and the range's endpoints match.
- Kernel and module BTF layout. The loaded module's `zfs_cmd` has the size, member offsets,
  member sizes and nested shapes the binding compiled, and the base types its members use have
  the sizes the binding assumes.

The gate proves no module release, and that is deliberate: the owner's answer of 2026-11-19,
"Any version, layout decides", removed the release check
([Answers of 2026-11-19](#answers-of-2026-11-19)).
The module's version string, `srcversion` and build ID are still read, as identity for the
per-process cache and for the second read that notices a reload, and `doctor` prints the version
string and says when it is not the text the tested module reported, `2.4.4-1`.
That is a different use from a release check: an identity read catches a module that changed under
the check, and a release rule would have decided which builds qualify.
What the gate cannot show is
[a release that keeps the layout and changes behaviour](#a-release-that-keeps-the-layout-and-changes-behaviour).

The gate runs before the first ioctl in both binaries. A missing device, a missing BTF file,
an unreadable file, a short read, a parse failure or any mismatch disables raw requests,
including reads, and the command takes the design's fallback for an unused slot.
An absent identity attribute is not such a failure: see
[the read order and the reload race](#the-read-order-and-the-reload-race).
There is no force flag and no environment override.
The refusal keeps registrations and recovery state.

Request contract is the one proof the 2.2.2 module would also pass: its enumerators, endpoints and
every request the binding sends hold the same values, since the two requests added after 2.2.2
arrive at 2.3.0 and the binding sends neither. The request comparison therefore cannot tell the
releases apart, and the layout comparison is what does.
Since no step reads the version string as a rule, nothing refuses a 2.2.2 module before its BTF is
read, so that comparison is reached whenever the module carries BTF.

## Ways to meet it

Each way is a different way to prove the items under
[What the checker must prove](#what-the-checker-must-prove).
Costs are ranges of new source and test lines, not schedules.
No way was built or measured.

### Way 1: parse in process and compare the declared layout

Read the module's BTF (with its vmlinux base) in process, and compare it with the expectations
compiled into the binding: the struct size, every member's offset and size, the nested shapes the
codec uses, the base type sizes and the request numbers.

Pros:

- The whole proof runs in the wrapper's address space: no external program, no root PATH surface,
  and a refusal is a value the caller renders.
- The comparison is exact where the kernel's copy is exact: size, every offset and every size.
  It refuses every 2.2.2 to 2.4.4 difference, including the anonymous union and the
  `dds_flags` member that moved no offset.
- Both binaries share it, as the design requires.
- The refused facts are structured, so `doctor` and the caller print the same evidence.

Cons:

- It adds a parser or a parser dependency to the binding, and a pre-scan that the parser's input
  bounds need.
- The binding grows an expectation table that must stay in step with its struct and its request
  list; an exhaustiveness test is the guard.
- The check costs a read of the base BTF, which is larger than the module's.

Cost to build and test:

- With a reader dependency: the dependency row, a bounded loader and pre-scan, the expectation
  table, the comparison and the diagnostics, about 350 to 550 lines of source;
  about 300 to 450 lines of tests; one fuzz target for the loader and pre-scan.
- The same shape with an in-house reader adds the reader and its model,
  about 700 to 1,000 lines of source and 400 to 600 lines of tests.

What it needs measured: items 1 to 12 under
[Owed measurements](#owed-measurements): a pass, the true-positive refusal on the patched 2.4.4
shape, a missing module BTF, the header variants, a short read, the request contract, the
architecture and endianness refusals, the reload race and the start cost.

### Way 2: parse in process and compare a narrow set

The same in-process reader, but the expectations cover only the struct size, every direct
member's bit offset, and the offsets (not the sizes or the anonymous shapes) of the nested fields
the codec reads.

Pros:

- A smaller table and fewer refusal paths than Way 1.
- It catches a moved field the codec uses and a changed total size.

Cons:

- It does not meet the answer. It compares no member size and no anonymous container, and it
  never names a member of `dmu_objset_stats` or `zinject_record`, so every 2.2.2 to 2.4.4
  difference goes uncompared and passes it, whether or not the codec reads one of those members.
  Whether it even accepts a 2.2.2 module needs an assumption the design must state: that a
  member of the approved profile which sits inside the anonymous container may be matched by a
  flattened name, and that a member the module lacks is not checked. Neither is stated today.
- A nested type change passes it when it moves no compared entry. Inside `dmu_objset_stats` or
  `zinject_record` the full comparison refuses such a change and this narrow set does not; inside
  `drr_begin`, `zfs_stat_t` or `zfs_share_t` the full comparison accepts it too, because it
  compares those structs only as members ([Limits](#limits)).

Cost to build and test:

- About 250 to 400 lines of source and 200 to 300 lines of tests with a reader dependency.
- The same reader and loader cost as Way 1, so the saving is small against the loss.

What it needs measured: item 2 and item 16, where the narrow comparison must accept the patched
module that the full comparison refuses, plus items 1 and 10.
That result is why this way is not ranked: it is the cheaper variant the comparison under
[The comparison](#the-comparison) rules out, and the saving is small against the loss.

### Way 3: start an external tool

Spawn `pahole` or `bpftool` at start, parse the type listing it prints, and compare it.

Pros:

- The parser is the tool's: `pahole` is the command the design already names for the test-time
  comparison, and `bpftool` carries libbpf's reader.
- No new Rust dependency in the binding.

Cons:

- The product gains a runtime program it must find, verify and version.
  A missing tool would look like an ABI refusal unless the checker adds its own step for it.
- The root handler would execute an external binary; its path and library resolution become
  part of the privileged boundary the design otherwise keeps small.
- Text output is not a machine interface: `pahole` prints cacheline comments and a human layout,
  `bpftool`'s raw dump needs a type-id resolver built in the wrapper anyway.
- The tool's version drifts with the distribution, and the tool must be present in every
  deployment, including the smallest.

Cost to build and test:

- A spawn wrapper and output parser per tool, about 200 to 300 lines of source and
  250 to 400 lines of tests.
- A packaging and installation requirement in the host and guest fixtures, and a version matrix
  per tool release in the VM.
- A path-verification test for the root handler's invocation.

What it needs measured: items 1, 2 and 10, plus item 13 for the tool's output on a compatible and
a mismatched module and its absence path, and item 14 for the root handler's verified path.

### Way 4: compare a BTF digest

Hash the module BTF (or a selected part of it) and compare with a value baked into the release.

Pros:

- No parser and no per-member table; a single comparison.
- A rebuilt module with any difference at all is refused.

Cons:

- It is the alternative the project did not select: "Exact approved kernel/module BTF profiles were
  the alternative the project did not select, and they would have disabled clones after a rebuild"
  ([ABI verification][design-project-decisions]).
  A digest refuses every rebuild, including one that keeps the approved layout.
- It proves nothing about which field changed, so the refusal cannot name the field.

Cost to build and test:

- A digest table for each approved build and the release step that fills it, about 50 lines of
  source, plus item 15. The digest must be regenerated for every approved build, which is the
  maintenance the answer rejected.

This way is bounded out by the accepted answer. It is listed so the boundary is visible.
What it needs measured: item 15, where a rebuild that keeps the approved layout must be shown
refused, which is the cost the answer rejected. No reason to build it otherwise.

### Way 5: prove the size with a request

Issue one harmless request whose name is known to fail, with a user buffer larger than any
candidate struct, and read the error: a kernel whose struct fits the buffer copies the request in
and out and returns the name error.

Pros:

- No BTF reader at all, and it proves the end-to-end copy on the running kernel.
- Small: one request and one error interpretation.

Cons:

- It proves only that the kernel's struct fits the buffer it was handed and that dispatch
  reached a handler. A moved field, a changed member size, a renumbered request or a kernel
  struct larger than the buffer passes or corrupts, and no error names the difference.
- It needs `/dev/zfs`, so it cannot run where the device is missing, and its request choice must be
  side-effect free on every pool state.
- It overlaps the start probe, which the design asks for separately for kernel releases.

Cost to build and test:

- About 50 lines of source and a device arm in the guest.
- It cannot replace the layout check; it is at most a control beside Way 1.

What it needs measured: item 16, where a moved field of the same size must be shown passing it,
and the buffer bound it can claim. A pass alone says nothing.

This way is bounded out by the gate's order, not only by its own cons. The accepted gate says
"At start, before the first ioctl, both binaries must prove compatibility"
([ABI gate and tests][design-abi-gate]), and Way 5 obtains its proof by sending an ioctl.
The proof itself is smaller than its name claims: the kernel copies a fixed
`sizeof (zfs_cmd_t)` (`module/os/linux/zfs/zfs_ioctl_os.c:139-150`), so a success shows only that
`/dev/zfs` exists and that dispatch reached a handler.
It is listed so the boundary is visible, and it is not ranked.

### Way 6: keep the release string check only

Refuse when the module version is not the approved string, and do not check the layout.

Pros:

- No parser, no BTF dependency, the smallest gate.

Cons:

- It is what the design had before the layout answer, and that answer added the layout check.
- The owner's answer of 2026-11-19 removed even this check, so a release rule is not part of the
  gate at all ([Answers of 2026-11-19](#answers-of-2026-11-19)).
- A distro or patched build that keeps `2.4.4` and changes a field passes, and the kernel copies
  the wrong shape.

Cost to build and test:

- The version read and the release rule, about 20 lines of source, plus a test of the rule.
  It is the gate the design had before the layout answer.

This way is bounded out twice: by the layout answer, which added the check it omits, and by the
owner's answer of 2026-11-19, which removed the release rule it is built on.
What it needs measured: item 16, where a patched module that keeps the version and moves a field
must be shown passing it. No reason to build it otherwise.

### Ranking

Ranking: Way 1 > Way 3. Both prove the items before the first ioctl and compare the declared
layout, so each meets the answer.

- Way 1 beats Way 3 because the proof stays in the wrapper's address space: no runtime program
  for the product to find, verify and version, no root-spawned binary whose path and library
  resolution join the privileged boundary, and no text format to parse back into members.
  Way 3 proves the same facts and pays those costs for them.

The further ways are not ranked, each for a stated reason:

- Way 2 does not meet the answer. It compares no member size, no anonymous container and no
  member of `dmu_objset_stats` or `zinject_record`, so every 2.2.2 to 2.4.4 difference goes
  uncompared, and passes it whenever the flattening assumption it needs holds.
- Way 4 is bounded out by the accepted answer: it would refuse every rebuild.
- Way 5 is bounded out by the gate's order, since it obtains its proof by sending the ioctl the
  gate must precede, and it proves only that the caller's buffer covers the copy.
- Way 6 is the gate the answers replaced: the layout answer added the check it omits, and the
  owner's answer of 2026-11-19 removed the release rule it rests on.

The reader inside Way 1 is the owner's pick;
the ranked reader options are
under [The profile-checking dependency](#the-profile-checking-dependency).

## The shape this proposal designs

This section proposes the shape of Way 1, with the placement the owner answered on 2026-11-19.
The reader is a seam that [Q1](#q1-which-reader-parses-the-module-btf)'s pick fills, and the rest
of the shape stays the same whichever option it picks.

### The seam and the placement

The binding package owns the request format, the `zfs_cmd` representation and the codec
([Binding crate and compatibility][design-binding]).
The owner answered on 2026-11-19 that the same package holds the layout reader,
"Binding package (Recommended)", so the binding owns:

- the bounded file loader and the pre-scan;
- the expectations derived from its `#[repr(C)]` type and its request list;
- the comparison and its refusal value;
- the reader itself, once [Q1](#q1-which-reader-parses-the-module-btf) is settled, or the call
  into the dependency that reader names.

and the wrapper and the handler own:

- the file paths, the read order and the per-process cache;
- the target allowlist (architecture);
- the identity reads the cache compares;
- the words the caller and `doctor` print, including the version string and the sentence printed
  beside it.

Both binaries call the same entry point, as the design requires for reads as well as writes.
The record decides that the binding owns the request format and that both binaries share the
binding and the gate, and both binaries live in one cargo package
([Binding crate and compatibility][design-binding]); rule `OWK` made the remaining placement the
named owner's, and the answer settles it.
The seam stays one function wide: it reads the two files, compares them with the binding's
expectations and returns the refusal value, and the wrapper supplies the paths, the target
allowlist and the wording.
Two of the refusal's steps are the wrapper's to build, not the seam's: the wrapper checks the device
and reads the identity, so it constructs the `Device` and `Identity` values itself, while the binding
holds the one type so both binaries render them alike; the seam never returns those two.
[Answers of 2026-11-19](#answers-of-2026-11-19) records the answer and what it did not take.

The crate exists on `main` since `e260ae257b`, merged as plan step 1 at version `0.1.0-alpha.1`,
with `src/layout`, `src/names`, `src/nvlist` and `src/requests`, the `zfs_cmd_t` mirror and
`Operation::ALL` among them. Step 6 is what edits it, adding the checker's folder, and nothing this
section designs has been written.

The checker's files cannot live inside `layout`. `src/requests/uses/layout` already links those two
folders, so a file in `layout` that read the request list would make the folders use each other,
which rule `LYL` forbids and the repository linter fails
(`package/linter/monochromatic-lint/src/layout_loops.rs`); and `src/layout/mod.rs` states that
nothing in that folder knows what a request means.
So the files live in a sibling source folder of `layout` and `requests`,
`src/compatibility_check/`, which carries `uses/layout` and `uses/requests` links of its own and
lists its files in dependency order in its top doc, as rule `LYF` asks: the refusal value first,
since the loader builds its `Header`, `BaseBtf` and `ModuleBtf` steps, then the bounded loader and
the pre-scan, the expectations and the comparison.
`src/lib.rs` declares it and re-exports its entry point beside `layout`'s, and the crate root gains
a `src/uses/compatibility_check` link, as `src/uses/` already links each folder `src/lib.rs`
declares.
The name says what the folder does, comparing two files, while the plan's word "gate" also covers
the device and identity steps the wrapper holds; the alternative is `compatibility_gate`, and the
name is open to veto, with "veto none" as the default.
Its expectations read `Operation::ALL` and `Operation::code()` from `src/requests/operation.rs`,
which are public, and `ZfsCmd`, `DmuObjsetStats`, `ZinjectRecord`, `ZcInjectOverlay` and
`ZcDestroyControl` from `src/layout/zfs_cmd_t.rs`.

### The read order and the reload race

The gate, for a caller that needs the device:

1.  Check that `/dev/zfs` exists and that the caller can open it.
    A missing or unopenable device refuses with that step and starts no read.
2.  Read the module's identity once: `/sys/module/zfs/version` and `srcversion`, each bound to one
    page and stripped of one trailing newline, and `notes/.note.gnu.build-id`, bound to one page
    and compared as bytes.
    That read is identity, never a rule: no step compares the version text with an approved string,
    and the text is kept for the cache, for the second read and for `doctor` to print
    ([Answers of 2026-11-19](#answers-of-2026-11-19)).
3.  Read the two BTF files, each opened with `O_RDONLY | O_CLOEXEC | O_NOFOLLOW`,
    each read to a bounded size, each checked against its own header and file size.
4.  Read the same identity again. If any of it differs, retry the whole check once; refuse with
    "module changed while checking" only when the retry's own two identity reads also differ.

`doctor` runs the same facts without step 1, so it can report what a caller would meet.

An absent identity attribute is not a refusal. A module built without a build-id note, or one whose
`srcversion` is missing, still reaches the layout comparison, because the answer leaves only the
layout and the request numbers to decide and a missing identity file is not a ground it kept. The
identity reads then compare whichever of the three values the module carries, plus each file's
device, inode, size and modification time, and `doctor` says which attribute is absent so a reader
can tell a weaker reload check from a refused one.
When all three values are absent the two identity reads compare nothing the check trusts: the file
times do not stand in, because this host's `/sys/kernel/btf/zfs` modification time moved from
16:47:51 to 21:39:40
while its `srcversion` and `version` stayed at 19:51:21 and 19:03:56, which is consistent with a
time moving without a reload.
So the check then has no term that could differ between its two identity reads and never refuses,
and only the cache recheck compares the file facts there, keeping the verdict it holds while each
file's device, inode, size and modification time hold ([The cache](#the-cache)); a module replaced
while those hold goes unseen, and `doctor` says so.
A missing `/dev/zfs` or a missing BTF file is a
refusal, as [What the checker must prove](#what-the-checker-must-prove) states.
The two alternatives this paragraph did not take are refusing when no identity value exists, which
would refuse a module the layout comparison can still judge, and parsing the two files again on
every use when none exists, which costs a parse per use only for such a module and would see a file
swapped while the other facts hold.
The rules in this paragraph are open to veto, with "veto none" as the default.

The second identity read closes the window in which the module unloads or is replaced between the
first identity read and the BTF reads. It does not close the window during a single read;
the header and size checks refuse a short or inconsistent file.
A replacement that keeps the version string is what `srcversion` and the build ID narrow. This
host's `/sys/module/zfs` carries `srcversion` (mode `0444`, 4096 bytes,
`A3553C308C29FD4B6595A72`) and `notes/.note.gnu.build-id` (mode `0444`, 36 bytes, a 20-byte GNU
build ID that plain `ls` hides and `ls --all` shows), and the account reads both. Across the two
reads, `srcversion` moves when a same-version module is built from other source, and the build ID
moves when one is recompiled even from identical source. A replacement that keeps the version,
`srcversion` and the build ID is still invisible to this check.
The retry is what keeps a reload that has already settled from reading as a refusal: only a
second attempt that also sees two different identity reads refuses, and a second attempt that sees
one module passes.
`doctor` runs the same identity reads without the device step and prints the version string beside
its verdict, saying when it is not the text the tested module reported, `2.4.4-1`; that sentence is
the only place the release string reaches a reader ([The refusal](#the-refusal)).

### The bounded pre-scan

Before any parser sees the bytes, the loader checks:

- the magic is `0xeB9F` in one byte order, the version is 1, and the flags are zero;
- `hdr_len` is at least the 24 bytes of the oldest header and at most the read bound;
- the type, string and, if present, layout sections, each offset counted from the end of the
  header (`hdr_len`, so the header's own bytes are in no section), sorted by offset, are
  contiguous with no gap and no overlap, and the last of them ends at the read length. That is
  the order `v7.3` accepts (`kernel/bpf/btf.c:5697-5735`, where `expected_total` is the read
  length less `hdr_len`), and this host's vmlinux is laid out under it as type, layout, strings;
- with a 24-byte header the string section ends the file, which `v6.8`'s checker requires
  unconditionally (`kernel/bpf/btf.c:5160-5164`) and `v7.3` still requires below the 32-byte
  header (`:5625-5629`);
- every type entry's length is computable with the widest encoding
  (24-bit `vlen`, 7-bit kind), and any value the chosen reader cannot represent is a refusal,
  not a misread.

The last check matters for the reader chosen at `2.0.1`: its released masks are the older
5-bit and 16-bit ones, so a future file that uses a wider value must be refused before it is
parsed. The check is a linear walk that skips each type by its entry length, not a second parser.

### The expectations

The expectations come from the binding's own type, so one change moves both sides:

- `size_of::<ZfsCmd>()`, and `offset_of!` for every field the Rust type can name;
- the union and anonymous members, whose Rust side has named fields and whose BTF side has an
  empty name: a path step names the kind and the expected bit offset, then the inner names;
- every member of `dmu_objset_stats` and `zinject_record`, with its offset and size, not only
  the members the codec reads, and the two types' own sizes (288 and 368), as listed under
  [What the BTF carries](#what-the-btf-carries);
- the base integer sizes the members use;
- every request constant in the binding's request list, which is `Operation::ALL` in
  `src/requests/operation.rs`, each with the value `Operation::code()` gives it, plus
  `ZFS_IOC_FIRST` and `ZFS_IOC_LAST`.

A path grammar, as it would live in the binding:

```text
# package/zfs/request-to-openzfs-kernel-device/src/compatibility_check/expectations.rs
zfs_cmd
  zc_name              [0, 4096]          # [bits from outer start, bytes]
  zc_nvlist_src        [32768, 8]
  ...
  <anon-union @ 106368>
    zc_inject_record   [106368, 368]
    <anon-struct @ 106368>
      zc_pad1          [106368, 352]
      zc_defer_destroy [109184, 4]
      zc_flags         [109216, 4]
      zc_action_handle [109248, 8]
  ...
```

The numbers in the table are bits from the start of the outer struct, with a byte size beside
each entry. BTF gives a member's offset relative to its container, so the comparison sums the
container offsets along the path: `zc_defer_destroy` is 2,816 bits into the union at 106,368,
which is 109,184 bits from the struct's start. A hand-written table of literals would drift;
the table is generated from the Rust type at compile time where the language allows it,
and the few members it cannot express (the anonymous containers and union members) carry a
literal with a test that recomputes them from the type's size and the neighbours.

An exhaustiveness test fails when the struct gains a field with no path, or the request list gains
a constant with no expected value, so a new field cannot silently go unchecked.

### The comparison

For each expected entry, resolve the BTF type and compare:

- the struct's size with `size_of::<ZfsCmd>()`, in bytes;
- each member's bit offset with the expected bit offset, and the member type's size with the
  expected byte size;
- an anonymous container by its kind and bit offset, then its inner members by name and offset;
- each base type size the member types resolve to;
- each request constant with its `zfs_ioc` enumerator, by name and value;
- `ZFS_IOC_FIRST` and `ZFS_IOC_LAST`.

The comparison stops at the first mismatch and reports its path, expected and actual values.
It also reports a missing type, a missing enumerator, and a member the expectations do not cover
when the table's count check fails.

The walk has a fixed order, so each arm under [Owed measurements](#owed-measurements) can assert
which finding comes first:

1.  The type's own size, before any of its entries.
2.  One pass over that type's entries in bit-offset order. Each expected entry is checked for
    kind, name, bit offset and member type size. A named entry is matched by name; an anonymous
    container is matched by kind and bit offset, since BTF gives it no name. A nested container is
    checked by its kind and bit offset in this pass, and its interior is not entered yet. Entries
    that share an offset are checked in the order the expectation table lists them.
3.  The member count check against the expectation, after that pass.
4.  Only if the pass and the count match, descent into the interiors of the types the compared set
    covers, in the order their containers stood in the parent, each interior repeating this order.

An entry that expected a container and met a named member, or the reverse, is a `Type` mismatch.
A missing, extra or misplaced member is `Member`, and a member whose type size differs is
`Member` with both sizes.

The 2.2.2 to 2.4.4 differences therefore land at these steps:

- A 2.2.2 module has no anonymous union where the expectations place one. Its outer pass reaches
  the container at 106,368 with every entry at a lower offset matching, so its first refusal is
  `Type`, naming the member found there, `zc_inject_record`, where the anonymous union was expected.
  The `dds_flags` refusal is a `Member` finding in the interior pass, which this module's outer
  pass never reaches.
- `zinject_record_t`'s size is the difference no offset shows: 352 against 368 bytes, with
  `zc_defer_destroy` at 13,648 in both. Only a member type-size comparison refuses it, and that
  refusal comes in the union's interior pass, after `dmu_objset_stats`'s, on the union's
  `zc_inject_record` entry. It is reachable only for a module that carries the anonymous union at
  all; a 2.2.2 module refuses earlier, at `Type` in its outer pass.
- `dds_flags` is the difference no size or offset of any other member shows: it is one byte at
  2,296 bits, filling the byte that was tail padding, and `dmu_objset_stats` stays 288 bytes.
  Only an expectation that names `dds_flags` at all refuses it, in the interior pass of
  `dmu_objset_stats`.
- The request contract refuses none of them: every request the binding sends and both
  endpoints hold the same value in 2.2.2, so the layout comparison is what carries the refusal.

A checker that compares the total size and the offsets only, and that flattens a member found
inside the module's anonymous container to the name the approved profile expects, therefore
accepts the 2.2.2 shape ([the vet][vet], lines 430-446, which names the union and the grown record
but not `dds_flags`). Without that flattening it refuses at the container instead, so on either
reading Way 2 must state the assumption it takes.

### The refusal

One value carries the outcome, with the step that failed, the file it read, and the detail:

```text
# package/zfs/request-to-openzfs-kernel-device/src/compatibility_check/refusal.rs
Refused {
  step: Device | Identity | BaseBtf | ModuleBtf | Header | Type | Member | Enum | BaseType,
  file: Option<PathBuf>,
  detail: String,            # the field path, the expected and actual values, or the read error
}
```

`Identity` is the step that says "module changed while checking", and it is the only step the
version text takes part in, as one of the three values the two identity reads compare.
No step is named for a release rule, because the gate has none
([Answers of 2026-11-19](#answers-of-2026-11-19)).
`Device` and `Identity` are built by the wrapper, which owns those two reads; the seam returns
neither of them ([The seam and the placement](#the-seam-and-the-placement)).

The caller prints the loaded module's version string, the target architecture, the files it read
and the failing step, and takes the design's fallback for an unused slot.
It never offers a bypass. `doctor` prints the same facts, with the healthy case as an explicit
"compatible" line naming the module, the release string it read, the sentence that says when that
string is not the text the tested module reported, `2.4.4-1`, and the file sizes, and does not
change its exit status.
Its refused case carries the same release string and sentence beside the failing step, so a reader
can tell a module the design never tested from one whose layout differs.
The release sentence is reporting, not a rule, and it costs the reader a sentence, never a refusal.
`doctor` compares the string it read with the exact text the tested module reports, `2.4.4-1`
([The module release string](#the-module-release-string)), and prints the sentence whenever the two
differ, so a distribution's `2.2.2-0ubuntu9.5` and a checkout build's `2.4.4-1_g1a2b3c4` both get it.
The sentence quotes both strings, the one the module reports and the one the tested module reported,
`2.4.4-1`, so a reader can tell a module reporting `2.4.4-2` or a Debian-style
`2.4.4-1~bpo12+1` from the build the design tested:
the same version with a different text is reported as a different text, not as a different version.
That comparison is the writer's, open to veto, with "veto none" as the default: the answer says
only "says when it is not the tested `2.4.4`" and records no matching rule, and a prefix test on
`2.4.4-` would silently pass the checkout build `2.4.4-1_g1a2b3c4`, which is the build the sentence
exists to flag.

### The cache

The result is cached per process in a `OnceLock`, following
`git_version.rs` (`package/git-policy/cli/src/native/invocation/git_version.rs:23-60`).
Before each use, the cache's identity is rechecked against those three identity values and the two
files' identity (device, inode, size and modification time); a change re-runs the check.
A sysfs modification time does not track a module load on this host (boot 11:12:11,
`/sys/module/zfs/version` 19:03:56, `srcversion` 19:51:21), so
the recheck can re-run a check that would have passed. The cost is a needless parse, and a stale
pass only where the cache has nothing but each file's device, inode, size and modification time to
compare, which is the all-absent case
[The read order and the reload race](#the-read-order-and-the-reload-race) states: there a
replacement that keeps those four facts is reused.
The BTF file is the clearest case: `/sys/kernel/btf/zfs` read 16:47:51 in the revision at
`fa66afaed0`, which first recorded it, and 21:39:40 when this revision read it, while `version` and
`srcversion` read the same times and values as before, which is consistent with the time moving
without a reload.
That is an inference from two readings and not a measurement of a reload.
Where the file times come from is unverified, so they are no substitute for an identity value:
when all three values are absent the reload check has no usable term, as
[the read order and the reload race](#the-read-order-and-the-reload-race) states.
Whether a reload changes the inode number is unverified, and
[item 9's second arm](#owed-measurements) is the one that tests a cache spanning a reload.
The per-process `OnceLock` and its identity recheck are the writer's rule, open to veto, with
"veto none" as the default; the alternative is to parse the two files on every use. The disk cache
is not this design's to settle: "no disk cache" is
[Q3](#q3-may-a-verification-be-cached-across-processes)'s option 1, so the project decides it when
owed measurement item 10 has run, and this design writes no verdict to disk meanwhile.

The wrapper already writes one disk cache: a private per-user file holding the text `git version`
prints, keyed by the executable's canonical path, size, modification time, change time, device and
inode (`package/git-policy/cli/src/native/invocation/git_version_cache.rs:1-30`).
That precedent does not carry over. The cached `git version` text is descriptive, and a wrong
entry changes a warning; a compatibility verdict is the gate that enables raw requests, so a
forged or stale entry in the account's own cache would authorize requests on a module whose layout
nothing compared. That is a different trust class, so this design writes no verdict to disk.
[Owed measurements](#owed-measurements) item 10 measures the cost, and
[Q3](#q3-may-a-verification-be-cached-across-processes) is held until it runs.

### What the shape does not cover

- The nvlist keys and the handler's behaviour.
- A patched module with the approved layout.
- A base BTF whose primitive sizes are checked but whose other types the checker does not read.
- A file the kernel exposes that is internally inconsistent in a way its own load check accepted;
  the pre-scan refuses only what it can see.

## The profile-checking dependency

This section follows
[`choosing-technology`][skill]:
hard constraints, a candidate ledger with a source audit per candidate, hard-gate outcomes,
a scored comparison of the finalists, and a sensitivity pass.
The pick is the owner's, so this section ends with ranked options and no choice.

The skill's report artifact (`doc/audit/tech-...-vet-YYYY-MM-DD.md`) and its
pre-discovery check-in are not done, because this brief permits edits to this file only and
allows no questions. Both are owed before adoption; see
[The vet artifacts this brief could not write](#the-vet-artifacts-this-brief-could-not-write).

### Method and hard constraints

Base category for every candidate: inspectable open-source local technology.
Overlays: high-trust execution (the root handler runs the same check),
native boundary (a parser of kernel-produced bytes), and multi-platform claim
(the wrapper's two GNU release targets).

Hard constraints, each with the design sentence it comes from:

- H1. Parses split BTF from a module file with its vmlinux base, as
  [A module BTF needs the vmlinux base](#a-module-btf-needs-the-vmlinux-base) shows.
  The design's start-time layout proof needs module types whose ids and names cross into the base.
- H2. License compatible with `LGPL-3.0-or-later` (the wrapper's license;
  the binding is a new crate in the same repository).
- H3. Builds for `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu` with no C build step
  and no load-time library from the distribution, because the wrapper's release is two GNU
  archives that must start on a machine without OpenZFS
  (`toolchain.lock.json:614-653`;
  [the vet][vet], "Where the wrapper ships and what it links today").
- H4. Reads a file the caller names, with no BPF syscall, no root requirement and no network.
- H5. Exposes named struct members with their bit offsets and type sizes, and enum members with
  their values, at the depth the anonymous union needs.
- H6. Source is inspectable and its provenance maps to the published artifact.

This screening applies H1 to H6 before any score. H5 is the reason a low-level `object`-style
ELF reader is not a candidate by itself: it exposes a section's bytes, not BTF types.
H3 constrains what the binding builds and links, and H2 what it may include; a spawned tool adds
no link, so the tools meet H3 and carry the runtime-program cost in C2 instead. Dropping H3 would
re-admit `libbpf-rs` and a `dlopen` of `libbpf`, which is a choice for the owner, not a fact.

### Candidate ledger

#### btf-rs 2.0.1

- Discovery: crates.io query `btf`, first result by recent downloads among parser crates.
- Version and revision: 2.0.1, tag `v2.0.1`, commit `03fd01bd9505d0888b568cd3aaa9b232b7e3fb20`,
  published 2026-11-12. License `LGPL-2.1-or-later`.
  The crates.io archive's SHA-256 is
  `fa34ec915ede6b7eaeebcb38fdb749fcac1538ec1b94dcfde63a034ad8f7963b`,
  which is the index entry's `cksum`.
- Dependencies: `btf-rs-derive`, `byteorder`, `fallible-iterator`, `memmap2`, `thiserror`;
  optional `elf`, `regex`, `bzip2`, `flate2`, `liblzma` and `zstd` for features this use does not
  enable. The default feature set is empty. No C build.
- Source audit: 2,749 lines under `btf-rs/src`.
  Split support is explicit: `Btf::from_split_file` and `Btf::from_split_bytes`
  (`btf-rs/src/btf.rs:66-107`) take a base and reject a base that is itself split.
  The cached backend keys module strings past the base's string length and numbers types from the
  base's count (`btf-rs/src/section.rs:190-265`), and name lookup falls back to the base
  (`btf-rs/src/btf.rs:190-200`).
  The mmap backend is lazy and checks the string section against the file length
  (`btf-rs/src/section.rs:348-390`), but its `iter_over_names` compares an offset against the
  module's own string length (`:405-417`), which is wrong for a split file; split files do not use
  that backend, because both split constructors use `from_reader`.
  Kinds cover `ENUM64`, `DECL_TAG` and `TYPE_TAG` (`btf-rs/src/btf.rs:303-355`), and members carry
  bit offsets and bitfield sizes (`:583-598`).
  The header reader is 24 bytes and section offsets use `hdr_len`
  (`btf-rs/src/cbtf.rs:182-191`; `section.rs:182,206`), so a 32-byte header parses.
- Tests and CI: an integration test with checked-in BTF assets
  (`btf-rs/tests/integration_test.rs`), and one CI workflow running format, Clippy,
  documentation, six feature builds and the tests (`.github/workflows/ci.yaml:1-40`).
  The published archive ships no tests:
  its manifest sets `autotests = false`,
  and its `include` list holds only `examples/`,
  `src/` and `LICENSE`.
  The suite therefore ran from the repository at tag `v2.0.1`.
  On 2026-11-20,
  168 integration tests and four doctests passed under `--all-features`,
  and 64 integration tests passed under the default features
  ([Finalist validation](#finalist-validation)).
  No fuzz target.
- Maintenance: releases `v2.0.0` (2026-04-16) and `v2.0.1` (2026-11-12);
  commits through 2026-10-29; one merged fix for an out-of-bounds mmap name read
  (PR 42, merged 2026-10-29, included in 2.0.1); two open items, PR 37
  "Add kind layout support" and issue 1 "Add some BTF building capabilities";
  4 forks, a small maintainer group.
  The 2.0.1 tree lacks the widened `vlen` and kind masks that the default branch carries at
  `cdf9ea46`; the branches diverged, and the release lags its own main on that point.
- Probe: parses this host's pair, resolves `zfs_cmd` by name, walks the anonymous union to
  `zc_defer_destroy`, and returns 104 `zfs_ioc` values; refuses truncated inputs with an error.
- Gates: H1 pass (split constructors plus the probe), H2 pass (LGPL-2.1-or-later can be used
  under LGPL-3.0; the wrapper is already LGPL-3.0-or-later), H3 pass on source (pure Rust, default
  features off; the aarch64 build is unrun), H4 pass, H5 pass, H6 pass.
- Validation:
  the library parsed this host's pair through `from_bytes` and `from_split_bytes`,
  with every compared member and enumerator asserted and four negative controls refused
  ([Finalist validation](#finalist-validation)).
  The `aarch64` target remains unrun.
- Finalist.

#### ebeepf 0.1.0

- Discovery:
  class 3 query 3 of [the round of 2026-11-20](#the-round-of-2026-11-20),
  whose 49-row page named it.
- Version and revision:
  0.1.0,
  published 2026-10-06,
  at commit `5de05465bd9445d89b13a967e198d0aeffd71ec4`.
  That commit is the repository's `HEAD` and the one `.cargo_vcs_info.json` names.
  The license is `LGPL-2.1-only OR BSD-2-Clause`,
  read from the version's own metadata,
  and a dual-licensed crate may be taken under its `BSD-2-Clause` arm.
  The archive's SHA-256 is
  `f5f4162021ff5dce1b6c5a657267523d7d41c6818bae2adc0adbfbf4b5b6fa96`,
  which is the version entry's `cksum`.
- Dependencies:
  `bitflags`,
  `flate2`,
  `goblin`,
  `libc`,
  `object`,
  `sha2` and
  `thiserror`,
  with `bpfcov` optional behind the `coverage` feature.
  No C build runs for this use,
  and the default feature set is empty.
- Source audit:
  1,926 lines in `src/btf.rs`.
  Split support is explicit:
  `Btf::parse_split` and the private `parse_with_base` clone the base's type and string tables,
  number the module's types from the base's count,
  and refuse a byte order that differs from the base's
  (`src/btf.rs:403-475`).
  Members carry a bit offset and a bit-field width,
  and enumerators carry a value
  (`src/btf.rs:94-121`).
  `size_of` walks typedef and qualifier chains and multiplies array elements
  (`src/btf.rs:576-720`),
  so the binding needs no array-size shim.
  The header reader accepts any `hdr_len` of 24 or more and places both sections from it
  (`src/btf.rs:438-450`),
  so a 32-byte header parses.
  Type and string sections are bounds-checked,
  every offset goes through `checked_add`,
  and `validate_references` runs before a table is returned
  (`src/btf.rs:441-475`).
  Two residuals remain.
  The kind and member-count masks are the 5-bit and 16-bit fields the `btf-rs` release also uses
  (`src/btf.rs:1430-1431`).
  A table passed as the base of another split table is not refused,
  because `parse_split` compares only the byte order
  (`src/btf.rs:411-419`),
  and the consumer probe observed exactly that.
- Tests and CI:
  102 unit tests in the library,
  of which 7 are BTF tests,
  plus `tests/kernel_smoke.rs` and one doctest.
  The archive ships all of them.
  The repository has no CI workflow,
  and `gh api repos/kxxt/ebeepf/contents/.github` answers 404.
  No fuzz target.
  On 2026-11-20 the archive's own `cargo test` passed:
  102 unit tests,
  `kernel_smoke` with 1 passed and 22 ignored,
  and 1 doctest.
  `cargo fmt --check` passed,
  and so did `cargo clippy --lib` with `-D warnings`.
  The suite needs `clang` for its BPF fixtures:
  without it 94 unit tests pass and 8 fail at `src/test_bpf.rs:53`,
  so the run used an image that has clang.
- Maintenance:
  one release,
  2026-10-06,
  one maintainer,
  0 stars,
  0 forks,
  0 open issues,
  no CI,
  no commit after the release,
  and a README whose first line reads "This repo contains AI-generated code that has not been
  thoroughly reviewed".
- Probe:
  [Finalist validation](#finalist-validation) records 195 of 196 design checks passing on copies
  of this host's pair.
  The one failure is the unrefused split base above.
- Gates:
  H1 pass on the split constructors and the probe;
  H2 pass on the `BSD-2-Clause` arm;
  H3 pass on source,
  pure Rust with the default features off,
  while the `aarch64` build is unrun;
  H4 pass,
  since `parse` and `parse_split` take bytes the caller read and `from_path` reads one named
  file;
  H5 pass on member bit offsets,
  bit-field widths and enumerator values;
  H6 pass on the archive's checksum and vcs information.
- Finalist.

#### btfparse 1.3.9

- Discovery: crates.io query `btf`.
- Version and revision: 1.3.9, commit `3788c4410a6cfd64ff57aa5bca20b2ee4eed3cd5`, 2026-01-08.
  License `Apache-2.0`.
- Dependencies: none. Edition 2024. 5,876 lines under `src/`.
- Source audit: one `TypeInformation::new` over one `Readable`, numbering types from 1
  (`src/btf/type_information.rs:435-490`); no base file, no split id or string offset.
  Its `id_of` and `offset_of` API is the closest to what the check wants
  (`:491-520`, `:607-761`), but it cannot represent a module file whose ids and names cross into
  the base.
- Tests and CI: unit tests embedded in each type module, with byte fixtures; five workflows
  (`check`, `clippy`, `format`, `main`, `posix_build`); no fuzz target.
- Gates: H1 fail. The rest were not scored.
- Exit.

#### aya-obj 0.6.0

- Discovery: crates.io query `btf`, by downloads.
- Version and revision: 0.6.0, `aya-rs/aya` at `44a6709164d2ac0c19c7845101e7b033e25b3f3f`.
  License `MIT OR Apache-2.0`.
- Dependencies: `bytes`, `log`, `object` (ELF), `thiserror`.
- Source audit: the BTF reader parses one object's `.BTF` section and has no base
  (`aya-obj/src/btf/btf.rs:340-365`; the `base` at `:355` is a byte offset, not a base BTF).
  It is written for eBPF object loading and relocations.
- Gates: H1 fail. Its `object` dependency is a fit concern, not a gate. Exit.

#### btf 0.5.1

- Discovery: crates.io query `btf`.
- Version and revision: 0.5.1, `arcjustin/btf` at `3b57cdab2ea80d13c3b5abd400bb2e4bfc0d6292`,
  last commit 2023-02-08. License `MIT`.
- Dependencies: `byteorder`, `thiserror` 1.
- Source audit: one blob parser with a 24-byte header and no base
  (`src/btf.rs:15-81`); no split support.
- Maintenance: no commit since 2023.
- Gates: H1 fail; the maintenance signal is noted but the gate is enough. Exit.

#### btfdump 0.0.5

- Discovery: crates.io query `btf`.
- Version and revision: 0.0.5, `anakryiko/btfdump` at `e5951d4b2e2e7ce6c36001c1b48c1c239c9759dc`.
  License `BSD-2-Clause`.
- Dependencies: `clap`, `object`, `memmap2`, `scroll`, `scroll_derive`, `regex`, `bitflags`.
- Source audit: a tool first, a library second; it dumps one BTF file and has no split base
  (`src/types.rs:1089-1120`, `src/lib.rs`). Its README names only an ELF file or a raw BTF file,
  not a module pair.
- Gates: H1 fail as a library; as a tool it is Way 3 with an extra package. Exit.

#### padlock-dwarf 0.82.0

- Discovery: crates.io query `btf`.
- Version and revision: 0.82.0, `gidotencate/padlock` at `6a7d045f2c6b0cb64549ddebe5825411e25178fa`.
  License `MIT OR Apache-2.0`.
- Dependencies: `padlock-core`, `gimli`, `object`, `pdb`, `miniz_oxide`.
- Source audit: a 794-line BTF reader inside a DWARF/PDB struct-layout analyzer,
  single blob, no base (`crates/padlock-dwarf/src/btf.rs:114-152`).
  The analyzer's output is a whole-program struct list, not the named type model the check needs.
- Gates: H1 fail; H5 fail for fit. Exit.

#### libbpf-rs 0.27.2 and libbpf through FFI

- Discovery: crates.io query `libbpf`, and repository precedent.
- Version and revision: 0.27.2, `libbpf/libbpf-rs` at
  `6d771ee9835fcf517dbcf1d29e25a97480425d45`. License `LGPL-2.1-only OR BSD-2-Clause`, read from
  the crates.io metadata for 0.27.2; its `libbpf-sys` dependency is `BSD-2-Clause`.
- Source audit: the safe wrapper around `libbpf-sys`; its default feature is
  `libbpf-sys/vendored-libbpf`, and the manifest says it "requires other necessary libs to be
  present on the system" (`libbpf-rs/Cargo.toml:20-26`).
- Gates: H2 pass, because a dual-licensed crate may be taken under its `BSD-2-Clause` arm, which
  is compatible with the wrapper's `LGPL-3.0-or-later`. H3 fail: the default feature runs
  libbpf's C build and needs the system's libraries.
  A `dlopen` of the system's `libbpf` avoids the link but keeps the native provenance and adds a
  privileged library load in the handler, so H3 and H6 fail there too. Exit on H3.

#### bpftool 7.8.0

- Discovery: this host (`/usr/bin/bpftool`), and the libbpf reader it carries.
- Version: 7.8.0 on libbpf 1.8; license of the tool is the kernel's (GPL-2.0 with BSD-2 parts),
  and it runs as a separate process.
- Source audit: the tool's `btf dump` reads one `BTF_SRC` with an optional `--base-btf`,
  and formats `raw` or `c` (`bpftool btf help dump`).
  Probe: on the sysfs path it resolves the base itself; on a copied file it needs `--base-btf`.
- Gates: H1 pass, H2 pass for a separate process, H3 not applicable as a library but the product
  needs the program at run time, H4 pass, H5 pass, H6 pass.
- Validation:
  the harness ran the tool on copies of this host's pair with `--base-btf`,
  checked every compared member and enumerator,
  and refused the module copy without its base
  ([Finalist validation](#finalist-validation)).
- Finalist as a Way 3 tool.

#### pahole 1.32

- Discovery: this host, and the design's test-time inspection command
  ("ABI gate and tests").
- Version: 1.32; license GPL-2.0 for the dwarves package, run as a separate process.
- Source audit: the documented `--btf_base` option and `-C` type printer
  (`pahole --help`); probe output for `zfs_cmd`, `zfs_ioc` and `dmu_objset_stats`.
  On a copied module it needs `--btf_base`; on the sysfs path it finds the base.
- Gates: H1 pass, H2 pass as a separate process, H3 as for `bpftool`, H4 pass, H5 pass
  (the printer gives member offsets and sizes), H6 pass.
- Validation:
  the harness ran the tool on copies of this host's pair with `--btf_base`,
  checked every compared member and enumerator,
  and refused the module copy without its base
  ([Finalist validation](#finalist-validation)).
- Finalist as a Way 3 tool.

#### Writing the reader in-house

- Discovery: repository precedent ("Our own small binding", the binding's own unsafe boundary),
  and the skill's existing-tools rule.
- Source: none; the model and the walker would follow `include/uapi/linux/btf.h` and the
  kernel's validation (`kernel/bpf/btf.c:5195-5325`).
- License: the repository's.
- Gates: H1 to H6 pass by construction, but the skill's rule is that
  "A custom implementation is eligible only when every existing tool fails a named hard constraint."
  Every existing candidate passes every hard gate, so this reader is eligible only if the owner
  names one hard constraint they all fail at once; refusing a copyleft dependency fails only
  `btf-rs`, since `pahole` and `bpftool` are separate programs.
- Conditional candidate, outside the eligible ranking, priced in the scored comparison so the record
  shows what the route would have cost; ineligible since the answer of 2026-11-19, and not a finalist
  while the existing candidates pass every hard constraint.

#### Not candidates

- `zfsdev-rs`'s layout code: it hard-codes a layout and does not parse BTF, so it cannot serve a
  runtime check. The design already reuses its codec and layout code for the binding.
- `object`, `goblin`, `scroll` and `libelf`: they expose an ELF section or a byte stream, not BTF
  types, so they are building blocks of the in-house reader, not candidates.
- `trailofbits/btfparse` (C++): discovery found it; a C++ build fails H3.

### Discovery record

The skill asks for a frozen query schedule with a page rule and, for each query, the literal
query, its provider, its filters, its sort order, its page or cursor, its result count, the
candidates it added and the survivors it added, then one terminal result. The schedule was frozen
in two rounds: the initial round is the first version's queries, run between
2026-11-19T18:10:30-05:00 and 2026-11-19T18:53:37-05:00, and one expansion round of the taxonomy
terms those found, run between 2026-11-19T19:30:00-05:00 and 2026-11-19T19:38:00-05:00, after
which the schedule was frozen. The page rule is `per_page=100`, the provider's maximum, for every
registry and repository query, first page only; each count in the schedule is the provider's own
total, so that page held every row and no cursor was needed. The first version read 50 of 81 rows at
`per_page=50`, which is a provider cap, so this revision completes every registry query at the
maximum page size and reads every row. No query sent an exclusion filter, and no result was
truncated after the page rule.

Frozen schedule. Ordering is the provider's default in every query, with no `sort` parameter sent.
"None new" means the query added no candidate and no survivor beyond the ledger.

Initial round:

1.  crates.io `q=btf`, no filter, first page of one at `per_page=100`: 81 rows of 81. New
    candidates: `btf`, `btf-rs`, `btfparse`, `btfdump`, `aya-obj`, `padlock-dwarf`. New
    survivors: `btf-rs`, `btfparse`, `btfdump`, `aya-obj`. All 81 rows are screened in the
    listing under class 1.
2.  crates.io `q=split btf`, no filter, first page of one: 17 rows of 17, every one also in the
    `q=btf` listing. None new; the only BTF parser among them is `btf-rs`, and the rest are
    `aya-obj`'s forks and eBPF tooling.
3.  crates.io `q=btf kernel`, no filter, first page of one: 50 rows of 50, every one also in the
    `q=btf` listing. None new.
4.  GitHub repository search `BTF parser in:name,description language:rust`, filter
    `language:rust`, first page of one: 1 row, `retis-org/btf-rs`. None new.
5.  GitHub code search `from_split_bytes language:rust`, filter `language:rust`, first page of
    one: 25 rows over 6 repositories. None new; the rows are under class 2.
6.  Web search, provider's own ranking, one call: under class 3.

Expansion round:

7.  crates.io `q=libbpf`, no filter, first page of one: 91 rows of 91. None new: the BTF-related
    rows are `libbpf-rs`, `libbpf-sys`, `aya-obj` and its forks, and the rest are eBPF loaders,
    bindings, schedulers, profilers and tools, with no pure-Rust BTF parser by description and no
    crate naming a split base.
8.  GitHub repository search `BTF parser in:name,description`, no `language:` filter, first page
    of one: 7 rows, listed under class 2. None new.
9.  GitHub repository search `btf in:name,description language:rust`, filter `language:rust`,
    first page of one: 37 rows, all 37 listed under class 2. None new.
10. GitHub repository search `topic:btf language:rust`, filters `topic:btf` and `language:rust`,
    first page of one: 3 rows, under class 2. None new.
11. GitHub repository search `topic:btf-parser`, filter `topic:btf-parser`, first page of one:
    0 rows. None new.
12. GitHub releases for each audited repository, and the organization project boards, both under
    class 2. None new.

The tagging comes from a crate's or repository's name and description only, except where a source
audit exists in [Candidate ledger](#candidate-ledger).

- Class 1, the crates.io registry. The `q=btf` query, all 81 rows:

```text
# crates.io q=btf, per_page=100, 81 of 81 rows, read 2026-11-19T19:31:00-05:00
1. btf 0.5.1 (parser, H1 fail)
2. btf-rs 2.0.1 (parser, finalist)
3. ebeepf 0.1.0 (eBPF loader, reads BTF as an input)
4. corelox-build 0.1.0 (CO-RE relocations, reads BTF as an input)
5. btf-derive 0.3.2 (macros for the btf crate, not a parser)
6. btfparse 1.3.9 (parser, H1 fail)
7. btfdump 0.0.5 (parser and tool, H1 fail as a library)
8. btf-rs-derive 2.0.1 (macros for btf-rs, not a parser)
9. breda-btf-data-gen 0.0.0 (reserved)
10. breda-btf-generator 0.0.0 (reserved)
11. libarena-rs 0.3.4 (BPF arena allocator, not a parser)
12. aya-obj 0.6.0 (parser with no base, H1 fail)
13. profile-bee-aya-obj 0.2.2 (fork of aya-obj, same H1 fail)
14. zejtron 80.2.2 (host evidence ledger that reads BTF, a tool)
15. aya-friday-obj 0.2.2 (fork of aya-obj, same H1 fail)
16. aya-obj-nightly 0.0.0 (same as aya-obj)
17. bpf-linker 0.13.0 (BPF linker, reads BTF as an input)
18. btf2wit 0.1.2 (prints BTF as wit, a tool)
19. ntpsec-rs-xdp 0.3.50 (XDP program, not a parser)
20. whytime 0.1.2 (timing profiler, not a parser)
21. padlock-dwarf 0.82.0 (parser in a DWARF analyzer, H1 fail)
22. aa-ebpf 0.0.1-rc.7 (kernel monitoring, not a parser)
23. orb8-proto 0.0.6 (protocol definitions, not a parser)
24. orb8-common 0.0.6 (observability toolkit, not a parser)
25. orb8-cli 0.0.6 (observability toolkit, not a parser)
26. fsblame 0.1.1 (file-change attribution, not a parser)
27. mquire 1.4.1 (memory forensics that reads BTF, a tool)
28. orb8 0.0.6 (observability toolkit, not a parser)
29. orb8-agent 0.0.6 (observability toolkit, not a parser)
30. sipnab-bpf-types 0.1.3 (record layout shared with a BPF program, not a parser)
31. suitesparse_sys 0.1.4 (sparse matrix bindings, not a parser)
32. sipnab 0.5.267 (SIP capture, not a parser)
33. redbpf 2.3.0 (eBPF runtime, reads BTF as an input)
34. rustnet-host 0.6.0 (socket inventory, not a parser)
35. ghostscope-platform 0.1.5 (kernel and BTF quirks behind a debugger, not a parser)
36. linprov-common 0.2.0 (types for linprov, not a parser)
37. linprov 0.4.0 (file policy, not a parser)
38. mem4n6 0.2.0 (memory forensics that reads BTF, a tool)
39. aya 0.16.0 (eBPF library, reads BTF as an input)
40. rvf-types 0.3.1 (vector format types, not a parser)
41. memf-symbols 0.2.4 (symbol backends that read BTF, a tool)
42. procinsh 0.1.20 (process inspector, not a parser)
43. profile-bee-aya 0.13.2 (fork of aya)
44. bioformats 0.1.8 (image format translation, not a parser)
45. bpf-script 0.5.4 (eBPF scripting, reads BTF as an input)
46. aya-friday 0.13.2 (fork of aya)
47. scx_pandemonium 5.23.1 (sched_ext scheduler, not a parser)
48. bpf-api 0.3.1 (eBPF bindings, reads BTF as an input)
49. assayist-capture-resident 0.1.3 (residency capture, not a parser)
50. aya-nightly 0.0.0 (eBPF library, reads BTF as an input)
51. rvf-runtime 0.5.6 (vector format runtime, not a parser)
52. prevail 0.2.6 (eBPF verifier, reads BTF as an input)
53. zelynic 80.4.0 (rate limiter, not a parser)
54. zylaxion 80.2.3 (keyboard sounds, not a parser)
55. ltop 0.2.13 (network TUI, not a parser)
56. padlock-cli 0.82.0 (the padlock analyzer command, not a parser)
57. ebpf-ifc-engine 0.1.8 (eBPF policy engine, not a parser)
58. nalgebra_block_triangularization 0.1.0 (matrix decomposition, not a parser)
59. actplane-ifc-compiler 0.1.8 (eBPF policy compiler, not a parser)
60. bpfix 0.1.15 (verifier diagnostics, reads BTF as an input)
61. tracekit-wire 0.1.0 (kernel and user ABI types, not a parser)
62. actplane-runtime 0.1.8 (eBPF policy engine, not a parser)
63. actplane 0.1.8 (eBPF policy engine, not a parser)
64. wits 2.2.3 (profiler that reads BTF, a tool)
65. bpfdoctor 0.1.0 (BPF doctor, reads BTF as an input)
66. procctx 0.1.0 (process context for eBPF tools, not a parser)
67. bpf-script-derive 0.5.4 (macros for bpf-script)
68. scx_cake 1.2.2 (sched_ext scheduler, not a parser)
69. iodyne 0.9.0 (disk IO TUI, not a parser)
70. arena-linker 0.2.0 (BPF linker, not a parser)
71. compendium 0.2.1 (strace viewer, not a parser)
72. ebpfsieve 0.1.6 (eBPF offload, not a parser)
73. electric-eyed 0.1.0 (activity monitor, not a parser)
74. snitchit 0.1.0 (session recorder, not a parser)
75. btfm-api-structs 0.3.1 (Discord bot API, not a parser)
76. btfs 0.0.4 (in-memory filesystem, not a parser)
77. allenap-libtftp 0.2.2 (TFTP library, not a parser)
78. btfm 0.21.1 (Discord bot, not a parser)
79. btfm-cli 0.7.0 (Discord bot command, not a parser)
80. libtftp 0.1.2 (TFTP server, not a parser)
81. redoubtful 0.0.1 (sandbox runner, not a parser)
```

  No new parser candidate came from the rows the first version did not read: `padlock-cli` is the
  padlock analyzer's command, and the remaining rows are eBPF tooling and unrelated crates, tagged
  as such in the listing.
- Class 2, the repository host, the queries at schedule items 4, 5 and 8 to 12.

  The first version recorded 7 rows for `BTF parser in:name,description` with `language:rust`;
  today's re-run returns 1 row for that filtered query and 7 for the unfiltered one, so which
  filter the index applied then is not knowable now. Both listings are screened under class 2 and
  add no survivor. The 7 unfiltered rows:

  - `trailofbits/btfparse`: a C++ BTF parser, already in the ledger and failing H3.
  - `retis-org/btf-rs`: the finalist.
  - `zeroeffects/btf`: a single-header parser for 6D BTF, a graphics transfer format, not BPF BTF.
  - `luzhixing12345/btfparser`: a C function-declaration parser, not a Rust BTF library.
  - `genksome/kxray`: an Android boot-image offset tool.
  - `ti-mo/btf-datasec-repro`: a reproducer for a kernel verifier bug.
  - `IoTAccessControl/eBPF-Linux-Patch-BTF-Parser`: an eBPF patch parser, no description.

  The wider `btf in:name,description language:rust` query's 37 rows, screened by name and
  description only:

  1.  `jellyf1s/btf`: a fork of `arcjustin/btf`, the ledger's `btf` row, whose README and badge
      still point at `arcjustin/btf` (H1 fail, no commit since 2023).
  2.  `anakryiko/btfdump`: the ledger's `btfdump` row.
  3.  `vobst/btf2json`: a Volatility3 profile generator that consumes BTF, a tool.
  4.  `retis-org/btf-rs`: the finalist.
  5.  `trailofbits/mquire`: memory forensics that reads kernel BTF, a tool.
  6.  `eunomia-bpf/btf2wit`: a BTF to wit converter, a tool.
  7.  `vadorovsky/aya-btf-map`: BTF map support for Aya, not a parser.
  8.  `rkhudov/btf`: no description and a one-line README; not audited, no split base claimed.
  9.  `alessandrogario/btfparse`: the ledger's `btfparse` row.
  10. `kotagit75/btfy`: a blockchain project, unrelated.
  11. `jeremycline/btfm`: no description; not audited, no split base claimed.
  12. `eunomia-bpf/rust-struct-bindgen`: CO-RE binding generation from BTF, a generator.
  13. `saiintbrisson/bepeefe`: an eBPF VM with BTF support, a runtime and not a parser library.
  14. `BTF-Kabir-2020/wireglass`: a DNS, TCP and TLS measurement command, unrelated.
  15. `BetterTommrow/BTFirstRepository`: an unrelated first repository.
  16. `leonardogiovannoni/BTF2LLVM`: a BTF to LLVM generator, a tool.
  17. `eurecom-s3/lemon-btf2json`: a BTF to JSON tool.
  18. `buffalojoec/btf-dumps`: a BTF dump inspection repository, research.
  19. `stultusverus/btfind`: a BitTorrent sniffer, unrelated.
  20. `KevinGuancheDarias/btf-editor`: an editor for a game's `.btf` localization files, unrelated.
  21. `kuopinghsu/BTFViewer-rust`: a BTF viewer, a tool.
  22. `chernyshev-alex/mci-btfaang`: LeetCode exercises, unrelated.
  23. `sysdiglabs/corelox`: CO-RE relocation generation, a build tool.
  24. `flier/rust-btf`: a small implementation of eBPF ELF operations, not a BTF type parser.
  25. `vadorovsky/aya-btf`: experimental CO-RE support for Aya, a library around eBPF objects.
  26. `masmullin2000/btf_container_test`: no description; a test repository, not audited.
  27. `joshuataylor/aya-btf-error`: no description; an error type for Aya's BTF work, not a parser.
  28. `Stringy/bpf-mcp`: an MCP server for BPF and BTF inspection, a tool.
  29. `newsniper-org/edge-btf-module-wasm`: no description; not audited, no split base claimed.
  30. `JeffResc/FSSnapshotBackup`: a ZFS and Btrfs snapshot backup tool, unrelated.
  31. `BTF-Kabir-2020/NebulaGrid`: an infrastructure control plane, unrelated.
  32. `BTF-Kabir-2020/aurum-rs`: a financial engine, unrelated.
  33. `Rithvik1124/eBPFBoilerplateHelper`: generates code from `/kernel/btf` C definitions, a tool.
  34. `varpeti/notflix-rs`: magnet links over btfs, unrelated.
  35. `kmulemg/vivo-xflip-gki-profile`: an Android GKI build profile that carries BTF, unrelated.
  36. `BTF-Kabir-2020/CS-1.6-Tool-v2`: a game memory tool, unrelated.
  37. `BTF-Kabir-2020/dnstt-kit`: a DNSTT toolkit, unrelated.

  None of the 37 is a BTF reader that names a split base, and none is audited here.

  The code search's 25 rows cover 6 repositories: `retis-org/btf-rs` (its parser, its ELF helper and
  its integration test), `quickwit-oss/quickwit` and its `gqf2008` fork (bundle storage),
  `apache/paimon-rust` (C bindings), `ostrium-labs/loams` (bundle storage), and
  `likewhatevs/ktstr` (a vmlinux-only struct-field resolver inside a larger tool). Only
  `btf-rs` is a BTF reader with a split constructor; the rest match `split_` helpers or unrelated
  code.

  The topic queries return 3 rows for `topic:btf language:rust` (`vobst/btf2json`,
  `jellyf1s/btf` and `copyleftdev/skbx`, a packet-path tracer) and 0 rows for
  `topic:btf-parser`. No new candidate.

  GitHub's REST API has no release search, so releases were read for each repository the ledger
  audits, at `per_page=100`, which is above every repository's count: `retis-org/btf-rs` has 5
  releases, `alessandrogario/btfparse` 12, `anakryiko/btfdump` 5 and `libbpf/libbpf-rs` 31. The 31
  is the total, not the API's default page size of 30, so a reader that trusted the default would
  undercount by one. Releases do not announce a new BTF reader: the two latest bodies read describe
  an out-of-bounds fix in `btf-rs` 2.0.1 and an arena map variant in `libbpf-rs` 0.27.2.

  Organization project boards. `retis-org` and `aya-rs`, the two organizations the ledger's
  candidates sit in, each answer the GraphQL `projectsV2(first: 1) { totalCount }` query with a
  count of 0, so there is no board to read there. `libbpf`, `trailofbits` and `eunomia-bpf` each
  return 1, and a board's contents need the `read:project` scope, which this session's token does
  not carry, so their titles and items stay unread. A board is not a candidate index, and no
  candidate in the ledger came from one; the gap is recorded rather than filled.
  The round of 2026-11-20 read those three boards through their public pages,
  and their organizations' repository lists,
  as recorded under
  [class 2 of this round](#class-2-of-this-round-the-organization-lists-and-boards).
- Class 3, broader web search. The literal query `Rust crate parse BTF split BTF module vmlinux
  base BTF parser library` was run once through this session's first provider, which returned 10
  result lines, that provider's page size; the first version's report of 44 lines under a 10-line
  cap could not be reproduced.
  One independent enumeration was then tried, once, through a second provider on the same literal
  query: it reported a listing of 44 result lines, of which the reader saw 10. Every page read
  points at an artefact another class already holds, namely `btf-rs` on crates.io (2.0.0), on
  docs.rs and at `retis-org/btf-rs`, `btfparse` on crates.io and a mirror, an ecosystems index
  entry for `btf-rs`, and the crates.io `btf` keyword index that the class 1 query already
  enumerated. No parser survivor, no name the ledger lacks, and no second listing read.
  Neither listing is provably exhaustive: the first stops at its provider's page size and the
  second reached this reader only in part, so this class stays blocked and no further provider was
  tried. The first provider's page carries the `btf-rs` README's statement that `Backend::Mmap` is
  only supported for base BTF files.
- Class 4, this repository. The OpenZFS vet, the design's `pahole` command, the repo's `zfsdev`
  reuse. These are the incumbent and prior-report class, and the vet already names both tools.

#### The round of 2026-11-20

This round ran the pre-discovery check-in this record owed,
the class 3 queries,
and the class 2 repository and project lists.
It ran in the worktree `zfs-layout-checker-reader-validation` on 2026-11-20,
adopted nothing,
and added no crate to any manifest.
Every command,
page and result count is below.
Every new name's screening is in
[Candidate ledger](#candidate-ledger).

#### The pre-discovery check-in of 2026-11-20

The skill's "Check in before broad discovery" asks one question per vetting process.
It states the subject,
the context forks settled so far,
the source classes,
and the draft query schedule.
It also offers who runs the enumeration:
this session,
a model the user names,
or the user.
The question was put to the project on 2026-11-20.
Its subject was the BTF reader Q1 picks.
Its settled forks were the design's three:
a library the binding links or a program the gate starts;
no hard requirement that the parser be our own code;
and the two GNU release targets.
Its source classes were the four the skill requires.
Its remaining schedule was the three class 3 queries and the two class 2 list reads below.
The answer of 2026-11-20 was "This session's writer (Recommended)".
That writer was described as a `DeepSeek V4.1 Flash` writer through Pi that runs the remaining
queries now.
It records each one with its literal query,
provider,
page and result count.
It ends in "no new candidate" or in new candidates checked the same way.
This subsection is that artifact:
the question,
the answer and its date.

#### Class 3 of this round: the three queries

Each query ran on the DeepSeek provider through this session's search tool.
None sent an include or exclude filter,
none sent a sort parameter,
and each took the provider's own ranking.
Each returned one page.
That page holds at most 50 rows.
Each returned fewer rows than that cap and offered no cursor.
The provider therefore reported its listing ended,
and no page 2 exists for these queries.
No result was truncated after that page,
and no filter can hide a row.

1.  Literal:
    `Rust crate parse BTF split BTF module vmlinux base BTF parser library`.
    Page 1 of 1.
    44 rows.
    Names the ledger lacks:
    `bpf-compatible-rs` and `vock`.
    Both are screened in [Candidate ledger](#candidate-ledger),
    and neither is a candidate.
    New survivors:
    none.
2.  Variant,
    C libraries that parse BTF:
    `C library parse BTF split BTF module vmlinux base BTF parser`.
    Page 1 of 1.
    46 rows.
    Names the ledger lacks:
    `libabigail` and `parttimenerd/btfparse`.
    Both are screened and neither is a candidate.
    New survivors:
    none.
3.  Variant,
    alternatives to libbpf's `btf__parse`:
    `alternatives to libbpf btf__parse BTF parser library`.
    Page 1 of 1.
    49 rows.
    Names the ledger lacks:
    `ebeepf`,
    `cilium/ebpf`,
    `libbpfgo`,
    `gbpf`,
    `coolbpf` and
    `bpf-compatible`.
    `ebeepf` passed every hard gate and is this ledger's fourth finalist.
    The other five are screened and not candidates.
    New survivors:
    one.

#### Class 2 of this round: the organization lists and boards

The repository lists came from the GitHub API.
`gh api --paginate --slurp "orgs/<org>/repos?per_page=100&type=public"` read every public
repository each organization's own `public_repos` count names.
Each list therefore ends on the provider's exhaustion rather than on a page cap.

- `libbpf`:
  15 of 15 repositories.
  The BTF-adjacent ones are the ledger's `libbpf-rs` and `libbpf-sys`,
  `bpftool` (a finalist),
  `blazesym` (symbolization),
  `veristat` (a tool),
  `libarena`,
  `usdt`,
  `vmlinux.h` (a header mirror),
  `libbpf-cpp`,
  and `bpfvv`.
  None is a BTF parser the ledger lacks.
- `trailofbits`:
  298 of 298.
  The BTF-adjacent ones are `btfparse` (refused on H3),
  `mquire` (the ledger's memory-forensics tool),
  `nate` (a Rust DWARF parser),
  `ebpfpub`,
  `ebpf-common`,
  `ebpfault`,
  and `bpf_tutorials_code`.
  None is new.
- `eunomia-bpf`:
  157 of 157.
  The BTF-adjacent ones are `bpf-compatible` (C),
  `btfhub-archive` (BTF files),
  `btf2wit`,
  `rust-struct-bindgen` and `c-struct-bindgen` (generators),
  `btf-relocate-demo`,
  `multiarch-relocate-example`,
  a `bpftool` mirror,
  a `libbpf-rs` fork,
  and a `vmlinux` mirror.
  None is new.

The boards came from two routes.
`organization { projectsV2(first: 1) { totalCount } }` answers 1 for each of the three
organizations,
so each holds one board.
Titles and items need `read:project`,
which this session's token lacks.
Its scopes are `gist`,
`read:org`,
`repo` and `workflow`.
The REST project endpoints answer 404 because they are sunset.
The boards' public HTML pages answered instead:

- The `libbpf` board is project 1.
  It is "BPF Verifier Visualizer" and holds Todo 0 and In Progress 0.
- The `trailofbits` board is project 150.
  It is "[ENG] Sigstore Transparency Log Monitoring",
  a Sigstore monitoring board with no BTF item.
- The `eunomia-bpf` board is project 1.
  It is "eunomia-bpf" and no column holds an item.

No board names a candidate,
which is what the earlier round predicted a board could not do.
What changed is that the three titles are read rather than assumed.

Terminal result:
saturated with at least two survivors.
Class 1 read all 81 rows of its registry query.
Class 2 now covers the three organizations' complete repository lists and all three of their
boards.
Class 3 read each of its three queries to the end its provider reported.
Class 4 is unchanged.
Screening survivors are these four:
`btf-rs` 2.0.1,
`ebeepf` 0.1.0,
`pahole` 1.32 and
`bpftool` 7.8.0.
Each is validated under [Finalist validation](#finalist-validation).
The ledger is dated by the commit that carries this record.
[The vet artifacts this brief could not write](#the-vet-artifacts-this-brief-could-not-write)
records what is still owed.

### Finalist validation

This round ran the first item Q1 lists,
the finalist validation the skill requires,
on 2026-11-20,
in the worktree `zfs-layout-checker-reader-validation`.
It adopted nothing,
added no crate to a manifest,
and ran no product code.
Every step's command,
output,
exit status and checksum is kept under
`/home/user/temp/agent/zfs-layout-checker-reader-finalist-validation-2026-11-20/keep/`.
The crate's logs are in `keep/evidence/btf-rs-2.0.1/`,
and the tools' outputs are in `keep/evidence/`,
both under that root.
This section cites those files by name.

The untrusted build ran in one environment.
The tool is `podman` 6.2.0.
The image is `docker.io/cachyos/cachyos:latest`
at digest `sha256:ab3e1698b7ce5eaa4f6342d7a0f687ce731c443706238a49338f46239036a185`.
The caps are 2 CPUs,
2 GiB of memory and 512 processes.
No home directory is mounted:
`HOME` is `/work/home` inside the container.
The two BTF copies are mounted read-only at `/btf`.
Dependencies were fetched in one step with the network on,
and every build and test ran in a later step with `--network none`
and `CARGO_NET_OFFLINE=true`.
The toolchain is stable 1.100.0,
installed by `rustup` 1.29.3 in the fetch step.
The host's active rustup toolchain is nightly 1.102.0,
with neither the `rustfmt` nor the `clippy` component,
while the crate's CI runs stable,
so the fetch step brings the toolchain and components the CI path needs.

The archive was checked against the registry before it ran.
The sparse index entry for 2.0.1 at `https://index.crates.io/bt/f-/btf-rs`
gives `cksum`
`fa34ec915ede6b7eaeebcb38fdb749fcac1538ec1b94dcfde63a034ad8f7963b`,
and the downloaded archive's SHA-256 is that value.
Its `.cargo_vcs_info.json` names commit
`03fd01bd9505d0888b568cd3aaa9b232b7e3fb20`,
which is the commit tag `v2.0.1` points at.

The published archive ships no tests.
It holds 15 files.
Its manifest sets `autotests = false`,
and its `include` list is `examples/`,
`src/` and `LICENSE`,
so `cargo test` on the archive runs no unit or integration test,
and its four doctests only compile.
The ledger's integration test therefore ran from the repository at that commit,
cloned from `retis-org/btf-rs` at tag `v2.0.1`,
with the same commit at `HEAD`.
Its `btf-rs/tests/assets` folder is 56 MB.

The CI-equivalent path passed on stable 1.100.0,
with the network off:
`cargo fmt --check`,
`cargo clippy --locked --all-features -- -D warnings` for the library and the example,
`cargo doc --locked --all-features` under `RUSTDOCFLAGS=-D warnings`,
the six feature builds the workflow runs,
and `cargo test --locked --all-features`.
The suite ran 168 integration tests and four doctests,
with 0 failed and 0 ignored,
in 11.16 seconds.
Their names group as `btf_api`,
`resolve_anon_name`,
`resolve_anon_regex`,
`chained_type_iter`,
`bijection` and `resolve_function` with 18 cases each,
`btfc` and `btfc_resolve_regex` with 12 each,
`resolve_split_func`,
`resolve_split_struct` and `resolve_regex` with 9 each,
`base_btf` and `split_btf` with 4 each,
and `wrong_input` with 1.
The log at `ci-cargo-test-all-features.log` lists every case's result.

The suite needs what the repository carries and lacks the product's own case.
It needs 56 MB of checked-in assets,
a C compiler for the `zstd` and `xz` optional dependencies when `elf-compression` is on,
and no network,
no root and no `/sys` access.
It has no fixture of a live kernel and module pair,
so it cannot exercise the pair the gate reads.
Its feature-gated cases are 104 of the 168:
with the default,
empty feature set it runs 64 integration tests and four doctests,
and nothing is marked `#[ignore]`.
The crate still has no fuzz target.

The design's checks ran from a consumer program that depends on the library by path.
It reads the two copies into bounded buffers,
calls `Btf::from_bytes` for the base and `Btf::from_split_bytes` for the module,
which is the loader route the product designs,
and asserts the compared set.
That set is both files' 32-byte header,
the base's 151,181 types and the module's 13,141 types from 151,182 to 164,322,
`zfs_cmd` at 13,744 bytes with its 32 members and their bit offsets and byte sizes,
the anonymous union at 106,368 bits with `zc_inject_record` and its anonymous struct,
`dmu_objset_stats` at 288 bytes with `dds_flags` at 2,296 bits and size 1,
`zinject_record` at 368 bytes with its 19 members,
the 104 `zfs_ioc` enumerators with the seven values this design names,
and the base integer sizes those members use.
It ran 183 checks with 0 failures.
Its negative controls are a base and a module truncated to 1,024 bytes,
a module whose magic byte changed,
and a split object passed as a base;
the library refused each with its own error,
and none panicked or stalled.
The log at `design-consumer-check-on-host-btf-copies.log` holds every check.

The tool finalists ran through a scratch harness,
`tools-layout-check.ts`,
that runs each tool on the same two copies and checks the same members.
`pahole` is 1.32 at `/usr/bin/pahole`,
with SHA-256 `00acbd859cd0cca6e71dca93d885ed2483c5a52a78d3df1a9bcc9da82881503c`.
`bpftool` is 7.8.0 on libbpf 1.8 at `/usr/bin/bpftool`,
with SHA-256 `475a53182f50231c40e40ce002f2e032dc418489d56a2f8f902aa317aed03d20`.
The harness ran 323 checks with 0 failures,
at four `pahole` invocations and at `bpftool btf dump file` for the module and the base.
Its negative controls are both tools on the module copy without a base:
`pahole` exits 1 with libbpf's "did you forget to provide base BTF" message,
and `bpftool` exits 234 with "Invalid argument".
Two output facts bear on C4.
`pahole` prints absolute byte offsets at every nesting depth,
so its anonymous containers need no path arithmetic.
`bpftool` prints `bits_offset` relative to the containing type,
renders an empty BTF name as `(anon)`,
and prints no member size,
so a consumer must resolve each `type_id` through both dumps.
The harness does that,
and that resolver is the cost C4 prices.
The raw outputs are `pahole-*-output.txt` and `bpftool-*-raw-output.txt`,
and every check is in `tool-harness-checks.txt`.

The skill's validation now holds for `btf-rs` on the one platform this host has.
The default CI path passed,
the suite passed,
and the consumer boundary ran on copies of this host's own pair,
with controls that show the reader refuses bad bytes.
It does not hold for `aarch64`:
no `aarch64` build or run happened,
so H3's second target is still source evidence only,
as it was before this round.
The archive's missing tests are a provenance fact and not a gate failure:
the vcs information maps the archive to the commit whose suite passed,
which is what H6 asks for.

The tool finalists are validated at their output boundary.
Each ran on the same copies and its output carried the compared members,
and a negative control showed the base file is required.
Their own upstream suites were not run and are not the product's surface,
because the product starts the packaged program.
The product's own arms for a tool route,
owed measurements 13 and 14,
and the release-version matrix those items name,
stay unrun.

No rating moved.
C5 and C6 for `btf-rs` rest on the upstream record the suite confirms,
and a green suite neither raises nor lowers them.
C4 for `bpftool` stands at 2,
now with the resolver's cost demonstrated rather than read.
C7 stays unmeasured,
and this round took no timing,
so it says nothing about start cost.
Among those three the order stays `btf-rs` 2.0.1 > `pahole` 1.32 > `bpftool` 7.8.0.
The fourth finalist changes the order once it is scored,
and [Scored comparison](#scored-comparison) records that order.

#### ebeepf 0.1.0

This finalist's validation ran the same three steps as `btf-rs`'s.
The archive was checked against the registry first:
the version entry's `checksum` equals the downloaded archive's SHA-256 recorded in
[Candidate ledger](#candidate-ledger).
Its `.cargo_vcs_info.json` names commit `5de05465bd9445d89b13a967e198d0aeffd71ec4`,
which is the commit the repository clone carries at `HEAD`.

The environment is one image built for this round,
because the published archive's own suite needs `clang` for its BPF fixtures.
The image is `localhost/ebeepf-validation:rust-1.100.0-clang` at digest
`sha256:d201d863b813804a9bc89db027eb51ed6f9839c4d8d363a2ab77e1363bed2a05`.
It was built from `docker.io/cachyos/cachyos:latest` at digest
`sha256:ab3e1698b7ce5eaa4f6342d7a0f687ce731c443706238a49338f46239036a185`,
plus the distribution's Rust 1.100.0 and clang.
Every step is a `podman run --rm` with 2 CPUs,
2 GiB of memory and 512 processes.
No home directory is mounted,
and `HOME` is `/work/home` inside the container.
The two BTF copies are mounted read-only at `/btf`.
Dependencies were fetched in a step with the network on.
Every build,
test and check then ran with `--network none` and `CARGO_NET_OFFLINE=true`.
A rustup route was tried twice for a cross `aarch64` toolchain and failed both times,
with a TCP connect timeout to the distribution host while `curl` from the same container reached
it.
The `aarch64` target therefore stays unrun,
as it does for `btf-rs`.

The suite passed.
On the archive,
`cargo test` ran 102 unit tests,
`kernel_smoke` with 1 passed and 22 ignored,
and 1 doctest,
all green.
`cargo fmt --check` passed.
`cargo clippy --lib -- -D warnings` passed with one manifest warning about a redundant `homepage`.
The repository's own workspace adds a `bpfcov` member whose `libbpf-rs` dependency does not
resolve offline,
so the repository's `cargo clippy` was not a usable step there.
The archive is the published artifact and the one this record cites.

The design's checks ran from a consumer program that depends on the archive by path.
It reads the two copies into bounded buffers,
calls `Btf::parse` for the base and `Btf::parse_split` for the module,
which is the loader route the product designs,
and asserts the same compared set as `btf-rs`'s run:
both files' 32-byte header,
the base's 151,181 types,
the module's 13,141 own types numbered from 151,182,
`zfs_cmd` at 13,744 bytes with its 32 members,
the anonymous union at 106,368 bits,
`dmu_objset_stats`,
`zinject_record`,
the 104 `zfs_ioc` enumerators,
and the base integer sizes the members use.
195 of its 196 checks passed.
The failing one is a negative control:
a module passed as the base of another split parse is accepted rather than refused,
which is the residual the ledger records.
Its other negative controls passed,
namely a base truncated to 1,024 bytes,
a module truncated the same way,
and a module whose magic byte changed,
each refused with its own message and none panicking or stalling.
The evidence sits under
`/home/user/temp/agent/zfs-layout-checker-reader-discovery-finished-2026-11-20/keep/evidence/ebeepf-0.1.0/`.

The skill's validation therefore holds for `ebeepf` on the one platform this host has,
at the same depth as `btf-rs`'s:
a CI-equivalent build and lint path,
the crate's own suite,
and the consumer boundary on copies of this host's pair.
It does not hold for `aarch64`,
and the start cost stays unmeasured.

### Scored comparison

Criteria and weights, frozen before the ratings:

- C1. Bounded, fail-closed handling of untrusted bytes, weight 5.
- C2. Release and deployment fit: no runtime program, no C build, both GNU targets, weight 5.
- C3. Privilege surface in the root handler, weight 4 (higher is smaller).
- C4. Diagnostic precision: member-level expected and actual values, weight 4.
- C5. Maintenance and upstream responsiveness, weight 3.
- C6. Integration and test cost, weight 3 (higher is cheaper).
- C7. Start cost, weight 2, low confidence because it is unmeasured.

Four candidates are finalists,
and each passes every hard constraint:
`btf-rs` 2.0.1,
`ebeepf` 0.1.0,
`pahole` 1.32 and
`bpftool` 7.8.0.
The in-house reader passes H1 to H6 by construction, but it is not eligible as a candidate while
an existing tool passes them, which is the skill's existing-tools rule and the reason
[Candidate ledger](#candidate-ledger) calls it conditional. It therefore stands outside the
eligible ranking, as an exception to that rule that the owner declined to grant on 2026-11-19 and
not as a finalist: the skill's "Weighted scoring" section says to score only validated finalists,
and this reader does not exist yet.

To make the reader eligible, the owner would have to name one hard constraint that every existing
candidate fails at once, such as "the parser is our own code and the product starts no second
program".
`btf-rs` fails the first half, because its parser is third-party code, and `pahole` and `bpftool`
fail the second, because each is a program; that single constraint would admit the reader and remove
each of them.
Refusing a copyleft dependency alone eliminates only `btf-rs`, since the two tools
are separate GPL-2.0 programs that are never linked, so that ground does not make the reader
eligible.
The owner was asked that question in this form on 2026-11-19 and answered "No, not required
(Recommended)", so no such constraint exists and the reader stays ineligible
([Answers of 2026-11-19](#answers-of-2026-11-19)).

The reader is still scored, so its price is on record beside the order it stands outside of, and
the arithmetic marks that score as priced, not ranked.

Ratings, 0 to 4, with confidence:

- `btf-rs` 2.0.1: C1 3 high, C2 4 high, C3 4 high, C4 4 high, C5 3 medium, C6 3 medium,
  C7 2.5 low (range 2 to 3).
- `ebeepf` 0.1.0:
  C1 3 medium,
  C2 4 high,
  C3 4 high,
  C4 4 high,
  C5 2 medium,
  C6 3 medium,
  and C7 2.5 low,
  ranging 2 to 3.
- In-house reader: C1 4 high, C2 4 high, C3 4 high, C4 4 high, C5 3 medium, C6 2 medium,
  C7 2.5 low (range 2 to 3).
- `bpftool` 7.8.0: C1 3 medium, C2 1 high, C3 1 high, C4 2 medium, C5 4 high, C6 3 medium,
  C7 1.5 low (range 1 to 2).
- `pahole` 1.32: C1 3 medium, C2 1 high, C3 1 high, C4 3 medium, C5 4 high, C6 3 medium,
  C7 1.5 low (range 1 to 2).

Evidence for the ratings:

- C1: `btf-rs` refuses truncated input and its input is kernel-validated, but it has no section
  check of its own and its released masks are old, so the pre-scan is ours (3).
  The in-house reader can enforce every bound in one place (4).
  The tools refuse through their own parser and exit status, and the wrapper must bound the file
  before handing it over (3).
- C2: both libraries are pure Rust with no default features (4);
  both tools add a program the product must find and version (1).
- C3: in-process code keeps the privileged handler to its own binary (4);
  a root-spawned tool adds a path and library resolution to that boundary (1).
- C4: `btf-rs` and an in-house model expose names, bit offsets and sizes (4);
  `pahole`'s `-C` printer maps directly to the compared members (3);
  `bpftool`'s raw dump needs a type-id resolver in the wrapper (2).
- C5: `btf-rs` is active but small, with two open items and a release branch behind its main (3);
  the tools follow their projects' release cadence (4); in-house depends on this repository's own
  review (3).
- C6: the library routes share most of the work, so the difference is small and both are
  rated close. The shared part is the bounded loader, the pre-scan and its widest-encoding walk
  over every kind, the expectation table with its path grammar, the comparison, the refusal value
  and their tests: the 350 to 550 source lines and 300 to 450 test lines Way 1 prices with a
  dependency. The dependency route still owes the binding one piece of arithmetic: `btf-rs` gives
  a `size()` for the int, struct, enum, data-section, section-info, float and `enum64` kinds
  (`btf-rs/src/btf.rs:462, 550, 638, 908, 936, 958, 1065`), and its `chained_type_iter` walks a
  type chain (`:220-228`), but its array type exposes only `len()` (`:512`) and no size, so array
  byte sizes are the binding's own multiplication, while typedef chains come from the crate's
  chain walk. The in-house route adds the reader and its model, about 350 to 450 source lines and
  100 to 150 test lines on the same shape, plus its fuzz target and its review.
  About two to one in effort, not four to one, so 3 and 2.
  The tools need a spawn wrapper and an output parser each, plus the packaging and the version
  matrix (3).
- C7: unmeasured; `btf-rs` parses the base's type offsets and the module's types;
  in-house can read lazily; the tools pay a process start and a full dump.
- The `ebeepf` ratings hold criterion by criterion:
  C1 3 medium,
  because it bounds both sections,
  validates every type reference,
  and refuses truncation and a wrong magic,
  but its own README declares the tree AI-generated and unreviewed,
  and it accepts a split table as a base,
  so a 4 would claim a trust the evidence does not hold;
  C2 4 high,
  pure Rust with the default features off;
  C3 4 high,
  in-process code with no second program;
  C4 4 high,
  since members carry bit offsets and widths and enumerators carry values,
  and `size_of` already multiplies array elements,
  which `btf-rs` leaves to the binding;
  C5 2 medium,
  because it has one release,
  one maintainer,
  no CI,
  no commit after that release,
  and a self-declared unreviewed tree;
  C6 3 medium,
  the same shared work as `btf-rs` minus the array-size shim,
  but with no CI to lean on;
  C7 2.5 low,
  ranging 2 to 3,
  unmeasured,
  and `parse_split` copies the base's 151,181 types into the module's table,
  which is a memory cost `btf-rs` does not pay.

Arithmetic:

- `btf-rs`: `5*3 + 5*4 + 4*4 + 4*4 + 3*3 + 3*3 + 2*2.5` = 90 of 104, 86.5.
- `ebeepf` scores `5*3 + 5*4 + 4*4 + 4*4 + 3*2 + 3*3 + 2*2.5` = 87 of 104,
  which is 83.7.
- In-house: `5*4 + 5*4 + 4*4 + 4*4 + 3*3 + 3*2 + 2*2.5` = 92 of 104, 88.5.
- `pahole`: `5*3 + 5*1 + 4*1 + 4*3 + 3*4 + 3*3 + 2*1.5` = 60 of 104, 57.7.
- `bpftool`: `5*3 + 5*1 + 4*1 + 4*2 + 3*4 + 3*3 + 2*1.5` = 56 of 104, 53.8.

Low-signal endpoint totals, over every combination of the ranges' endpoints:

- `btf-rs`: 89 to 91 (85.6 to 87.5).
- The `ebeepf` endpoints are 86 to 88,
  which is 82.7 to 84.6.
- In-house: 91 to 93 (87.5 to 89.4).
- `pahole`: 59 to 61 (56.7 to 58.7).
- `bpftool`: 55 to 57 (52.9 to 54.8).

The weights, the ratings, the confidence levels and the ranges stated in this section are every
input the arithmetic uses, and the arithmetic and the matrix under [Sensitivity](#sensitivity)
follow from them.
Every score here is provisional:
`btf-rs`'s suite,
`ebeepf`'s suite and the two tools' output boundary have run
([Finalist validation](#finalist-validation)),
the start cost is unmeasured,
and the in-house reader does not exist,
so its ratings price an intention.

The in-house reader's 92 leads `btf-rs`'s 90 at this writer's weights, and that lead does not
survive its own inputs. At equal weights (every criterion 1, which the skill sets for every
criterion whose priority is unspecified) the pair ties at 23.5. The lead comes from C1 and C4,
both rated high for code that does not exist: at medium confidence C1 alone puts the reader at
87 and C4 alone at 88, both below 90, and both at medium give 83. Code that has not been written
cannot hold a rating better than the release that was probed, so 92 is a ceiling, not a
measurement.

`ebeepf`'s 87 sits between `btf-rs`'s 90 and the tools' 60.
C5 is where it loses first place:
a step to 3 there ties the pair at 90,
and its own C6 at 4 ties it again,
so the pair's separation rests on the maintenance and trust evidence,
not on the arithmetic alone.
Both ties are recorded under [Sensitivity](#sensitivity).

### Sensitivity

One input at a time, as the skill requires: every weight from 1 to 5, every medium-confidence
rating one step down and up, and both endpoints of every low-signal range (the endpoints are the
plausible values, so no step is taken outside a range).
The 12 one-input tests change an order: 8 reverse it and 4 leave an exact tie. The tests take the
confidence labels as the ratings list them, and the count is of those labels. The in-house reader's
C1 and C4 are labelled high for code that does not exist; were they medium, the same method would
add two reversals, C1 alone putting the reader at 87 and C4 alone at 88 against `btf-rs`'s 90, and
that tally would not include them.

Library pair, `btf-rs` against the in-house reader, 90 against 92 at the baseline:

- weight C1 at 1 reverses, 78 against 76;
- weight C1 at 2 reverses, 81 against 80;
- weight C1 at 3 ties, 84 each;
- weight C6 at 5 ties, 96 each;
- `btf-rs` C5 from 3 to 4 reverses, 93 against 92;
- `btf-rs` C6 from 3 to 4 reverses, 93 against 92;
- in-house C5 from 3 to 2 reverses, 89 against 90;
- in-house C6 from 2 to 1 reverses, 89 against 90.

Tools pair, `pahole` against `bpftool`, 60 against 56 at the baseline:

- `bpftool` C1 from 3 to 4 reverses, 61 against 60;
- `pahole` C1 from 3 to 2 reverses, 55 against 56;
- `bpftool` C4 from 2 to 3 ties, 60 each;
- `pahole` C4 from 3 to 2 ties, 56 each.

No test moves a tool past a library, and more than twenty points of 104 separate them.

A new library pair is `btf-rs` against `ebeepf`,
90 against 87 at the baseline.
Its one-input matrix runs 42 tests over the same method.
One reverses the pair,
four tie it,
and thirty-seven leave the order.
The tests that move it:

- `ebeepf` C1 from 3 to 4 reverses the pair to 92 against 90;
- `ebeepf` C5 from 2 to 3 ties the pair at 90 each;
- `ebeepf` C6 from 3 to 4 ties the pair at 90 each;
- `btf-rs` C5 from 3 to 2 ties the pair at 87 each;
- `btf-rs` C6 from 3 to 2 ties the pair at 87 each.

The rest leave it:
C1 weight 1 puts them at 78 against 75,
C1 weight 2 at 81 against 78,
C5 weight 5 at 96 against 91,
C6 weight 5 at 96 against 93,
and C7 weight 5 at 97.5 against 94.5.
Both C7 endpoints leave it too,
`btf-rs`'s low end at 89 against 87 and `ebeepf`'s low end at 90 against 86.

The one reversal is `ebeepf`'s C1 at 4,
and the evidence does not support that step.
The crate's own README declares its tree AI-generated and unreviewed,
and it accepts a split table as its own base,
so 3 is the rating its bounds and its provenance support.
The four ties are C5 and C6 steps on either side,
and the maintenance evidence decides those:
`btf-rs` carries two releases,
a merged out-of-bounds fix and a CI workflow on every push,
against one release,
one maintainer and no CI.
The pair's order therefore stands on that evidence,
not on the baseline arithmetic alone.

A tie is not an order the arithmetic decides, and the skill's rule for both a tie and a change is
to gather decisive evidence or ask only the controlling preference.
For the tools, the C4 evidence decides: `pahole`'s `-C` output carries the compared members, while
`bpftool`'s raw dump needs a type-id resolver in the wrapper, so `pahole` ranks ahead of
`bpftool` when the rating moves.
For the scoring's baseline pair the evidence does not decide: both routes' C4 and C6 rest on
readings of two different code bases, and the pair is 92 to 90 with 8 tests tying or reversing it.
That pair is no longer a choice: the owner answered on 2026-11-19 that the in-house reader is not
offered, so those tests record what the route would have cost and decide nothing.
Stability here does not cover simultaneous multi-input changes, and this document makes no claim
it does.
The evidence that would settle the remaining ratings is owed:
[Owed measurements](#owed-measurements) item 10 would settle C7,
and the suite this round ran supports C5 and C6 without moving them
([Finalist validation](#finalist-validation)).
The controlling preference that remains is the owner's, not a weight on this writer's scale: a
library the binding links, an `LGPL-2.1-or-later` dependency whose release lags its own main, or a
program the gate starts at each run, which the product must find and version and the root handler
starts as root.
[Q1](#q1-which-reader-parses-the-module-btf) asks that in those words.
Until then this section gives a conditional ranking and recommends none.

### Ranking and pros and cons

Ranking of the eligible candidates:
`btf-rs` 2.0.1 > `ebeepf` 0.1.0 > `pahole` 1.32 > `bpftool` 7.8.0.
The in-house reader stands outside it as an exception the owner declined to grant on 2026-11-19,
priced at 92 of 104 while ineligible.

- `btf-rs` beats `pahole` because its proof runs in the wrapper's address space, with no runtime
  program and no root-spawned binary, which neither tool can offer; the arithmetic agrees, 90
  against 60.
- `btf-rs` beats `ebeepf` because the pair's 90 and 87 separate on maintenance and trust:
  two releases,
  a merged out-of-bounds fix and a CI workflow on every push,
  against one release,
  one maintainer and a tree its own README calls unreviewed AI-generated code.
  The arithmetic agrees,
  and the pair's one reversal and four ties are the C1 and C5 steps that evidence settles.
- `ebeepf` beats `pahole` because it is a library in the wrapper's address space,
  with a split parser and no runtime program,
  while `pahole` costs the deployment and privilege points C2 and C3 price.
  The arithmetic agrees at 87 against 60.
- `pahole` beats `bpftool` because its `-C` output carries the compared members directly, while
  `bpftool`'s raw dump needs a type-id resolver in the wrapper before it can compare anything.
  The arithmetic agrees, 60 against 56; a C4 step ties the pair and a C1 step reverses it.
- The in-house reader is not ranked: the owner's answer of 2026-11-19 left it ineligible, and its
  92 of 104 against the tools' 60 stays in [Scored comparison](#scored-comparison) as what the route
  would have cost ([Answers of 2026-11-19](#answers-of-2026-11-19)).

The pros and cons of each candidate stand under
[Q1](#q1-which-reader-parses-the-module-btf). No candidate is chosen here.

### The vet artifacts this brief could not write

The skill asks for a vet report under `doc/audit/` when an evaluation crosses its threshold.
That report carries a fingerprint,
a lock,
the full evidence records and the pre-discovery check-in.
This brief allowed edits to this file only and allowed no questions,
so this record stands in place of that report,
and one of the two artifacts it owed is now written:

- the ledger,
  gates,
  source audits,
  arithmetic,
  sensitivity and the full discovery listing are here instead;
- the `doc/audit/tech-...-vet-YYYY-MM-DD.md` report is still owed before any dependency is
  adopted;
- the pre-discovery check-in is written under
  [The pre-discovery check-in of 2026-11-20](#the-pre-discovery-check-in-of-2026-11-20),
  with its subject,
  its settled forks,
  its source classes,
  its schedule,
  its options and the answer the project gave on that date;
- the finalist validation of the four eligible finalists is recorded under
  [Finalist validation](#finalist-validation):
  `btf-rs`'s own suite ran from its repository at tag `v2.0.1`,
  since its archive ships none;
  `ebeepf`'s suite ran from its published archive and from its repository;
  and `pahole` and `bpftool` ran through a repeatable harness.
  The `aarch64` target stays unrun for both libraries;
- discovery's terminal result is saturated with at least two survivors
  ([Discovery record](#discovery-record)),
  so the earlier reason a later round could add a candidate is closed:
  a new name now needs a fresh schedule rather than a finished one;
- no candidate is adopted and no crate is added to a manifest.

## Owed measurements

None of these ran. Each runs in a throwaway VM with a file-backed pool, never this host's pool and
never with the host's `/dev/zfs`, except items 5 and 10, which are device-free. Those two run on a
throwaway host build against copies of this host's files, open no ZFS device and load no module.
Item 5 feeds byte buffers to the loader. Item 10 calls the comparison entry point directly, which
is the route that skips the gate's device step, and reads copies of the version, `srcversion` and
BTF files rather than the live sysfs paths.
Each names its control, because a pass alone does not show the checker can fail.
Every measurement stays owed, and no item here touches this host's pool or modules.

1.  A compatible module passes, in the guest. Build or install a module 2.4.4 with BTF in the
    guest, run the built gate, and assert it reports compatible and that a real read request then
    runs. Control: the same binary on a module without BTF must refuse, so a pass is not vacuous.
2.  A changed layout refuses, in the guest. The patched arms start from the 2.4.4 source with
    `META` untouched, so the module still reports `2.4.4-1`; no step compares that string, so only
    the comparison can refuse these arms
    ([Answers of 2026-11-19](#answers-of-2026-11-19)). Both arms drop members the source uses, so
    each patch also removes or stubs every use of them: in `zfs-2.4.4` the two counters are read
    and written throughout `module/zfs/zio_inject.c`, from line 165 to line 789, and printed at
    `cmd/zinject/zinject.c:474, 511, 548`; `dds_flags` is written at
    `module/zfs/dsl_dataset.c:2891, 2893` and read at `lib/libzfs/libzfs_crypto.c:1827`.
    A kernel-module-only build avoids `cmd/zinject/zinject.c` and `lib/libzfs/libzfs_crypto.c`,
    and the kernel files still need the edits or the arm does not build.
    First arm, a member size: drop `zi_match_count` and `zi_inject_count` so
    `zinject_record_t` is 352 bytes, and patch the anonymous struct's pad to the literal
    `char zc_pad1[352]`. The pad's own expression is `sizeof (zinject_record_t) - 16`
    (`include/sys/zfs_ioctl.h:544-552`), so the second patch is what keeps the union at 368 and
    the total at 13,744; without it the struct shrinks to 13,728 and the module fails its own
    `_Static_assert` instead of loading.
    Assert the gate refuses at `Member`, in the union's interior pass, with the path of
    `zc_inject_record` at 13,296 bytes (106,368 bits), expected 368 bytes and actual 352, while
    the struct's total size and the defer offsets match.
    Second arm, a member no offset shows: drop `dds_flags` from `dmu_objset_stats`. Nothing moves,
    the nested struct stays 288 bytes and the total stays 13,744.
    Assert the gate refuses at `Member`, in the first interior the descent reaches, naming
    `dds_flags` missing at 2,296 bits.
    Control for both arms: the unpatched module from the same build passes the same binary.
    Third arm, only if the guest's packaged 2.2.2 module carries BTF: it reports
    `2.2.2-0ubuntu9.5`, and no step compares that string, so the layout comparison runs on it
    without a seam. Assert the refusal is the container step at `Type`, naming the member found
    where the anonymous union was expected (`zc_inject_record`), which is the first refusal under
    [The comparison](#the-comparison)'s walk order. The first version needed a test seam that
    skipped the release rule to reach this arm; the 2026-11-19 answer removed that rule, so the
    seam goes with it and the arm now proves the comparison, not the rule.
    Control: the unpatched 2.4.4 module that carries BTF passes the same binary in the same guest.
    Every arm's refusal names the field and both values.
3.  A missing module BTF refuses and falls back, in the guest. Use the tarball build path that
    skipped BTF (`guest-openzfs-build.log:834-838`) or hide the base file from the checker's
    namespace, and assert raw requests are disabled, that the fallback is offered for an unused
    slot, and that registrations and recovery state survive.
    Control: the same guest after providing the base and rebuilding with BTF passes.
4.  A missing or unreadable file refuses, in the guest. Hide `/sys/kernel/btf` from a mount
    namespace or make a copy unreadable, and assert the refusal names the file and the step.
    Control: the same binary in the same guest with the file readable passes.
5.  A short read refuses, and this host's pair passes, device-free. A test harness fed a truncated
    module and base pair must refuse with the section or header step, not stall; the unit test and
    the fuzz target carry this, and the guest arm repeats it through the product's loader only if a
    seam exists.
    The same device-free harness runs the comparison on copies of this host's two BTF files, the
    module reporting `2.4.4-1` with `srcversion` `A3553C308C29FD4B6595A72`, and asserts it reports
    compatible with both `hdr_len` 32.
    That is the only arm before step 23 that asserts a pass on the 32-byte-header shape, because a
    6.8 guest's tools are expected, unmeasured, to write the 24-byte one; item 10 feeds copies of the
    same pair to the comparison, which reaches the shape without asserting a pass on it.
    Its negative controls are the same two files with one byte of the magic changed, which must
    refuse at `Header`, and with the byte edit that moves the two members' offsets as item 16's
    patch would, which must refuse at `Member` with both offsets.
    Control: the untruncated pair passes.
6.  The 24-byte header passes in the guest, and the 32-byte one device-free. A module whose BTF an
    older tool emitted with the 24-byte header, and a module whose BTF reports `hdr_len` 32 (the
    host's kernel is `7.3.5-1-cachyos`, unqualified for the product until step 23), must both parse
    and compare.
    The 24-byte case's route is item 11's: copy `/sys/kernel/btf/vmlinux` into the guest's kernel
    build directory before the module build, which is what step 2's fix round did.
    A 6.8 guest's tools are expected, unmeasured, to emit no 32-byte file, so that shape is asserted
    by item 5's device-free arm rather than here.
    Control: a file with `hdr_len` 16, and a file whose sections overlap, must refuse.
7.  The request contract is checked, in the guest. Renumber one `zfs_ioc` value in a patched module
    and assert the refusal names the request and both values. Control: the unpatched module passes,
    and the patched module still has the same struct layout, so the refusal can only come from the
    request comparison.
8.  Architecture and endianness refuse, in the guest. On an `aarch64` guest (unqualified) the gate
    must refuse before any ioctl; on a big-endian BTF fixture the endianness step must refuse.
    Control: the `x86_64` little-endian guest passes.
9.  A module reload between reads is caught, in the guest. Unload and load a different module
    between the gate's first and second identity reads, through a test seam or a scripted guest,
    and assert the checker retries once and refuses only when the retry's own two identity reads
    differ as well.
    A second arm unloads and loads between two uses of the cache in one long process, and asserts
    the identity recheck re-runs the check rather than returning the stored verdict, since a sysfs
    modification time does not track a load.
    Record also that a replacement which keeps the version string, `srcversion` and the build ID is
    invisible to this step, so the arm can show it only through a seam that observes more than
    those bytes, as [The read order and the reload race](#the-read-order-and-the-reload-race)
    states.
    Control: a single load passes, and a reload that has settled before the retry passes, which is
    the arm that shows the retry is not a refusal in disguise.
10. Start cost and memory, device-free on a throwaway host build. Call the comparison entry point
    directly, with the file paths injected, which is the route that runs below the gate's device
    step, and feed it copies of this host's BTF files. Measure the comparison's wall time and peak
    memory against those copies, interleaved with a same-binary control that runs the same code
    without the ZFS path, and report the ratio and the run-to-run spread, per the repository's
    timing rule. Measure the base parse and the module parse separately, and the handler's start as
    a one-process-per-ZED-event child as well as the wrapper's longer process, because the two pay
    the parse differently; those two runs need the device step stubbed in the throwaway build, or
    each stops there and never pays the parse.
    Control: the no-op path in the same round, and a run over a copied version file only, which
    reads no BTF and so separates the read cost from the parse cost.
    Failing threshold: the arm fails if the added wall time's median exceeds 100 ms, or if the
    resident peak exceeds 256 MiB for the pair of files measured here. Both numbers are this
    writer's proposal and not a measurement. The one reading behind them is the gate review's
    single probe run, which reported a base parse of 42.8 ms, a whole run of 49.3 ms and a
    `VmHWM` of 57,808 kB, with no control and no spread, in
    `notes/btf-rs-shape-probe-output.txt` under
    `/home/user/temp/agent/zfs-runtime-layout-checker-design-gate-review-2026-11-19/`.
    One run without a control is not a finding under the repository's timing rule, so the
    thresholds sit above that reading as a guess, and this item is the run that replaces them.
    The peak threshold and the base BTF cap under
    [What a writer can settle without asking](#what-a-writer-can-settle-without-asking) are one
    decision: at the eight bytes of memory per input byte that reading suggests, a 64 MiB base
    could reach about 512 MiB, past this threshold, so this item sets each of them and
    [Q3](#q3-may-a-verification-be-cached-across-processes)'s trigger is this same threshold.
11. The guest fixture for step 6, in the guest. The tarball module the design's guest built carries
    no BTF, so the design's own fixture still needs the route: repeat item 1 with a packaged module
    that carries BTF or with a tarball build given a `vmlinux` in its build directory, and record
    which route produced the BTF and its release string.
    Control: the same guest's module built without BTF, which must refuse with the missing-BTF step
    and offer the fallback.
    The requirement this item holds step 2 to is the plan's, not this design's: step 2's layout
    comparison test reads the loaded module's BTF file against the vmlinux base, "The layout
    comparison test against the running module's BTF"
    (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`, line 443 on `main` at
    `6c838ba11f`) and "Verify exact ioctl values and Rust layout against the running module's BTF"
    (`:468`), so the test compares the file the gate reads and not the module's DWARF.
    It does not require step 2 to use this design's reader; that choice is step 2's.
    Where step 2 stood at `08c9dea25f`, read at 2026-11-19T23:04:00-05:00. That commit was its
    branch's tip when that read was taken, and the integrate branch's merge `58a1bba2f5` has it as
    its second parent; it is not on `main` at `6c838ba11f`.
    Its recipe copies `/sys/kernel/btf/vmlinux` into the guest's kernel build directory before the
    build, asserts `/sys/kernel/btf/zfs` exists after `insmod`, and the layout test reads that file
    with `pahole --btf_base /sys/kernel/btf/vmlinux`. Its record says `KBUILD_BUILTIN` "decides when
    the command is re-triggered, not whether it runs" (record lines 163 to 165), which is the
    Makefile reading under [When the module carries no BTF](#when-the-module-carries-no-btf), and
    its kept recorded run reports this layout line (record line 204) on `6.8.0-147-generic` (record
    line 56), with the module at `2.4.4-1` (record lines 44, 70 and 72):
    `layout zfs_cmd bytes=13744 dmu_objset_stats=288 zinject_record=368 drr_begin=304 source=/sys/kernel/btf/zfs`
    `main`'s handover records that fresh guest run and the recipe's two steps, at lines
    40200 to 40223 of the file at `6c838ba11f`, and it holds no copy of the record itself.
    The fix round began at `4c7a347ea2`, where the test read the module's DWARF and the record gave
    `KBUILD_BUILTIN` as the reason the guest could produce no module BTF.
    The guest's own `/sys/kernel/btf/vmlinux` is recorded: the step 2 short review's probe read it as
    raw BTF with magic `9f eb 01 00` and copied it into the build directory, which is the `cp` the
    recipe now runs and which fails there without the file under `set -eu`.
    The owner question belongs to the record at `4c7a347ea2`, which asked whether step 6's gate needs
    the BTF file. The accepted gate answers it, since a missing required BTF disables raw requests
    and "kernel/module BTF layout" is one of its proofs (see
    [What the checker must prove](#what-the-checker-must-prove)), and the record at the tip does not
    ask it.
    This item stays open until a run of the route lands on `main` for the design's own fixture:
    step 2's branch carries one, and `main` carries the handover's summary of it.
    The revision that answers the diff-only gate check read the branch again at
    2026-11-19T23:40:09-05:00, when its tip was `69cef8fe3c`, four commits on from `08c9dea25f`;
    the integrate branch was still at `58a1bba2f5`, and the recipe's copy of
    `/sys/kernel/btf/vmlinux`, its `test -f` after `insmod`, the layout test's `pahole --btf_base`
    read and the record's layout line and `KBUILD_BUILTIN` sentence say what this paragraph states.
12. The refused state in `doctor`, in the guest. With a mismatched module loaded, assert `doctor`
    names the loaded module, the target and the file, keeps its exit status, and does not print a
    bypass; then with a compatible module assert it prints the compatible line.
    Control: both states in the same guest.
13. A tool-based checker's output and absence (Way 3), in the guest. With the tool's packaged
    version, run the tool arm on a compatible and on a mismatched module, and assert the refusal
    names the member and both values. Then remove the tool from the guest and assert the refusal
    names the missing program and its version, not the ABI. Repeat with a second tool version.
    Control: the same module with the tool present and matching passes.
14. The root handler's tool path (Way 3), in the guest. As root, run the handler with a writable
    `PATH` containing a decoy `pahole` or `bpftool`, and assert the handler uses a verified absolute
    path or refuses, never the decoy.
    Control: the verified tool at its absolute path passes, and the decoy is never executed.
15. The digest way's cost (Way 4), in the guest. Rebuild the same 2.4.4 source with a different
    compiler or flag set, record whether the module BTF bytes differ, and assert the digest refuses
    while the full layout check passes. If the bytes are identical, use a patched source change that
    keeps the layout. Control: the full check on the same two modules passes both.
16. The blind spots of Ways 2, 5 and 6, in the guest. Use the patched module whose
    `zinject_record` is 352 rather than 368, and a module with two same-size members swapped
    inside `zinject_record`: declare `zi_duration` before `zi_iotype`, both 32-bit, so the record
    keeps 368 bytes, the union keeps 368 and the total keeps 13,744, while `zi_iotype` sits at
    13,620 bytes from the struct's start rather than 13,616. The full comparison compares
    `zinject_record` member by member, so it refuses there at `Member`, expected `zi_iotype` at
    108,928 bits and actual 108,960. The narrow comparison, the request probe and the
    release-string-only check accept both; the full comparison alone refuses them, and the gate as
    now designed has no release step beside it
    ([Answers of 2026-11-19](#answers-of-2026-11-19)).
    Control: the full comparison on the unmodified 2.4.4 module passes.
17. A release that keeps the layout and changes behaviour, in the guest. Take the 2.4.4 source and
    change one compared request's handler behaviour while leaving its request number, its key
    table and every struct alone: for example, make `zfs_ioc_destroy_snaps` treat a flag it
    ignores today as a refusal, or make `zfs_ioc_get_bookmarks` skip a bookmark it returns today.
    The module still reports 2.4.4-1, its BTF still matches the expectations, and its request
    numbers still match, so the gate must report compatible and let a raw request through.
    Assert exactly that, and record what the request then returns, because that answer is the
    blind spot's own evidence and not a defect in the gate.
    Control: the unpatched 2.4.4 module from the same build passes and answers as before, so the
    arm shows a difference the gate cannot see rather than a difference it missed.
    [A release that keeps the layout and changes behaviour](#a-release-that-keeps-the-layout-and-changes-behaviour)
    says what the gate would do and what, if anything, catches it.

## Questions for the owner

One question is answered in part and one is held.
Q1 was answered in its narrower form on 2026-11-19 and its pick is still conditional: it goes only
after the items its own text lists close
([Answers of 2026-11-19](#answers-of-2026-11-19)).
Q3 is held until [Owed measurements](#owed-measurements) item 10 runs, and nothing needs its
answer before then.
The Q2 and Q4 questions of the first versions are answered, and their answers are recorded under
[Answers of 2026-11-19](#answers-of-2026-11-19) rather than here.
Options are ranked, best first, each option says why it sits where it does, and the pick is the
owner's.

### Q1: which reader parses the module BTF

Answered in its narrower form, and the pick is still conditional.
The owner was asked "is it a hard requirement that the parser is our own code and the product
starts no second program" and answered "No, not required (Recommended)" on 2026-11-19
([Answers of 2026-11-19](#answers-of-2026-11-19)).
So no named hard constraint admits the in-house reader.
The skill's existing-tools rule keeps it ineligible.
The eligible order is options 1 to 4.
No option is recommended, and `btf-rs` 2.0.1 ranks first among the eligible options
on eligibility and not on validation: no hard gate fails on the source evidence read, and the
crate holds the split model, the type kinds and the tests the check needs, while this proposal's
pre-scan closes the residuals it names.
Finalist validation,
the first item that list held,
is closed under
[Finalist validation](#finalist-validation),
with the `aarch64` target unrun as H3's own limit.
The pick waits until these close:

- the `doc/audit/` vet report does not exist;
- the `aarch64` target has no build or run for either library finalist,
  which is H3's own limit;
- the start cost of the two libraries is unmeasured,
  so C7 rests on a low-confidence range ([Owed measurements](#owed-measurements)).

The scoring's baseline pair also flips or ties on one criterion at a time
([Sensitivity](#sensitivity)).
That section does not cover simultaneous multi-input changes.
That pair is `btf-rs` against the declined in-house reader.
The pair the live choice turns on is `btf-rs` against `ebeepf`,
and it reverses or ties on the same method too,
which is why the order rests on the maintenance and provenance evidence rather than on the
arithmetic.
The choice that remains is a library against a spawned program.

BTF (BPF Type Format) is the kernel's own description of its struct and enum layouts, published
as files under `/sys/kernel/btf`. The gate reads the kernel's base file and the ZFS module's file
and compares their field offsets and sizes with the ones the binding was built for. A dependency
here is a library the binding links, or a program the gate starts.

In plain words the choice is between options 1 and 2 and options 3 and 4.
Options 1 and 2 take a library under the GNU Lesser General Public License version 2.1 or later,
or its BSD arm,
which our `LGPL-3.0-or-later` product may link and redistribute.
Option 1's 2.0.1 release lags its own main branch on the widened BTF masks.
Option 2 is one release old and calls its own tree unreviewed AI-generated code.
Options 3 and 4 start `pahole` or `bpftool` at each gate run,
two separate programs the product must find and version,
one of them in the root handler.
A fourth route, about 350 to 450 source lines and 100 to 150 test lines of our own reader plus a
fuzz target (a test that feeds random and mutated bytes to the parser and fails on a panic or a
stall) and a review of it, stays priced under [Scored comparison](#scored-comparison) and is not
offered: the owner answered on 2026-11-19 that our own parser and no second program is not a hard
requirement.

Terms used here. "C6" is the integration and test cost criterion of
[Scored comparison](#scored-comparison), where a higher rating is cheaper. The "widened BTF
masks" are the newer header's 24-bit member count and 7-bit kind fields, against the 16-bit and
5-bit fields that the `btf-rs` 2.0.1 release still masks with. "The existing-tools rule" is the
choosing-technology skill's rule that a custom implementation is eligible only when every
existing tool fails a named hard constraint. "Split BTF" is a module's BTF, which extends the
kernel's base BTF instead of standing alone. A "finalist" is a candidate that passed every hard
constraint, and its "validation" is reading its source, tests, releases and upstream record the
same way as every other finalist's. A "`-C` printer" is an option that prints one named type's
members; a "type-id resolver" maps the numeric type references of a raw listing to the types they
name; an "array-size shim" is arithmetic the binding would add for array byte sizes that a library
does not give.

Scenario: a future kernel writes a BTF file whose member count needs the 24-bit field. With
option 1, the released masks misread it or this design's pre-scan (the header and section checks
made before any parser runs) refuses the file, so raw requests stay disabled and the command takes
the Git fallback until `btf-rs` publishes a release with the wider mask. With options 2 and 3 the
outcome depends on the installed tool's version, which this file has not checked.

Options:

1.  `btf-rs` 2.0.1,
    the first of the eligible options,
    with its finalist validation recorded in
    [Finalist validation](#finalist-validation).
    Pros: split BTF and the type kinds this check needs; pure Rust with default features off;
    a documented API and tests; the probe parses this host's files and walks the anonymous union.
    Cons: `LGPL-2.1-or-later` adds a copyleft-family dependency; no section validation in the crate,
    so the binding keeps the pre-scan; its release's masks lag its main branch; a small maintainer
    group;
    the published archive carries no tests,
    so its suite runs from the repository at tag `v2.0.1`.
    Cost: the shared shape, about 350 to 550 lines of source and 300 to 450 of tests, plus the
    array-size shim and one fuzz target.
    Why here: it is the only eligible library, since the skill admits a custom implementation only
    when every existing tool fails a hard constraint, and the owner named no such constraint on
    2026-11-19; 90 of 104, and a library needs no program the product must find and version.
2.  `ebeepf` 0.1.0,
    a second library,
    ranked after option 1.
    Pros:
    a split parser with a byte-level API and a path constructor;
    `size_of` already multiplies array elements,
    so the binding needs no array-size shim;
    pure Rust with the default features off;
    its suite and the design's checks ran
    ([Finalist validation](#finalist-validation)).
    Cons:
    a dual `LGPL-2.1-only` and `BSD-2-Clause` crate,
    so take it under the BSD arm;
    one release,
    one maintainer and no CI;
    its own README calls the tree AI-generated code that has not been thoroughly reviewed;
    its released masks are the same 5-bit kind and 16-bit count fields as `btf-rs`'s;
    it accepts a split table as its own base;
    and `parse_split` copies the base's types into the module's table.
    Cost:
    the shared shape without the array-size shim.
    Why here:
    87 of 104,
    three points behind option 1 on maintenance and provenance alone,
    and ahead of the tools because a library needs no program the product must find and version.
3.  `pahole` 1.32 as a start-time program,
    ranked after option 2.
    Pros: the command the design already names for the test-time comparison; `-C` prints the
    compared members; no new Rust dependency.
    Cons: a runtime package; a root-spawned program in the handler; a text format; an invocation
    each for the struct, the nested type and the enum.
    Why here: 60 of 104, and a runtime program costs the deployment and privilege points that
    options 1 and 2 do not.
4.  `bpftool` 7.8.0 as a start-time program,
    ranked after option 3.
    Pros: libbpf's reader; a documented base option and a raw type listing.
    Cons: the same runtime-program and root-path costs; the raw listing needs a type-id resolver
    in the wrapper before it can compare a member; the tool's version tracks the kernel's tools,
    not the distribution's.
    Why here:
    56 of 104,
    the lowest of the four options,
    for the resolver its output needs and
    its medium diagnostic rating.

Not offered: an in-house reader, because the owner answered on 2026-11-19 that our own parser and
no second program is not a hard requirement, which leaves it ineligible under the skill's
existing-tools rule while it stays priced under [Scored comparison](#scored-comparison);
`btfparse` (no split base), `aya-obj` (no base, ELF-oriented), `btf` 0.5.1
(unmaintained, no base), `btfdump` (no base, tool first), `padlock-dwarf` (no base, wrong fit),
`libbpf-rs` and `libbpf` (C build), and a BTF digest (the answer the project did not select).

Ranking:
option 1 > option 2 > option 3 > option 4.
Option 1 leads option 2 because both are libraries in the wrapper's address space,
and the pair's 90 and 87 separate on maintenance and provenance:
two releases,
a merged out-of-bounds fix and a CI workflow on every push,
against one release,
one maintainer and a self-declared unreviewed tree.
Option 2 leads option 3 because a library needs no program the product must find and version and no
binary the root handler starts,
and the arithmetic agrees at 87 against 60.
Option 3 leads option 4 because `pahole`'s `-C` output carries the compared members while
`bpftool`'s raw dump needs a type-id resolver before anything can be compared,
and the arithmetic agrees at 60 against 56.
The in-house reader has no place in this order: the owner's answer of 2026-11-19 left it
ineligible, and [Scored comparison](#scored-comparison) keeps its price as what the route would
have cost.

### Q3: may a verification be cached across processes

Held until [Owed measurements](#owed-measurements) item 10 measures the start cost.
Nothing needs the answer before that run: the per-process cache is open to veto as a writer's rule,
so the gate works the moment it is built, and a veto of that rule means parsing the two files on
every use rather than holding a disk cache.
The question returns only if item 10 measures the added time above its 100 ms median threshold or
the peak above 256 MiB. Those thresholds are this file's proposal, not a measurement: item 10
names the one reading behind them, the gate review's single probe run with no control and no
spread, which is not a finding under the repository's timing rule. Until then it returns with
these options:

Scenario: a person runs three ZFS commands in a row on one machine. With option 1 each command
pays the parse of the two BTF files; with option 2 the first pays it and the others read a file.

Terms used here. The "boot id" is the kernel's identifier for one boot, so a cache keyed by it is
discarded at the next boot. The "build id" is the identifier the linker stamped into the binary,
so a rebuilt binary is not mistaken for the one that wrote the file. A "verdict" is this gate's
answer, compatible or refused. "Root-owned" means a file only the root account can write, which
the root handler can trust as much as its own binary.

The "identity recheck" under
[The cache](#the-cache) is a comparison of the module's version, `srcversion` and build ID, and of
both BTF files' device, inode, size and modification time, before a stored verdict is reused.

1.  No disk cache, ranked first while the cost is unmeasured.
    Pros: no stale verdict, no new state, no new privilege question; the identity recheck stays
    inside the process. Cons: every ZFS command pays the parse.
    Why here: it is the only option that adds no new input to the gate, and nothing measured says
    the parse needs saving.
2.  A per-boot cache bound to the boot id, the module version, both BTF files' device, inode, size
    and modification time, and the binary's build id, ranked second.
    Pros: the parse is paid once per boot. Cons: a forged entry in the account's own cache would
    enable raw requests on a mismatched module, so the file must be root-owned for the handler and
    validated before use, and that validation is the identity check again.
    Why here: it saves a cost not yet measured, and it needs a root-owned file and an extra trust
    argument before it saves anything.

Ranking: option 1 > option 2 while the cost is unmeasured, because option 1 adds no new input to
the gate and no new trust argument, while option 2 saves a cost nothing has measured and needs a
root-owned file plus the identity check it must re-run.

The wrapper's disk cache of the `git version` text is not a precedent for this one: it holds a
descriptive string, and a wrong entry changes a warning, while a verdict here gates raw requests
([The cache](#the-cache)). That difference in trust class, not the parse cost, is why the first
version made this choice; the cost is what may reopen it.

## What a writer can settle without asking

The rules marked here as open to veto are the writer's, each with "veto none" as its default: the
folder's name, the `doctor` sentence's exact-string trigger, the absent-identity rules and the
per-process cache.
Each says so where it stands, and the sections that use them say so too.
The disk cache is not among them: [Q3](#q3-may-a-verification-be-cached-across-processes) holds it
for the project, and this design does what option 1 says until that question is answered.

- The seam's shape, in the crate the owner named: the binding package holds the loader, the
  pre-scan, the expectations, the comparison and the reader, and the wrapper supplies the paths,
  the target allowlist and the wording
  ([Answers of 2026-11-19](#answers-of-2026-11-19)).
- The files' folder inside the binding, `src/compatibility_check/` with `uses/layout` and
  `uses/requests`, because a file in `layout` that read the request list would make two folders use
  each other ([The seam and the placement](#the-seam-and-the-placement)).
  The crate root links it from `src/uses/compatibility_check`, as it links each folder `src/lib.rs`
  declares, and the folder lists the refusal value before the loader, because the loader builds the
  refusal's `Header`, `BaseBtf` and `ModuleBtf` steps.
  The name is open to veto, with "veto none" as the default.
- That step 6 now touches two packages, so it owes both gates: the binding package's own tasks, its
  `cargo nextest` run and the type gate `package/jsonc-edit/core-rust` defines, and the wrapper's
  container gate for every change under `package/git-policy/cli`
  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`, "Where the tests run",
  lines 1984 to 1988).
- That the loader's fuzz target lands in the existing sidecar
  `package/zfs/request-to-openzfs-kernel-device.fuzz`, beside the crate, per rule `SBS`.
- That a dependency taken in the binding is added through `rust-catalog.toml`, whose
  `file-enforcer` output is the crate's `[workspace.dependencies]` table, never by editing that
  table by hand (rule `WC2`).
- The gate runs after the device check and before the first ioctl; `doctor` runs the facts without
  the device check.
- The read order and the second identity read, with one retry, as
  [The read order and the reload race](#the-read-order-and-the-reload-race) states. Those identity
  reads are the module's version, `srcversion` and build ID, taken before and after the BTF reads.
- No release rule: nothing compares the module's version text with an approved string, the text is
  identity and reporting only, and `doctor` prints it with the sentence that says when it is not
  the tested `2.4.4` ([Answers of 2026-11-19](#answers-of-2026-11-19)).
- The sentence's trigger: `doctor` compares the string with the tested module's exact text, so a
  checkout build's longer string gets it, and a prefix test does not; the answer records no matching
  rule, so this is open to veto, with "veto none" as the default ([The refusal](#the-refusal)).
- An absent identity attribute degrades the reload check instead of refusing, while a missing device
  or BTF file refuses ([The read order and the reload race](#the-read-order-and-the-reload-race)).
  When all three values are absent the reload check has no usable term and the cache keeps the
  verdict it holds while each file's device, inode, size and modification time hold.
  Both rules are open to veto, with "veto none" as the default.
- The pre-scan rules, including the sorted-by-offset section check, the widest-encoding walk and the
  refusal of a value the chosen reader cannot represent.
- The file bounds, as proposals pending [Owed measurements](#owed-measurements) item 10:
  a page for each identity file, 64 MiB for the base BTF and 16 MiB for the module BTF.
  The caps are this writer's multiple of the sizes measured here (6,579,400 and 570,234 bytes), and
  the multiple is unexplained until item 10 sets the constant, so it is not a shipped limit yet.
  An ordinary debug kernel fits, but the only memory measurement is that reading and caching the
  pair of 7,149,634 bytes peaked at 57,808 kB resident, about eight bytes of memory per input byte,
  so a 64 MiB base could reach about 512 MiB, past the 256 MiB failing threshold item 10 states.
  Item 10 sets the peak threshold and this cap together. A file that exceeds its cap is a refusal.
- The compared set: the struct size, every member of `zfs_cmd` with its offset and size, the
  anonymous containers and their members, every member of `dmu_objset_stats` (including
  `dds_flags`) and of `zinject_record`, the base integer sizes, and every request constant plus
  `ZFS_IOC_FIRST` and `ZFS_IOC_LAST`.
  The interiors of `drr_begin`, `zfs_stat_t` and `zfs_share_t` stay unchecked beyond their own size
  and offset, as [Limits](#limits) says.
- The expectations come from the Rust `#[repr(C)]` type where `offset_of!` reaches, with literals
  for the anonymous containers and union members, guarded by an exhaustiveness test.
- The refusal value, its steps, the order the comparison walks and the words the caller and
  `doctor` print, including the loaded module, its release string, the target, the file and the
  first mismatch, and no bypass.
- The per-process `OnceLock` and the identity recheck, open to veto with "veto none" as the
  default; the alternative is parsing the two files on every use. The disk cache is Q3's, held on
  item 10.
- The architecture allowlist: `x86_64` only until an `aarch64` guest qualifies the managed route.
- Endianness from the BTF magic against the target.
- The test placement and fixtures: device-free unit tests with small generated BTF blobs, the
  exhaustiveness tests, the diagnostic tests, the `doctor` rendering test, and the fuzz target for
  the loader and pre-scan.
  The device-free arm under [Owed measurements](#owed-measurements) item 5 runs the comparison on
  copies of this host's two BTF files, and it is the only passing assertion the 32-byte header gets
  before step 23.
- Step 2 picks its own reader for the layout test: the plan leaves that choice to step 2
  ([Dependencies and their state][plan-deps], lines 126 to 132), where the design's command is
  `pahole --btf_base`. If Q1's reader serves both, that is step 2's choice to make; this design
  does not assign it.
- Step 2's layout test reads the loaded module's BTF file with the vmlinux base. That is the plan's
  requirement of step 2 and not this design's, because the test then compares the file the gate
  reads and not the module's DWARF ([Owed measurements](#owed-measurements) item 11 cites the
  plan's lines).
  At `08c9dea25f`, read at 2026-11-19T23:04:00-05:00, its recipe copies `/sys/kernel/btf/vmlinux`
  into the build directory and reads the module's BTF file; at `4c7a347ea2` its test read DWARF.
  [Owed measurements](#owed-measurements) item 11 is the run that records the design's own fixture's
  route.
- The refusal's interaction with the design's fallback for an unused slot and the failure path for
  a bound clone; the design decides the behaviour, the writer writes the words.
- The check's placement in step 6, beside the start probe, and the step-2 test remaining the
  proof that the compiled layout matches the pinned headers.
- No external program if Q1 picks a library, no C build, no `zfs` invocation, no `/dev/zfs` read
  for the layout check itself, and no force flag or environment override.
  If Q1 picks a tool, the external program is that tool and
  [Owed measurements](#owed-measurements) items 13 and 14 qualify it.

## What this proposal does not do

- It ran no guest, no built product and no measurement in
  [Owed measurements](#owed-measurements); every one is unrun.
- It changed no file except this one.
- It made no choice that belongs to the owner. The owner's answers of 2026-11-19 settled the
  reader question's narrower form, the release string and the crate placement, and this revision
  applies them ([Answers of 2026-11-19](#answers-of-2026-11-19)). The per-process `OnceLock` and
  its recheck stand open to veto with "veto none" as the default, and
  [Q3](#q3-may-a-verification-be-cached-across-processes) holds the disk-cache question behind a
  measurement.
- It edited neither the design proposal nor the implementation plan, and
  [the amendment these answers owe](#the-amendment-these-answers-owe-the-proposal-the-plan-and-the-binding-readme)
  writes out the text they need.
- It does not design the start probe or the staging folder, the other two blocking designs.
- It contains no timing claim and no forecast.

## Links

[design-open-items]: cli-git-zfs-worktree-design-proposal.md#open-items-the-decisions-leave
[design-project-decisions]: cli-git-zfs-worktree-design-proposal.md#project-decisions
[design-binding]: cli-git-zfs-worktree-design-proposal.md#binding-crate-and-compatibility
[design-requirements]: cli-git-zfs-worktree-design-proposal.md#requirements-and-module-ownership
[design-record]: ../decision/cli-git-zfs-worktree-design.md
[vet]: ../audit/tech-rust-crate-for-openzfs-vet-2026-11-18.md
[skill]: ../../.agents/skills/choosing-technology/SKILL.md
[design-abi-gate]: cli-git-zfs-worktree-design-proposal.md#abi-gate-and-tests
[design-doctor]: cli-git-zfs-worktree-design-proposal.md#what-doctor-checks-and-prints
[design-decided]: cli-git-zfs-worktree-design-proposal.md#decided-behaviours-with-no-test-yet
[plan-file]: cli-git-zfs-worktree-implementation-plan.md
[plan-deps]: cli-git-zfs-worktree-implementation-plan.md#dependencies-and-their-state
[plan-open-items]: cli-git-zfs-worktree-implementation-plan.md#open-items-and-the-step-that-needs-each
[plan-start]: cli-git-zfs-worktree-implementation-plan.md#what-can-start-now
[vm-fixture]: cli-git-zfs-worktree-design-proposal.md#vm-measurements-preceding-acceptance
