# cli-git ZFS runtime layout checker

## Status

Accepted 2026-11-20 in the question tool.
The answer was "Accept as written (Recommended)",
and it records no veto of the design's four veto-open choices.
The design is [`doc/planning/cli-git-zfs-runtime-layout-checker-design.md`][design] at `22703f23df`.
It is accepted as written.
Its body is the evidence:
the three-release behaviour comparison,
the reader candidates with their source audits and scores,
the ways it refused,
its limits,
and the measurements it still owes.
Those stay there.
This record copies none of them.
The acceptance changed no line of the design's body.
One note at its top gives the acceptance date and names this record.
The answers of 2026-11-19 live in the design's own "Answers of 2026-11-19" section.
Each is quoted again below with the choice it settled.

This record is an agent's writing of the project's decision.
It is unapproved and non-final (rule `SGO`).
The project accepted the design.
The project may correct any line here.
It was written at 2026-11-20T05:55:11-05:00,
which is the clock reading taken while it was written.

Two names this record uses.
The binding package is `package/zfs/request-to-openzfs-kernel-device`.
That crate owns the `/dev/zfs` request format,
the `zfs_cmd` type and the codec.
The gate is the compatibility check both binaries run before their first ioctl.

## Decision

In plain words,
the project accepted this:

- The binding package parses the loaded ZFS module's BTF file with the kernel's base BTF.
  It compares the declared layout with the expectations compiled from its own `#[repr(C)]` types.
  Those expectations are `zfs_cmd`'s size,
  every member's bit offset and size,
  the anonymous union and struct with their members,
  every member of `dmu_objset_stats` and `zinject_record`,
  the base integer sizes those members use,
  and every request number the binding can send with the range's two endpoints.
- No step reads the module's release string as a rule.
  The layout comparison and the request numbers decide.
  `doctor` prints the release string beside its verdict.
  It says when that string differs from the text the tested module reported,
  `2.4.4-1`.
- A missing device,
  a missing or unreadable BTF file,
  a short read,
  a parse failure or any mismatch disables raw requests,
  reads included.
  There is no force flag and no environment override.
  The wrapper takes the applicable reflink or ordinary Git route for an unused slot.
  Registrations and durable recovery state survive a refusal.
- The check runs in one process and remembers its verdict there.
  An identity recheck runs before every reuse.
  No verdict goes to disk while the disk-cache question is held.
- The binding package holds the bounded loader,
  the pre-scan,
  the expectations,
  the comparison and the reader once it is picked.
  The wrapper and the root handler hold the paths,
  the read order,
  the per-process cache,
  the target allowlist and the words the caller and `doctor` print.
