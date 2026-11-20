# The start probe for the ZFS worktree route

**Non-final, unapproved proposal by an agent.** This document designs one open item of the accepted ZFS worktree
design, "The start probe a kernel-release answer asks for", so that step 6 of the implementation plan can be
written. A Pi session wrote it on 2026-11-19 and revised it in the rounds listed under [Revisions](#revisions); it
is not accepted. It was written without running a VM, a probe or any pool command, from the sources listed under
[Sources read](#sources-read). What it checked: the
accepted design, the plan (every section list and step it names) and the measurements record, the seven answers the
question round returned and the threat-model sentence recorded beside them, OpenZFS `zfs-2.4.4` sources, the
running kernel's exported system calls, this host's `include/uapi/linux/mount.h`, systemd v262 `service.c`, `unit.c`
and `exec-invoke.c`, Linux v6.8 and v7.3 `fs/super.c`, `fs/fsopen.c` at v5.2, v6.5, v6.6 and v7.3, Linux v7.3
`kernel/user_namespace.c`, `include/linux/user_namespace.h`, `kernel/ucount.c`, `kernel/nsproxy.c`,
`kernel/capability.c`, `kernel/fork.c`, `fs/proc/namespaces.c`, `fs/namespace.c`, `fs/namei.c`, the SELinux hooks
`security/selinux/hooks.c` and `security/selinux/avc.c`, and `mm/shmem.c`,
Ubuntu 24.04's release notes for the user-namespace restriction its default AppArmor profile applies, and the
wrapper's own code (fetched, because this host's 7.3.5 tree ships no `fs` sources). No kernel source
was read for the first draft, which is how a non-existent mount attribute and a wrong `errno` survived it; the
kernel statements under [The kernel the design names](#the-kernel-the-design-names) name the version each was read
from. What stays open: every measurement under [Owed measurements](#owed-measurements), none of which ran. The
questions this design put to the owner were answered on 2026-11-19, and [Answers of 2026-11-19](#answers-of-2026-11-19)
records each one with the option the project chose; the body beside it follows those answers. The subject of that
acceptance is the ZFS worktree design, and it is accepted with its open items still open; this file changes no
decision of it. Nothing here authorizes work on this host's pool.

## Revisions

Each revision is labelled with a time and names the commits the round made before it, and each entry says whether
its label is a commit's own time. A label written into a commit cannot name that commit's own time or
hash, so the commit carrying an entry always comes after its label. The entries labelled `18:02:41` through
`20:31:12` carry the `%cI` of a commit their round made. The entry labelled `20:58:00` carries a committer date that
round pinned on `e850455f55` and `3dff37b556`, whose author dates are `20:57:31` and `20:58:12`, so it is not the
time they were made. The entry labelled `21:44:02` carries the real `%cI` of `1a9c0b27b2`, a time that round also
pinned on the label and wrap commits after it. The entry labelled `22:37:46` states the worktree's reflog times,
because the commits it lists carry `23:20:00` as their `%cI`. The entry labelled `23:20:02` is the time its text was
first written.

- `2026-11-19T18:02:41-05:00` (commit `4167f50724`): the first draft.
- `2026-11-19T18:40:12-05:00` (`eee32e31fd`): the answer to a gate review of that draft, by the commit that says
  so.
- `2026-11-19T18:41:09-05:00` (`fa52e2e948`): the mount header fields cited at their exact lines.
- `2026-11-19T19:27:20-05:00` (`45aca9ab4a`): the answer to a first gate re-check.
- `2026-11-19T19:33:27-05:00` (`370c6cb9e2`): the start-probe arms, the txg window and the owner questions
  tightened.
- `2026-11-19T20:31:12-05:00` (`10f2435d34`, `e205be87f7` and this entry's commit): the answer to a second gate
  re-check. Its findings were weighed one by one, and every one earned a fix, with none refused: the duplication in
  the source question, which listed one option twice; the start question, which was a confirmation with one option and
  moved to the writer's list; the move and unmount arm, which could not unmount while the worker held references on
  the mount; the holder protocol, now stating every pipe and its endpoints, the parent-death race and the
  process-group call; the txg window, which now carries a start fence, an end fence and an idle control; and
  `ExecStartPre=-`, which keeps a probe crash from skipping the reconcile. Also fixed: the marked options and the
  costs charged to one way, the sanoid facts the source question left out, the probe dataset's layout facts, the
  fold-in that cost nothing, the scope of the "never opens `/dev/zfs`" statement, the teardown arm's own unmount,
  two option sentences, the control-directory path, the unnamed handler modes and the prose the checkers do not
  count. The measurements this round added are under [Owed measurements](#owed-measurements).
- `2026-11-19T20:58:00-05:00` (commits `daa7a93f67`, `01e51e24c1`, `e850455f55` and this entry's commit): the
  owner's answers of 2026-11-19 applied. A new "Answers of 2026-11-19" section states each question, the option
  the project chose with the round's timestamp range, the option it did not take and why, and the threat model the
  answers assume. The question sections and the ways this design weighed are gone, leaving a pointer to the answers
  and one shape. The body follows the answers: the source is the dedicated dataset
  `zpcachyos/nosnap/git-worktree-probe` with its `ready` snapshot, so the account-influenced name, the
  `.zfs/snapshot` read and the reuse-skip go; the move and
  unmount arms stay in step 6; a failed arm refuses new attaches only; an unexercised arm blocks the route; a
  missed deadline fails closed. New design work: the account companion, with its namespace setup, its arms, its
  refusal, its skip, its printed lines, its per-add cost and the measurement that sizes it, and the `first-attach`
  record, with its writer, its claim, its staged rename, its keys, its reader and the steps that carry it. A new
  "Notes for the plan" section names what each answer adds to steps 4, 6, 7, 8, 10, 18, 19, 20 and 22. The
  measurements gained M11 and a control for M8; M1, M2, M6 and M10 follow the dedicated source, and the
  access-time and account-influenced-name work they gated is deleted. The three checkers ran on the design and on
  a broken copy, each failing there and passing here.

- `2026-11-19T21:44:02-05:00` (commits `92503d3f71`, `5af5c0fbc9`, `b5a635e448`, `d457df96b0`, `a11d124b64`,
  `586177747c` and `1a9c0b27b2`, and the label and wrap commits after them, this one included): the answer to a gate
  review of the applied answers. Its findings were weighed one by one, and every one earned a fix, with none refused.
  `doctor` now runs the account companion under the shared preflight, so `doctor` prints the companion's own outcome
  beside the probe's record. The companion's skip is this design's own choice, with its reason and the alternative,
  and marked open to the project's veto. Its child's start is specified: a fork on the blocking pool whose child runs
  a raw-syscall island, because the source guard forbids the re-exec the review offered, with `setpgid` called at once
  and the scratch directory placed outside the managed tree. `first-attach` gains its step names, `errno none` for a
  check, the rule that a refusal at the key check takes no claim, `0644` writes both ways, a path open, and the
  ordering a later sweep really has. The deadline's value is sourced from step 6's own M4 run and step 10's own M11
  run, with what both steps do until those exist, and a timed-out record names the arm in flight. The plan notes were
  rechecked against the plan as main has it, now cite steps by heading, and carry the duties of steps 8, 14, 16, 18,
  19 and 20 and each measurement's step. The lows: the skip errno set checked against the kernel source (`ENOSPC`
  since Linux 4.9, `EACCES` from SELinux and AppArmor, no `EUSERS`, with each hook's lines read in the v7.3 file),
  the effective Linux 6.6 floor with its amendment note, the dead `.zfs/snapshot` and `atime` facts removed, the
  ninth exclusion's amendment note, the header's subject, the answer record cited with a commit line, and the two
  host premises of the source section and of its assumptions checked at setup rather than asserted. The three
  checkers ran on the design and on a broken copy, each failing there and passing here.

- `2026-11-19T22:37:46-05:00` (commits `ac8a46addb` at 22:28:26, `8bfdcd546e` at 22:32:59, `0536918c5a` at
  22:33:58, `85cadbb152` at 22:34:15, `e7326f2bc2` at 22:34:24, `fe5271211e` at 22:34:37, `a7c8686afa` at 22:34:50,
  `8d9d1cb280` at 22:35:06, `32583a3331` at 22:36:15, `5ea3a66008` at 22:36:52 and `4258b3c3c9` at 22:37:46, all on
  2026-11-19 and all `-05:00`, and the small text, wrap, naming and label commits after them): the answer to a
  re-check of the applied answers. Eleven commits are listed. The ten from `8bfdcd546e` through `4258b3c3c9` carry
  author and committer dates that were set by hand to `2026-11-19T23:20:00-05:00`, which is not when they were made,
  so `git log --format=%cI` shows a value that was never the time of the commit; `ac8a46addb` carries its own real
  dates, 22:28:26 as author and committer. The times this entry states are the worktree's own reflog times for all
  eleven, and a corrective commit comment on GitHub names the same ten pinned commits. Its findings were weighed one
  by one and every one earned a fix, with none refused.
  `first-attach` then took its claim at the first attempt that passes the key check, and that one rule was stated in
  the claim sentence, the step order, the M8 control and the operation-outcome sentence, with the reason for that
  boundary and the key check the only step outside it. The companion's endings gained a class and print line each: a
  pass, an arm failure, a skip, a deadline expiry, a refused `fork` under the wrapper's retried
  start, an unusable scratch directory, a refused mountinfo read past the wrapper's own bound, and a death by
  signal, with `doctor` printing what it cannot prove as `unknown` and saying its run is a fresh one; that set was
  not closed, and the fifth revision closes it. The skip's
  boundary moved to the end of the namespace setup, because Ubuntu 24.04's default AppArmor profile, on the guest
  every measurement runs on, grants the user namespace and denies the capabilities inside it, where its first
  mapping write answers `EPERM` (a reading of mainline `kernel/user_namespace.c:986` through
  `kernel/capability.c:450` to `:458` and AppArmor's `apparmor_capable` at `security/apparmor/lsm.c:198` to `:210`;
  the fifth revision corrects this sentence, which then named `unshare(CLONE_NEWNS)`), and the wrapper's own test
  control already treats a failure anywhere in that setup as unavailable. M11 gained that restricted guest, a
  test-only switch and a test deadline, and M6 says it is a separate arm from plan step 3's H1 gate 7. The lows: the
  companion child's `_exit` on every path, its `close_range` sweep and the reaper for a child `SIGKILL` cannot end,
  its parent-death signal with the pool's keep-alive; the re-exec bar cited from its decision document and a sentence
  on why no separate helper executable is proposed; the sweep that proves a directory dead before removing it, with
  the kernel rule that makes that care necessary, and the PID-namespace and birth-identity guards beside it; a
  heading and name for the pipes and parent-death protocol, so no pointer by position remains; arm 8's move, unmount
  and closing scan each on a status line of their own; the LSM hook cites; the path open named as a departure from
  the receipts rule; the `doctor` write stated; the two veto-open choices taken out of the writer's list and named in
  the questions section; and the deadline's margin sentence. The plan notes were rechecked against main as it stands
  (`59152e567b`), and a fourth amendment, for plan step 22, joins the three. The three checkers ran on the design and
  on a broken copy, each failing there and passing here.

- `2026-11-19T23:20:02-05:00` (commits `467efadbaa`, `98e9d232cf`, `bac8e1227c`, `1f4e5d2d29` and `7b9f0fe62c`, and
  every commit this round made after it): the answer to a narrow gate check of the fourth
  revision, read on Claude Sonnet from the diff `c134086edd..4258b3c3c9`. This label is the time the entry's text was
  first written, not a commit time, and it names no commit of its own: the commit that carries it, the wrap and label
  commits after that one, and this entry's own hash all come after it, so none of them can be named here. The
  check's findings were weighed one by one: every one earned a fix, and one part of a low was dismissed after
  reading the source. The three mediums: `first-attach`'s claim now covers the descriptor walk and the slot's
  first-observation record, each with a step name, so no refusal inside the claim reaches the reader unnamed, and
  M8 gained the nested-mount control that must write `step walk` with `EXDEV`; the companion's endings gained the
  refused blocking-pool thread, a failed pipe and a child that ends with no status line, the fork now rides the
  wrapper's budgeted runner and its one-shot claim, and the two sentences that still described the old set now say
  what `doctor` prints; and the skip is drawn on the refusals that mean this host refuses the user namespace or the
  capabilities inside it, each errno named with the kernel line that answers it, so a defect in the companion's own
  setup code is an arm failure, and the veto sentence states which hosts skip. The lows: the Ubuntu `EPERM` is the
  first mapping write, read from mainline `kernel/user_namespace.c:986`, and M11 records which call it named; the
  M11 switch is a `CLI_GIT_TEST_ONLY_*` variable compiled into the release executable and inert unless set, per the
  project's own decision, and M8 and M9 name the same shape; `doctor`'s read-only limits state the one directory the
  companion makes under `$XDG_RUNTIME_DIR`; the scratch sweep is stated as the wrapper's own test with the
  parent-death signal bounding the child, the second child carries a signal of its own, a child `SIGKILL` cannot end
  keeps its directory until the wait returns, and a name from another PID namespace is judged by its record's age;
  the two plan locators that were main's now name their step and text, and the notes state main's plan as it is now;
  M6 and a new step 3 note require the user-namespace setting at 0 so neither arm passes for the wrong reason; and
  the questions section names all three veto-open choices. The plan notes and the four amendments were rechecked
  against main at `8b1cec7883`, read fresh, and every heading they quote is verbatim there. The one dismissal:
  `fs/namei.c` `detach_mounts` is at `:5521` in v7.3, so the citation `:5502` to `:5521` stands and needed no
  edit. The three checkers ran on the design and on a broken copy, each failing
  there and passing here.

## Answers of 2026-11-19

Every question this design put to the owner was answered in the question tool between 2026-11-19T20:35 and
2026-11-19T20:43:51-05:00. The round was first written down in `doc/handover/cli-git-rust-implementation.md` in the
main checkout: as of main's `92417f3758` it sits at line 40398, inside a paragraph and not at its start, and the
words "THE ROUND WAS ASKED" are inside that paragraph. A handover is pruned once its work lands, by rule `H0P`, so
this section, not that paragraph, is the durable record of the answers and of the threat model quoted under
[The threat model](#the-threat-model-these-answers-assume). This design now follows the answers: where its own
ranking differed, the answer wins, and the body, the record, the unit, the measurements and the plan-step notes
take the chosen shape.
Each entry gives the question, the option the project chose in the question tool's words, the round's timestamp
range, and one sentence naming the option it did not take and why, in the project's terms.

### Q1: whether the plan bars the probe's unmount arm

Chosen: "Product mounts only (Recommended)", in the round of 2026-11-19T20:35 to 2026-11-19T20:43:51-05:00. The
plan's sentence that no unmount runs before the third design covers the product's mounts, so the probe's own move
and unmount arms are built in step 6, inside the namespace and the tmpfs the probe makes for itself. Not taken: the
literal reading, that the sentence covers the probe's unmount too, which would build the arm in step 9 and leave
step 6 shipping a gate whose unmount half never ran.

### Q2: what a failed arm disables

Chosen: "Refuse new attaches only (Recommended)", in the round of 2026-11-19T20:35 to 2026-11-19T20:43:51-05:00. An arm
that ran and failed refuses new managed attaches: an unused slot takes the same-dataset reflink or the ordinary Git
route, `boot-reconcile` attaches nothing and keeps every intent and registration, removal of a clone that is not
mounted stays available, and removal of a mounted clone is refused only when the failed arm is the move, the unmount or
the scan. Not taken: mirroring the module gate, which would refuse every ZFS operation and strand the removal of an
unmounted clone until the kernel changed, the availability cost the allowlist answer was chosen against; and
warn-and-continue, which would proceed into the attach the probe had just seen fail.

### Q3: whether a setup-time probe dataset exists

Chosen: "Dedicated dataset (Recommended)", in the round of 2026-11-19T20:35 to 2026-11-19T20:43:51-05:00. The source is
the dataset `zpcachyos/nosnap/git-worktree-probe` with the empty snapshot `ready`, and the plan's steps 18 to 20 carry
its creation, its configuration field and its exclusion. Not taken: a snapshot of the checkout, which needs no layout
change, and which this design did not keep because the source can be absent right after a migration, the account holds
`snapshot` on the checkout and `destroy` on its descendants, and another namespace can hold every candidate, so arms
can silently not run.

### Q4: whether an unexercised arm may enable the route

Chosen: "Block until it has run (Recommended)", in the round of 2026-11-19T20:35 to 2026-11-19T20:43:51-05:00. A probe
arm that could not run keeps the managed route blocked, with the per-operation effects the failed-arm answer states.
Not taken: reporting the gap and keeping the route enabled, which would let a broken kernel pass with the gap visible
only in `doctor`, now that the source always exists.

### Q5: whether a missed deadline fails closed

Chosen: "Fail closed (Recommended)", in the round of 2026-11-19T20:35 to 2026-11-19T20:43:51-05:00. A probe that misses
its deadline writes a timed-out result, the record is not current, an add refuses and names the reason, an unused slot
takes the reflink or ordinary Git route, and existing bindings and intent stay. Not taken: failing open with a warning,
which would attempt an attach on the strength of a record that says the probe could not finish.

### Q6: whether the account runs a companion probe

Chosen: "Root plus account companion", in the round of 2026-11-19T20:35 to 2026-11-19T20:43:51-05:00. The account also
runs the small user-namespace companion before each managed add, and the root probe stays the only evidence about the
dataset mount path. Not taken: root only, this design's own first rank, which would leave an account-side regression of
the walk or the mount surface seen only where something mounts as the account; the project took the companion knowing
that it never touches ZFS and skips on a host that refuses user namespaces. This design's skip also covers a host
that grants the namespace and denies the capabilities inside it, the Ubuntu guest whose default profile does so
included; that reach was not in the question, and [The account companion](#the-account-companion) states which
refusals the skip covers, what follows a skip, and that both are this design's choice, open to the project's veto,
rather than the round's.

### Q7: whether the first real attach records its outcome

Chosen: "Write first-attach", in the round of 2026-11-19T20:35 to 2026-11-19T20:43:51-05:00. The handler process that
made the first real attach after boot writes its outcome to `first-attach` beside the probe's record, and `doctor`
shows it beside the probe's result. Not taken: the probe record only, this design's own first rank, which keeps no
outcome from a real clone, a real slot and the host namespace; the project took the write, with its duty on plan steps
4 and 8.

### The threat model these answers assume

The project stated, answering a question of the staging design the same evening: "We have to assume my machine
isn't compromised. sudo is passwordless here anyway." This design therefore treats the account as trusted and does
not treat root and the account as a security boundary. Nothing else in it moves because of that sentence: the
descriptor walk, the identity compare and the refusals stay as written, as checks that a path and an identity are
the ones the design pinned, not as a boundary between root and the account.

## The open item, the answer, and what bounds them

### The open item

The accepted design leaves this item undesigned, in the words of the [proposal][proposal], lines 1630 to 1635:

> The start probe a kernel-release answer asks for. The answer's words: "Probe at start". What the probe exercises
> (descriptor attach, staging unmount, the namespace refusal), in which namespace and as which user it runs, what
> it changes on the machine at start, what it leaves behind when it fails or is interrupted, and what a refusal
> prints for the caller and `doctor` are all undesigned. Until that exists, the runtime module gate stays the only
> start check this design describes ([ABI gate and tests](cli-git-zfs-worktree-design-proposal.md#abi-gate-and-tests)).

### The answer it carries

The item's answer is recorded under "Kernel-release readiness" in the [proposal][proposal], lines 1294 to 1306:

> Kernel-release readiness. Decided: "Probe at start", a functional probe at start, against the ranking the
> question round carried, which put an exact allowlist first. The probe's own steps, its side effects at start and
> the way it refuses have no design yet;
> [Open items the decisions leave](cli-git-zfs-worktree-design-proposal.md#open-items-the-decisions-leave)
> names them. The runtime module gate of "A: Test plus runtime gate" stays, and the unselected allowlist's facts stay:
> it would qualify a release only by built-product runs on that release and would disable the managed route after
> every CachyOS kernel package update until a run passes, which makes existing unmounted clones unavailable in the
> meantime.

The item continues with the state of `7.3.5-1-cachyos` qualification.

The same [proposal][proposal], lines 617 to 631, states what the project weighed and what the answer admits:

> The project decided a functional probe at start, answering "Probe at start", against the ranking the question
> round carried, which put an exact allowlist first. The probe's steps, its side effects at start and the way it
> refuses have no design yet ...; the project's answer, "test plus runtime gate", covers the ZFS module check
> rather than a kernel-release list, and that gate stays. The unselected allowlist would qualify a release only by
> passing built handler attach, staging unmount, refusal and namespace controls on that release, compared
> independently of the ZFS ioctl ABI gate; fixture results alone do not qualify it, and under that allowlist
> `doctor` and preflight would report the running release and any missing qualification. ... The probe admits
> untested releases and leaves unmeasured behaviour to appear on the host, where a failure shows as a refused
> mount or as one the probe did not detect (reasoning); its own design is not described here, and neither is the
> allowlist's qualification run, which is named under
> [Acceptance tests the build still owes][owed-tests].

### The decisions that bound it

Each of these is decided, and this proposal keeps every one of them.

- The start-time module gate stays exactly as decided, in the [proposal][proposal], lines 235 to 244:

  > At start, before the first ioctl, both binaries must prove compatibility with the loaded module and target
  > ABI, including module release, architecture, endianness, request contract and kernel/module BTF layout. A
  > missing device, missing required BTF or mismatch disables raw requests, including reads; no force flag. ...
  > On gate refusal the wrapper uses the applicable same-dataset reflink or ordinary Git route for an unused
  > slot. ... Report what failed, including when an update makes existing clones temporarily unavailable. A
  > failed gate does not authorize an unrelated mount or cleanup; keep registrations and durable recovery state
  > intact.

  The probe adds a functional check beside that gate and replaces none of it; the quoted decision covers the ZFS
  ioctl device and layout, not a kernel-release list (the [proposal][proposal], lines 617 to 620).

- The unselected allowlist's qualifying arms name what the answer was chosen against: "built handler attach,
  staging unmount, refusal and namespace controls on that release" (the [proposal][proposal], lines 620 to 623).
  They are the shape of the functional evidence the probe is expected to seek, though the answer does not bind
  the probe to every one of them.

- The probe admits untested releases: "The probe admits untested releases and leaves unmeasured behaviour to
  appear on the host" (the [proposal][proposal], lines 629 to 630). A partial pass is therefore not a defect of
  this design, provided the missing piece is reported; the project's answer keeps the route blocked until every
  arm has run, and the record names the arm that did not run
  ([Q4](#q4-whether-an-unexercised-arm-may-enable-the-route)).

- Existing bindings and intent survive an unavailable route: "it preserves existing bindings and intent while
  the route is unavailable" (the [proposal][proposal], line 1306), and an unused slot takes "the applicable
  same-dataset reflink or ordinary Git path" (line 626).

- A failed start check is not permission to touch anything else: "A failed gate does not authorize an unrelated
  mount or cleanup; keep registrations and durable recovery state intact" (the [proposal][proposal], lines 243
  to 244).

- `doctor` is read-only: "Setup findings go to standard error and do not change `doctor`'s exit status. `doctor`
  does not start a service, create a test dataset, repair registrations or change configuration" (the
  [proposal][proposal], lines 1153 to 1155). The root probe's result reaches `doctor` by being read, never by
  `doctor` running that probe. The account companion is the one check `doctor` runs itself, under the shared
  preflight the proposal gives the two callers (the [proposal][proposal], lines 348 and 1152): it creates no
  dataset, starts no service, and its namespaces die with its child, so the limits quoted here hold for it. It makes
  one directory under the account's `$XDG_RUNTIME_DIR` for the run, mounts its child's tmpfs inside that directory,
  and removes it when the child ends; that directory is the one change `doctor` causes, it lies outside the
  checkout, the slots, the staging folder and the record's directory, and the next preflight sweeps one a kill left
  ([The account companion](#the-account-companion)).

- The wrapper must start without OpenZFS: "the wrapper must start even when OpenZFS is absent" (the
  [proposal][proposal], lines 152 to 153).

- Each add requires a current readiness report: "Require a current handler readiness report for the actual pool,
  checkout and boot, not just a present executable" (the [proposal][proposal], lines 351 to 352). The probe's
  result can live beside that record, which the wrapper already must read.

- The wrapper owns snapshot and clone creation, and the handler does not destroy anything: the decision record
  gives the wrapper "unprivileged snapshot and clone creation" and the unit "clone mount and unmount, not Git or
  automatic deletion" ([decision][decision], lines 22 to 26). A root probe that created or destroyed a snapshot
  or clone at start would need a request the accepted design does not have (the [proposal][proposal], lines 211
  to 212).

- The build owes one acceptance test for it: "Run the built probe against an untested release and against one it
  must refuse, and record what it changes at start, what it leaves behind and what it prints" (the
  [proposal][proposal], lines 1841 to 1843). Step 6 also owes its own VM arms: "an untested release, and a
  module whose field layout differs from the approved one, each asserting that raw requests are disabled" (the
  [plan][plan], lines 739 to 745).

- No second request channel is added for mounting: the design keeps "the project's ZED handler for in-session
  requests **plus** a systemd unit for boot and shutdown, not a second request channel for mounting" (the
  [proposal][proposal], lines 478 to 480), and fixes the intent vocabulary to `v1:<nonce>:mount` and
  `v1:<nonce>:unmount` (lines 537 to 539). A probe cannot be triggered by a third intent or a new channel, which
  is why this design places it in the existing boot unit.

- The handler's `boot-reconcile` and `shutdown-unmount` modes "Run both modes in the host mount namespace" (the
  [proposal][proposal], line 496), and this proposal keeps that. The probe is not one of those modes: it attaches
  nothing for a worktree, it runs in a child mount namespace of its own, and the limit that it therefore cannot
  prove the host's propagation is stated under [Limits](#limits).

- No production kernel request and no unmount runs early: the [plan][plan], lines 82 to 88 record that "No
  production kernel request runs before the first two, and no unmount runs before the third", and the third is
  the staging design. Step 6 is the start probe's own step, and step 9, which builds the product's unmount and
  holder scan, depends on steps 2, 4, 6 and 8 (the [plan][plan], lines 967 to 971). The project read that
  sentence as covering the product's mounts ([Q1](#q1-whether-the-plan-bars-the-probes-unmount-arm)), so the
  probe's own move and unmount arms are built in step 6; they are probe-private code either way, because the
  product's held-descriptor unmount does not exist when step 6 lands.

- The plan files this item as "step 6, blocking, writer design work" (the [plan][plan], lines 2200 to 2203), and
  lists the start probe among "The three blocking designs, as documents ... the critical path for steps 6 and 9"
  (the [plan][plan], lines 2102 to 2107).

## Facts

### The design's own statements

- The start-time gate runs in both binaries "At start, before the first ioctl" (the [proposal][proposal], line
  235). That gate is a set of reads. This proposal reads "start" in its sentence as the start of a process that
  will make raw requests, which covers every wrapper invocation and the boot unit's `ExecStart` (the
  [proposal][proposal], lines 500 to 505). A functional probe needs root and mounts, so it cannot run in every
  wrapper invocation; this design places it in the boot unit and settles that as a writer's item under
  [What a writer settles without asking](#what-a-writer-settles-without-asking).
- The probe is a separate check from the module gate, and the gate's failure path is decided while the probe's is
  not (the [proposal][proposal], lines 617 to 620 and 1294 to 1306).
- The syscall floor is stated as Linux 5.6 for `openat2` and 5.2 for the mount APIs, and the demonstrated guest
  baseline is `6.8.0-147-generic` with OpenZFS 2.4.4 (the [proposal][proposal], lines 612 to 615).
- The descriptor route needs a verified private immediate parent and verified private staging, and a shared
  parent refused the staging move (the [proposal][proposal], lines 738 to 739; [measurements][measurements], item
  3).
- The mount handler runs in the host mount namespace, `ExecStart` verifies ABI, keys, checkout and the private
  propagation the boot unit sets, then reconciles intent and publishes readiness (the [proposal][proposal],
  lines 492 to 500).
- `doctor` extends the read-only filesystem report and never starts a service or creates a test dataset (the
  [proposal][proposal], lines 1149 to 1155).
- The wrapper has no ZFS code today: `package/zfs` does not exist in this checkout, and no source file under
  `package/git-policy/cli/src` calls `fsopen`, `fsconfig`, `fsmount`, `move_mount` or `setns` (repository
  search, 2026-11-19). Tests do start the `unshare` program to enter a namespace, which is how the existing
  private-namespace tests run (for example
  `package/git-policy/cli/src/native/binary/binary_worktree_removal_around_a_volume_tests.rs:166` and `:194`);
  no wrapper code path enters one. The probe is new code, not a change to existing wrapper behaviour.

### OpenZFS at `zfs-2.4.4`

These lines are from the tag's commit `71a9f9578616`, cloned on 2026-11-19 from `openzfs/zfs`.

- The ZFS filesystem type declares `FS_USERNS_MOUNT` (`module/os/linux/zfs/zpl_super.c:1096` and `:1098`), so a
  user namespace may ask for the filesystem, which is the declaration the design's H1 gate 7 already expects to
  be refused in a confined namespace (the [proposal][proposal], lines 1940 to 1942).
  [M6](#m6-user-namespace-limits) is a separate arm that expects the same refusal for the probe's own source as the
  account, and it repeats nothing of gate 7.
- The kernel's own check refuses that request: `zfs_domount` returns `EPERM` when the caller is not in the global
  zone and the dataset is not visible there (`module/os/linux/zfs/zfs_vfsops.c:1357` to `:1366`).
- `inglobalzone` is true only for a credential whose user namespace is `init_user_ns`
  (`module/os/linux/spl/spl-zone.c:396` to `:403`), and a dataset can be made visible in another user namespace
  only by `zone_dataset_attach`, whose credential check requires uid 0 (`:157` to `:164`, called at `:192`).
  Only a process in the init user namespace can attach a dataset there, and the ioctl that asks for it is
  registered with `zfs_secpolicy_config` (`module/os/linux/zfs/zfs_ioctl_os.c:215` to `:216`), the pool
  administration policy (`module/zfs/zfs_ioctl.c:1155` to `:1160`).
- A name that is not a dataset or a snapshot of one has no objset to hold, and the mount path reports that as
  `ENOENT`: `zpl_get_tree` returns the error from `dmu_objset_hold`
  (`module/os/linux/zfs/zpl_super.c:857` to `:859`), and the hold sets `ENOENT` for a missing head dataset and
  for a missing snapshot (`module/zfs/dsl_dataset.c:825` to `:847`). The probe therefore detects a missing probe
  dataset, or a missing snapshot of it, only as the `ENOENT` from `fsconfig(create)`, and an unimported pool answers
  the same `ENOENT` from `spa_open` (`module/zfs/spa.c:6478` to `:6482`), so the record names the arm and not the
  cause.
- The fs_context path takes `fc->source` as the dataset name, holds that objset, and calls `sget_fc` keyed on it;
  it calls `zfs_domount` only when the returned superblock has no root yet
  (`module/os/linux/zfs/zpl_super.c:850` to `:920`). A second attach of an already mounted dataset therefore
  reuses the existing superblock and skips the dataset mount work, including the key check
  (`zpl_test_super` at `:345` to `:357`, the `s_root` test at `:905` to `:921`).
- A read-only request against an existing read-write superblock refuses with `EBUSY`, except for a snapshot
  (`module/os/linux/zfs/zpl_super.c:936` to `:943`).
- The fs_context accepts `source` and the negatable `exec`, `suid` and `dev` options
  (`module/os/linux/zfs/zpl_super.c:579` to `:583`, applied at `:669` to `:676`).
- The fs_context operations table wires `parse_param`, `get_tree`, `reconfigure` and `free`
  (`module/os/linux/zfs/zpl_super.c:1022` to `:1032`). `zpl_get_tree` sets the context's root dentry
  (`:945` to `:949`), which is what `fsmount` turns into a mount and `move_mount` then places.
- The dataset mount path in the kernel makes no ZFS ioctl request: `zpl_get_tree` calls `zfs_domount`, which calls
  `zfsvfs_create` (`module/os/linux/zfs/zfs_vfsops.c:1347` to `:1375`; the call is at `:1375`). A probe on this
  path therefore needs no `/dev/zfs`, and it proves nothing about the device or the ioctl contract.
- The snapshot branch of `zfs_domount` is not the dataset branch: for a snapshot it sets `z_issnap` and skips
  `zfsvfs_setup` (`module/os/linux/zfs/zfs_vfsops.c:1414` to `:1437`), and `zfsctl_create` is skipped too
  (`:1456` to `:1457`). So the probe's snapshot mount runs neither the callback registration, the ZIL open and
  replay and the unlinked-set drain that `zfsvfs_setup` does (`:725` to `:823`), nor the `.zfs` control
  directory's creation. Attaching an existing snapshot checks the key on the way: `zfsvfs_create` calls
  `dmu_objset_own` with decryption asked for (`module/os/linux/zfs/zfs_vfsops.c:650` to `:660`, the `B_TRUE`
  argument at `:660`; the function is defined at `module/zfs/dmu_objset.c:816`), which is the path item 5
  measured refusing with `EACCES` while the key was unavailable.
- Delegation is checked for clone creation, not for the kernel's attach: the clone policy requires `clone` on the
  origin (`module/zfs/zfs_ioctl.c:1138` to `:1141`) and `create` and `mount` on the parent (`:1143` to `:1148`),
  while `zpl_get_tree` and `zfs_domount` check no delegation. A root-run attach therefore proves the kernel path,
  not the account's grants.
- An attach of an encrypted dataset with an unavailable key refuses at `fsconfig(create)` with `EACCES`
  ([measurements][measurements], item 5, line 231), which is the observable that shows `zfs_domount` ran.
- The CLI's legacy-mountpoint refusal is the userspace branch for an explicit `zfs mount`: it prints
  `cannot %s '%s': legacy mountpoint` (`cmd/zfs/zfs_main.c:7240` to `:7248`), not the fs_context path; item 10
  measured that branch and the descriptor attach beside it ([measurements][measurements], item 10, lines 359 to
  375).
- No pool-history call exists on the mount path: the only history write in the VFS file is the upgrade
  (`module/os/linux/zfs/zfs_vfsops.c:1911` to `:1964`). Pool history records administrative commands, so it
  cannot show a mount or an atime write either way. The per-pool txg kstat can:
  `/proc/spl/kstat/zfs/<pool>/txgs` is installed by `module/zfs/spa_stats.c:288` to `:304`.
- A dataset that is mounted keeps its wrapping key: a key mapping holds the wrapping key
  (`module/zfs/dsl_crypt.c:624` to `:626`), and `spa_keystore_unload_wkey_impl` refuses with `EBUSY` while that
  refcount is nonzero (`:902` to `:903`). Item 5 unmounted the checkout and home before unloading the key
  ([measurements][measurements], item 5, lines 221 to 222).

### The kernel the design names

- This host boots `7.3.5-1-cachyos` with module version `2.4.4-1` ([measurements][measurements], lines 22 and
  671 to 681; read-only).
- On that running kernel, `/proc/kallsyms` exports `__x64_sys_fsopen`, `__x64_sys_fsconfig`,
  `__x64_sys_fsmount`, `__x64_sys_move_mount`, `__x64_sys_openat2`, `__x64_sys_mount_setattr` and
  `__x64_sys_unshare` (read-only grep of the symbol names, 2026-11-19). The syscalls the design names exist
  there; the listing proves presence on this boot, not behaviour.
- `/sys/kernel/btf/zfs` and `/sys/kernel/btf/spl` are readable, `/proc/filesystems` lists `nodev zfs`, and
  `/dev/zfs` is mode `0666` on this host (read-only, 2026-11-19). Device presence alone is weak evidence, which
  is why the design's own gate checks access and layout.
- The module's `srcversion` reads `A3553C308C29FD4B6595A72` and its `version` reads `2.4.4-1`, while
  `/sys/module/zfs/notes/` is empty, so a build ID is not exported there and `srcversion` is the readable build
  identity on this kernel (read-only, 2026-11-19).
- The root filesystem is on ZFS and the module is in use, so the module cannot be unloaded or reloaded without a
  reboot: `findmnt --target /` names `zpcachyos/ROOT/cos/root` ([measurements][measurements], lines 26 to 29) and
  `/proc/modules` lists `zfs` with a use count of 10 (read-only, 2026-11-19).
- Unprivileged user namespaces are enabled here: `/proc/sys/user/max_user_namespaces` reads `252369` and
  `/proc/sys/kernel/unprivileged_userns_clone` reads `1` (read-only, 2026-11-19).
  `kernel.apparmor_restrict_unprivileged_userns` is absent here, because no AppArmor module is loaded:
  `/sys/kernel/security/lsm` reads `capability,landlock,lockdown,yama,bpf` (read-only, 2026-11-19), and that
  setting is a distribution patch, as `unprivileged_userns_clone` is.
- The probe dataset and the configuration field that names it are the setup steps' output, and no step runs
  them on this host (the [plan][plan], steps 18 to 20). Until the owner runs the commands those steps print, the
  probe finds no name the parse accepts and records its attach arm and every arm after it as not exercised, as
  [The source](#the-source-the-probe-dataset) states.

### The kernel headers and source

- This host's `/usr/include/linux/mount.h` (package `linux-api-headers` 1:7.1-1) lists every `MOUNT_ATTR_*`
  value (`:118` to `:128`) and `struct mount_attr` with separate `attr_set`, `attr_clr`, `propagation` and
  `userns_fd` fields (`:133` to `:138`). There is no `MOUNT_ATTR_PRIVATE`; propagation is its own field of that
  struct (`:136`), and it takes the `MS_*` propagation values (`MS_PRIVATE` at `:32`, the value list beside
  `mnt_propagation` at `:171`).
- That copy is byte-identical to this host's 7.3.5 tree, `/usr/src/linux-cachyos/include/uapi/linux/mount.h`,
  apart from the include guard name (diff, 2026-11-19), so the mount-header facts this design cites are the
  running kernel's own `UAPI`, not another release's.
- Linux v6.8 `fs/super.c` (fetched; the 7.3.5 tree on this host ships no `fs` sources) states the matched
  superblock rule in the `sget_fc` doc comment: an extant superblock is returned unless the caller's user
  namespace differs or the context asked that reuse is not allowed, and both cases return `EBUSY`
  (`fs/super.c:715` to `:718` for the rule, the check at `:779`, the return at `:786`). Linux v7.3 has the same
  check at `fs/super.c:906` to `:915`. `zfs_domount`'s `EPERM` is therefore reached only for a new superblock.
- Exclusive reuse is the freshness oracle this design uses. `FSCONFIG_CMD_CREATE_EXCL` is the command (value 8:
  this host's `/usr/include/linux/mount.h:106`; Linux v7.3 `fs/fsopen.c:294`). It sets `fc->exclusive`
  (`fs/fsopen.c:217` to `:229`), which `sget_fc` turns into `EBUSY` for any matched extant superblock. ZFS
  reaches that check and is not a legacy context, so the command is accepted: `zpl_get_tree` calls
  `sget_fc(fc, zpl_test_super, set_anon_super_fc)` (`module/os/linux/zfs/zpl_super.c:872`).
- The command is newer than this design's stated syscall floor. Linux v6.6 carries it (`fs/fsopen.c:293`) and
  v6.5 does not: there an unknown command returns `EOPNOTSUPP` (`fs/fsopen.c:360` to `:361`). A kernel that
  refuses the command has no freshness oracle, which [Limits](#limits) states and
  [M1](#m1-which-source-exercises-the-dataset-mount-path) records.

### The measurements record

- Item 2 measured the descriptor walk refusing a symbolic link, an escape, a procfs magic link, bind and ZFS
  mount crossings, and changed device, inode, uid, mode and ACLs, with a valid attach as the success control
  ([measurements][measurements], item 2, lines 74 to 116).
- Item 3 measured a shared parent refusing the staging move with `EINVAL` and only a private checkout allowing
  the move and unmount ([measurements][measurements], item 3, lines 118 to 168).
- Item 4 measured a fresh process reopening and moving an attached clone, a busy unmount leaving it staged, and
  a wrong-source control that reused mount ID, device and inode and was still refused
  ([measurements][measurements], item 4, lines 172 to 213).
- Item 5 measured `fsconfig(create)` refusing with `EACCES` while the key was unavailable, before `fsmount`
  ([measurements][measurements], item 5, line 231), and a later descriptor attach passing after key load
  (lines 243 to 245). It unloaded the key only after unmounting the checkout and home (lines 221 to 222).
- Item 7 measured `nosuid` stopping setuid elevation against an unrestricted positive control
  ([measurements][measurements], item 7, lines 283 to 300).
- Item 15 measured a root scan grouping tasks by mount namespace, finding both post-attach holders, and finding
  none after they stopped ([measurements][measurements], item 15, lines 612 to 660). It recorded that a
  process-visible scan is not proof of complete absence (lines 662 to 666).
- Item 16 recorded that no built-in or local image booted `7.3.5-1-cachyos` with OpenZFS `2.4.4`, and that the
  remote catalog was not queried ([measurements][measurements], item 16, lines 668 to 706).
- The guest every measured item ran on is `6.8.0-147-generic` with OpenZFS userspace and module `2.4.4-1`
  ([measurements][measurements], lines 37 to 39, 669 to 673). Its distribution's default AppArmor profile denies
  capabilities inside an unprivileged user namespace (Ubuntu 24.04 release notes), and no item in the record ran an
  unprivileged user namespace on it, so [M11](#m11-the-account-companion-per-add)'s pass controls run with
  `kernel.apparmor_restrict_unprivileged_userns` at 0 and its skip arm runs with the setting at 1.

### The wrapper's code

- The no-crossing walk already exists: `RESOLVE_NO_XDEV`, `RESOLVE_NO_MAGICLINKS`, `RESOLVE_NO_SYMLINKS` and
  `RESOLVE_BENEATH` are defined at `package/git-policy/cli/src/native/subvolume/fs_beneath.rs:44` to `:54`,
  `ObjectId` at `:80`, the resolve mask at `:163`, `open_root` at `:184` and `open_directory_at` at `:203`. The
  wrapper already calls `libc::syscall` for `openat2` (`:167`), so the probe's other syscalls need no new
  dependency. `open_root` and `open_directory_at` open with `O_RDONLY` and `O_DIRECTORY` and read no directory
  entry (`:190`, `:204`), so a walk changes no access time.
- Mount facts are already read from `/proc/self/mountinfo`: `MOUNTINFO_PATH` at
  `package/git-policy/cli/src/native/subvolume/fs_mount_options.rs:37`, `MountFacts` at `:52` and
  `read_mount_facts` at `:339`.
- A boundary walk that refuses a nested mount and names it already exists: `find_boundary_below` at
  `package/git-policy/cli/src/native/subvolume/subvolume_remove_walk.rs:465` and its `_except` form at `:483`,
  with `device_of` at `fs_beneath.rs:244` and `handle_path` at `:307`. The probe's mount-crossing refusal control
  can reuse them.
- The `doctor` filesystem report is the section to extend: `SECTION_MARKER` at
  `package/git-policy/cli/src/native/health/doctor_filesystem.rs:51`, `gather_facts` at `:227`, `render_section`
  at `:366` and `checkout_section_with` at `:455`.
- The report's read ceiling is `REPORT_SCAN_CEILING`, 5 s, at
  `package/git-policy/cli/src/native/service/wait_ceilings.rs:1280`, measured on this host on 2026-11-09
  (`:1261` to `:1279`). The design calls it a guest trial budget, not a shipped mount limit (the
  [proposal][proposal], lines 750 to 752), and the plan records that the design's citation of it is its owner's
  to correct (the [plan][plan], lines 250 to 259).
- A cached start check already exists: `binary_identity`, `read_version_text` and `store_version_text` in
  `package/git-policy/cli/src/native/invocation/git_version_cache.rs:1` to `:23` (module doc),
  `binary_identity` at `:91`, `read_version_text` at `:238` and `store_version_text` at `:283` key a per-user
  cache on the executable's canonical path, size, times, device and inode.
- The Git check is the precedent for a cheap start check: `MINIMUM_GIT_VERSION` at
  `package/git-policy/cli/src/native/invocation/git_version.rs:52`, one version question per invocation, a
  warning and continued command on an older Git, and a stop only when the version cannot be read
  (`package/git-policy/cli/SPEC.md:18651` to `:18680`).
- The wrapper's tests already run an unprivileged private mount namespace and skip with a stated reason where
  user namespaces are refused: `a_private_mount_namespace_is_available` and `inside_a_private_mount_namespace` in
  `package/git-policy/cli/src/native/binary/binary_worktree_removal_around_a_volume_tests.rs:159` to `:200`, where
  a refused setup fails the run unless it states that the host has none
  (`CLI_GIT_NO_USER_NAMESPACE_ACKNOWLEDGED`).

## Assumptions

Each assumption is stated so a measurement can break it, and the measurement that would is named.

- A private mount namespace made by root from the init user namespace can attach a ZFS dataset, because
  `zfs_domount` sees the init user namespace and `zone_dataset_visible` returns visible and writable there
  (`module/os/linux/spl/spl-zone.c:297` to `:307`). The measurement that breaks or confirms it is
  [M1](#m1-which-source-exercises-the-dataset-mount-path), which shows the attach happening at all, and
  [M2](#m2-the-mount-table-and-the-pool-are-unchanged), which shows it staying inside the namespace.
- Once the probe's namespace has been made private, its attaches and unmounts do not reach the host mount table.
  This is reasoning from mount propagation, not a measurement in this design's shape.
  [M2](#m2-the-mount-table-and-the-pool-are-unchanged) measures it, and item 3 shows a shared parent does
  propagate and refuses the staging move ([measurements][measurements], item 3).
- A source that no namespace mounts yet reaches `zfs_domount` and the key check, while a source with an extant
  superblock skips both. The oracle is `FSCONFIG_CMD_CREATE_EXCL`, which answers `EBUSY` instead of attaching when
  any extant superblock matches ([The kernel headers and source](#the-kernel-headers-and-source)), and
  [M1](#m1-which-source-exercises-the-dataset-mount-path) measures it. The probe's source is the dedicated
  dataset's `ready` snapshot, whose key comes from the encryption root setup checks and which the boot unit
  already waits for before `ExecStart`, so the refusal itself is a regression arm only, as
  [M1](#m1-which-source-exercises-the-dataset-mount-path) states.
- A snapshot attach records no pool history entry and changes no dataset property. This is an inference from the
  kernel's mount path having no history call (`module/os/linux/zfs/zfs_vfsops.c:1911` to `:1964`), and
  [M2](#m2-the-mount-table-and-the-pool-are-unchanged) reads the txg kstat, which can show a write, beside it.
- A killed probe leaves nothing in the host mount table and nothing outside the record's directory, because
  every mount it makes lives in a namespace that dies with it and every file it writes lives in that directory or
  on the tmpfs mounted inside it. The measurement is [M3](#m3-interruption-leaves-nothing).
- The account cannot attach any ZFS source, even inside a user namespace, so every ZFS arm of the probe runs as
  root. This rests on the `EPERM` check, the init-namespace test and the uid 0 attach check named under OpenZFS
  at `zfs-2.4.4`, together with the design's own H1 gate 7 expectation (the [proposal][proposal], lines 1940 to
  1942); the measurement is [M6](#m6-user-namespace-limits), which expects `EPERM` on a source no namespace
  mounts and `EBUSY` on one that is mounted.
- The probe's cost is small enough for the boot unit to pay on every boot of this host. No measurement exists,
  and this design refuses to state a duration; [M4](#m4-what-the-probe-costs) measures it against a control arm
  of the same binary, and measures the delay it adds to the first login.
- Every arm can be exercised without creating a clone, using the dedicated dataset's empty `ready` snapshot as
  the attach source. The clone's exact shape (its encryption root and its slot properties) is then not exercised,
  which is the clone-shape limit under [Limits](#limits);
  [M1](#m1-which-source-exercises-the-dataset-mount-path) and
  [M8](#m8-the-real-clone-attach-as-evidence) measure what the snapshot arm does and does not cover.
- The account can make a user namespace and a mount namespace on this host, map its own ID to 0 there, and mount a
  tmpfs inside them, so the companion runs. This rests on `user_namespaces(7)` and
  `/proc/sys/user/max_user_namespaces` reading `252369` and `/proc/sys/kernel/unprivileged_userns_clone` reading
  `1` here (read-only, 2026-11-19); [M11](#m11-the-account-companion-per-add) measures the arms and the skip. It also
  needs a kernel that grants the namespace and still allows the mount namespace and the tmpfs inside it, which a
  kernel carrying Ubuntu 24.04's default AppArmor profile does not: there the setup answers `EPERM` and the companion
  skips, which M11 measures with `kernel.apparmor_restrict_unprivileged_userns` at 1 and at 0.
- tmpfs accepts the descriptor calls the companion makes, so its attach, move and unmount arms reach a real mount
  without ZFS: it declares `FS_USERNS_MOUNT` and carries fs_context operations (`mm/shmem.c:5103`, `:5284`,
  `:5311` and `:5316`, read at v7.3). [M11](#m11-the-account-companion-per-add)'s success control breaks or
  confirms it.

## Limits

- The probe cannot prove host-namespace propagation, a clone's real slot, or ZED ordering. Only the handler's
  actual attach at a real slot does that, which is why
  [the first real attach's record](#the-first-real-attachs-record) keeps its evidence and why
  [M8](#m8-the-real-clone-attach-as-evidence) measures it.
- The probe's mount lands in a directory of its own tmpfs, not in the checkout's slot, so the probe does not
  show the real slot, the account-owned staging folder, or the checkout's propagation into a shared parent.
  Those belong to step 9's staging design and to [the first real attach's record](#the-first-real-attachs-record).
- The probe's unmount arm is probe-private. It proves that a second process can move the mount to a move-target
  directory in the probe's own root-only tmpfs and unmount it there by path. It does not exercise the product's
  unmount from a held descriptor, which has no design until step 9 (the [proposal][proposal], lines 681 to 683;
  the [plan][plan], line 265).
- The probe's namespace scan is probe-private. It does not exercise step 9's holder scan (the [plan][plan], lines
  909 to 916), and a process-visible scan is not proof of absence (the [proposal][proposal], lines 708 to 710).
- The snapshot branch of `zfs_domount` skips `zfsvfs_setup` and `zfsctl_create`
  (`module/os/linux/zfs/zfs_vfsops.c:1414` to `:1457`), so the probe's mount runs neither the callback
  registration, the ZIL open and replay and the unlinked-set drain, nor the control directory's creation. The
  dataset shape those cover is only exercised by a real dataset attach.
- The probe reports the flags it asked for and reads them back from `mountinfo`. It does not prove that a later
  account mount is refused the same flags.
- The key check is regression coverage only in the product: the probe attaches the dedicated dataset's `ready`
  snapshot, whose key comes from the encryption root setup checks and which the boot unit waits for, so no arm
  produces the `EACCES` refusal on a healthy boot. Item 5's unloaded-key procedure runs in the VM fixture, not at start
  ([M1](#m1-which-source-exercises-the-dataset-mount-path)).
- The probe's verdict is a property of the running kernel and module build. A livepatch, or a container runtime
  with a different namespace setup, can change the answer after the record was written. The ZFS module cannot be
  unloaded and reloaded on this host without a reboot, so a module change arrives with a reboot that runs the
  probe again ([The kernel the design names](#the-kernel-the-design-names)); the probe also runs again when the
  checkout provider is explicitly stopped or restarted, because `Requires=` on that provider stops or restarts
  this unit and every start re-runs its probe ([Who runs it, and where](#who-runs-it-and-where)). Nothing else
  re-runs it between boots.
- The probe proves the syscall surface on the build it ran on, not on every newer build than the stated syscall
  floor.
- A probe refusal names an arm and an errno. It cannot say whether one particular clone would attach, and it is
  not a substitute for the module gate's device and layout checks.
- The module gate the mode runs first may open `/dev/zfs` for the access check the accepted gate asks for (the
  [proposal][proposal], lines 235 to 237); the probe's own arms never do. A device that fails the gate's check is
  therefore a gate verdict, not an arm's, and the scope of "never opens `/dev/zfs`" is the arms.
- The probe reads no file content from the attached source; it proves the mount through `fstatfs`, a bounded
  directory read and mount facts. A source whose root directory is unreadable to root is therefore not detected
  by content.
- A probe that holds the `ready` snapshot for its duration makes a concurrent `zfs destroy` of that exact
  snapshot refuse `EBUSY`; nothing else snapshots or destroys that dataset, so the window is not expected to be
  met, and [M2](#m2-the-mount-table-and-the-pool-are-unchanged) records what the hold shows. The holder and the
  unmount child each carry a parent-death signal set after their own fork and the pipes their forking process
  owns, so an interruptible child does not outlive that thread unless it died between
  the fork and the call that sets the signal, which the child's `getppid()` recheck covers
  ([M3](#m3-interruption-leaves-nothing)). A child inside an uninterruptible kernel wait survives its `SIGKILL`
  until the wait returns, exactly as the worker does, so those bounds cover an interruptible child rather than
  every child. The residue is the worker: it sets no parent-death signal, so a mode killed alone leaves it
  running, and a `SIGKILL` cannot interrupt a kernel wait, so a worker inside one holds the snapshot until the wait
  returns.
- The probe exercises one source, the configured dataset's `ready` snapshot, so an attach that succeeds there says
  nothing about another dataset's mountpoint or flags. The clone's shape and its encryption root are covered only
  by the first attach's record ([M8](#m8-the-real-clone-attach-as-evidence)).
- The freshness oracle needs Linux 6.6 or newer. `FSCONFIG_CMD_CREATE_EXCL` is refused with `EOPNOTSUPP` by an
  older kernel ([The kernel headers and source](#the-kernel-headers-and-source)), so on such a kernel the attach
  arm is unexercised and the probe never claims a fresh superblock. Where the command exists, it tells a fresh
  attach from a held superblock even when no mount of that superblock is visible in any readable namespace,
  because `sget_fc` matches on the objset rather than on a mount; the cost is that such a source is skipped
  rather than attached. The managed route's effective floor is therefore Linux 6.6, not the 5.6 and 5.2 the
  [proposal][proposal] states (lines 612 to 615): on a kernel that refuses the command, the attach arm is
  unexercised and [Q4](#q4-whether-an-unexercised-arm-may-enable-the-route) blocks that route. That is an amendment
  to the proposal's floor sentence, which [Notes for the plan](#notes-for-the-plan) records.
- The companion never reaches the dataset mount path, the key path or `/dev/zfs`, so its pass cannot predict the
  probe's or the product's attach, and its failure refuses an add without evidence that the root attach would
  fail. It is skipped on a host that will not give it a user namespace (a kernel built without them included), has
  none left, or grants the namespace and denies the capabilities or the mount inside it; the skip is printed and the
  add goes ahead after it. That reach is the refusals the skip paragraph lists and no other failure, bar the one
  mapping-write residue that paragraph names, so a defect in the companion's own setup code is an arm failure; the
  skip's printing and the add going ahead after it are this design's choices, open to the project's veto, and
  [The account companion](#the-account-companion) states the reason and the alternative.
- The companion's cost on every managed add, and the deadline that bounds it, are unmeasured until
  [M11](#m11-the-account-companion-per-add); no value is stated here.
- The `first-attach` file records one operation and is not a compatibility verdict. It is written only by the
  process that made that attach, it is read by `doctor` alone, and no route turns on it.

## The shape the answers chose

The shape has three parts, and the answers fix all three: a root probe at boot on the dataset the source answer
names, the account companion before each add, and the first real attach's record. Every arm of the probe runs on
every boot, because the source exists and nothing else mounts it.

The probe is a new mode of the handler binary, in step 6, root-run, inside a private mount namespace of its own, and
its source is the dedicated dataset's `ready` snapshot. The companion is account-side code the add preflight runs,
in a user and mount namespace of its own, touching no ZFS. The record is a write in the handler's attach path, in
the process that made the attach.

Two shapes are not taken. A probe on a snapshot of the checkout needs no layout change, and the project did not
choose it for the reasons [Q3](#q3-whether-a-setup-time-probe-dataset-exists) records. A probe that created and
destroyed a clone at every start is ruled out rather than ranked. The accepted design gives the wrapper
"unprivileged snapshot and clone creation" and the unit mount and unmount, "not Git or automatic deletion" (the
[decision][decision], lines 22 to 26). The handler "has no Git, pool administration, or dataset destruction
operation" (the [proposal][proposal], lines 160 to 161), and "There are no pool management, dataset rename, or
generic dataset creation requests" (lines 211 to 212). Such a probe would also need `/dev/zfs` at boot, on the pool
whose health is the question, so it would add a boot-time write on that pool, a stranded object after an
interruption and `ENOSPC` at boot.

## The proposed probe

This section proposes the shape the answers settle, in the detail the open item asks for: one root mode at boot on
the dedicated source, one account companion in the add preflight, and one outcome file the first real attach
writes. Every part of it is a writer's proposal inside an undesigned item, and the answers under
[Answers of 2026-11-19](#answers-of-2026-11-19) are the only decisions it takes from the owner.

### The source: the probe dataset

The source is the dataset `zpcachyos/nosnap/git-worktree-probe` with the empty snapshot `ready`, which the source
answer chose. It sits under the container `zpcachyos/nosnap`, which is unmounted (`mountpoint=none`,
`canmount=off`) and carries `autosnap = no` with each of its children (the [proposal][proposal], lines 1098 to
1124); those children are mounted data directories, and the container is the one dataset there that nothing
mounts. Its properties are `mountpoint=legacy`, `canmount=off`, `com.sun:auto-snapshot=false` and the user property
`org.monochromatic:git-worktree-probe-source=v1`. Nothing mounts a legacy dataset automatically, and `canmount=off`
stops `zfs mount -a` and the mount generator too (the [proposal][proposal], lines 262 to 270), so only a root
process that asks for it attaches it.

Root-owned configuration pins its name and GUID beside the pool and dataset names that configuration already pins
(the [proposal][proposal], line 319). The probe reads that name and checks it as a name, because it makes no ioctl;
`doctor` is where the GUID and the user property are compared, through the ioctl the probe does not make. The
snapshot is empty. Its key is the one its encryption root holds, and the dataset inherits that root from
`zpcachyos/nosnap`; step 18's `doctor` reads the root and prints it beside the pool's, so this design reads it at
setup rather than asserting it here. An unavailable key is an `EACCES` refusal from the attach arm rather than a
hidden gap, and a root that is not the pool's is a setup finding `doctor` prints, not a probe verdict. No sanoid
section is needed to keep it out of snapshots: the only recursive section is the home template,
`zpcachyos/ROOT/cos/home`, and `zpcachyos/nosnap` is outside that subtree (the [proposal][proposal], lines 1081 to
1092). The section this design
adds in step 20 follows the configuration's convention of naming every exclusion, so a later recursive ancestor
cannot silently include it.

What the account can do to it: nothing, once `doctor` has shown that no grant reaches it. Delegation is scoped to
the checkout dataset and the clone parent (the [proposal][proposal], lines 319 to 327), and that same passage says
not to put unrelated ZFS descendants there. This dataset is under neither, so the account is expected to hold no
`create`, `mount`, `destroy` or `userprop` on it and to be unable to create, rename, mount, snapshot or destroy
it; that expectation is a reading of the configuration, and step 18's `doctor` checks the effective delegation,
ancestor grants included, and prints any grant that reaches the dataset (the [plan][plan], step 18). No
account-run process and no automatic job pins or destroys it, so it cannot collide with the space guard or the
sweep, which enumerate the pinned clone parent's children and never this dataset (the [proposal][proposal],
lines 540 to 541).

How its existence is detected: the probe asks the kernel, not the pool. It attaches `<dataset>@ready` with
`FSCONFIG_CMD_CREATE_EXCL`, and a missing dataset or snapshot is the `ENOENT` from `fsconfig(create)`, as the
OpenZFS facts state. `ENOENT` alone does not name the cause: `spa_open` returns it for a pool that is not imported
(`module/zfs/spa.c:6478` to `:6482`), so the record names the arm and the call rather than the cause. An existing
dataset whose attach answered `ENOENT` is a failure `doctor` reports as such, after it checks that the configured
dataset exists; a dataset the configuration names and the pool does not hold is reported as an unexercised attach
arm and is never created. Until the setup steps run, that is the reading every boot gets: the configuration names a
dataset the pool does not hold, so the attach arm is unexercised and the managed route stays blocked
([Q4](#q4-whether-an-unexercised-arm-may-enable-the-route)), with removal of an unmounted clone still available.

What it changes at start: nothing beyond the attach the arms describe. There is no directory read of
`.zfs/snapshot`, no account-influenced name and no access-time question. The dataset itself outlives every run by
design, and the plan's steps 18 to 20 build the commands and configuration that create it, which the owner runs, as
[Notes for the plan](#notes-for-the-plan) records.

### The arms

1.  Namespace and scratch. The mode process forks a worker and calls `setpgid(worker, worker)` at once, so the
    group exists even if the worker dies before its own call, and the worker calls `setpgid(0, 0)` as well. The
    worker then calls `unshare(CLONE_NEWNS)`. It makes every mount in the new namespace private
    (`mount_setattr` with `AT_RECURSIVE` and `attr.propagation = MS_PRIVATE`, or
    `mount(NULL, "/", NULL, MS_REC | MS_PRIVATE, NULL)`), and mounts a fresh tmpfs on the scratch mount point the
    mode created. It then creates the slot, the move-target directory and a refusal tree inside the tmpfs. Until
    the `unshare` returns, no mount exists that could propagate, so a `SIGKILL` before it leaves nothing.
2.  Walk and refusal controls, over trees the probe owns. The worker opens the tmpfs root, walks to the slot with
    the wrapper's no-crossing `openat2` flags, and records the slot's `ObjectId`. The refusal controls run in the
    refusal tree: a symbolic link at a component must refuse `ELOOP`, a `..` escape must refuse `EXDEV`, a bind
    mount planted across a component must refuse `EXDEV`, and a recorded identity that changed must refuse. Each
    refusal must name the component or field, the way item 2's refusals did ([measurements][measurements], lines
    80 to 92). Separately, and read-only, it opens the checkout root through the wrapper's `open_root`, which is a
    plain `open` with a canonical-path check rather than the `openat2` mask, and records its `ObjectId`: that
    proves the root itself opens and reads no entry there.
3.  Source, from the root-owned configuration. The worker reads the dataset name from the configuration
    `boot-reconcile` reads, accepts it only as a name (each `/`-separated component a valid ZFS component name,
    within the dataset-name bound, with no `@` and no `#`), and assembles `<dataset>@ready` from the snapshot name
    the design fixes. A configuration that is absent, unreadable or names nothing the parse accepts records the
    attach arm and every arm after it as not exercised, with that reason. The probe reads no property and no
    directory at start: the configuration is the only place the name comes from, and the user property that
    names the source is read by `doctor`, through an ioctl the probe does not make.
4.  Attach, with the freshness oracle. `fsopen` on the `zfs` filesystem type, `fsconfig` the source name,
    `fsconfig` the `nosuid` and `nodev` options, read-only through the generic `ro` flag, then
    `FSCONFIG_CMD_CREATE_EXCL`, then `fsmount` with `MOUNT_ATTR_NOSUID`, `MOUNT_ATTR_NODEV` and
    `MOUNT_ATTR_RDONLY`, and `move_mount` from the mount descriptor onto the held slot descriptor. The exclusive
    create is what proves the superblock is new: a matched extant superblock answers `EBUSY` before `zfs_domount`
    runs, whether or not any mount of it is visible. A missing dataset or a missing `ready` answers `ENOENT` from
    the same call, which the record attributes to the arm and the call, never to the missing object. A kernel
    older than 6.6 refuses the command with `EOPNOTSUPP`, which records the attach arm as not exercised and stops
    the probe there rather than falling back to a plain create that could reuse a superblock.
5.  Verify. Read `/proc/self/mountinfo`, require the expected mount ID, filesystem `zfs`, exact source, and
    `nosuid,nodev,ro` flags; `fstatfs` the mount and read a bounded number of directory entries; no file content
    is read. The arm reports the flags it asked for; it does not prove that a later mount is refused them.
6.  A holder for the namespace arm. The mode process forks the holder, not the worker, so the holder inherits no
    descriptor on the worker's mount and no mount it did not make. It unshares its own mount namespace, makes it
    private, and attaches the same source read-only at the directory `holder`, which the mode created beside
    `scratch`: its mount lives in its own namespace, and the worker's tmpfs is not visible to it. It uses the plain
    `FSCONFIG_CMD_CREATE`, because reusing the worker's superblock is what this arm wants: it needs a mount in a
    second namespace, not a second mount record. It starts after arm 5 and before the move, because a probe that
    has already unmounted has no mount for a holder to hold, and it waits on its control pipe for the mode's
    signal. The mode stops and reaps it between arm 7's two scan passes, and reads the outcome line it wrote to
    its status pipe. Its pipes and its parent-death signal are the protocol under
    [The pipes and parent-death signals each child owns](#the-pipes-and-parent-death-signals-each-child-owns).
7.  Namespace scan, in two passes, before anything is unmounted. Run the probe's own scan: group live tasks by
    `/proc/<pid>/ns/mnt`, read a representative `mountinfo` per namespace, and match the ZFS filesystem and exact
    source. In the first pass, while the worker's mount and the holder's both exist, require the set of
    namespaces that mention the source to be exactly the worker's and the holder's: the exclusive create proved
    no superblock existed when the probe attached, so a further namespace mentioning it is either a mount the
    probe leaked or a mount another root process made during the run, and both are recorded as a failure of this
    arm rather than hidden. In the second pass, after the mode stops the holder and reaps it, require the worker's
    namespace to be the only one that mentions the source. A process-visible scan is not proof of absence, and a
    pinned namespace with no live member stays invisible (the [proposal][proposal], lines 708 to 710), so a clear
    report is not absence. This scan is probe-private; step 9 builds the product's (the [plan][plan], lines 909 to
    916), and this design proposes that step 9 reuse this grouping and source match rather than write a second
    scanner of the same mechanism.
8.  The move target and the unmount, from a second process. Before it forks, the worker closes the mount
    descriptor `fsmount` returned and every descriptor whose path is on the attached mount, including the ones the
    verify arm opened there. Each of them holds a reference on that mount, and a non-lazy unmount refuses while the
    mount's count exceeds the one its parent holds plus the one the unmounting path walk takes
    (`fs/namespace.c:2010` calls `propagate_mount_busy(mnt, 2)`; the count test is `fs/pnode.c:371` to `:374`,
    reached from `:501` to `:513`; the `fsmount` file holds such a reference because `dentry_open` puts the path in
    it, `fs/namespace.c:4705`, and marks it `FMODE_NEED_UNMOUNT`, `:4715`). A child's close cannot do that work for
    the worker: `fork` copies descriptors, and the open file description lives while any copy does. The worker then
    forks a child with its own control pipe and status pipe, and the child rechecks `getppid()`, resolves the slot
    and the move-target directories inside the tmpfs by path, and moves the mount with `move_mount` between those
    two paths. `move_mount` looks both paths up with `filename_lookup` and drops those references when it returns
    (`fs/namespace.c:4747`, `:4789` to `:4812`), so the child holds nothing when it unmounts. It verifies the mount
    point from `/proc/self/mountinfo`, which is not on the mounted tree, and unmounts by path with `umount2`,
    which must not answer `EBUSY`: an `EBUSY` there fails this arm. The worker waits for the child,
    bounded by the mode's deadline, and then runs the scan once more: no namespace may mention the source. It never
    unmounts a path in the account's tree, which is the only tree the accepted rule forbids (the [plan][plan], line
    275). This is probe-private code: it proves that a second process can move and unmount what another process
    attached, and it does not exercise the product's unmount from a held descriptor, which step 9 owns.
9.  Teardown. The worker writes its last outcome line, closes its status pipe and exits; the death of the mount
    namespace removes the tmpfs and everything mounted inside it, so the probe unmounts no path of its own. The
    holder is already gone, because the mode stopped it between the scan's passes. The mode process writes the
    record only after the worker exits or its deadline passes.

If an arm fails, the worker stops at that arm, records its name and errno, tears the namespace down, and reports.
A source another namespace holds is not a probe failure: arm 4 records `EBUSY` with the source's name, and records
the attach arm and every arm after it as not exercised, because the design has no second source to try. The refusal
controls in the walk arm are expected refusals: an arm that attaches instead of refusing fails the probe, so the
harness has a positive control for its own check.

### The pipes and parent-death signals each child owns

The mode, the worker and each child speak over pipes that belong to one child each, so no end of file can arrive
from another process's copy and no child inherits a channel meant for a sibling. Every pipe is created
immediately before the fork that needs it, and the forking process keeps one end and closes the others as soon as
the fork returns: the mode keeps one request pipe and one reply pipe to the worker, and one control pipe and one
status pipe to the holder, and the worker keeps one control pipe and one status pipe to the unmount child. The
mode cannot own that child's ends, since the child is its worker's, so the worker relays its outcome line to the
mode and the mode's bound on that child is the process group's `SIGKILL`. The holder and the unmount child each
move their control read end to descriptor 3 and their status write end to descriptor 4, then close every
descriptor numbered 5 or higher with `close_range(5, ~0U, 0)`, so those two pipes survive the sweep that clears
everything else. A child sets `PR_SET_PDEATHSIG` to `SIGKILL` and then rechecks `getppid()`, because the kernel
sends the signal from `forget_original_parent`, which runs for the thread that forked the child rather than for
the process (`kernel/exit.c:729` to `:732`, `PR_SET_PDEATHSIG(2const)`, CAVEATS), and because a forking thread
that died before the call leaves `getppid()` as the only signal (`kernel/sys.c:2552` to `:2561`). The mode is
single-threaded and forks from the thread that lives until it exits, so every child's parent-death signal has a
live thread behind it.

Requests are single bytes. The worker asks for the holder to start before the move and to stop between the scan's
passes; the mode forks, stops and reaps the holder on those requests and writes the outcome back on its reply
pipe. The mode does no work between reading a request and answering it, so neither process can wait on the other
for a pipe the other is waiting to fill. The holder and the worker write one line when each step of theirs ends,
naming the arm, the step and its errno, to their own status pipes, which the mode reads until end of file; arm 8's
move, arm 8's unmount and the closing scan are steps of their own, so the mode can name which one was in flight. The
unmount child writes to the pipe its worker owns and the worker relays that line. The mode therefore writes every
field of the record from values it read itself. The worker sets no parent-death signal of its own, because it must
finish its unmount and report even when the mode dies; [M3](#m3-interruption-leaves-nothing) records that kill, and
[Limits](#limits) names the residue.

### The account companion

This is the second check the companion answer chose. It runs as the account, from the wrapper's own process, before
each managed add, and it never touches ZFS, the key path or `/dev/zfs`.

How the child starts. The add preflight forks one child, and the fork is made on the blocking pool rather than on a
thread the runtime polls, because the wrapper's own source scan keeps blocking work off the runtime's workers
(`package/git-policy/cli/src/native/source_guard/blocking_code_off_the_runtime_tests.rs:1` to `:40`). The wrapper's
runtime is multithreaded and never dropped (`.../foundation/lazy_runtime.rs:1` to `:22`), and a fork leaves a child
holding only the forking thread, so the child runs a fixed sequence of raw syscalls and nothing else: every buffer
it writes is built before the fork, it formats its numbers into a stack array by hand, and no allocating call, no
runtime call and no library call that takes a lock runs in it. A re-exec of the wrapper's own executable would give
it a fresh single-threaded process, and this design cannot use one: the bar is the project's own decision in
`doc/planning/cli-git-no-self-invocation.md`, enforced by a textual scan that fails any release line starting the
wrapper itself, an `argv[0]` dispatch included
(`.../source_guard/no_self_invocation_tests.rs:1` to `:14`), and by `clippy.toml`'s ban on
`std::env::current_exe`; the scan's two `ALLOWED` entries name identification and a shim text, so no entry could
excuse a start. A separate small helper executable is not barred, because it is not the wrapper, and it is not
proposed: plan step 5, "The second executable and what the release ships", already prices a second executable for
this release (its `[[bin]]`, its install paths, the binstall table, the `checkBins` and `pinFiles` rows, the lock's
`bins` row, the sidecar-lock refresh and the install text, and the one-executable test it changes), and a helper
would add a third executable and a version handshake between it and the wrapper for no evidence the island does not
already give. The consequence is stated rather than hidden: the companion repeats the probe's syscall sequences in
that island instead of calling the probe's allocating helpers, and
[M11](#m11-the-account-companion-per-add) runs both against the same controls, so the checks stay the same.
The parent calls `setpgid(child, child)` as soon as the fork returns, before any deadline can fire, and the child
calls `setpgid(0, 0)` too, so the group the deadline kills exists even if the child dies before its own call,
which is the rule the probe's arm 1 states for its worker.

The island's shape is the wrapper's own, and three conditions come with it. The release code forks this way in a
possibly multithreaded process, builds every buffer before the fork, and has the child call only the file's own
reads, its `write` and `_exit`
(`package/git-policy/cli/src/native/process/open_file_of_a_descriptor_in_a_child.rs:101` to `:134`). First, the
companion's child ends in `_exit` on every path, because the release profile unwinds a panic
(`package/git-policy/cli/Cargo.toml:157`), and a child that did not end there would unwind into the parent's frames
and run a second copy of the wrapper. Second, the child closes every descriptor it inherited, moving its two pipe
ends down and then calling `close_range(5, ~0U, 0)`, the way the probe's holder and unmount child do
([The pipes and parent-death signals each child owns](#the-pipes-and-parent-death-signals-each-child-owns)): the
runtime holds epoll, event and socket descriptors, a caller's pipe may be open, and a copy of a pipe end or of a
lock descriptor in a child that outlives the wrapper holds that pipe or lock open. Third, the parent never waits on
a child that `SIGKILL` cannot end: it kills the child it can and leaves a thread to wait for it in the shape the
wrapper's reaper states (`package/git-policy/cli/src/native/process/stuck_children_reaped_when_the_kernel_lets_go.rs:1`
to `:32`; its `reap_soon` takes a `std::process::Child` at `:57`, so this raw-forked pid needs that rule written
for a pid, which step 6 does), which is the rule [Limits](#limits) states for the probe's own child.

Its parent-death signal, and what a killed wrapper leaves. The child sets `PR_SET_PDEATHSIG` to `SIGKILL` and
rechecks `getppid()`, as the probe's children do, so a wrapper killed outright takes its child with it, and the
child's namespace and the tmpfs inside it go when the child ends, which leaves the empty directory the next
preflight sweeps. The second child that moves and unmounts is forked by the first, and it sets its own
parent-death signal after that fork and rechecks `getppid()`, as the probe's children do, so it goes when the first
child dies: the single-threaded first child blocks until it has reaped it, which is what keeps that signal's forking
thread alive. That signal follows the forking thread rather than the process
(`doc/troubleshooting/pdeathsig-fires-when-the-parent-thread-ends.md`), and a blocking-pool thread idles out after
ten seconds without work (tokio 1.54.1 `src/runtime/blocking/pool.rs:231`, `KEEP_ALIVE`; the wrapper's builder sets no
`thread_keep_alive`, `.../foundation/lazy_runtime.rs:164` to `:170`), so the signal is safe here only because the
thread that forked the child blocks until it has reaped that child or its deadline fires.

What it runs. The parent records the account's effective user and group, and whether `/proc/self/ns/user` exists,
before the fork: a map line may name only the opener's effective ids, and the ids read after the `unshare` are the
overflow ids, which build a line the kernel refuses. The child calls `unshare(CLONE_NEWUSER)`, writes `deny` to
`/proc/self/setgroups`, writes one line each to `uid_map` and `gid_map` mapping the account's own effective user and
group to 0 in the new namespace, then calls `unshare(CLONE_NEWNS)`, makes every mount private, and mounts a tmpfs at
the scratch directory the parent made for the run. Those writes are the ones an unprivileged caller may make: one
line, its own effective ID, and `setgroups` denied before `gid_map` (`user_namespaces(7)`, the sections on
defining the two mappings and on `/proc/pid/setgroups`). tmpfs declares `FS_USERNS_MOUNT` and carries fs_context
operations, so the descriptor calls reach it without a path mount (`mm/shmem.c:5103`, `:5284`, `:5311` and `:5316`,
read at v7.3).

Then it walks its own tmpfs and makes the probe's calls there: the no-crossing walk from the tmpfs root to the slot
with the `ObjectId` compare, `fsopen` on `tmpfs` with `fsconfig`, `fsmount` and `move_mount` onto the held slot
descriptor, the mount facts read back from `/proc/self/mountinfo`, the four refusal controls, and the move and the
unmount from a second child it forks, which is safe because the first child is single-threaded by then. It makes no
ZFS call, reads no file content, and mounts nothing outside its own namespaces. Its unmount is its own tmpfs mount
in its own namespace, and the unmount answer leaves it unbarred for the same reason it leaves the probe's arm
unbarred: neither is a product mount.

Where its scratch directory lives, and who removes it. The parent creates it under the account's runtime directory,
`$XDG_RUNTIME_DIR`, in a subdirectory of its own for the run, the way the wrapper already keeps its own runtime
state under that directory (`package/git-policy/cli/src/native/service/service_provision.rs:24`). It is not under
the checkout, the slot tree, the staging folder or the clone parent, so no path mount or unmount touches the tree
the plan's rule forbids (the [plan][plan], line 275), and it is not under the root-owned record directory. The
parent removes its subdirectory when the child ends, a kill included, and only after it has reaped the child, so a
child that `SIGKILL` cannot end keeps its directory, its namespace and the mount in it until the wait returns, and
that directory is the next preflight's to sweep once the wait returns; no other process removes it. The next
preflight sweeps the directories a kill leaves, and it removes only a directory it can prove dead: the name
carries the pid and the birth identity the wrapper records for its own processes
(`.../process/process_identity.rs:1` to `:22`), and the sweep removes a name
only when its pid names no process in this namespace (`kill(pid, 0)` answering `ESRCH`) and no process holds that
recorded identity. That test is the wrapper's, not the child's: the directory is made before the fork, so the name
carries the wrapper's pid and identity, and what the sweep proves is that this run's wrapper is gone. A live child
of a dead wrapper, which a signal not yet delivered or an uninterruptible wait leaves, therefore reads as dead to
the sweep, and that is benign: that run's wrapper is gone, nothing reads its answer, and the mount the `rmdir`
detaches then lives in that child's own namespace. The name carries the PID namespace too, as
`current_pid_namespace()` gives it (`.../process/pid_namespace.rs:122` to `:133`): a name written by a wrapper in
another PID namespace names another process here, so the sweep judges it the wrapper's own way, by the directory's
own file time as its age (`foreign_record_age`, `:202`), rather than by `kill(pid, 0)`, and past
`FOREIGN_OWNER_PATIENCE` (`:108`) it refuses to remove the directory and names it (`:147` to `:200`). That care is
load-bearing: the tmpfs mount inside a live child's directory lives in that child's namespace, `rmdir` of the
directory from the host namespace is not refused for such a mount, because `is_local_mountpoint` looks at the
caller's namespace, and the call then detaches that mount in every namespace
(`fs/namei.c:5502` to `:5521`, `detach_mounts`), so a sweep that guessed would end a live run's mount. The
parent-death signal makes a wrapper's own death leave an empty directory rather than a live one, because the child
goes with the thread that forked it. Nothing survives a logout, which removes `/run/user/<uid>`.

When, and as whom. In the add preflight, after the wrapper's own start gate and before the add's claim, so that a
refusal leaves no managed object, which is the preflight order the proposal already requires (the
[proposal][proposal], lines 348 to 355). `doctor` runs the same check, under the shared preflight the proposal gives
the two callers (the [proposal][proposal], lines 1152 to 1155), so the reasons an add refuses are the classes
`doctor` prints from its own fresh run, which can differ for a transient cause, and it prints `unknown` for what it
cannot prove. It runs as the account and never as root; the managed attach itself still runs as root in the handler,
so the companion is never evidence about that attach.

What it refuses. A companion arm that ran and failed refuses that add's managed attach, prints one line, and leaves
an unused slot to the same-dataset reflink or ordinary Git route; it keeps every existing binding and intent. It
refuses no removal, no reconcile and no other wrapper command, and it never writes the probe's record: `doctor` runs
it and prints its own outcome, which is the companion's verdict and is kept apart from the probe's arms and the
first attach's outcome.

Its deadline. The parent bounds the child with a monotonic deadline and, on expiry, kills the child's process group
and refuses the managed attach as unproven, which is the same posture the deadline answer gives the probe. A refused
start is retried inside the wrapper's five-second budget before any child exists, so that budget is spent outside
this bound; the runner then awaits a place for one ceiling and the job for another, both the companion's deadline,
so the add's whole wait is that budget and those two ceilings and nothing else. No value is stated, and step 10
cites the value beside [M11](#m11-the-account-companion-per-add)'s run; until that run exists, a shipped build's
companion does not run in the preflight or in `doctor`, as [Who runs it, and where](#who-runs-it-and-where) states
for the probe's own deadline, and the guest run of M11 is the one exception, through a `CLI_GIT_TEST_ONLY_*` switch
and a test deadline that are compiled into the release executable and inert unless set (the decision in
`doc/planning/cli-git-rust-open-decisions.md:3646` to `:3649`, which keeps the test-only crash switch in the release
executable because container tests exercise the installed one, and
`doc/planning/cli-git-shipped-limits-audit.md:117` to `:119`). The hold is the missing value rather than the
switch: a shipped run names nothing, so the switch stays inert, and the guest run sets it and a test deadline of its
own, which is a control value and never a shipped one.

Its skip. The skip is drawn on the refusals that mean this host will not give the companion a user namespace, or
will not let it use the capabilities inside one. Each is a call and an errno, with the line of the kernel or of a
security module that answers it:

- `unshare(CLONE_NEWUSER)` answering `ENOSPC`: `kernel/user_namespace.c:91` sets it for the depth limit and for
  `inc_user_namespaces` answering none, which `user.max_user_namespaces` bounds (`kernel/ucount.c:77` installs the
  setting).
- `unshare(CLONE_NEWUSER)` answering `EPERM`: the same file, `:105` for a chrooted caller, `:113` for a caller with
  no mapping in the parent namespace, and `security_create_user_ns` (`:118`) answering `EPERM` from a policy hook.
- `unshare(CLONE_NEWUSER)` answering `EACCES`: that same hook, SELinux's `avc_has_perm`
  (`security/selinux/hooks.c:4734` to `:4739`, `security/selinux/avc.c:1188`) and AppArmor's `apparmor_userns_create`
  (`security/apparmor/lsm.c:1050`) reaching `aa_check_perms` (`security/apparmor/task.c:374` to `:394`, its denial at
  `security/apparmor/lib.c:440`).
- A kernel built without user namespaces, where `unshare(CLONE_NEWUSER)` answers `EINVAL` and `/proc/self/ns/user`
  does not exist: `include/linux/user_namespace.h:222` to `:227` makes `unshare_userns` an inline that answers
  `EINVAL` for `CLONE_NEWUSER` when the option is off, `ksys_unshare` calls it (`kernel/fork.c:3304`), and
  `/proc/<pid>/ns` carries the user entry only with `CONFIG_USER_NS` (`fs/proc/namespaces.c:29` to `:31`). The run
  reads that entry before the setup, so the same errno with the entry present stays an arm failure.
- The first mapping write answering `EPERM`, where the line is the one this design builds: `map_write` checks
  `file_ns_capable(file, map_ns, CAP_SYS_ADMIN)`
  (`kernel/user_namespace.c:986`) through `security_capable` (`kernel/capability.c:450` to `:458`), which
  AppArmor's `apparmor_capable` (`security/apparmor/lsm.c:198` to `:210`) answers under Ubuntu 24.04's default
  profile. A SELinux denial reaches the caller as this same `EPERM`, because `file_ns_capable` answers a bool
  (`kernel/capability.c:450` to `:462`) and `map_write` had already set `-EPERM` (`kernel/user_namespace.c:982`),
  so the hook's `-EACCES` (`security/selinux/avc.c:995` to `:1004`) never leaves that function. That check runs
  before the line is parsed and before the permission check `new_idmap_permitted` makes (`:1054`), so with a line
  of the one extent this design builds, mapping the account's own effective id, this errno is the host denying the
  capability rather than the companion's line.
- `unshare(CLONE_NEWNS)` answering `EPERM`: `unshare_nsproxy_namespaces` requires `CAP_SYS_ADMIN` in the caller's
  user namespace (`kernel/nsproxy.c:221` to `:223`).
- A capability check a mount call makes answering `EPERM`: `may_mount` requires `CAP_SYS_ADMIN` over the mount
  namespace's own (`fs/namespace.c:2063` to `:2066`) and answers a bool, so a policy hook's denial there reaches the
  caller as `EPERM`; `fsopen` asks it (`fs/fsopen.c:127` to `:128`), as do `mount` (`fs/namespace.c:4270` to
  `:4271`), `mount_setattr` (`:5298`), `fsmount` (`:4619`) and `move_mount` (`:4754`).
- A `mount` call answering `EACCES`: `path_mount` asks `security_sb_mount` before `may_mount`
  (`fs/namespace.c:4267`), and SELinux's `selinux_mount` answers `EACCES` (`security/selinux/hooks.c:2820` to
  `:2833`, wired to that hook at `:7694`), which is mount mediation the errno list would otherwise miss. It covers
  the `mount` that puts the scratch tmpfs in place, and the propagation when the companion makes that with `mount`;
  `mount_setattr` and the descriptor calls ask no mount hook, so none of them answers this `EACCES`.

Everything else the setup can answer is an arm failure, so a defect in the companion's own code lands on the
refused side rather than the skip: `EINVAL` from any of those calls where `/proc/self/ns/user` exists, which is the
malformed line at `kernel/user_namespace.c:990` or a `unshare(CLONE_NEWUSER)` from a caller whose thread group is
not empty (`kernel/fork.c:3188` to `:3191`, an implied `CLONE_THREAD`); an `EPERM` from the
`setgroups` write, which `proc_setgroups_write` gives only to a `deny` written after a `gid_map` (`:1264` to
`:1278`, a defect of the companion's order) and not to this design's own write, which asks for no capability; an
`EPERM` from the second mapping write, which the one-write rule answers (`kernel/user_namespace.c:978`); `ENOMEM`;
and any errno a call was not named here to give, a seccomp filter's arbitrary answer included. That failure refuses
the add as unproven. `EUSERS` is not in either set on any kernel this design supports: `unshare(2)` gives it only
from Linux 3.11 to Linux 4.8, and `kernel/user_namespace.c` at v7.3 names it nowhere.

The line can be drawn because the refusals these calls answer today are named here, each with the line that
answers it, and because the capability check inside `map_write` runs before both the parse and the permission
check, so the one errno the Ubuntu restriction and a malformed line could share is told apart. Two cases are left
where the reading cannot separate a host refusal from a defect. One is a first mapping write whose line or opener
is not the one this design builds: a line that is not one extent of count one mapping the opener's effective id,
or a mapping file opened before the `unshare`, also answers `EPERM` (`proc_uid_map_write` refuses that opener at
`kernel/user_namespace.c:1127` to `:1131`, and `new_idmap_permitted` refuses that line at `:1171` to `:1212`), so a
companion that built the wrong line, or read the ids after the `unshare` and got the overflow ids, would skip. The
run reads the ids before the fork, and [M11](#m11-the-account-companion-per-add)'s pass control, where no pass arm
prints a skip line, is what catches a companion that misreads them. The other is a host whose `/proc` is not
mounted, where the absence of `/proc/self/ns/user` reads as a kernel without the option. The design accepts that
residue rather than hide it, and the skip line names the call and the errno, which is what a person needs to see it.
What the rule costs in the other direction is that a sandbox that answers an arbitrary errno at a setup call is now
a refused add rather than a skip, which is the fail-closed side, and this machine's threat model trusts it against
that case.

The wrapper's own control draws a related line: it runs
`unshare --user --map-root-user --mount --propagation private /bin/true` and fails the run unless the run states the
host has no user namespaces, by setting `CLI_GIT_NO_USER_NAMESPACE_ACKNOWLEDGED`
(`package/git-policy/cli/src/native/binary/binary_worktree_removal_around_a_volume_tests.rs:165` to `:183`,
the variable at `:38`). The companion is a shipped check rather than a control, so it prints the skip instead of
failing every add on such a host, and what follows a printed skip is this design's choice rather than the project's,
marked open to the project's veto where [The account companion](#the-account-companion) states it with its reason.

The Ubuntu restriction, in detail. A kernel can grant the user namespace and take the capabilities inside it away,
and Ubuntu 24.04's default AppArmor profile does exactly that, as its release notes say: the profile "allows the use
of user namespaces for unprivileged and unconfined applications but will deny the subsequent use of any capabilities
within the user namespace". That guest is the one every measurement in this design runs on. Reading mainline v7.3, the
call that answers there is the first mapping write, not `unshare(CLONE_NEWNS)`: `map_write` asks for
`CAP_SYS_ADMIN` over the new namespace before it parses anything (`kernel/user_namespace.c:986`), and the profile
is what answers that check, so `unshare(CLONE_NEWUSER)` succeeds, the `setgroups` write succeeds because
`proc_setgroups_write` asks for no capability to write `deny`, and the first map write answers `EPERM`. That is a
reading of mainline source, not a run on that guest's patched 6.8 kernel, and
[M6](#m6-user-namespace-limits) and [M11](#m11-the-account-companion-per-add) record which call it named rather
than assuming this one. A boundary at the first call would make it a failed arm, so every managed add on such a
host would be refused and the probe's pass controls could not pass there either.

The companion then prints one line naming the call, the errno and the settings it read
(`/proc/sys/user/max_user_namespaces`, `/proc/sys/kernel/unprivileged_userns_clone` and, where the running kernel
has it, `kernel.apparmor_restrict_unprivileged_userns`), records its arms as not exercised, and does not refuse the
add.

That last step, and the printing of the skip, are this design's own choice, not the project's, and one the project
can veto separately from the boundary: Q6's option text named a skip on a host that refuses user namespaces as one
of its costs, so that a skip exists was before the project, while the add going ahead after it was not. The skip's
reach matters to that choice, and the skip's boundary keeps it to the refusals the skip paragraph lists, bar the
residue that paragraph names, with the Ubuntu guest among the hosts that skip. The choice differs from the
direction Q4 chose for the probe's arms, where an arm that could not run keeps the route blocked. The reason: the
companion touches no ZFS and its pass cannot
predict the root attach, so a host that refuses user namespaces would lose the whole managed route to a check that
cannot speak about it, while the root probe's arms and the module gate still bound that route. The alternative is
Q4's rule applied here: a skipped companion refuses the managed add, and the managed route is then off on hosts
that refuse user namespaces, the Ubuntu guest included, with an unused slot taking the reflink or Git route. This
design adopts the first and marks it open to the project's veto at acceptance, with veto none as the default; a veto
takes the second, and nothing else in the design moves. `doctor` prints the settings it read and the companion's own
outcome, so a host cannot lose the check without a printed reason.

Every ending, and the line it prints. The companion's run ends in one of these, and each has a class and a line of
its own:

- A pass: nothing is printed, and the add goes on.
- A refusal on the skip paragraph's list: a skip, printed with the call and its errno as the skip line gives it,
  and the add goes on, which is this design's choice open to the project's veto.
- Any other failure of any call, in the setup or after it: an arm failure, a refusal printed on the failed-arm
  line.
- A refused `fork`: a refusal, because a start the system refused started nothing and the wrapper's own rule is to
  start again: the parent pauses and retries within the wrapper's five-second budget
  (`package/git-policy/cli/src/native/push/start_retry.rs:33`, `START_RETRY_BUDGET`, over `retry_refused_start` at
  `:246` and its `RefusableStart` at `:60`), and a budget that runs out is an arm failure naming `fork`, whose text
  is the retry's own `WouldBlock` error with the count of refusals. This host's own record has such refusals as
  ordinary here (`doc/troubleshooting/run-wrapper-pids-max-refuses-fork-and-threads.md`), and a shortage that
  outlasts the budget is a host fault the add itself would meet.
- A refused blocking-pool thread, which can arrive before any child exists: the fork goes through the wrapper's
  ceilinged runner, `run_a_job_once_awaited_no_longer_than`
  (`.../foundation/blocking_jobs_within_the_budget.rs:529`), awaited with the companion's deadline as its ceiling,
  because the budget-only variant takes a place with no ceiling and would wait for one nothing gives back (`:508`
  to `:520`). It takes the wrapper's one-shot claim on the job's work before that work's first effect
  (`.../foundation/the_claim_a_job_takes_before_running_it.rs:1` to `:24`), so a refusal the runner can promise
  about is `NoThreadToStart` and nothing was made, while one it cannot is reported as started or as maybe-run
  (`.../foundation/blocking_job.rs:38` to `:67`). Both variants share that claim and the place's release
  (`run_a_job_holding_the_place`, `:220` to `:257`). Either is an arm failure naming the pool thread and the
  claim's answer, so the add is refused as unproven; without that claim a companion run could start after the add
  had already answered.
- A failed pipe, or a child that ends with no status line, or with one the parent cannot parse: an arm failure
  naming the pipe's failing call and its errno (`EMFILE` or `ENFILE` from a `pipe2`, or the child's own prelude
  failing), or naming `no status line` when the child's `_exit` came before its line.
- A scratch directory that cannot be made or written, because `$XDG_RUNTIME_DIR` is unset, unwritable or full: a
  refusal as unproven, printed with the failed-arm line, whose arm is `scratch` and whose call and errno are the
  failing one, in the wording the wrapper's provisioning already uses, which names `XDG_RUNTIME_DIR` and what to do
  (`.../service/service_provision.rs:306` to `:308`, `:408` and `:427`).
- A mountinfo read past the buffer the parent built before the fork: an arm failure naming the read and its bound,
  which is the wrapper's own `MOUNT_TABLE_LIMIT`, where a table past the bound is refused rather than read whole
  (`.../foundation/plain_file.rs:209`; `.../subvolume/fs_mount_options.rs:344` to `:348`), so the add is refused as
  unproven.
- A child that dies of a signal before its first status line, from a seccomp `SIGSYS` or the memory killer: an arm
  failure naming the signal with no errno, printed on the failed-arm line, so the add is refused as unproven.
- A deadline expiry: a refusal as unproven, printed as
  `managed add refused: the account companion did not answer within its deadline; this slot takes <route>.`

The lines themselves, then. On a failed arm:
`managed add refused: the account companion's <arm> failed: <what failed>; this slot takes <route>.`
The `<what failed>` is `<call> (errno <n>)` when a call answered, or `the child (signal <n>)`, or
`the pool thread (no thread to start)` or `the status pipe (no status line)` when none did.
On a skip: `account companion skipped: <call> answered <errno>; user namespaces are unavailable here`, followed by
the values it read, `user.max_user_namespaces=<value>`, `kernel.unprivileged_userns_clone=<value>` and
`kernel.apparmor_restrict_unprivileged_userns=<value>`, with `absent` printed for each setting the running kernel
does not have: `unprivileged_userns_clone` is a distribution patch and is absent on every mainline kernel at v7.3,
and the AppArmor setting exists only where that module is. On a pass it prints nothing. The lines go to standard
error under the design's own reporting rules (the [proposal][proposal], lines 755 to 762).

`doctor` prints the same classes: a pass, a failed arm with its call and errno, the skip with its settings, and
`unknown` for the deadline, a refused `fork` that outlasted its budget, a refused pool thread, a failed pipe or a
missing status line, an unusable scratch directory, a refused mountinfo read and a signal death, which is the word
plan step 18 uses for a check it cannot prove (the [plan][plan], step 18's acceptance text, "reports an unreadable
or unproven check as unknown"). It runs a fresh companion, not the add's run, so a transient cause that refused one
add can leave `doctor` printing a pass, and that add's own line is the record of it.

What it costs per add, and what sizes that. One fork, two `unshare` calls, two mapping writes, one tmpfs mount and
its unmount, the walk and its identity compare, the refusal controls, and one more child for the move and the
unmount, on every managed add. No duration is stated: [M11](#m11-the-account-companion-per-add) measures the cost as
a ratio to the same add with the companion disabled, interleaved in the same round, with the range between rounds,
and carries the controls that make that ratio able to fail; the same run sizes the per-add deadline, which ships
only with that value beside it.

### The first real attach's record

The write the record answer chose keeps the outcome no probe can produce: a real clone, at a real slot, in the host
mount namespace. The file is `first-attach`, beside `result` in `/run/git-worktree-clone-mount-probe/`, root-owned
mode `0644` in that root-owned `0755` directory, read-only to the wrapper and `doctor`.

Which process writes it, and when. The handler process that makes the first real clone attach after the current boot
writes it: `boot-reconcile` or the sweep, whichever attaches first, the sweep being the zedlet's default
invocation with no argument (the [plan][plan], step 5) rather than a second path. `First` means first to claim the
file, not first to attach: two handler processes can finish out of order, and the one that claims the name first
is the one whose outcome is kept. The claim is taken by the first attempt that passes the key check, after that
check and before the walk that opens the slot, and not by an attempt the check refuses: plan step 8 checks the
encryption root and key state before the attach, and it "reports an unavailable key separately from identity or
permission failures", so a key refusal is a finding about the key, which `doctor` and the handler's journal already
report, and not an attach
outcome this file keeps. This design chose that boundary over taking the claim at `fsopen`, and the reason is the
order the handler runs: the slot's identity compare is a check of the real slot before the attach, and a slot whose
identity has drifted is exactly the operation failure this file exists to record, so the claim is taken before that
check and not after it. The plan puts both checks before the attach (step 8's key check, and step 4's recorded slot
identity that the compare reads) and does not order the slot's first-observation record against the key check, so
this design takes the claim before that record: a slot whose identity cannot be read or written is an outcome the
file keeps. The key check and the sweep's own refusals before any attach attempt are what stay outside the claim:
the clone's type, its GUID, its origin and properties, the slot's derivation and the intent parse all precede the
walk in plan step 8, and none of them takes a claim. Every refusal point from the walk onward has a step name, and
a refusal inside the claim writes `outcome refused` with the failing call's
errno name, or with `errno none` where the refusal was a comparison or a policy check.
It is that process, not the mode that supervises a probe and not the wrapper. It writes after the attach's outcome
is known, either the verified mount at the slot or the failing step, and before it publishes this boot's receipt.
When the writing process is `boot-reconcile`, it also writes before readiness, so a reader that sees the
readiness published by `boot-reconcile` sees the file too; a sweep that attaches later writes after readiness was
published, and the file's own time is what orders the two. A crash between the write and the receipt leaves an
outcome without a receipt rather than a receipt without an outcome.

How that rule reads against [Q7](#q7-whether-the-first-real-attach-records-its-outcome), as the project was asked it:
the question asked for the one outcome from a real clone, a real slot and the host namespace, and this rule keeps
the first attempt that could have produced one. A refusal at a check the claim covers, the walk, the slot's
first-observation record or the identity compare, therefore becomes that boot's record, and an attempt that attached
later in the same boot writes nothing, so `doctor` shows that boot's first attempt as refused and the later pass in
the handler's journal alone. Such a refusal may never have reached a kernel attach call, so the file can hold a
boot's outcome that is no evidence about the kernel's attach path, where Q7 asked for the outcome of a real clone
that `doctor` compares with the probe's result on the same build. The alternative, claiming the name at `fsopen`,
keeps only the attach calls' outcomes and loses a drifted slot's record, which is one of the failures the write
exists to keep. The design keeps the first attempt and states this effect in place of hiding it; the alternative is
named so the project can weigh it, it is not taken here, and
[Questions the owner answered](#questions-the-owner-answered) puts the placement to the project as a question.

How one writer keeps it. The writer claims the name with an exclusive creation, the shape `write_exclusive_synced`
has (`package/git-policy/cli/src/native/foundation/private_storage.rs:500`), writing the started line; a process that
finds the claim answers `EEXIST` and writes nothing. Both writes set `0644` explicitly, because the helpers it takes
their shape from publish at `0600`: the claim is published by `write_then_publish` with its mode argument at
`0644`, which `protect_path` applies through `set_permissions` (`:175`), the argument `write_private_file` already
passes `0600` through (`:424` and `:392` to `:397`), and the staged outcome, which `replace_private_file` creates at
`0600`, is `fchmod`ed before its rename
(`.../foundation/private_storage_replacing_a_record.rs:54`). The winner then stages the outcome in a sibling in the
same directory and renames it over the claimed name in one step, in that `replace_private_file` shape, with the
record's own owner and mode rather than the private `0600` the wrapper's helpers give the account's own files. A
reader therefore sees the started line or the whole outcome, and a winner that dies between the claim and the rename
leaves the started line, which every reader reports as begun and not finished, as it reports the probe's started
record. The claim is per boot because the directory is on `/run`; the outcome also carries the boot ID, and a reader
whose boot differs reads the file as stale.

Its format. One `key value` line per field, fixed keys and no free text, in the shape the probe's record uses:
`v1` first, then `kind operation-outcome`, `state started` or the outcome, `boot <id>`, `kernel <release>`,
`module <version>`, `srcversion <value>`, `handler <binary identity>`, `time <ISO 8601 with offset>`,
`operation attach`, and then either `outcome attached` with `mount <mount ID>`, `flags <the flags read back>`,
`slot <the slot name>`, `clone <the clone's dataset name>` and `guid <the clone's GUID>`, or
`outcome refused` with `slot <the slot name>`, `clone <the clone's dataset name>`, `guid <the clone's GUID>`,
`step <the step>` and `errno <a name or none>`. The steps are the handler's own, in the order it runs them:
`walk`, `identity-record`, `identity-compare`, `fsopen`, `fsconfig`, `fsconfig-create`, `fsmount`, `move-mount` and
`verify`. `fsconfig` carries the source write and the option writes and `fsconfig-create` the one
`FSCONFIG_CMD_CREATE` call, so every `fsconfig` call the attach makes has a name.
`walk` is the no-crossing `openat2` walk from the pinned checkout, `identity-record` is the slot's first-observation
record that plan step 8 writes before the attach, and `identity-compare` is the compare step 4 makes against a
recorded identity. The key check is not one of them, because a refusal there takes no claim and writes nothing. The
walk and the attach calls write the errno name their call gave, `identity-compare` writes `errno none` because its
refusal is a comparison, and `identity-record` and `verify` write whichever their refusal was: a call's errno name
when a read or a write answered, `errno none` when a check did. So no refusal the claim covers reaches the reader
without a name it can parse. A reader
refuses a set of lines with an unknown key, a repeated key, a missing required key or a value it cannot parse, and
reports the file as unreadable rather than guessing.

How `doctor` reads it. In the same section as the probe's result, opened by path with `open_regular_file` and
bounded by `read_regular_file`'s limit, so a link or a named pipe at the name is refused rather than waited on
(`.../foundation/private_storage.rs:785` and `:817`). That open takes a path and is not a directory-relative open,
which is enough here: the name sits in a root-owned `0755` directory and the open refuses a link at it. It is a
deliberate departure from the proposal's sentence for receipts, which are "opened relative to the trusted
directory" (the [proposal][proposal], lines 550 to 553): these two runtime files are not receipts that authorize
anything, and root and the account are trusted here under the threat model the answers assume. It prints
the outcome and, for a refusal, the step and the errno; an absent file as no real attach this boot; a started line
as begun and not finished; a boot ID that differs from this boot as stale; and an unreadable file as unreadable.

How it stays an operation outcome. Its own `kind` line says `operation-outcome` and it has no verdict field, and no
consumer gates on it: the add preflight reads the probe's record and the module gate, never this file, and a
first-attach refusal neither enables nor disables any route. `doctor` prints it under its own label and keeps the
two questions apart: the probe says whether this kernel and module can attach at all, and this file says what one
attach did on this boot, so a slot whose identity had drifted or a device that failed reads as an operation failure
and the probe's arms are unchanged, while a clone whose key was unloaded writes nothing here, because the key check
refuses before the claim, and `doctor` prints that refusal from the handler's own report instead. Where the two
differ, `doctor` prints the difference as a difference and names both, without calling either a verdict on the other.

Which plan steps carry the duty. Step 4, "The descriptor attach, its identity checks and its rollback", builds the
outcome line, the writer and the reader of the host facts the fingerprint holds (kernel release, module version,
`srcversion` and the handler's binary identity), because it is where the failing step and the errno are known. Step
8, "The attaching sweep for a clone that carries a mount request", calls it, and its sweep is the path that makes
the first attach after boot. Step 7, "The intent property, the receipts and the readiness record", references the
file rather than owning it, as it does the probe's record. Step 18, `doctor`, gains the reader beside the probe's
result. [M8](#m8-the-real-clone-attach-as-evidence) measures the file against the probe's snapshot arm on one
build. The line numbers inside [Notes for the plan](#notes-for-the-plan) name the plan's base.

### Who runs it, and where

Root, from the handler binary, inside the private mount namespace the run creates. The probe never runs in the
host mount namespace, never runs as the account, and never runs inside `doctor`.

Placement: a new mode of the handler binary, `start-probe`, beside `boot-reconcile`, `shutdown-unmount` and
`adopt-worktree-parent` (the [plan][plan], lines 640 to 641 list those; step 6 adds `start-probe` to the list),
run as `ExecStartPre=-` of the existing boot unit. The dash prefix is load-bearing: without it a probe that exits
non-zero, panics or dies of a signal makes the unit fail before `ExecStart`, so the reconcile is skipped and the
unit's stop action runs at boot, which is the outcome the placement exists to avoid. With the prefix such an exit
is recorded and has no further effect, and `ExecStart` still runs (`systemd.service(5)`, the special executable
prefixes). The prefix covers a non-zero or signal exit and not a timeout, which is one reason the unit sets none:
an expiry in `ExecStartPre=` sends the unit to its stop action whatever the prefix says (`systemd.service(5)`,
`ExecStartPre=`; systemd v262 `service.c:3047` to `:3057` spawns that command with the start timeout, and `:5156`
to `:5166` turns an expiry there into the stop path).
`reconcile-git-worktree-clone-mounts.service` already runs as root at boot, already orders after the key and
checkout providers, and already publishes readiness (the [proposal][proposal], lines 492 to 505). `ExecStartPre=`
is the service manager's own child process, so the `unshare` inside the probe cannot change the namespace the
unit's `ExecStart` reconciles in, which the accepted design requires to be the host namespace (the
[proposal][proposal], line 496). No new unit and no new request channel are added, which matters because the
design forbids a second mount request channel and fixes the intent vocabulary to `mount` and `unmount` (the
[proposal][proposal], lines 478 to 480 and 537 to 539). An on-demand probe would need a third intent kind or a
new channel, and this proposal does not propose one.

The mode runs the module gate first, in the same process, before it makes any mount. A gate refusal records the
probe as skipped with the gate's own reason and enables nothing, because the gate's refusal "does not authorize
an unrelated mount" (the [proposal][proposal], lines 243 to 244); a skipped probe is not a probe verdict either
way.

The mode process enforces the deadline rather than the worker: it forks the worker, puts it in a process group of
its own, waits for it with a monotonic deadline, and on expiry sends `SIGKILL` to that whole group and to the
holder, reaps the worker only with `WNOHANG`, records a timed-out result, writes the record and exits without
waiting further. It exits successfully whenever it reaches its own end, because the unit's start must continue: a
failed or timed-out probe is a verdict in the record, never a failed unit. A mode that panics, aborts, takes a
signal or is killed for memory never reaches that end, and the dash prefix is what keeps the unit's start going
then; what the next boot reads in that case is the started record that
[the interruption section](#what-it-leaves-when-it-fails-or-is-interrupted) describes. The worker's mounts die
with its namespace, so a worker that survives an uninterruptible wait, which `SIGKILL` cannot interrupt, holds
nothing in the host mount table; it can hold the snapshot busy until the wait returns, which is under
[Limits](#limits). The holder and the unmount child each carry a parent-death signal and the pipes their forking
process owns, as
[The pipes and parent-death signals each child owns](#the-pipes-and-parent-death-signals-each-child-owns)
states, so an interruptible child does not outlive the thread that forked it, and a child inside an uninterruptible
kernel wait can outlive it until that wait returns; the unit's own `KillMode=`
default of `control-group` sends nothing until the unit stops, so it bounds a leaked process only after a failed
or finished run (`systemd.kill(5)`, `KillMode=`). A missed deadline writes a timed-out result and the route fails
closed ([Q5](#q5-whether-a-missed-deadline-fails-closed)). That result names the step in flight when the deadline
fired and the step the last status line named: the mode reads one line per step, so the step after the last line it
read is the one that was running, and arm 8's move, arm 8's unmount and the closing scan each have their own line.
Removal therefore does not guess: [Q2](#q2-what-a-failed-arm-disables)'s rule treats a move, unmount or scan step
that was in flight as a failed arm, so removal of a mounted clone is refused when the deadline caught one of those
three, and a timeout that caught any other step keeps the failed-arm answer's effects for new attaches only.

Where the deadline's value comes from. No workload measurement exists, and rule `LQM` asks for a measurement or an
owner decision beside every shipped limit, so no number ships uncited. Step 6 runs [M4](#m4-what-the-probe-costs)'s
arm and whole-probe timings in its guest arms, through the transient unit
[M9](#m9-the-untested-release-and-the-refused-release) describes, and the value ships beside that run; step 19's
boot arms re-measure the login delay at boot, and the value is re-cited there if the boot reading differs. Until
step 6's run exists, step 19 does not add the unit's `ExecStartPre=` for the probe, so no wedge can hold up
`systemd-user-sessions.service` in the meantime, and the probe runs only from the transient unit. Step 10 does the
same for the companion, citing the per-add deadline beside [M11](#m11-the-account-companion-per-add)'s run, and the
companion does not run in the preflight until that value exists. Both of those are this design's own choices,
pending the project's veto, with veto none as the default: the first is no deadline value before step 6's
measurement, and the second is no companion deadline before step 10's. A provisional ceiling set now by the project's
own decision would also satisfy `LQM` and let step 19 add the `ExecStartPre=` sooner, and a provisional ceiling for
the companion would likewise let it run in a shipped preflight and in `doctor` before M11's value exists; this design
proposes neither number, and those choices are the project's.

The deadline's margin. The value ships as the slowest measured arm or whole-probe run plus a margin that covers the
range between the interleaved rounds, so a run inside the range the harness already saw does not fail closed on
noise. The measurement runs on a guest and the deadline guards this host, and a file-backed guest pool measures
semantics rather than this host's storage (the [plan][plan], step 22's storage half, which makes that half the
owner's act on the owner's authorisation); a reading of the same run on this host's storage is the owner's act for
the same reason, and until it exists the value is the guest's.

The unit sets no `TimeoutStartSec=`. A start timeout bounds a oneshot's `ExecStart` as well as its
`ExecStartPre=`: the timer is armed per command with `s->timeout_start_usec` (systemd v262 `service.c:2991`,
armed at `:2178`), and a command that times out sends the unit to its stop action, which for this unit is
`shutdown-unmount`. The reconcile's duration is unsized until plan step 22, so any value that fits the probe could
kill the reconcile mid-attach and then unmount what it had attached. The oneshot default is no start timeout at
all (systemd v262 `service.c:1225` to `:1227`; `systemd.service(5)`, `TimeoutStartSec=`), the probe's own deadline
bounds the worker and the holder, and the mode's own phases are short and non-blocking rather than timed. No step
sizes a unit start timeout: plan step 22 sizes the mount-request deadline of step 7 and the unmount deadline of
step 9, and asks for workloads to size any shipped deadline (the [plan][plan], lines 1778 to 1779 and 1811 to
1812). A mode wedged in one of its own phases would therefore hold up `systemd-user-sessions.service` until it was
killed by hand, which [M4](#m4-what-the-probe-costs) reports beside the login delay it measures. The accepted unit
is ordered before `systemd-user-sessions.service` (the [proposal][proposal], lines 496 to 498), so the probe delays
a login by its own run time; [M4](#m4-what-the-probe-costs) measures that delay and [Limits](#limits) records that
the probe does not run in the host namespace.

Nothing re-runs the probe between boots except a restart of the checkout provider: the unit requires that
provider and orders after it (the [proposal][proposal], lines 483 to 486), and `Requires=` stops or restarts the
requiring unit when the required one is explicitly stopped or restarted (`systemd.unit(5)`, `Requires=`, systemd
262). Every start of this unit runs its `ExecStartPre=` again, so the probe and its record are refreshed then. A
module reload cannot happen here without a reboot, so a module change arrives with a reboot that runs the probe
again ([The kernel the design names](#the-kernel-the-design-names)). A record whose fingerprint no longer matches
the running kernel, module or boot is stale, and the wrapper reads stale readiness as not current rather than
running a probe.

The wrapper's own per-invocation start check stays the module gate. The wrapper reads the probe's result from the
record it already must read before an add (the [proposal][proposal], lines 351 to 352) and reports it.

The account companion is not part of this placement: it is the wrapper's per-add check, described under
[The account companion](#the-account-companion), `doctor` runs it under the shared preflight, and no boot unit
runs it.

### Where the verdict is enforced, and where the record lives

The record is `/run/git-worktree-clone-mount-probe/result`: a root-owned file, mode `0644`, in a root-owned
directory of mode `0755` that the mode process creates itself if it is absent, the same shape as the receipts
directory `/run/git-worktree-clone-mount-results` (the [proposal][proposal], lines 550 to 553). The writer of
`first-attach` creates the same directory with the same owner and mode when it is absent, because a sweep can
reach its first attach before the boot probe has written anything. The mode creates that directory rather than
relying on the unit's `RuntimeDirectory=`, because systemd removes a runtime directory
when the unit reaches its dead state, failed included (systemd v262 `service.c:2591` to `:2599`, `unit.c:6452` to
`:6459`; `systemd.exec(5)`, `RuntimeDirectory=`), and that would delete the record exactly when a failed arm's
verdict is what `doctor` and the add preflight must read. Because the directory is on `/run`, the record is
per-boot state that no disk holds. The mode also creates `scratch` and `holder` inside that directory, as the
mount point for the worker's tmpfs and the directory the holder attaches at; both mounts live only in a child
namespace, so the host sees empty directories there after a run and the record file beside them. `first-attach`
lives beside `result` in that directory, with the same owner and mode, and the handler process that made the first
real attach after boot writes it ([The first real attach's record](#the-first-real-attachs-record)); the probe
mode neither writes nor reads it.

Step 6 builds the mode and its writer. The unit that carries the `Type=oneshot`, `RemainAfterExit=yes` and
`ExecStart` running `boot-reconcile` is step 19's (the [plan][plan], lines 1591 to 1593), and it gains
`ExecStartPre=-` running the probe; step 7's readiness record references the probe's record rather than owning it
(the [plan][plan], lines 762 to 779 and 825 to 829). Until step 19 lands, the probe runs from the guest's
transient test unit named under [M9](#m9-the-untested-release-and-the-refused-release), so the design does not
wait on the unit.

Every process that makes a raw ZFS request reads it before doing so: the wrapper's add preflight, where the
proposal already requires a readiness report (the [proposal][proposal], lines 351 to 352), the handler's
`boot-reconcile` mode, its `shutdown-unmount` mode, and the sweep the no-argument invocation runs, which a ZED
event starts (the [plan][plan], lines 640 to 647). The wrapper's other invocations make no raw request and read
nothing. The sweep and the boot unit are independent triggers (the [proposal][proposal], lines 478 to 480 name
both), so a sweep can reach the record before the unit's probe has written it; an absent or stale record then
refuses the attach and keeps the intent, which is the decided behaviour for a missing readiness report (the
[proposal][proposal], lines 351 to 352) and needs no new rule. The module gate binds both binaries (the
[proposal][proposal], lines 235 to 237); the probe's verdict binds at least every attach path, and a failed arm
refuses new managed attaches and keeps every other operation ([Q2](#q2-what-a-failed-arm-disables)).

### What it changes at start

- One worker process, the mode process that forks it and enforces its deadline, a holder the mode forks, and an
  unmount child the worker forks, each short-lived, with one control pipe and one status pipe per child.
- One new private mount namespace for the worker and one for the holder, made private before any mount is made.
- One tmpfs for scratch, no larger than the probe needs to hold a few directories and a link.
- One mount of a read-only source, and the superblock it creates, for the duration of the run.
- One bounded directory read on the probe's own read-only mount and one `fstatfs`.
- One root-owned directory under `/run` and its record, rewritten by atomic rename.
- Journal lines from the boot unit.

It creates no dataset, snapshot, bookmark or property, makes no ZFS ioctl request, starts no service, and writes
no file outside the record's directory and the tmpfs inside it.

The account companion adds nothing here: it runs in the add's own process, forks its child on the blocking pool,
and its tmpfs lives and dies with that child's namespace. Its one residue is the empty scratch directory under
`$XDG_RUNTIME_DIR`, which the parent removes and the next preflight sweeps
([The account companion](#the-account-companion)). Neither does the first real attach, beyond the one file it
writes at the attach's end.

### What it leaves when it fails or is interrupted

A failed arm tears the namespace down and leaves the record naming the arm, the errno, the fingerprint and the
fallback. The record's directory is not a runtime directory, as
[the record's own section](#where-the-verdict-is-enforced-and-where-the-record-lives) states, so the record
survives the unit's failure and a manual stop. A `SIGKILL` leaves nothing in the host mount table and nothing
outside the record's directory, which [Assumptions](#assumptions) states and
[M3](#m3-interruption-leaves-nothing) measures, because the namespace and the tmpfs die with the process. If the
record is absent, the wrapper's add preflight already treats the readiness as not current and refuses the managed
attach, which is the decided behaviour for a missing readiness report (the [proposal][proposal], lines 351 to
352), not a new rule.

The one residue a killed probe can leave is a started record: the mode process writes one, with the fingerprint
and no result, by atomic rename before it forks the worker, and the final record replaces it the same way. Both
live in the record's directory on `/run`, so neither is on disk, and `doctor` can say the probe began and did not
finish. A reader that finds a started record with no result reads it as not current rather than as a pass, which is
the same rule as an absent record, so a mode that panicked or was killed leaves a boot whose adds refuse the
managed attach. The next boot's probe replaces it, and the next boot starts with an empty directory.

The `first-attach` file has the same started state: a claim carrying the started line and no outcome is read as
begun and not finished, never as a pass and never as a verdict.

The companion leaves no record and no mount. Its child's namespace dies with it, so the tmpfs mount goes with it and
the host sees an empty directory; that directory, under `$XDG_RUNTIME_DIR`, is the whole residue, removed by the
parent when the child ends and by the next preflight after a wrapper kill. A killed child holds no mount outside
its own namespace and no snapshot, so the probe's residue rules cover it as well.

### What a refusal prints

The wrapper prints one line to standard error on the managed operation it refuses, naming the probe arm, the
running kernel release, the module version, the failed call and its errno, the route the slot will use instead,
and the `doctor` command that prints the last result. It never suggests a force flag, because the gate has none
and this probe adds none. A failed arm refuses new managed attaches and keeps every other operation, with the
per-operation effects [Q2](#q2-what-a-failed-arm-disables) states.

`doctor` prints a new section under the existing filesystem report, headed by its section marker pattern, with
the fingerprint, when the last probe ran, each arm and its outcome, any difference between the recorded
fingerprint and the live kernel, module or boot, the sources and mounts the probe used, and which route remains
available. It prints the first real attach's outcome beside the probe's result, under its own label
([The first real attach's record](#the-first-real-attachs-record)), and the account companion's own outcome from
the run `doctor` makes: a pass, a failed arm with its call and errno, the skip with the three user-namespace
settings it read, or `unknown` for every ending it cannot prove
([The account companion](#the-account-companion)). The section goes to standard error and changes
no exit status, as the design requires for setup findings (the [proposal][proposal], lines 1153 to 1155).

The companion prints its own line on the add it refuses or skips, as [The account companion](#the-account-companion)
states, and the wrapper's refusal line names the companion's arm and errno beside the probe's when both apply.

The handler's journal line names the same arm and errno under `reconcile-git-worktree-clone-mounts.service`, so a
boot failure is visible without a caller.

### The result record and its fingerprint

The record holds the fingerprint: kernel release from `uname`, module version from `/sys/module/zfs/version`,
the module's `srcversion`, the boot ID from `/proc/sys/kernel/random/boot_id`, and the handler binary's identity
in the shape `binary_identity` records. A module's build ID where the kernel exports one, under
`/sys/module/zfs/notes/`, joins the fingerprint too; that directory is empty on this host, so `srcversion` is the
build input that is always readable there. The record also holds the time of the run, each arm's outcome, errno
and source, the step in flight and the last step that finished when it timed out, the holder's and the worker's
namespace identities, the chosen source, whether the source existed, and the resulting verdict. The probe's own
fields are written only by the mode process, from the lines it reads on the status pipes. The first real attach's
outcome goes into a second file beside it, `first-attach`, written by the handler process that made that attach, as
[The first real attach's record](#the-first-real-attachs-record) states:
that file has its own writer per boot and its own claim, so each file has one writer and no lock is needed. The
wrapper and `doctor` read both and write neither.

The record is not authority to act. It is a cache of an observation, and the wrapper rebuilds nothing from it
except the verdict and its wording, the way the design treats its other runtime files (the [proposal][proposal],
lines 555 to 556).

### What the result gates

A result with every arm passed enables managed attaches for that boot and module identity.

A result naming a failed attach, walk, move or unmount arm, or a failed namespace arm, is a real observation, and
it refuses new managed attaches while it keeps every other operation ([Q2](#q2-what-a-failed-arm-disables)): an
unused slot takes the same-dataset reflink or ordinary Git route, `boot-reconcile` attaches nothing and keeps
every intent and registration, removal of a clone that is not mounted stays available, and removal of a mounted
clone is refused and reported when the failed arm is the move, the unmount or the scan.

A result naming an unexercised arm, because the configured source is absent, another namespace holds it, the
configuration names nothing the parse accepts, or the kernel refused the exclusive create, blocks the managed
route ([Q4](#q4-whether-an-unexercised-arm-may-enable-the-route)) with those same per-operation effects, and
`doctor` reports which arm did not run and why.

A result naming a missed deadline is written as timed out, and the route fails closed
([Q5](#q5-whether-a-missed-deadline-fails-closed)): the record is not current, it names the step in flight and the
last step that finished, and a move, unmount or scan step in flight is treated as a failed arm for removal while
every other timeout carries the failed-arm answer's effects for new attaches only
([Q2](#q2-what-a-failed-arm-disables)).

A module or kernel whose fingerprint changed since the record makes the record stale; the next boot's probe
replaces it, and an add in between reads stale readiness, which the preflight must report as not current rather
than as a pass.

### What the probe never does

Its arms never open `/dev/zfs`, because the dataset mount path in the kernel makes no ZFS ioctl request:
`zpl_get_tree` holds the objset and reaches `zfs_domount`, which calls `zfsvfs_create`
(`module/os/linux/zfs/zpl_super.c:850` to `:920`, `module/os/linux/zfs/zfs_vfsops.c:1347` to `:1375`). The module
gate the mode runs first may open it, because the accepted gate checks device access (the [proposal][proposal],
lines 235 to 237) and step 6 designs that check; `stat`, `access(2)` and a plain `open` all answer "missing
device" without a raw request, so this design does not choose between them. Beyond that, it never mounts by path on
an account-controlled tree, never runs the `zfs` program, never creates or destroys a dataset or snapshot, never
promotes, never sets a property, never writes a registration or a receipt, never runs in the host mount namespace,
never runs from `doctor`, and never offers a force flag. It never reads `.zfs/snapshot` at all: its source is the
configured name, so the `.zfs` control directory is never opened, and no entry name in it is ever looked up.

The account companion never touches ZFS, the key path or `/dev/zfs`, never runs as root, mounts only inside the
user and mount namespaces it makes for itself, and writes no record; `doctor` runs it and prints its outcome, which
is the only companion verdict there is. The mounts the probe and the companion make live only in their own
namespaces.

## Notes for the plan

These are notes for the plan's own text, and this design edits no plan step. Each note names its step by heading,
because the plan's line numbers move: every plan line number this document cites elsewhere is a line of the plan at
commit `73eaad0846`, the branch's base, and the two places this design reads a line of main's copy name their step
and its text rather than a number (`git show main:doc/planning/cli-git-zfs-worktree-implementation-plan.md`, read at
main's `8b1cec7883`, whose plan is 2451 lines). Main's copy is thirteen lines longer before step 4 than the base,
four of those lines from the merge `30431ecc14` inside step 1 and six of them before the earliest passage this
document cites, so one offset does not serve the whole file.

- Step 3, "ZFS recognition in the reflink rung and the ladder's refusals", owns H1 gate 7, which exercises a
  confined consumer's clone attach. That gate meets Ubuntu 24.04's capability denial when the setting is at 1, so
  its `EPERM` reading must be taken with `kernel.apparmor_restrict_unprivileged_userns` at 0, and its control needs a
  tmpfs mount in the same user namespace beside the permitted host mount, so a denial cannot pass for the gate's own
  refusal. [M6](#m6-user-namespace-limits) states the same rule for its arms.
- Step 4, "The descriptor attach, its identity checks and its rollback", gains the `first-attach` writer, its format
  and its exclusive claim, because the attach module is where a failing step and its errno are known. It also owns
  the reader of the host facts the fingerprint holds (kernel release, module version, `srcversion` and the handler's
  binary identity), which step 6's `result` reuses rather than reading a second way, and its writer creates the
  record's directory with the same owner and mode the mode process gives it, because a sweep can reach its first
  attach before the boot probe has written anything.
- Step 6, "The start-time compatibility gate and the start probe", gains the `start-probe` mode, run as
  `ExecStartPre=-` of the boot unit, with its attach, move, unmount and namespace arms, which the unmount answer
  places in this step, and the account companion's own check code beside them. Its guest arms gain the timing run
  that sizes the probe's deadline ([M4](#m4-what-the-probe-costs)'s arm and whole-probe timings, through the
  transient unit [M9](#m9-the-untested-release-and-the-refused-release) names).
- Step 7, "The intent property, the receipts and the readiness record", gains the probe's record and `first-attach`
  as files it references rather than owns. It does not gain the preflight that reads them: that is step 10's shared
  `doctor` preflight, which treats an absent, stale, started, timed-out or blocked record as not current.
- Step 8, "The attaching sweep for a clone that carries a mount request", gains the call to the `first-attach`
  writer, with the order this design chose as its duty: the sweep takes the claim on the first attempt that passes
  the key check and before it writes the slot's first-observation record, its sweep being the path that makes the
  first attach after boot, and it reads the probe's record before any attach: on an absent, stale, started,
  timed-out, failed or unexercised record it attaches nothing and keeps
  every intent and registration, which is what [Q2](#q2-what-a-failed-arm-disables) and
  [Q4](#q4-whether-an-unexercised-arm-may-enable-the-route) decide.
- Step 10, "The add route from registration to verified fill", gains the account companion in its shared preflight
  before the claim, the refused-arm answer's refusal rule, the unexercised-arm answer's block, and the timing run
  that sizes the companion's per-add deadline ([M11](#m11-the-account-companion-per-add)). `doctor` runs the same
  companion under the shared preflight step 10 builds here, which step 18's `doctor` gains the checks for.
- Step 14, "Removal: the lineage proof, the plan and the removal intent", and step 16, "The snapshot batch, the head
  destroy, origin cleanup and the Git settle", carry the removal half of the failed-arm answer: removal of a mounted
  clone is refused when the failed or timed-out arm is the move, the unmount or the scan. Step 14 builds the
  `unmount` request with its proof and step 16 takes the fresh all-namespace verification from step 9's scan, so
  those are the two places the refusal is read.
- Step 18, `doctor`, prints the probe dataset's creation, its `ready` snapshot, its user property and the
  configuration field that names it, as root-once commands it prints and never runs. It gains the reader of the
  probe's record and of `first-attach`, the companion's outcome with the settings it read, and the checks
  that the dataset's encryption root is the pool's and that no delegation reaches the dataset.
- Step 19, "The one-time migration, the immutable entry and the boot and shutdown units", builds the unit that
  carries the probe: `ExecStartPre=-` running the `start-probe` mode, before its `ExecStart` running
  `boot-reconcile`. Its setup text stays the owner's act, printed by `doctor` and never run by the step, and the
  configuration field step 18 prints is applied by the owner there.
- Step 20, "Sanoid coverage, the exclusions and the guard's report", owns the `sanoid.conf` text for the probe
  dataset, the section that names it as an exclusion beside the others; no step applies it on this host.
- Step 22, "The measurements that size waits and retention", keeps sizing the mount-request deadline of step 7 and
  the unmount deadline of step 9. The probe's and the companion's deadlines are not its work: steps 6 and 10 cite
  them beside their own runs, because those steps ship before step 22, and that narrowing of step 22's "every shipped
  limit cited beside its measurement" and its "to size any shipped deadline" is a plan change the amendments name.
- The measurement homes, one place each: M1, M2, M3, M5, M7 and M9's fault arms in step 6's guest arms, with
  M9's untested-release arm in step 23, which qualifies the CachyOS template; M4's arm and whole-probe timings in
  step 6 and its boot-and-login reading in step 19's boot arms; M8 in step 8's sweep on the guest; M10's setup half
  with step 19's guest setup and its refusal readings in step 6; M11 in step 10's guest arms, which gain a run with
  the companion enabled by a test-only switch, a boot with unprivileged namespaces disabled and a boot with
  `kernel.apparmor_restrict_unprivileged_userns` at 1; and M6's root-and-account half in step 6, with its
  restricted-namespace arm repeated in step 10's guest arms. M6 is a separate arm from step 3's H1 gate 7, which
  exercises a confined consumer's clone attach, and it replaces nothing there. No value ships before the step that
  runs its measurement, and step 22 sizes only the waits it names.

### Amendments these notes ask for

Four sentences outside this design need an amendment note beside them once the branch lands. None of them is edited
here.

- The plan's preface, "no unmount runs before the third": [Q1](#q1-whether-the-plan-bars-the-probes-unmount-arm)
  read it as covering the product's mounts only, and step 6 builds the probe's own move and unmount arms, so the
  sentence needs the note that says which mounts it covers.
- The proposal's floor sentence, "The syscall floor is Linux 5.6 for `openat2`; the other mount APIs used here start
  at 5.2" (the [proposal][proposal], lines 612 to 614): the managed route's effective floor is Linux 6.6, because a
  kernel that refuses `FSCONFIG_CMD_CREATE_EXCL` leaves the attach arm unexercised and
  [Q4](#q4-whether-an-unexercised-arm-may-enable-the-route) blocks that route, as [Limits](#limits) states.
- The proposal's exclusion count, "Use the eight exclusions named in the planning decision" (the
  [proposal][proposal], line 1083): the probe dataset's own section is a ninth exclusion, and step 20 writes it.
- Plan step 22, "The measurements that size waits and retention": its "every shipped limit cited beside its
  measurement" and its acceptance text "to size any shipped deadline or optional scan interval" would reach the two
  deadlines too, though steps 6 and 10 cite them beside their own runs before step 22 ships, so the sentence needs
  the note that narrows it to the mount-request deadline of step 7 and the unmount deadline of step 9.

## Owed measurements

Every item here is owed before the probe can be trusted. None ran for this proposal, and each runs on a
disposable file-backed pool inside a throwaway VM, never on this host's pool and never with its `/dev/zfs`.

The chosen shape needs every item here. The root probe needs M1, M2, M3, M4, M5, M7, M9 and M10, and M6 for the
`EPERM` it expects the account to meet. The account companion needs M6 and M11. The first real attach's record
needs M8 and M9.

### M1: which source exercises the dataset mount path

Attach the dedicated dataset's `ready` snapshot, which no namespace mounts, and the already mounted checkout
dataset beside it, and record which one runs `zfs_domount` and the key path. The oracle for a fresh superblock is
`FSCONFIG_CMD_CREATE_EXCL`: the fresh source must attach, and the already mounted one must answer `EBUSY` from
`fsconfig(create)` rather than succeed while reusing a superblock. A successful attach alone proves nothing about
the key check, because the source's key is the one its encryption root holds and is loaded; the two `EBUSY` and
`EACCES` controls carry that evidence. Control: item 5's unloaded-key procedure, which
unmounts the checkout and home first ([measurements][measurements], lines 221 to 222), unloads the key, and
attempts the fixture's `ready` source; require `EACCES` from `fsconfig(create)` (item 5, line 231). A second
control: the same attach of a name that the pool does not hold, which must answer `ENOENT`, so that a harness
reporting every refusal as a missing dataset fails its own control. Also record whether a read-only request
against a read-write dataset superblock refuses `EBUSY` (`module/os/linux/zfs/zpl_super.c:936` to `:943`), adjust
the arm's flags if it does, and record what a kernel older than 6.6 returns for the exclusive create, so the
unexercised arm's wording is measured rather than assumed.

Gates: the probe's attach arm and its source, the exclusive create as the freshness oracle, the claim that a
source no namespace mounts is the one that reaches the mount path, and the limit that the key refusal is regression
coverage only.

### M2: the mount table and the pool are unchanged

Read the guest's initial mount namespace's `/proc/self/mountinfo`, and a representative `mountinfo` for every
other visible namespace, before and after a complete probe. Read `/proc/spl/kstat/zfs/<pool>/txgs` before and
after, plus the dataset list and the pool history as secondary readings. The txg
kstat is the one that can show a write: pool history records administrative commands only, since the mount path
has no history call (`module/os/linux/zfs/zfs_vfsops.c:1911` to `:1964`) and the kstat is installed at
`module/zfs/spa_stats.c:288` to `:304`. Its columns are `txg`, `birth`, `state`, `ndirty`, `nread`, `nwritten`,
`reads` and `writes`, followed by the completion times (`module/zfs/spa_stats.c:206` to `:214`).

The txg comparison is by row, not by the highest txg: an idle pool commits a txg on its own timer, every
`zfs_txg_timeout` seconds (5 by default, `module/zfs/txg.c:532` to `:553`), so the highest txg always moves and a
before-and-after comparison of counters proves nothing. The window carries a start fence and an end fence, both a
sync. Run `zpool sync` in the guest, then read the kstat and call the then-open row's txg `T0`: the sync committed
everything written before it, so `T0` starts clean and the probe's writes can land only in `T0` or later. Run the
probe, run `zpool sync` again, and read the kstat again: that sync commits everything the probe wrote, so the
highest committed row is the window's end, and a late write cannot hide in the row that is still open. Require,
for every committed row whose `txg` is no lower than `T0`, `ndirty`, `nwritten` and `writes` to be zero, and
require such a row to exist: the end sync commits `T0` itself, so that requirement is met unless the history list
has already dropped the row, which the `zfs_txg_history` bound turns into an unmeasured report rather than a pass.

An idle txg writes nothing: with nothing changed in it, an empty config dirty list and no multihost protection,
the uberblock write is skipped (`module/zfs/vdev_label.c:2064` to `:2079`), so a committed row with zero `ndirty`,
`nwritten` and `writes` is the expected reading for a probe that wrote nothing. The measurement therefore needs a
pool the harness keeps otherwise idle, because a write from anything else lands in the same window and cannot be
told from the probe's; the harness records the window's rows with their counters so a nonzero row can be
attributed. Read the second time soon enough that `T0` is still in the list, because the list keeps
`zfs_txg_history` rows, 100 on this host (read-only `/sys/module/zfs/parameters/zfs_txg_history`); with the
parameter at 0 the reading is empty, which the design reports as unmeasured rather than as zero writes.

Controls, each fixture-only and never shipped: the idle window, which runs the same syncs and readings with
`sleep` standing in for the probe and must read zero on every row of the window, so a harness that reports zero
for a pool something else is writing fails its own control; a deliberate mount in the guest's initial namespace,
which must show the mount-table difference the harness looks for; and a deliberate write in a scratch dataset,
which must leave a nonzero committed row in the window, including one made inside `T0` itself, so a harness that
never reads the file, or that only looks past the window, fails its own control. A deliberate read of the attached
mount's own directory belongs beside them, since the probe makes one and the harness must show it changing no pool
row.

Gates: the probe and the account companion; also the claim that a probe leaves no pool write, and the
snapshot-hold window.

### M3: interruption leaves nothing

Kill the probe with `SIGKILL` after each step: after `unshare`, after the tmpfs, after `fsconfig(create)`, after
`fsmount`, after `move_mount`, after the holder starts, between the scan's two passes, after the move, and before
the unmount. Kill each process separately as well as the whole run: the mode process alone (the worker must then
survive or die cleanly, and the record must say what happened), the worker alone while the mode lives, the holder
alone, and the unmount child alone. Record the host mount table, every namespace's representative `mountinfo`,
the slot, the move-target directory, the record and its directory, whether any namespace still mentions the
source after the kill, and whether a second run succeeds. Control: the same probe allowed to finish, and a run
killed before `unshare` returns.

Hold one arm past the deadline, once with the move step of arm 8 and once with its unmount step, and require the
timed-out record to name the step in flight as that one and the last step that finished as the one before it, and
require removal of a mounted clone to be refused after each. Control: the same hold with the deadline raised past
the step's measured duration, which must record that step as passed, so a mode that names the step in flight always
fails its own control.

Gates: the claims that it leaves nothing in the host mount table and nothing outside the record's directory, the
started record, the parent's deadline path, the step a timed-out record names, and each child's parent-death signal,
`getppid()` recheck and pipes.

### M4: what the probe costs

Measure each arm and the whole probe as a ratio to a control arm of the same binary with the probe disabled,
interleaved in the same round, and report the range between rounds. Measure at boot, with the boot unit, the
delay it adds to the first login that `systemd-user-sessions.service` admits, and measure any other placement a
later answer asks for. The companion is not measured here; [M11](#m11-the-account-companion-per-add) measures it.
The arm and whole-probe timings run in step 6's guest arms, through the transient unit
[M9](#m9-the-untested-release-and-the-refused-release) names, because step 6 ships the probe; the boot-and-login
half runs in step 19's boot arms, once the unit exists. Control: the interleaved arm of the same binary with the
probe disabled; items 4 and 15's timings are not a control, because they measure other work.

Gates: the probe's placement, its own deadline, and the time-to-login cost of a boot ordered before user sessions.
No value is stated before this measurement, and a value ships only beside it or beside an owner decision that sets
it.

### M5: the refusal arm refuses for the right reason

Run every refusal control the walk arm lists and the success control, and require the success case to attach
where the refusal cases refuse. Control: the unmodified slot must attach, so a harness that refuses everything
fails its own control.

Gates: the probe's walk, its refusal controls and the wrapper's `openat2` flags.

### M6: user namespace limits

As the account, attempt the probe's attach of a ZFS source that no namespace mounts, inside a user namespace, and
require `EPERM` from `zfs_domount`, with `kernel.apparmor_restrict_unprivileged_userns` at 0 on a guest that has
the setting, so a capability denial cannot answer in place of `zfs_domount`. Then repeat with the source mounted in
the guest's initial namespace and require `EBUSY` from the matched extant superblock instead, because `sget_fc`
refuses a superblock from another
user namespace before `zfs_domount` runs (Linux v6.8 `fs/super.c:779` to `:786`). Control: a tmpfs mount in the
same user namespace succeeds, and root's attach in the init namespace succeeds. Then disable unprivileged
namespaces in the guest (`user.max_user_namespaces` set to 0) and record that the account arm is skipped with a
stated reason rather than failing the boot. Then boot a guest with `kernel.apparmor_restrict_unprivileged_userns` set
to 1, where the user namespace is granted and its capabilities are denied, and require the same skip, naming the
setup call that failed and `EPERM`, which is the restricted arm [M11](#m11-the-account-companion-per-add) runs.

M6 is a separate arm from H1 gate 7, which plan step 3 owns: gate 7 exercises a confined consumer's attach of the
clone, while M6 exercises the probe's own source as the account, and neither replaces the other. Both run on the
Ubuntu guest, so both meet the restriction [The account companion](#the-account-companion) states for the skip, and
both therefore run with `kernel.apparmor_restrict_unprivileged_userns` at 0 for their refusal readings: with the
setting at 1 an `EPERM` from the capability denial looks the same as the `EPERM` M6 and gate 7 expect, so M6's tmpfs
control and gate 7's permitted host mount would pass for the wrong reason. Gate 7's own note is under
[Notes for the plan](#notes-for-the-plan).

Gates: the assumption that every ZFS arm needs root, the companion's `EPERM` reading, and the skip path
[M11](#m11-the-account-companion-per-add) measures.

### M7: the namespace arm sees a known holder and reports only the probe's own namespaces

Record the scan's two passes and what each must show: the worker's and the holder's namespaces while both mounts
exist, the worker's namespace alone after the holder is stopped, and no namespace after the unmount. Repeat with
the source already mounted in a second namespace before the probe runs, and require the attach arm to answer
`EBUSY` and the probe to record that source as held and the later arms as not exercised instead of refusing
spuriously. Repeat with a root mount of the same source made in a further namespace during the run, and require
it to name that mount in the first pass and fail the arm. Repeat with a namespace pinned and no live member in it,
and record that the scan does not name it and reports the source clear: a clear report is not absence, which the
document says wherever it relies on the scan.

Gates: the probe's namespace arm, the exclusive create as the freshness oracle, and the limit that a
process-visible scan is not proof of absence.

### M8: the real clone attach as evidence

Record the handler's first real attach after boot as the `first-attach` file writes it, written by the process
that made the attach, and compare its outcome with the probe's snapshot arm on the same build. Controls, each of
which must fail its arm when it is broken: an unloaded-key clone attach must write nothing, because plan step 8's
key check refuses it before any call, and the handler's journal and `doctor` must name the unavailable key instead;
with the `CLI_GIT_TEST_ONLY_*` switch that skips that check set, the same fixture must write `outcome refused` with
`step fsconfig-create` and the `EACCES` item 5 measured, on the same boot as a passing probe arm; a second attach in
the same boot must write nothing, because its claim answers `EEXIST`; a handler check that refuses inside the claim,
such as the identity compare, must write `errno none` and pass the reader; a slot behind a nested mount, which the
no-crossing walk must refuse, must write `step walk` with `EXDEV`, so a refusal at the walk is a named step rather
than an unnamed one; and a writer killed after the claim must
leave the started line, which `doctor` reports as begun and not finished. Compare the file against the probe's arm
both ways: a passing probe with a refused first attach must leave the probe's record unchanged, and a failing probe
arm with an attached first attach must not turn that probe arm into a pass.

Gates: the first real attach's record, its one-writer claim, and the limit that a snapshot arm is not a clone arm.

### M9: the untested release and the refused release

Run the built probe on a release it must accept and on one it must refuse, and record what it changes at start,
what it leaves behind and what it prints, as the design's acceptance item requires (the [proposal][proposal],
lines 1841 to 1843), together with step 6's own VM arms (the [plan][plan], lines 739 to 745). Control: the
accepted release must pass, so a probe that refuses everything fails its own control.

The refused-release arm needs a release, or a fault, that makes an arm fail. A fault is buildable in the guest
today: a transient unit named `git-worktree-probe-test.service`, created with `systemd-run` and carrying
`SystemCallFilter=` for `move_mount` or `fsmount` together with `SystemCallErrorNumber=EPERM`, runs the
`start-probe` mode as its `ExecStart=`, which makes the move or the attach arm fail with `EPERM` and the probe
refuse. Without the error number a system call filter kills the process with `SIGSYS` rather than failing the call
(`systemd.exec(5)`, `SystemCallFilter=`), which would test the signal path and not the arm. The same transient
unit carries the passing control arm, and it is also what runs the probe before step 19's unit exists. The
untested-release arm needs the custom CachyOS template that no built-in or local image provides (the
[proposal][proposal], lines 1826 to 1834; [measurements][measurements], item 16, lines 668 to 706); until it
exists, run this arm on the Ubuntu guest with a labelled limit that says it is not the host release.

Two more arms use deliberate faults rather than releases. The handler's `CLI_GIT_TEST_ONLY_*` switch that makes it
keep one extra descriptor open on the attached mount must make the move and unmount arm fail with `EBUSY`, and the
same switch turned off must pass, so that arm's unmount is shown to check the mount rather than pass vacuously
(`fs/namespace.c:2010`). A
`SIGKILL` to the mode process must leave the unit's `ExecStart` to run, which is what `ExecStartPre=-` buys, and
must leave a started record that the add preflight and `doctor` read as not current rather than as a pass.

Gates: the built probe, the built companion and the acceptance item.

### M10: the probe dataset

Measure the setup command's creation of `zpcachyos/nosnap/git-worktree-probe`, its `ready` snapshot, its user
property and its configuration field, and its idempotent second run. Measure the space it takes, that nothing else
mounts it, that the sweep and the destroy-anyway path never see it, and that the sanoid configuration excludes it.
Record what the probe writes when the dataset is absent, when its `ready` snapshot is absent, and when the
configuration names a dataset the pool does not hold: the first two answer `ENOENT` from `fsconfig(create)` and
the record names the arm, and the third is the same arm reported from an absent name. Record, from `doctor`, the
encryption root the dataset inherits and the effective delegation including ancestor grants, so the two premises
[The source](#the-source-the-probe-dataset) reads at setup are measured rather than assumed. Controls: the same attach
of a name that is absent while the configuration names the real dataset, which must record the arm as not
exercised rather than as a pass, and the healthy dataset beside it, which must attach.

Gates: the probe's source and the setup steps.

### M11: the account companion, per add

Run the built companion in the add preflight on the guest. The guest run enables it through a
`CLI_GIT_TEST_ONLY_*` switch that carries a test deadline of its own, both compiled into the release executable and
inert unless set: in a shipped build, until this item's value exists, the companion does not run in the preflight
(as [The account companion](#the-account-companion) states). Record every arm the run exercises, and require that no
pass arm prints a skip line, so a companion that skipped while the run calls it a pass fails its own control. The
arms are the user namespace and its mapping writes, the mount namespace and its private propagation, the tmpfs
scratch mount, the walk and its `ObjectId` compare, the descriptor attach of the second tmpfs, the flags read back,
the four
refusal controls, and the move and the unmount from the second child. Every refusal control needs its success control
beside it, the shape [M5](#m5-the-refusal-arm-refuses-for-the-right-reason) gives the probe's own, so a companion
that refuses everything fails its own control, and every arm here but the restricted one runs with
`kernel.apparmor_restrict_unprivileged_userns` at 0 on a guest that has the setting.

Measure what the companion costs one add as a ratio to the same add with the companion disabled, interleaved in the
same round, and report the range between rounds; size the per-add deadline from that measurement and from no other.
Controls that make the number and the verdict able to fail: the interleaved disabled arm, which must show the
difference the ratio reports; a `CLI_GIT_TEST_ONLY_*` fixture whose child answers `EPERM` from `fsmount`, which must
refuse the add, name the call and take the fallback route, with the switch off passing; and a guest booted with
`user.max_user_namespaces` set to 0, which must print the skip line naming `ENOSPC` and the setting, must not
refuse the add, and must leave `doctor` naming the setting and the skip. A transient unit carrying
`SystemCallFilter=` for `unshare` with `SystemCallErrorNumber=EPERM` is the other refusal a host can show, and it
must print the same skip line with that errno; a filter answering `EACCES`, the errno the SELinux and AppArmor hooks
give, must print the same line; and a filter for `mount` answering `EACCES`, the mount mediation the skip list
names, must do the same, naming `mount`. The other half of the line needs its own control: a filter answering `EINVAL`
at a setup call on a guest whose `/proc/self/ns/user` exists, which is what a defect of the companion's own code
would give, must refuse the add as a failed arm and must print no skip line; a `CLI_GIT_TEST_ONLY_*` fixture whose
child answers `EPERM` at the second mapping write must do the same, and it needs that seam because
`SystemCallFilter=` takes system call names and cannot single out one `write` of three (systemd.exec(5),
`SystemCallFilter=`), so a skip rule that took every errno from a call in the setup fails its own control. A guest
whose kernel is built without `CONFIG_USER_NS`, so `/proc/self/ns/user` is absent, is the other host refusal the
skip list names: it must print the skip line naming `unshare` and `EINVAL`, must not refuse the add, and until such
a guest exists this arm is unrun and this item says so. A guest booted with
`kernel.apparmor_restrict_unprivileged_userns` at 1, on a
distribution whose default profile denies capabilities inside the namespace, is the arm the skip boundary exists
for: it must print the skip line naming the setup call the kernel denied and `EPERM`, must record which call it
named (the first mapping write on a mainline reading, as [The account companion](#the-account-companion) states),
must not refuse the add, and must leave `doctor` naming the setting and the skip. A guest whose kernel has no
`/proc/sys/kernel/unprivileged_userns_clone` must print `absent` for it rather than nothing. And because the child
repeats the probe's syscall sequence in its own no-allocation island, this
run compares that sequence against the probe's arms on the same controls, so an island that diverged from the probe
fails its own control.

Gates: the account companion's arms, its skip path, its per-add deadline, and the assumption that tmpfs accepts the
descriptor calls it makes.

## Questions the owner answered

The question round closed on 2026-11-19, and every answer is under
[Answers of 2026-11-19](#answers-of-2026-11-19), which states each question, the option the project chose, the
timestamp range and the option it did not take. No question of this design stays open. Three of its own choices stay
open to the project's veto, each with veto none as the default:

- The companion's printed skip letting the add proceed. A veto refuses the managed add instead, which turns the
  managed route off on every host that skips, the Ubuntu guest included.
- No probe deadline value before step 6's measurement. A veto needs the project to name a provisional ceiling, and
  step 19 then adds the unit's `ExecStartPre=` for the probe before that measurement, which is why the probe's
  deadline sentence points at step 19.
- No companion deadline before step 10's. A veto needs a provisional ceiling too, and the companion then runs in a
  shipped preflight and in `doctor` before [M11](#m11-the-account-companion-per-add)'s value exists.

Two questions travel beside those vetoes to the acceptance round, because each changes what the design keeps. One is
the companion's skip, ranked against refusing the add, with its reach and the Ubuntu guest stated. The other is where
the `first-attach` claim sits: before the walk, which keeps a drifted slot's refusal as that boot's record, against
claiming at `fsopen`, which keeps only the attach calls' outcomes. What stays open beside them is measurement: every
item under [Owed measurements](#owed-measurements) is unrun, and nothing here authorizes work on this host's pool.

## What a writer settles without asking

- The scratch location: the record's directory under `/run`, the tmpfs mounted at its scratch mount point, the
  slot, the move-target directory and the refusal tree inside that tmpfs, and that the probe never touches the
  account-owned staging folder or the checkout's own directories.
- The name rule: the source is the configured dataset's `ready` snapshot, the root-owned configuration is the
  only place the name comes from, an absent or unparseable configuration records the later arms as not exercised,
  and a missing dataset or snapshot is never created.
- The arm order, the private-namespace setup before any mount, and every syscall the probe uses.
- That the probe runs at boot only: one root process per boot, from `ExecStartPre=-` of the existing unit, with
  no on-demand trigger and no second writer of the record. A module cannot be reloaded on this host without a
  reboot, so a module change arrives with a reboot that runs the probe again
  ([The kernel the design names](#the-kernel-the-design-names)); the probe also runs again when the checkout
  provider is restarted, because the unit requires that provider
  ([Who runs it, and where](#who-runs-it-and-where)). A trigger keyed to a module load would add a second writer
  of the record and a probe that can race an attach, and this host cannot exercise it, so this design does not
  propose one. A record whose fingerprint no longer matches the running kernel, module or boot is stale, and the
  managed route then refuses the attach and keeps the intent (the [proposal][proposal], lines 351 to 352).
- That the holder is forked by the mode process and the unmount child by the worker, that each child carries a
  parent-death signal and pipes of its own as
  [The pipes and parent-death signals each child owns](#the-pipes-and-parent-death-signals-each-child-owns) states,
  and that the mode kills the worker's process group at the deadline.
- That the holder starts before the move and attaches its own mount of the same source.
- That [The pipes and parent-death signals each child owns](#the-pipes-and-parent-death-signals-each-child-owns)
  settles who closes which pipe end, when a child moves its ends to low descriptors, and that the mode writes the
  record from the outcome lines it reads.
- That the namespace arm's holder runs at every start, with its cost under [M4](#m4-what-the-probe-costs), and
  that the scan is judged against the probe's own namespaces rather than against an empty set.
- That the move and unmount arm closes every descriptor the worker holds on the attached mount before it forks,
  and that its unmount must not answer `EBUSY`.
- That the probe reads no file content, only `fstatfs`, a bounded directory listing on its own mount, and mount
  facts.
- The mount flags, the read-only request, the exclusive create as the freshness oracle, and the `EBUSY` handling
  [M1](#m1-which-source-exercises-the-dataset-mount-path) measures.
- That the unmount and the move are by path inside the probe's own tmpfs, which the unmount answer places in
  step 6 ([Q1](#q1-whether-the-plan-bars-the-probes-unmount-arm)), and that the probe therefore does not validate
  the product's held-descriptor unmount. The companion's tmpfs unmount is its own and is built with it.
- That the probe tears down without unmounting its own tmpfs, whose removal is the namespace's death.
- The wording of the wrapper line, the `doctor` section and the journal line, under the design's own reporting
  rules (the [proposal][proposal], lines 755 to 762).
- The record's directory, mode, fields and atomic-rename write, and the started record before the fork.
- The `first-attach` file's keys, its claim, its staged rename and the reader's refusals, as
  [The first real attach's record](#the-first-real-attachs-record) states, and that the handler process that made
  the attach is the only writer.
- That the account companion runs in the add preflight before the claim, in a child of the wrapper's process, and
  that its failure refuses only that add's managed attach.
- That `doctor` reads the result and never runs the root probe, and that no force flag exists.
- That the probe is a handler mode named `start-probe`, run as `ExecStartPre=-` of the existing unit, that it runs
  the module gate first and skips itself when the gate refuses, that the mode process enforces the deadline and
  exits successfully whenever it reaches its own end, and that no on-demand channel is added.
- That no unit start timeout is added and no plan step sizes one: step 22 sizes the mount-request deadline of
  step 7, the unmount deadline of step 9 and any shipped deadline those waits would need (the [plan][plan], lines
  1778 to 1779 and 1811 to 1812). The two deadlines' values are this design's own choice pending the veto, stated
  in the deadline's value and margin paragraphs under [Who runs it, and where](#who-runs-it-and-where).
- The fallback for an unused slot and the preservation of existing bindings, which the accepted design already
  decides.

## What this proposal does not change

It does not introduce an allowlist, does not withdraw or loosen the module gate, does not authorize a path mount
or a new request channel, does not add a force flag, does not make the probe a `doctor` action, does not create or
destroy a dataset at start (the probe dataset is the owner's act through the setup steps), does not change the
intent vocabulary, does not move the handler's `boot-reconcile` and `shutdown-unmount` modes out of the host mount
namespace, does not change any removal, promote, staging or delegation decision, and does not decide any of the
remaining blocking designs. It does add the two duties the answers chose: the account companion in the add
preflight, and the `first-attach` write in the handler's attach path, which the plan's steps 4, 6, 8 and 10 carry
as [Notes for the plan](#notes-for-the-plan) records. It also does not bring the discarded pool-writing probe
back: that one contradicts the accepted design, as [The shape the answers chose](#the-shape-the-answers-chose)
states.

## Sources read

- [The decision record][decision], whole.
- The answer record: `doc/handover/cli-git-rust-implementation.md` in the main checkout at main's `92417f3758`,
  line 40398, inside the paragraph that carries the words `THE ROUND WAS ASKED`. It holds the seven answers
  [Answers of 2026-11-19](#answers-of-2026-11-19) records and the threat-model sentence quoted there, which that
  section keeps because rule `H0P` prunes a handover once its work lands.
- [The proposal][proposal], the sections this document cites, and the section lists it names: "Open items the
  decisions leave", "ABI gate and tests", "Project decisions", "Decided defaults", "Descriptor mount and
  unmount", "Boot and shutdown unit", "Waiting and refusal", "What doctor checks and prints" and "Acceptance
  tests the build still owes".
- [The plan][plan], the sections this document cites, and the section lists it names: "Dependencies and their
  state", "Wrapper code the design reuses" (read whole), "What the design forbids", steps 4, 6, 7, 8, 9, 10, 14,
  16, 18, 19, 20 and 22, "What can start now", "Open items and the step that needs each", and "Where the design is
  silent: this plan's suggestions". Every plan line number this document cites is a line of the plan at commit
  `73eaad0846`, the branch's base; [Notes for the plan](#notes-for-the-plan) states that base and names each step
  by heading, and it names the two places that read main's copy by their step and text, because main's copy moved
  after the base and moved again after the fourth revision read it.
- [The measurements record][measurements], the items this document cites, each with the measured output beside
  it.
- OpenZFS `zfs-2.4.4` at commit `71a9f9578616`, cloned on 2026-11-19:
  `module/os/linux/zfs/zpl_super.c`, `module/os/linux/zfs/zfs_vfsops.c`,
  `module/os/linux/zfs/zpl_ctldir.c`, `module/os/linux/zfs/zfs_ctldir.c`,
  `module/os/linux/zfs/zfs_ioctl_os.c`, `module/os/linux/spl/spl-zone.c`, `module/zfs/zfs_ioctl.c`,
  `module/zfs/dmu_objset.c`, `module/zfs/dsl_crypt.c`, `module/zfs/dsl_dataset.c`, `module/zfs/spa.c`,
  `module/zfs/spa_stats.c`, `module/zfs/txg.c`, `module/zfs/vdev_label.c` and `cmd/zfs/zfs_main.c`.
- The kernel header and source named under [The kernel headers and source](#the-kernel-headers-and-source): this
  host's `/usr/include/linux/mount.h` (linux-api-headers 1:7.1-1), the 7.3.5 tree's
  `/usr/src/linux-cachyos/include/uapi/linux/mount.h`, Linux v6.8 and v7.3 `fs/super.c`, Linux v7.3
  `fs/fsopen.c`, `fs/namespace.c`, `fs/pnode.c`, `kernel/exit.c`, `kernel/sys.c` and `kernel/nsproxy.c`, with
  `fs/fsopen.c` fetched at v5.2, v5.11, v5.12, v5.15 and v6.0 to v6.7 as well. The round that added the
  parent-death and unmount citations fetched `fs/namespace.c`, `fs/pnode.c`, `kernel/exit.c` and `kernel/sys.c` at
  v7.3 itself. For the account companion, Linux v7.3 `kernel/user_namespace.c` and `kernel/ucount.c` (fetched, for
  the `ENOSPC` of an exhausted namespace count, the capability check `map_write` makes before its parse, and the
  setting that bounds the count), `kernel/capability.c` (fetched, for what `file_ns_capable` asks and that it answers
  a bool), `kernel/fork.c` (fetched, for the `EINVAL` a `CLONE_NEWUSER` unshare from a multithreaded caller answers
  and for `ksys_unshare`'s call to `unshare_userns`), `include/linux/user_namespace.h` (fetched, for the `EINVAL`
  the inline `unshare_userns` answers on a kernel built without user namespaces) and `fs/proc/namespaces.c`
  (fetched, for the user entry `/proc/<pid>/ns` has only with `CONFIG_USER_NS`), `kernel/nsproxy.c` and
  `fs/namespace.c` (for the capability the mount namespace's own setup needs, and for `mount_setattr`'s
  `may_mount` asking no mount hook), `fs/namei.c` (for `rmdir` of a mount point and `detach_mounts`) and
  `mm/shmem.c` (fetched, for tmpfs's `FS_USERNS_MOUNT` and its fs_context
  operations). For the errno a refused user namespace answers, Linux v7.3
  `security/selinux/hooks.c`, `security/selinux/avc.c`, `security/apparmor/lsm.c`, `security/apparmor/task.c` and
  `security/apparmor/lib.c` (fetched), each read at the hook `create_user_ns` calls, the one that hook reaches, the
  capability check the first mapping write makes, and the mount hook.
- Ubuntu's 24.04 release notes, "Unprivileged user namespace restrictions", for the default AppArmor profile that
  allows the user namespace and denies the capabilities inside it, which the design's own measurement guest carries.
- `user_namespaces(7)` and `unshare(2)` read on this host, for the mapping rules, the `setgroups` order and the
  errnos a refused namespace creation answers.
- systemd v262 `src/core/service.c`, `src/core/unit.c` and `src/core/exec-invoke.c` (fetched), and
  `systemd.service(5)`, `systemd.exec(5)`, `systemd.unit(5)`, `systemd.kill(5)` and `statx(2)` read on this host.
- `PR_SET_PDEATHSIG(2const)` on this host, for the parent-death signal and the cases that clear it.
- This host, read only, 2026-11-19: `uname --kernel-release`, `/sys/module/zfs/version`, `/proc/kallsyms`,
  `/proc/filesystems`, `/sys/kernel/btf/`, `/dev/zfs` mode, `/proc/modules` (for the `zfs` use count),
  `/proc/spl/kstat/zfs/zpcachyos/txgs`, `/sys/module/zfs/parameters/zfs_txg_history` and `zfs_txg_timeout`,
  `findmnt --target /`, `/proc/sys/user/max_user_namespaces`, `/proc/sys/kernel/unprivileged_userns_clone` and the
  absence of `kernel.apparmor_restrict_unprivileged_userns` with `/sys/kernel/security/lsm`.
- The wrapper's code: `package/git-policy/cli/src/native/subvolume/fs_beneath.rs`,
  `.../subvolume/fs_mount_options.rs`, `.../subvolume/subvolume_remove_walk.rs`,
  `.../health/doctor_filesystem.rs`, `.../service/wait_ceilings.rs`, `.../service/service_provision.rs`,
  `.../invocation/git_version.rs`, `.../invocation/git_version_cache.rs`,
  `.../foundation/private_storage.rs`, `.../foundation/private_storage_replacing_a_record.rs`,
  `.../foundation/plain_file.rs`, `.../foundation/lazy_runtime.rs`, `.../process/process_identity.rs`,
  `.../process/open_file_of_a_descriptor_in_a_child.rs`,
  `.../process/stuck_children_reaped_when_the_kernel_lets_go.rs`, `.../push/start_retry.rs`,
  `.../source_guard/no_self_invocation_tests.rs`,
  `.../source_guard/blocking_code_off_the_runtime_tests.rs`,
  `.../binary/binary_worktree_removal_around_a_volume_tests.rs`, `clippy.toml`, and `package/git-policy/cli/SPEC.md`
  "Compatibility and degradation".
- The project's own records the companion's child relies on: `doc/planning/cli-git-no-self-invocation.md` (the bar on
  starting the wrapper), `doc/troubleshooting/run-wrapper-pids-max-refuses-fork-and-threads.md` (a refused `fork`
  under a session's task budget) and `doc/troubleshooting/pdeathsig-fires-when-the-parent-thread-ends.md` (what the
  parent-death signal follows), with tokio 1.54.1 `src/runtime/blocking/pool.rs` for the pool's keep-alive.

[decision]: ../decision/cli-git-zfs-worktree-design.md
[measurements]: cli-git-zfs-measurements-owed-before-acceptance.md
[owed-tests]: cli-git-zfs-worktree-design-proposal.md#acceptance-tests-the-build-still-owes
[plan]: cli-git-zfs-worktree-implementation-plan.md
[proposal]: cli-git-zfs-worktree-design-proposal.md
