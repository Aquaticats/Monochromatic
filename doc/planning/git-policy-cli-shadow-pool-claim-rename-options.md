# A commit's claim of a pooled shadow, and the ways to take its rename off the commit's wait

What this answers:
what a commit waits for when it takes a finished shadow from the pool,
which part of that wait costs the seconds the owner's bar is about,
every way to take that work off the commit's wait,
what each way changes in the guarantee `SPEC.md` records for a claim,
and which of them ranks first.

What this record is not:
a change.
No production code follows it.
The claim is the owner's decision
(`doc/handover/questions-for-the-owner.md`,
"May a commit take its private repository from a pool made ahead?"),
and every option here either changes the mechanism the owner named
or narrows the check the pool's guarantee states,
so each needs the owner's call.

The question came from the agent of
`perf/git-policy-cli-commits-within-5-s-of-native-shadow-pool-index-and-staged-deletions`,
whose traced commit put 3,126 ms in the claim and 1,678 ms of that in one `rename`,
and whose section
"The pool's claim,
the index's refresh,
and the answer that holds no step"
(`doc/planning/cli-git-commit-latency-under-many-agents.md`,
commit `88ae81a78b` on that branch,
not yet in this branch's copy of the file)
left the two mechanisms it could see as the owner's call.
This record measures them and finds that neither of them touches the call that costs the seconds.

## The claim today, and the guarantee it keeps

### What a claim does

`shadow_pool::take` lists `<common-dir>/monochromatic-git-policy-cli-shadow-pool/<worktree>`,
renames the first `member-` entry onto the commit's `<scratch>/shadow`,
and `shadow_pool_member::refusal` checks the member that landed there
(`package/git-policy/cli/src/native/shadow/shadow_pool.rs`,
`shadow_pool_member.rs`).
The rename is the claim:
two commits cannot rename one member,
so there is no lock and no commit waits for another
(`AGENTS.md`,
`NLK`).
The service fills the pool beside the landing loop,
never on a commit's path
(`service_shadow_pool_fill.rs`).

### What `SPEC.md` records

`package/git-policy/cli/SPEC.md`,
section "A shadow taken from the pool",
states three things this record compares every option against:

- "A claimed member is checked before it is used,
  and everything the check reads is a read:
  its `config` must be the text this invocation's creation would write,
  its alternates must name this invocation's object directory,
  its `HEAD` must be the creation's own placeholder,
  every directory of the skeleton must be there,
  every entry a commit reads that the repository holds now must be a link to this repository's own entry,
  every entry the member holds must be one a creation makes or a link at one of those names,
  and no pack and no ref."
- "A claim is one `rename` of a member onto the commit's `<scratch>/shadow`,
  which fails for the second claimant,
  so no lock is taken and no commit waits for another."
- The owner's chosen option's own words:
  "The resident service fills a pool of finished shadows,
  a commit renames one into place,
  and it builds inline as today when the pool is empty."

No record in `doc/decision/` covers the pool:
`rg "pool" doc/decision/cli-git-concurrent-commits.md` and
`rg "shadow pool|shadow-pool|pool of finished" doc/decision/`
answer nothing.
The guarantee is the `SPEC.md` section and the owner's words,
and nothing else.

## Where the numbers come from

### The host and the volume

Every fixture ran on the repository's own volume.
`${HOME}/temp` is a symbolic link to `/var/mnt/encrypted/temp`,
so scratch under `${HOME}/temp/agent` (rule `ATH`) is on `/dev/mapper/crypt_sda`,
the btrfs volume the repository lives on,
which reported 89 percent of its space used when this was measured.
The host's load average was 74.93 over eight processors when the first run started
and 63.88 near the last one;
no figure here is a claim about a quiet host
(`AGENTS.md`,
`NQH`).
Two harnesses report separately below:
one measures calls against a fixture laid out the way the pool lays one out,
and one measures real wrapped commits.

### The arms, and the control beside them

`${HOME}/temp/agent/shadow-pool-claim-options/claim-arms.ts`
makes a fixture whose common directory holds four shared entries,
whose pool holds two finished members,
and whose scratch parent waits for a commit's tree.
It is one fixture for every arm,
not a reflink copy per arm:
the claim's calls depend on the entries a common directory shares,
which the fixture fixes,
and never on what a repository tracks,
so one fixture is what lets each arm pair with its control in the same round.
Each arm is one option's sequence of calls,
written out call by call from `take` and `refusal`,
and every arm alternates with a control:
the same binary runs today's sequence,
the order flips each round,
and the control's own figures sit beside each arm's
(table-free figures in "What each option measured").
300 rounds per arm,
and a control run beside each,
4,200 of them over the fourteen arms of the last run.
`trace-arms.ts` and `count-calls.ts` run one arm alone under `strace --syscall-times`
and count the calls between two marks the harness writes around its own arm,
so every count below is counted,
never inferred.

### The wrapped commits

`${HOME}/temp/agent/shadow-pool-claim-options/wrapped-claim.ts`
makes a disposable repository of 200 files on the same volume,
with its own copy of this branch's release wrapper,
its own `HOME`,
its own `XDG_RUNTIME_DIR`,
its own commit service and a throwaway signing key,
and makes 330 SSH-signed explicit-path commits over three runs.
A signed commit is one the service hands to real Git,
which is the route that builds a shadow and takes its claim from the pool.
Every commit is traced with `strace --follow-forks --syscall-times`.
`claim-share.ts` cuts each trace between the pool's listing and the claim's last emptiness question,
reads the one `rename` inside that window on its own,
and compares the window against the stretch the commit's own record names.
The two agree to a median of 0.25 ms and a largest of 0.86 ms over the 330 commits,
so the figure a record writes as `buildParts."shadow pool"` is this claim,
and the split inside it is the trace's.

### What these measurements cannot resolve

- The volume stalls calls in bursts,
  and the bursts do not fall evenly on the kinds of call.
  The arms and the primitives met almost none of them;
  the wrapped commits met 22 in 330 claims.
  So an arm's median says what an option costs when nothing stalls,
  and only the wrapped commits say what stalls.
- Whether another option's remaining write stalls as today's directory rename does
  can only be answered by building that option and committing through it.
  The one window in which the kinds were compared side by side is
  "The kind of call that waits".
- One host,
  one volume,
  one repository shape.
  The fixture's common directory shares four entries where this checkout shares four
  and a fresh `git init` shares two,
  so a call count below moves with that number and a median does not.

## What a commit's claim costs today

### The calls, counted

A claim in the wrapped fixture makes 41 calls:
8 `openat`,
which is 5 directory listings (the pool,
the member,
`objects/pack`,
`refs/heads`,
`refs/tags`)
and 3 files opened,
1 `rename`,
24 `statx`,
6 `read`
(the `config`,
the alternates and the `HEAD`,
each read twice),
and 2 `readlink`.
The arm fixture's common directory shares four entries instead of two,
so its version of the same sequence is 47 calls:
the two more shared entries are 4 `statx` and 2 `readlink`.
The creation the pool keeps a commit from making is a different order of magnitude:
the reading over 86 commits in
"What each part of a commit's private repository is for,
and how many commits need it"
names `shadow skeleton` 15,052 ms over 3 commits,
so a commit the pool cannot serve pays seconds where a claim pays a listing and a rename.

### The cost, over 330 commits

Every one of the 330 commits took a member;
none fell back to a creation.

- The claim's whole stretch:
  median 3.11 ms,
  ninetieth percentile 68.57 ms,
  ninety-ninth 975.22 ms,
  largest 5,622.57 ms.
  22 claims passed 100 ms,
  8 passed 500 ms,
  and 3 passed 2 s.
- The one `rename` inside it:
  median 0.08 ms,
  ninetieth percentile 24.07 ms,
  ninety-ninth 926.48 ms,
  largest 5,606.21 ms.
  17 renames passed 100 ms,
  7 passed 500 ms,
  and 3 passed 2 s.
- The other 40 calls together:
  median 3.00 ms,
  ninetieth percentile 12.39 ms,
  ninety-ninth 63.60 ms,
  largest 81.74 ms,
  and not one of them passed 100 ms in any of the 330 claims.

The claim is one stretch of a commit,
and in the commit whose claim took 5,622.57 ms the record's whole time was 7,007 ms,
so that claim alone was four fifths of it and longer than the owner's five seconds.

### Where a stalled claim's time sat

In 19 of the 22 claims that passed 100 ms the `rename` was the largest call,
and in the three where it was not the claim was 108,
108 and 124 ms
with 69,
64 and 82 ms of it in the reads.
The largest ones are all the rename:

- 5,623 ms claim:
  5,606 ms in the rename,
  16 ms in the other 40 calls.
- 3,080 ms claim:
  3,062 ms in the rename,
  17 ms in the other 40 calls.
- 2,995 ms claim:
  2,969 ms in the rename,
  26 ms in the other 40 calls.
- 975 ms claim:
  926 ms in the rename,
  49 ms in the other 40 calls.

So on this volume the claim's tail is one call:
a directory `rename` across two directories,
which is a metadata write that waits for a transaction,
while the forty reads around it are answered from the cache in microseconds
in the same commits.
The second agent's traced commit put 1,678 ms of 3,126 ms in the rename and the rest in the listings,
which is the same call waiting longer than this record saw and a longer burst catching the reads with it;
the two readings agree that the rename is the call that pays first.

## The kind of call that waits, measured in one window

`${HOME}/temp/agent/shadow-pool-claim-options/mix.ts`
times five kinds of call round after round in one fixture,
120,000 rounds of each,
while 150 wrapped commits ran on the same volume,
so every kind met the same stalls:

- A directory renamed across two parents,
  which is today's claim:
  median 0.076 ms,
  largest 4,787.009 ms,
  8 of 120,000 passed 100 ms,
  5 passed 500 ms,
  3 passed 1 s.
- A directory renamed within one parent,
  which is a claim that keeps the commit inside the pool's folder:
  median 0.073 ms,
  largest 3,141.981 ms,
  8 of 120,000 passed 100 ms,
  4 passed 500 ms,
  4 passed 1 s.
- One file renamed within the member's own directory,
  which is a claim by reservation:
  median 0.077 ms,
  largest 970.924 ms,
  2 of 120,000 passed 100 ms,
  1 passed 500 ms,
  none passed 1 s.
- One file created within the member's own directory,
  which is the other form of a claim by reservation:
  median 0.098 ms,
  largest 2,483.113 ms,
  3 of 120,000 passed 100 ms,
  1 passed 500 ms,
  1 passed 1 s.
- The member's listing:
  median 0.030 ms,
  largest 14.479 ms,
  none passed 100 ms.
- One live entry's type:
  median 0.020 ms,
  largest 5.747 ms,
  none passed 100 ms.

The reads did not wait once in 240,000 questions,
and the writes waited 21 times in 480,000.
The difference between one kind of write and another is not resolved:
2 file renames and 3 file creates against 8 directory renames in one window
is not enough events to separate them,
and the mechanism that would explain a smaller number for a file rename,
one directory's entry instead of two and the moved directory's parent pointer,
is reasoning and not a measurement.

## What each option measured

Every median here is one arm's 300 paired rounds against the control,
which measured a median of 1.256 ms over its 4,200 runs,
and every count is from `strace`.
The counts are the arm fixture's,
where today's sequence is 47 calls;
in the wrapped fixture's two shared entries the same sequence is 41,
and every count below is six lower there,
because the two more entries add the same six calls to each.
What the wrapped commits say an option keeps is said with each:
a claim that keeps its `rename` keeps 17 of 330 claims over 100 ms and 3 of 330 over 2 s.

### A. The service hands a drawn tree out

The service renames a member into a drawn tree of its own before the commit asks,
and the commit is handed that path.
Two forms:
it runs every check on the handed tree (the 45 calls of the reads,
no write),
or it trusts the service's answer (no calls at all).

- Measured:
  1.142 ms median for the checked form (45 calls,
  ninetieth percentile 2.453 ms),
  0.003 ms for the trusting form (0 calls),
  and one unix socket round trip for the answer,
  which a stand-in server answered in 0.043 ms at the median,
  0.069 ms at the ninetieth percentile,
  0.135 ms at the ninety-ninth,
  and 1.684 ms at the largest over 3,000 round trips.
- Pros:
  it is the only option that leaves no write on the commit's wait,
  and the checked form keeps every question `SPEC.md` records,
  so the guarantee is unchanged.
- Cons:
  the commit waits for a new answer from the service,
  which must come from outside the serialized landing step or every commit queues behind every other;
  a drawn tree's lifetime becomes the service's,
  so a killed client leaves one for the service's sweep and a worktree removal must leave a drawn one alone;
  an empty stock leaves the commit with today's rename or a five-second creation;
  and it changes what the owner chose,
  because the service renames and the commit is handed a tree
  rather than renaming one into place.
- What a person sees differently:
  nothing in a commit that runs as it does today,
  and one thing in a failure:
  a commit against a stopped service already ends with the platform's message before anything changes,
  and a commit that finds an empty stock is slower by exactly the rename or the creation.

### B. The claim renames a mark inside the member, and the commit runs in the member

The fill leaves a mark file in each member
(`available`),
and a claim renames that file inside the member's own directory,
which fails for the second claimant because the mark is gone.
The commit then runs in `<pool>/member-<name>` instead of `<scratch>/shadow`,
and every check runs on it.

- Measured:
  1.266 ms median (47 calls,
  the same count as today,
  with the one write's kind changed),
  against the control's 1.256 ms in the same run.
  In the one window that compared kinds,
  this write stalled in 2 of 120,000 calls
  where today's stalled in 8 of 120,000,
  which is not a resolved difference.
- Pros:
  every question `SPEC.md` records is still asked,
  the claim is still one `rename` that fails for the second claimant,
  so there is still no lock;
  and the write becomes a file inside one directory instead of a directory across two.
- Cons:
  it makes the pool a place where live commits run,
  which four recorded rules assume it is not:
  the pool is torn down with one `rename` of the whole folder before anything is deleted
  (`worktree_removal_clears_the_pool.rs`),
  a sweep never reaches a member,
  a commit's tree is released with its scratch directory,
  and the check that a commit leaves no scratch behind means what it says
  (`SPEC.md`,
  "A shadow taken from the pool").
  A claimed member is now a live tree those rules must leave,
  and a member spent by a commit that stops
  is a member the pool no longer has.
- What a person sees differently:
  a `git worktree remove` of a worktree another commit is committing in:
  today the running commit keeps its own tree in scratch and lands;
  under this option its tree is inside the folder the removal renames away,
  so the removal must be taught to leave a claimed member,
  and until it is,
  that commit's tree is deleted under it.
  A person who looks in the pool folder also finds live trees in it,
  not only standing ones.

### C. The claim trusts the fill, and the member is named for the template's digest

The fill's member is verified when it is made,
the member's own name is the digest of the template it was made from,
and the claim recomputes that digest from live facts and renames `<pool>/member-<digest>`.
Without the digest the form is the same trade with today's names:
a pool listing and the rename,
nothing else.

- Measured:
  0.165 ms median for the digest form (3 calls:
  one listing of the common directory,
  one question about `config.worktree`,
  and the rename),
  and 0.096 ms for the naming-free form (2 calls:
  the pool listing and the rename).
  The digest form's figure holds a JavaScript `SHA-256` of the template,
  which a Rust claim would pay in microseconds,
  so its real figure sits below the measured one.
- Pros:
  38 or 39 of the 41 calls go,
  and the median drops from 1.256 ms to about a thirteenth of it.
- Cons:
  it keeps the `rename`,
  so it keeps every stalled claim this record measured:
  17 of 330 over 100 ms and 3 of 330 over 2 s.
  It also narrows the pool:
  a member's name is its template's digest,
  so a second fill of the same template cannot publish beside the first,
  and the pool holds one member per template where it holds two of anything today.
  It saves at most 82 ms in the worst claim of 330,
  which is the other 40 calls' largest.
- What it changes in the guarantee:
  the member's tree on disk is no longer compared with what the claim would make.
  What a person sees differently:
  anything that writes to `<pool>/member-<name>/config` after the fill,
  a person,
  a script,
  a restore,
  or storage that hands back a different block,
  is used instead of refused.
  Concretely,
  a `core.hooksPath`,
  a filter or a `core.worktree` added there after the fill
  takes part in that commit,
  and Git writes its objects where that config says;
  today the same change makes the commit refuse the member,
  say nothing on the terminal,
  and build its own shadow,
  which lands the commit the person asked for.

### D. The fill writes a manifest the claim trusts

The fill writes the member's entries,
their link targets and the digests of its files into a manifest beside the member,
and the claim reads that one file instead of asking the member anything.

- Measured:
  0.201 ms median (6 calls:
  the pool listing,
  the rename,
  and the manifest's two reads).
- Pros:
  the member's entries are named in one place a diagnostic can print,
  and the claim is one read away from trusting nothing at all.
- Cons:
  it is C's trust change with C's exposure,
  and it costs one call more:
  it keeps the pool listing that the digest form removes,
  and it adds a file to read on the commit's wait.
  It does not touch the rename either,
  so it keeps the measured tail.
- What a person sees differently:
  exactly what C's section says,
  because the trust is the same and only the record of it differs.

### E. The claim keeps every check, with a smaller skeleton and one listing of the common directory

Two savings that change no question:
the creation leaves `objects/pack`,
`refs/heads` and `refs/tags` to Git,
so the claim answers "no pack and no ref" with one question about each absent directory
instead of listing three;
and the live shared entries come from one listing of the common directory
instead of nine type questions.
The change another branch already measured,
one link question per shared entry,
belongs beside them and is not this record's.

- Measured:
  0.775 ms median (23 calls),
  against 1.256 ms for today's sequence and 1.048 ms for the one-question change alone.
- Pros:
  every question `SPEC.md` records is still asked,
  the claimed member is still the tree a creation makes
  (the creation's skeleton moves with the check,
  so the two agree),
  and it is 24 calls fewer with no trust asked for.
- Cons:
  it does not touch the rename,
  so it keeps the measured tail and moves nothing about the owner's bar.
  It also changes what a creation makes,
  so `create_shadow_skeleton` and the pool's members change together,
  and Git must run in a shadow that has no `objects/pack`,
  `refs/heads` or `refs/tags`
  until it writes one:
  `write_shadow_refs` writes `packed-refs` and then per-worktree refs,
  and whether Git and those writes are content without those directories is a probe this record does not claim.
- What a person sees differently:
  nothing,
  if that probe passes.

### F. Nothing

- Measured:
  the 330 commits above.
- Pros:
  the guarantee stays exactly as `SPEC.md` states it,
  and no rule about the pool's removal,
  the sweep or a scratch release changes.
- Cons:
  22 of 330 commits wait over 100 ms in the claim,
  3 over 2 s,
  and the largest claim is 5,622.57 ms,
  which is longer than the owner's five seconds on its own.

## What was dismissed, and why

- A claim that renames the member within the pool's own directory.
  Measured head to head with today's across-parents rename:
  8 stalls of 120,000 calls against 8 in the same window,
  largest 3,141.981 ms against 4,787.009 ms,
  and 0.073 ms median against 0.076 ms.
  Same call,
  same cost,
  and it carries every consequence of B without B's one chance.
- A claim run beside the commit's own preparation.
  Two runs measured the same pair,
  with a stand-in for the stat pass in each:
  one measured the pass alone at 74.349 ms,
  77.127 ms one after the other with today's claim,
  and 80.716 ms at the same time;
  the other measured 66.829,
  68.738 and 67.738 ms.
  So in one run the claim's 1.9 ms was hidden and in the other it cost 3.6 ms more than running alone,
  which is the noise of this host and not a saving.
  The pass is 66.829 to 74.349 ms,
  so a claim that waits 5,623 ms would still leave seconds on the commit's wait behind it,
  unless the pass were stalled too,
  and then both waits would be waiting for the same volume.
- A digest the claim verifies from the member's own bytes.
  It costs the member's listing and every question a check makes,
  which is today's sequence with a hash on the end,
  so it removes nothing.

## The ranking

1.  **A,
    the service hands a drawn tree out.**
    It is the only option that takes the one call this record measured waiting off the commit's path,
    and its checked form keeps every question `SPEC.md` records,
    so the guarantee is whole while the 5,606 ms wait is not the commit's.
2.  **B,
    the claim renames a mark inside the member.**
    It can remove that same wait and keeps every check and the lock-free claim,
    so it ranks above every option that cannot touch the wait at all;
    it ranks below A because its prize is unmeasured,
    its write is still on the commit's path,
    and it turns the pool into a place live commits run,
    which four recorded rules about the pool contradict.
3.  **E,
    the smaller skeleton and one listing.**
    It is a free saving:
    24 calls and 0.775 ms against 1.256 ms,
    with the guarantee unchanged.
    It ranks below B because its saving is bounded by the other 40 calls' largest,
    81.74 ms,
    while B's is potentially seconds.
4.  **F,
    nothing.**
    It keeps the guarantee and the pool's rules exactly as they are.
    It ranks below E because E takes 24 calls off the same path and asks nothing in return,
    and above C and D because neither of those moves the tail either,
    and both trade a recorded check for a median saving
    that is a rounding error against the seconds the owner's bar is about.
5.  **C,
    the claim trusts the fill and the member is named for the template's digest.**
    It ranks above D because it removes one call more,
    the pool listing,
    and because a member's name is the fill's to choose anyway,
    while D adds a file for the claim to read.
6.  **D,
    the manifest.**
    It is the same trust change as C with one call more and the pool listing kept,
    so it is the last resort if a readable record of a member's contents is wanted for its own sake.

## What this record asks the owner

Three decisions,
separate from one another.

1.  May a commit stop renaming its own tree into place,
    so that the service renames it ahead and hands it out (A),
    or the claim renames a mark inside the member and the commit runs there (B)?
    Both change the option's own words in
    "May a commit take its private repository from a pool made ahead?",
    so neither is taken here.
    A needs the service to answer outside its serialized step and to own a drawn tree's lifetime;
    B needs the pool's removal,
    sweep and scratch release to learn a claimed member.
2.  May the claim stop checking the member it takes,
    trusting the fill's verification instead (C,
    D)?
    The measured saving is 0.82 ms at the median and at most 82 ms in the worst claim,
    and the cost is that a member changed after the fill is used rather than refused.
    This record ranks it below doing nothing,
    because the tail does not move.
3.  May the creation leave `objects/pack`,
    `refs/heads` and `refs/tags` to Git,
    and the claim read the live
    shared entries from one listing (E)?
    Nothing about the guarantee changes,
    and the price is a probe of Git in a shadow without those directories
    and the recorded ref writes against it.

## What this record does not claim

- That a file rename,
  a file creation or a service's answer would not stall.
  The mix found 2 file renames and 3 file creations in 120,000 that stalled,
  which is too few events to separate from the 8 directory renames beside them.
- That the reads can never stall.
  The other agent's traced commit put 1,448 ms of its claim in the listings and the waits between calls,
  which this record's 330 commits did not reproduce;
  the reads' measured ceiling here is 81.74 ms in one claim,
  not a bound.
- That a claim's cost is the same on a quiet volume or in a bigger repository.
  Every figure here is this host,
  this volume and a disposable repository of 200 files.
- That any of these options lands a commit in less time end to end.
  The claim is one stretch of a commit whose records ran from 424 ms to 17,795 ms here,
  and the harness measured the claim,
  never the whole command against native Git.

## Where the evidence sits

Under `${HOME}/temp/agent/shadow-pool-claim-options`,
outside the repository:

- `claim-arms.ts`,
  the arms and their control;
  `trace-arms.ts` and `count-calls.ts`,
  the call counts.
- `wrapped-claim.ts`,
  the 330 wrapped commits;
  `claim-share.ts`,
  their claims split into the rename and the rest.
- `mix.ts`,
  the five kinds of call in one window.
- `out/arms-tail.json`,
  `out/arms-mark.json` and `out/arms-final.json`,
  the paired arm medians;
  `out/counts*.json`,
  the traced counts;
  `out/wrapped.json`,
  `out/wrapped-2.json` and
  `out/wrapped-3.json`,
  the records of the commits,
  and `out/traces-wrapped-run*.tar.zst`,
  their traces;
  `out/claim-share-all.json`,
  the split over all 330;
  `out/prims.json` and `out/mix.json`,
  the primitive and mixed-window figures.

## The design the owner's decision made

The owner decided on 2026-11-11 for option A of this record,
in the checked form:
the resident service renames a pool member into place ahead of a commit,
hands the commit that path,
and the commit still runs every check this record's
"The claim today, and the guarantee it keeps" lists on the tree it is handed.
Two things are decided with it:

- Options C and D are not taken.
  A tree a commit uses is compared with what its own creation would have made,
  at the claim as today and again at the hand-out.
- Option E is taken when its own probe passes.
  The probe runs Git and `write_shadow_refs` in a shadow without `objects/pack`,
  `refs/heads` and `refs/tags`,
  and its result is in "The option E probe and its result" below.

What follows is the design that carries the decision out,
written before the code,
and the sections below name the paths and the functions the code uses.

### Where the drawn trees are kept

A drawn tree is a finished shadow the service has already renamed out of the pool,
so no commit's wait holds a rename of its own.
It lives in `<common-dir>/monochromatic-git-policy-cli-shadow-drawn/<worktree>`,
a folder beside the pool's own and never inside it,
with the same key rule the pool uses
(`pool_key` in `shadow_pool.rs`),
and one entry per tree, named `drawn-<random identity>`.
It is the wrapper's own state of the kind the pool is,
so no shadow links either folder
(`shadow_links.rs`,
`is_shared_shadow_entry`).

The reason it is a folder of its own and not an entry of the pool's:
a claim takes a member by renaming it out of the pool,
so nothing a commit uses is ever inside the pool's folder,
and the removal of a removed worktree takes that folder out of its name with one rename
before anything of it is deleted
(`worktree_removal_clears_the_pool.rs`).
A drawn tree is used where it stands,
so it would be inside a tree that rename takes.
Beside the pool a drawn tree is something a removal has to leave alone by asking whether a commit holds it,
which is the question the sweep asks.

A member is a skeleton, a `config`, an `objects/info/alternates`, a `HEAD` and one link per shared entry,
so the drawn folder holds about what the pool holds:
at most one drawn tree beside the pool's `POOL_LIMIT` members,
which is why the draw stops at `DRAWN_LIMIT` (one) and leaves the pool's own limit where it is.

### The message a commit sends, and the answer it gets

A commit asks on the service's socket,
on a connection of its own that ends with the answer,
and never through the landing queue:

- ``{"type":"shadow","gitDir":"<path>","scratch":"<path>"}``,
  `ClientMessage::Shadow`,
  sent once per attempt by `ask_for_a_drawn_shadow`
  (`service_client_shadow_hand_out.rs`).
  `scratch` is the commit's own scratch tree,
  whose name carries the request's identity and the attempt's number,
  so an answer can be given to a question about a tree this commit holds.
- `{"type":"drawn","tree":"<path>"}`, `ServiceMessage::Drawn`,
  or `{"type":"no-shadow"}`, `ServiceMessage::NoShadow`.

The answer is given by the connection's own task (`service_server_connection_handling.rs`),
which is where `Warm` is answered today,
and never by the landing thread:
the stock of drawn trees is a value of the service's own (`service_shadow_draw.rs`),
read and taken under a lock that guards nothing on disk and dies with the process,
so a commit that waits for its tree waits for a message and a mutex that guards nothing on disk, so nothing makes a commit queue behind a landing.

A commit that is handed a tree asks nothing more of the service for it:

- it runs `refusal` (`shadow_pool_member.rs`) on the tree,
  which is the same check a claim runs and is all reads;
- it renews the tree while it lives,
  the way it renews its own scratch tree (`service_scratch_renewal.rs`);
- the request it then sends names the tree in a new field, `shadow`,
  so the service removes it once that request's landing is over,
  beside the scratch tree it already removes (`service_scratch_release_removal.rs`).

### The sweep, and how a tree in use is told from an abandoned one

A tree in use is a fresh drawn tree,
and a tree nobody uses is a stale one,
which is the rule the wrapper already has for a commit's scratch tree:
the client renews the tree it was handed every `SCRATCH_REFRESH_EVERY` while its command lives,
so a drawn tree is never older than an hour while a commit holds it,
and a client killed at any instruction stops renewing it,
exactly as its scratch tree does (`service_scratch_abandonment.rs`).
No lock is written, so no lock can be left held:
the fact that decides is the modification time of a directory,
which the wrapper writes in one call, and the two ends of the question are a client that renews and one that stopped.

`sweep_drawn` (`shadow_drawn.rs`) removes the drawn trees of a worktree older than
`SCRATCH_TREE_ABANDONED_AFTER` (two hours),
the age the scratch sweep uses and the measured bound for how long the longest commit lives,
and removes a worktree's folder once nothing is in it.
It runs in three places:

- beside every wrapped command,
  next to the sweep of a gone worktree's pool (`sweep_gone_worktrees`),
  so a repository nobody commits in still loses a tree a killed client left;
- beside the service's own draw,
  on that thread, so a busy repository collects as it goes;
- when a worktree is removed (`worktree_removal_clears_the_pool.rs`),
  where only that worktree's folder is asked,
  and where a fresh tree is left alone:
  a live commit's tree is fresh, and a removal that took it would take the ref store that commit had just written.

The pool's own rule is unchanged:
the pool of a removed worktree is still taken out of its name with one rename and deleted,
because nothing a commit uses is in it.

### A commit that finds no drawn tree, or no service

The ask is an offer, never a step a commit needs:

- `ask_for_a_drawn_shadow` gives the whole exchange `DRAWN_ANSWER_DEADLINE` (250 ms,
  which is a hundred and fifty times the largest answer this record measured,
  1.684 ms over 3,000 round trips),
  and it starts nothing:
  its connect and its reads are inside that one span.
  The demand is bounded, the answer is one line, and the service is asked nothing else.
- An empty stock is `NoShadow`,
  a socket nothing listens on is no connection at all,
  and a service that does not answer inside the span is a service that did not answer.
  In every one of those cases the commit runs the path it runs today:
  `take` renames a member out of the pool,
  and a pool with none left leaves the creation, within the bounds it has today.
- The fallback says nothing on the terminal.
  A slow claim or a slow creation is named in the commit's timing record by the build parts it already writes
  (`shadow pool`, `shadow skeleton`, `shadow config`, `shadow links`),
  and the hand-out names itself beside them as `shadow hand-out`.
  A commit that took a drawn tree names no creation part,
  so a reading of a slow commit tells the three routes apart.
- A tree the service hands out that this invocation's `refusal` answers with a reason is handed back
  (removed in the background, as a refused member is today)
  and the commit takes the path above:
  the service's copy of the check is the fill's, and this commit's own facts are what decide.

### Where the rename happens now, and what it can delay

The rename moves from the commit to the service,
where it runs on a thread of its own named `shadow-pool-draw`
(`service_shadow_draw::start`,
called beside the fill in `land_group`,
after the landing rather than before it,
and making a pool member itself when the pool holds none: a draw that ran beside the fill of the same landing
found a pool the fill had not filled yet and drew nothing,
which the built wrapper showed on four commits),
so what it can delay is:

- the next draw, which a guarded flag serializes, and nothing else on that thread;
- nothing the landing thread does:
  the draw takes a pool member and checks it, and the landing holds no lock on either;
- nothing a client waits for:
  the answer is a value in memory, and a draw that stalls 5,606 ms leaves the stock empty for that time,
  which costs the commits in it today's path and costs no commit a wait.

The second write a drawn tree costs is the renewal every `SCRATCH_REFRESH_EVERY`,
on the client's own thread,
and it is the same call the scratch tree already pays:
no commit's wait holds it.

### What the record's four cons became

#### The commit waits for one more answer from the service

It waits for one line,
answered by the connection's task and never by the landing thread,
and bounded at `DRAWN_ANSWER_DEADLINE` before it falls back.
An answer that comes while another commit lands is measured in
"What the hand-out costs" below,
against the same question asked while nothing lands.

#### A handed-out tree's lifetime is the service's

It is the renewal's and the sweep's,
and a removal leaves a fresh tree alone.
The three cases the record names:

- a client killed after the hand-out leaves a tree that stops being renewed,
  which the sweep removes once it is two hours old,
  exactly as the tree it would have made in its own scratch would go;
- a worktree removal asks its own folder and leaves a fresh tree,
  so the tree of a commit that is running in that worktree is still there when the commit's own Git reads it;
- an answer the service gave that this commit did not use
  (a request the service never took, a commit sent back to be prepared again)
  is released beside the command in the background,
  the way the scratch tree of such a request is released today.

#### An empty stock, or a service that does not answer

The commit falls back to the claim and then the creation,
inside today's bounds,
and says nothing unless the fallback itself is slow,
which the timing record names.

One case is left out of the hand-out, and the code says so where it decides:
a commit that concludes a merge,
a cherry-pick or a revert keeps the path it has today.
The landing that concludes compares the state Git left in the shadow with the copy it kept,
and it reads the shadow at the path the commit's own scratch tree holds
(`conclusion_state_copy.rs`,
`remove_conclusion_state`),
which a tree used where it stands does not occupy.
Naming the tree the request built in at that comparison is the follow-up this leaves,
and until it is made those commits claim or create as they always did.

#### The hand-out must not reintroduce a wait for the rename elsewhere

The rename is on the draw's own thread, which delays nothing but the next draw,
and neither the claim nor the creation waits for it.

### The option E probe and its result

The probe passed, and option E is built.

The question was whether Git and the wrapper's own ref writes are content in a shadow that holds no
`objects/pack`,
`refs/heads` and `refs/tags`,
because the creation would stop making them and the claim would answer "no pack and no ref"
with one question about each absent directory instead of three listings.
`${HOME}/temp/agent/commit-hands-out-a-tree/probe-e.ts` makes a disposable repository with one commit and a tag,
makes a bare shadow of it,
removes those three directories,
writes the shadow's `HEAD`,
its `packed-refs` with the base ref,
and its `objects/info/alternates` into the person's object directory,
and then asks real Git to work in it.
Git 2.56.0's answers:

```
for-each-ref in the lean shadow:
refs/heads/main fc577a51670c88ff9c2089f9874bcd4bf9ea73ee
git commit in the lean shadow exited 0
git log exited 0: 3550fba probe
fc577a5 a
git rev-parse HEAD exited 0: 3550fba50e74dce46177d8f4f4828c4c53bde76d
git status exited 0: ?? a.txt
```

So Git makes the loose ref it writes under `refs` and the object's own directory under `objects`,
and the wrapper's ref store writes no loose ref on the files backend beyond `packed-refs`
and the symbolic ones whose parent directories it already makes.
The build is in the commit that names this record:
a creation makes the directory itself, `objects`, `objects/info` and `refs`;
the claim refuses a tree that holds any of the three, which is what a shadow a commit used holds;
and the shared entries come from one listing of the common directory in the order the named list has them.
The package's end-to-end control that builds one commit in a creation and the same commit in a claimed member
runs in the lean member and lands both.

### What the hand-out costs

Measured on this machine, after the change, with the harness of
"Where the numbers come from" and a disposable repository of sixty files rather than two hundred
(a claim reads none of the worktree, and the smaller fixture is fewer calls the stalled device can hold past a commit's own ceiling):
80 wrapped SSH-signed commits per arm, four arms alternating the two builds,
so each build met the load twice and the spread of an unchanged build sits beside the difference.
The host's load average was 149 to 168 over eight processors while these ran,
and no trace was taken: every figure below is the commit's own timing record,
which "The wrapped commits" showed agrees with its trace to a median of 0.25 ms.

A commit's tree, as its own record names it:
the hand-out (or the claim, and a fallback's hand-out too) or the creation's three stretches.

- Before, first arm: 80 claims, no hand-out and no creation.
  `median 0.00 ms, p90 110, p99 1,975, largest 1,975; 9 of 80 over 100 ms, none over 2 s`.
- Before, second arm: 80 claims.
  `median 0.00 ms, p90 10, p99 7,106, largest 7,106; 2 of 80 over 100 ms, 1 over 2 s`.
- After, first arm: 79 hand-outs and one fallback (an empty stock, 7 ms in the claim).
  `median 0.00 ms, p90 2, p99 143, largest 143; 1 of 80 over 100 ms, none over 2 s`.
- After, second arm: 80 hand-outs.
  `median 0.00 ms, p90 0, p99 1, largest 1; none over 100 ms, none over 2 s`.

What these say, and what they do not:

- The ceiling moved. A claim in these arms waited up to 7,106 ms in the call that takes no argument,
  and a hand-out waited at most 143 ms, which is inside the 250 ms the question is given.
  That is the difference the option was chosen for, and it is a bound rather than a smaller average:
  a hand-out cannot wait longer than its deadline, and a commit that runs out of it takes the claim or the creation.
- The counts over 100 ms are too few to separate the builds on their own:
  11 of 160 before against 1 of 160 after, where the unchanged build alone spread from 9 to 2 of 80.
  The record's earlier window found 22 of 330, so this one was calmer for the claim and rougher for the whole commit.
- The whole commit is not the measure of this change:
  its medians were 599.5 and 167.5 ms before against 122.5 and 41 ms after,
  with p90 of 6,654 and 1,339 against 603 and 61,
  and the volume's bursts fall unevenly on the arms.
  What a commit's tree costs is the figure above; the rest of a commit is the work this record never measured.