- The profile-reading dependency is still the project's to pick.
  The acceptance left that pick where the design left it,
  for the reasons under [the reader question](#the-reader-question-q1-no-not-required).

The four choices the design marked open to veto stand unvetoed.
Each is restated in its own section below.
The disk-cache question stays held.

## The answer of 2026-11-20: accept as written

The project was asked whether to accept the design at `22703f23df`.
It answered "Accept as written (Recommended)" on 2026-11-20.
The four choices the design marked open to veto were offered with that question.
The answer records that none of them is vetoed:
the folder name `compatibility_check`;
`doctor` flagging any module version text other than the tested `2.4.4-1` exactly;
carrying on with a weaker reload check when the module's identity files are missing;
and remembering the verdict per process.
So the design's writer-settled rules become the project's as the design states them,
each with "veto none" as its default.
Nothing in the four is left for a writer to decide again.
The next four sections state each choice with its reason,
its alternative and its cost.
A veto is closed now,
so a reader needs to know what each choice buys and forgoes.

### The folder name `compatibility_check`

The checker's files live in `src/compatibility_check/` inside the binding package.
The folder sits beside `layout` and `requests`.
It carries `uses/layout` and `uses/requests` links of its own.
The name says what the folder does:
comparing two files.
The folder cannot be a subfolder of `layout`.
`src/requests/uses/layout` already links the two folders,
and rule `LYL` forbids two source folders that use each other.
The alternative the design names is `compatibility_gate`.
That name is wider,
because the plan's word "gate" also covers the device and identity steps the wrapper holds.
A rename would cost the folder name and its `src/uses/` link.

### `doctor`'s exact-string sentence

`doctor` reads the string `/sys/module/zfs/version` reports.
It compares that string with the exact text the tested module reported,
`2.4.4-1`.
It prints its sentence when the two differ,
and the sentence quotes both strings.
So a distribution's `2.2.2-0ubuntu9.5`,
a checkout build's `2.4.4-1_g1a2b3c4` and a Debian-style `2.4.4-1~bpo12+1` each get the sentence.
The sentence says a different text was read,
not a different version.
It reports and never refuses,
so this choice cannot gate a request.
The alternative the design rejects is a prefix test on `2.4.4-`.
That test would silently pass the checkout build `2.4.4-1_g1a2b3c4`,
which is a build the sentence exists to flag.

### A weaker reload check when the identity files are missing

An absent `/sys/module/zfs/version`,
`srcversion` or build-id note is not a refusal.
The layout comparison still runs.
The two identity reads compare whichever of the three values the module carries,
plus each file's device,
inode,
size and modification time.
The reason is that the project's answer leaves only the layout and the request numbers to decide.
A missing identity file is not a ground that answer kept.
When all three values are absent the reload check has no usable term.
The cache then keeps its verdict while each file's four facts hold.
`doctor` says which attribute is absent,
so a reader can tell a weaker reload check from a refused one.
The design did not take two alternatives here.
One is refusing when no identity value exists.
That would refuse a module the layout comparison can still judge.
The other is parsing the two files again on every use when none exists.
That costs a parse per use for such a module,
and it would see a file swapped while the other facts hold.

### Remembering the verdict per process

The result is cached in a `OnceLock` per process,
following the wrapper's `git_version.rs`.
The identity recheck runs before each reuse.
It compares the module's version,
`srcversion` and build ID,
and both BTF files' device,
inode,
size and modification time.
The default parses the two files once per process.
It pays the stale-pass corner the design states:
where the module carries no identity value,
the cache compares only the two files' four facts,
so a replacement that keeps them is reused.
The alternative is to parse the two files on every use.
It pays a parse per use,
which owed measurement item 10 measures,
and it has no stale pass.
Nothing else separates the two arms,
since the check is the same code either way.

## The reader question, Q1: "No, not required"

The owner was asked whether it is a hard requirement that the parser is the project's own code.
The second half of that question was that the product starts no second program.
The owner answered "No,
not required (Recommended)".
So no hard constraint admits an in-house reader.
The `choosing-technology` skill's existing-tools rule keeps it ineligible.
The eligible order is `btf-rs` 2.0.1,
then `pahole` 1.32,
then `bpftool` 7.8.0,
on the design's scored comparison.
The acceptance took no reader.
The pick waits until the items Q1 lists close:
nobody ran `btf-rs`'s own test suite;
the `doc/audit/` vet report does not exist;
discovery's terminal result is blocked,
so a later round may add a candidate;
and the pre-discovery check-in is not written.
The pick is therefore a decision the project still owes.
The design's ranking is its input.

## The release string, Q2: "Any version, layout decides"

The answer is "Any version,
layout decides (Recommended)".
It was re-confirmed on 2026-11-19T21:31:47-05:00 as "Keep:
layout decides".
So the gate compares no release string at all.
The layout comparison and the request numbers decide.
`doctor` prints the loaded module's release string,
and it says when that string is not the tested text.
Two facts the exchange rested on stay in the design.
A release can change a request's answer while keeping the layout.
The 2.2.2 to 2.3.0 interval shows that for a volume's property set.
And the three-release comparison found no change that reaches the wrapper's own call shapes.
The wrapper sends two properties,
both of them user properties,
and both return at the user-property check before the handler's `switch` in all three releases.
The third option in that exchange,
a release list,
stays refused.
The second option was layout plus refusing a property set aimed at anything but a filesystem dataset.
It guards a request shape the wrapper does not send.

Two corrections the last check asked for stand in this section.
The record is what a later reader of that exchange meets,
so the corrections belong here.
The `snapdev` and `volmode` routing fact was read by the design's fifth revision.
It was not read by the revision that wrote the sentence saying the answer came before it.
On that reading the fact does not change the basis of the answer.
The project may ask it again,
since the answer is the project's and not the design's to reaffirm.
The design's clone-path count is five of the seven on the property-set path.
It comes from the fourth revision's narrow gate check,
in that check's `keep/evidence/`,
in
`show-which-differing-functions-the-clone-handler-alone-reaches-output.txt`.
The `trace-call-paths-output.txt` and
`compare-handlers-registrations-and-reachable-callees-output.txt` cited beside it sit in the third
gate re-check's `keep/evidence/`.

## The crate placement, Q4: "Binding package"

The answer is "Binding package (Recommended)".
So `package/zfs/request-to-openzfs-kernel-device` holds the bounded loader,
the pre-scan,
the expectations,
the comparison,
the refusal value and the reader once it is picked.
The wrapper and the handler hold the paths,
the read order,
the per-process cache,
the target allowlist and the wording.
The record decided that the binding owns the request format,
and that both binaries share it.
Both binaries live in one cargo package,
so either placement satisfied the record.
The answer settles which one is built.
The alternative was the wrapper package holding the loader,
the pre-scan and the comparison.
That splits the expectations from the `zfs_cmd` type they mirror,
and it turns one field change into a cross-package test failure.

## The disk cache, Q3: held

Q3 waits for owed measurement item 10.
Nothing needs its answer before that run.
The reason is that the gate works with option 1's behaviour either way.
The per-process cache is what ships,
so a veto of that rule means a parse per use and no disk file.
No answer of Q3 is needed for the gate to work.
Option 1 is no disk cache.
It ranks first while the cost is unmeasured,
and this design writes no verdict to disk meanwhile.
The question returns to the project when item 10's measurement crosses the thresholds Q3 states.
Those thresholds are the added wall time's median above 100 ms and the resident peak above 256 MiB.
The design calls them its own proposal,
taken from a single probe run with no control.
Item 10 is the run that replaces them.

## Step 2's standing at acceptance

The design records step 2's standing as dated reads of a branch.
The acceptance re-pins that standing to where the work landed:

- Step 2 is in `main` as `1ead5ca9fa`.
  That merge carries live requests,
  the layout comparison against the module's BTF file and the guest fixture.
- Step 2's lows round is merged as `dc13ae9243`.
  That merge carries the nine lows of step 2's integration re-check.

Both are `main`'s commits,
not a branch tip.
Both were checked as ancestors of `main` on 2026-11-20.
The design's dated reads stay true as dated records and need no correction.
They are `08c9dea25f`,
read at 2026-11-19T23:04:00-05:00,
and `69cef8fe3c`,
read at 2026-11-19T23:40:09-05:00.
The design's sixth revision wrote its own label,
`2026-11-19T23:44:09-05:00`.
It also wrote a step 2 read at 2026-11-19T23:40:09-05:00.
Both are clock readings that round did not keep.
Both are bounded by commits rather than by the reading itself.
The label sits between `8d516064df` at 23:43:18 and `242b4454ea` at 23:44:16.
The read sits inside the time `69cef8fe3c` was the branch's tip,
before `3c74141b5e` arrived.
The design's fifth revision bounds its own read by the branch's move at 23:04:45.
Its lower bound only restates the value it bounds,
so it adds no second bound.

## The last check's ten lows, corrected in this record

The check is the diff-only gate check of the design's sixth revision.
It is kept at
`/home/user/temp/agent/zfs-runtime-layout-checker-design-sixth-revision-diff-only-gate-check-2026-11-19/keep/review.md`.
It found no high and no medium finding.
It called the design ready to put to the project,
and it carried ten lows beside it.
The design's body is accepted and stays as it is.
Each low is therefore stated corrected here,
and the sections above carry the corrections where they restate the same matter.

### L1: the header sentence's verb

The design's opening section must read that the parts a writer can settle are under its writer list.
The corrected clause is "and each rule marked there as open to veto carries "veto none" as its
default".
So the last clause has its own verb,
and the sentence does not claim that every writer-settled part is marked.

### L2: the fifth revision's entry and the writer list

The fifth revision's entry keeps the statement that the writer list names the cache among those
open to veto.
The scoping of that veto to the per-process rule alone,
with the disk cache left to Q3,
is the sixth revision's change.
It stays in the sixth entry.
A reader who diffs `b8de4a383d` must not find the opposite of what the fifth entry says.

### L3: the fifth entry's "They"

The sentence after the inserted clock-reading sentence reads "The commits are the revision that
answers the narrow gate check of the fourth revision".
So the pronoun names the commits.
It does not name the label and the read that now sit nearest it.

### L4: what the sixth entry's last commit changes

The sixth entry says the commit carrying its closing sentence is the last fix after the round's
other commits.
It must say that the commit changes this record and the prose checks' count words.
`22703f23df` also changes the count words in "The prose checks".

### L5: the sixth revision's two times

The sixth revision's label and its step 2 read are clock readings that round did not keep.
They are bounded by commits,
as [step 2's standing](#step-2s-standing-at-acceptance) states.
The fifth entry's lower bound only restates the value it bounds.
It is not an independent bound.

### L6: the capitalised words count

"The prose checks" must say that emphasis held at 0.
It must say that capitalised words went from 22 to 21 as a sentence was removed.
It must say that count words went from 275 to 298.
The count fell by one,
so it did not hold.

### L7: the clone count's source

The design's Q2 paragraph must cite the fourth revision's narrow check for the clone path's five of
the seven.
It must name the third gate re-check's folder once for the two outputs it cites.
The correction is stated under
[the release string](#the-release-string-q2-any-version-layout-decides).

### L8: the per-process cache's reason, alternative and cost

The cache rule must state the reason for its default,
the alternative and what each arm costs.
[Remembering the verdict per process](#remembering-the-verdict-per-process) does:
one parse per process with the stated stale-pass corner,
against a parse per use with none,
which item 10 measures.
It must also say that a veto here overlaps the absent-identity paragraph's second alternative.
Both are a parse on every use,
so a module that carries no identity value meets the same trade twice.

### L9: when Q3 returns to the project

The design must say that the question returns to the project when item 10's measurement crosses the
thresholds Q3 states.
It must say that the design writes no verdict to disk meanwhile.
It must not say that the project decides once item 10 has run in every case.
The reason nothing needs the answer is that the gate works with option 1's behaviour either way.
The per-process rule being open to veto is not that reason.
[The disk cache](#the-disk-cache-q3-held) states both.

### L10: the evidence correction's revision and its voice

The design's evidence correction must say that the `snapdev` and `volmode` fact was one the fifth
revision read.
It must say that on this reading the fact does not change the basis of the answer,
while the project may ask it again.
It must not have the design reaffirm the project's answer.
[The release string](#the-release-string-q2-any-version-layout-decides) states both.

## Amendments on acceptance

The design names three documents whose text the project's release-string answer supersedes.
Each was right when it was accepted or merged,
and each needs the amended statement.
Two of them are historical records.
Each of those gets an in-place note giving the new statement and the date,
rather than a rewritten line.
The binding crate's `README.md` is a live document,
so its bullet is corrected in place.
Each change is listed here with where it sits.

- [`doc/planning/cli-git-zfs-worktree-design-proposal.md`][proposal],
  "ABI gate and tests":
  a note of 2026-11-20 follows the paragraph that lists the gate's proofs.
  It states that the gate proves no module release.
  It states that the layout comparison and the request numbers decide,
  by the answer "Any version,
  layout decides".
  It states that `doctor` prints the release string,
  and that the sentence says when the string is not the tested text.
- [`doc/planning/cli-git-zfs-worktree-implementation-plan.md`][plan],
  step 6 "What it builds":
  a note of 2026-11-20 follows the paragraph.
  It gives the gate's list without `module release,`.
  The list then reads architecture,
  endianness,
  request contract and the kernel and module BTF layout.
- [The same plan][plan],
  step 6 "Open items":
  a note of 2026-11-20 follows the section.
  It states that the architecture,
  endianness and request-contract checks are decided and can be written beside the two designs.
  It states that the module's release string is reported and not proved,
  by the answer of 2026-11-19.
- `package/zfs/request-to-openzfs-kernel-device/README.md`,
  "What this crate does not do":
  the first bullet drops `release` from the list the start-time gate compares.
  It gains the sentence that the release string is not part of the gate.
  That sentence states that the layout comparison and the request numbers decide,
  and that `doctor` prints the string and says when it is not the tested text.
  The design quoted that bullet as step 2's branch then wrote it.
  Step 2 merged before this amendment,
  so the bullet is re-quoted as `main` now has it.
  Step 6 deletes the bullet when the crate itself holds the gate.
- The plan's step 6 "VM measurements" needs no change.
  It is part of this amendment only by explanation.
  Its "untested release" arm belongs to the start probe.
  That is a kernel release and not a module release,
  so the design makes no assertion about it.
  The sentence that the BTF comparison runs against the guest's `/sys/kernel/btf/zfs` waits on a
  guest whose module carries that file.
  That run is owed measurement item 11's.
- `package/git-policy/cli/SPEC.md` needs no change on acceptance.
  The plan's step 6 adds the section "The start-time ZFS compatibility gate" under `## Lifecycle`
  when it lands.
  The plan's description of that section holds without a named release proof.
  That description is what is proved,
  what a refusal disables,
  what it prints and which route an unused slot takes.

## What this record does not do

- It does not carry the design's evidence,
  its candidate scoring,
  its refused ways or its owed measurements.
  The design's own text is where each stands,
  at `22703f23df`.
- It does not pick the profile-reading dependency or settle the disk cache.
  Both are the project's answers still owed.
- It claims no run.
  No gate,
  no reader,
  no guest measurement and no build was exercised for it.
  Every measurement the design lists stays owed.
- It marks itself unapproved and non-final (rule `SGO`).
  The project may correct it.

[design]: ../planning/cli-git-zfs-runtime-layout-checker-design.md
[proposal]: ../planning/cli-git-zfs-worktree-design-proposal.md
[plan]: ../planning/cli-git-zfs-worktree-implementation-plan.md
