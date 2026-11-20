# BTF reader for the ZFS layout checker: technology vet (2026-11-20)

Not final and not approved.
This report recommends no reader and adopts nothing (rule `SGO`).
Only the project declares the work complete.

Status:
complete as a report,
at lifecycle phase 7,
Scored.
The skill's terminal outcome for the evidence is
[sensitivity unresolved](#terminal-result-sensitivity-unresolved),
so the report gives conditional rankings and recommends none.
Two of the four finalists' pairs still move on one input at a time at the frozen baseline,
one weight moves a program past the second library,
and the input that settles that pair is a preference the project has never stated.
The choices the project has to make are put in
[Open questions for the project](#open-questions-for-the-project).
`btf-rs` 2.0.1 leads the refrozen baseline at 27.5 of 32,
and the rank is conditional rather than recommended.
Checked by this revision:
every evidence record below was read against the kept log or the kept API response it names;
the maintenance audit and the auditability measurements,
which are this revision's own reads,
with their commands kept;
the score arithmetic and every sensitivity row below,
recomputed from the published ratings;
the fingerprint recomputed from its printed input;
both libraries' archives measured against their tags and their manifests;
the two tools' package provenance read from the distribution's own database;
and the host facts re-read.
Not checked:
the suite runs themselves,
which this revision did not repeat,
and an `aarch64` run,
which no machine here can do.

## Report metadata

- Subject:
  BTF reader for the ZFS layout checker.
- Scope:
  which BTF reader the start-time ZFS layout check uses,
  a library the binding crate links or a program the gate starts,
  for question Q1 of [`doc/decision/cli-git-zfs-runtime-layout-checker.md`][record].
  The check compares the loaded ZFS module's declared layout
  with the binding's compiled expectations before the first `/dev/zfs` request,
  and a refusal disables raw requests.
- Started:
  2026-11-20T18:07:38-05:00,
  the clock reading taken as this session's work began.
  Last updated:
  2026-11-20T18:56:14-05:00,
  the write time of this revision.
- Revisions:
  two.
  The first wrote the report;
  the second is this one,
  after an independent gate review by a different vendor,
  Claude Sonnet 5.5,
  kept at
  `~/temp/agent/gate-review-of-the-btf-reader-vet-report-on-sonnet-2026-11-20/keep/review.md`.
  What it changed is listed in section "Changes in this revision".
- Governing skill:
  [`.agents/skills/choosing-technology/SKILL.md`][skill] at commit
  `45d42d9b23d2f2752f9c512683b45905e13afe99`,
  dated 2026-10-29T20:16:36-04:00,
  SHA-256 `f4cfa6bc5d1d71f4b136f92ff9fbdab68575c920bd334f824d253336c8782e1b`.
  The `.claude/skills/choosing-technology/SKILL.md` copy this session read is byte-identical:
  `diff` printed nothing and both hash to that value.
  That is the same revision the OpenZFS vet recorded.
- Compatibility fingerprint:
  `55c2be0183adaa6fbad912f852fb045a915c1593b344aa10095aa71bdec4677f`.
  Input:
  section "Fingerprint input".
  Script:
  `~/temp/agent/zfs-layout-checker-reader-vet-report-2026-11-20/keep/compute-the-vet-report-fingerprint.ts`,
  run twice in the first revision,
  the second at 2026-11-20T18:16:26-05:00,
  both exit 0 and both printing that value;
  the rerun in this revision is
  `~/temp/agent/zfs-layout-checker-reader-vet-report-gate-review-findings-fixed-2026-11-20/scratch/keep/compute-the-vet-report-fingerprint.ts`,
  a byte-for-byte copy of it,
  run against this revision's printed input and printing the same value.
  The value is unchanged because the subject,
  the scope,
  the hard constraints,
  the deployment,
  the trust boundary,
  the base category and the overlays are unchanged:
  the refreeze below changed weights and ratings,
  which the fingerprint does not carry.
- Lock:
  `~/temp/agent/technology-vet-locks/` + the SHA-256 of the absolute report path + `.lock`.
  The first revision created the report with create-new semantics,
  so its first lock record holds `preEditSha256: null`.
  This revision took the same lock in update mode:
  it wrote a sibling temporary file,
  re-read the report's SHA-256 under the lock,
  aborted if it had changed,
  and renamed the temporary file into place.
  Every record of both revisions,
  seven from the first and this revision's,
  is kept together in
  `~/temp/agent/zfs-layout-checker-reader-vet-report-gate-review-findings-fixed-2026-11-20/scratch/keep/lock-record.json`,
  It began as a copy of the first revision's file,
  which still sits at
  `~/temp/agent/zfs-layout-checker-reader-vet-report-2026-11-20/keep/lock-record.json`,
  and this revision's writer appended to it as the first one's did.
- Active audit owner:
  this revision's session,
  a Pi session in the worktree `.claude/worktrees/zfs-layout-checker-reader-validation`,
  on branch `docs/planning-cli-git-zfs-layout-checker-reader-validation`.
  No second agent wrote any line of this revision.
  The discovery enumeration's first twelve queries were run by earlier sessions in the same worktree;
  the project's answer of 2026-11-20 delegated the remaining ones to the first revision's writer,
  which ran them,
  and the first revision was that writer's synthesis,
  not a re-run.
  The finalist validation was run by an earlier session still.
  The gate review above was an independent read of the first revision and wrote nothing here.
- Prior compatible report:
  none.
  No file under `doc/audit/` carries this subject slug,
  and the one report holding ZFS layout evidence,
  [`tech-rust-crate-for-openzfs-vet-2026-11-18.md`](tech-rust-crate-for-openzfs-vet-2026-11-18.md),
  stores fingerprint `407e8ac931da8102b402d2356e322daaef1a0bdf224b8caf612c2c64d1d3ae2e` for a different subject.
  So the skill's rules for a new report apply,
  and no duplicate path exists.
- Report threshold:
  crossed,
  on three of the skill's conditions at once.
  External discovery and screening saturated with four serious alternatives;
  every finalist received execution evidence from this host's own BTF pair;
  and a required source class was blocked for a time
  before alternate enumeration finished it (section "Query schedule").

## Changes in this revision

This revision answers the gate review named under "Revisions" above,
written by a reviewer from another vendor after reading the first revision whole.
Each finding is listed with what became of it and the section that carries the change.

- H1,
  a recommendation the skill does not allow:
  fixed.
  The report states the skill's terminal outcome,
  [sensitivity unresolved](#terminal-result-sensitivity-unresolved),
  with a conditional ranking under each outcome-changing input,
  and the two choices the project must make are questions
  ([Open questions for the project](#open-questions-for-the-project)).
  The design's Q1 now says the same thing in its own words.
- H2,
  weights the skill does not allow,
  and a matrix never run at its baseline:
  fixed.
  The rubric is refrozen at the skill's weights,
  the complete matrix is rerun at them,
  and the design writer's weights are reported as a sensitivity arm only
  ([Refreeze of this revision](#refreeze-of-this-revision)).
- M1,
  no maintenance audit:
  fixed for all four finalists,
  from the repository hosts' own records,
  with every request kept
  ([Maintenance audit](#maintenance-audit)).
- M2,
  auditability unmeasured and unweighted:
  fixed for the four finalists and carried by a new criterion,
  C8
  ([Human auditability](#human-auditability)).
- M3,
  thinner evidence for the two tools:
  partly fixed.
  Their package provenance and its signature record are added,
  C3's root-child premise is corrected,
  and the one omission that cannot be closed here is stated as a depth gap
  rather than hidden under "equal depth"
  ([The two tools' provenance and suites](#the-two-tools-provenance-and-suites)).
- M4,
  `btf-rs` C1's confidence label:
  fixed by lowering it to medium,
  which puts that rating into the perturbation set,
  and the arithmetic is rerun
  ([C1](#c1-bounded-fail-closed-handling-of-untrusted-bytes-weight-1)).
- M5,
  the check-in's timing and scope:
  fixed
  ([The check-in of 2026-11-20](#the-check-in-of-2026-11-20)).
- M6,
  the design's stale Q1 text:
  fixed in the design,
  three statements.
- Lows:
  L1,
  L2,
  L3,
  L4,
  L6,
  L7 and L8 are fixed where they touch a fact this report states;
  L5 is fixed with the omission record the skill asks for;
  L9 needs no change and gains the review's own extra-query probe as corroboration,
  because the report already called its class 3 result an inferred exhaustion.

## Fingerprint input

Every field the skill names,
with its value.
The script sorts the three set-valued arrays (the hard constraints,
the base categories and the overlays)
and the deployment keys by raw UTF-16 code-unit order,
and the block below already shows that order;
it serializes the value as RFC 8785 canonical JSON and hashes the UTF-8 bytes with SHA-256.

```json
{
  "schemaVersion": 1,
  "subject": "BTF reader for the ZFS layout checker",
  "decisionScope": "Which BTF reader the start-time ZFS layout check uses: a library the binding crate links, or a program the gate starts, for question Q1 of doc/decision/cli-git-zfs-runtime-layout-checker.md",
  "hardConstraints": [
    "H1 parses split BTF from a module file with its vmlinux base",
    "H2 license compatible with LGPL-3.0-or-later, the wrapper license",
    "H3 builds for x86_64-unknown-linux-gnu and aarch64-unknown-linux-gnu with no C build step and no load-time library from the distribution",
    "H4 reads a file the caller names, with no BPF syscall, no root requirement and no network",
    "H5 exposes named struct members with their bit offsets and type sizes, and enum members with their values, at the depth the anonymous union needs",
    "H6 source is inspectable and its provenance maps to the published artifact"
  ],
  "deployment": {
    "architecture": "x86_64-unknown-linux-gnu and aarch64-unknown-linux-gnu",
    "distribution": "the wrapper release and the root handler, both relying on no OpenZFS userland at start",
    "input": "the kernel base BTF file and the ZFS module BTF file under /sys/kernel/btf",
    "os": "Linux"
  },
  "trustBoundary": "high-trust: the same check runs in the git wrapper and in the root ZFS event handler process, and parses kernel-produced bytes",
  "incumbent": null,
  "baseCategories": [
    "inspectable open-source local technology"
  ],
  "overlays": [
    "high-trust execution in an agent, plugin, hook, CI runner, or credential boundary",
    "multi-platform or browser-baseline claim",
    "native, Wasm, prebuilt binary, or generated-code boundary"
  ]
}
```

`incumbent` is `null` and that is a measured fact,
not an omission.
`rg --ignore-case 'from_split|hdr_len|0xeb9f|btf_header|parse_btf' package` returns nothing,
and the ten files under the binding crate that mention BTF either name the module's BTF path,
shell out to `/usr/bin/pahole` for the test-time comparison
(`package/zfs/request-to-openzfs-kernel-device/src/kernel_device/live_btf_tests.rs:1-40`),
or carry a layout constant read from that file.
Its `Cargo.toml` declares `libc` alone.
This worktree has no reader:
step 6 of the implementation plan,
which would have held one,
is unbuilt on the branch this worktree carries.
An in-house decoder does exist,
unmerged,
on `feat/git-policy-cli-zfs-start-time-compatibility-gate` at `ac59add52c`:
`btf_decode.rs` 125 lines,
`btf_entry.rs` 225,
`btf_header.rs` 354,
`btf_records.rs` 154 and `btf_types.rs` 242,
1,100 lines before tests,
with `btf_header_tests.rs` 137,
`btf_records_tests.rs` 121 and `btf_types_tests.rs` 164,
422 lines of tests.
The skill's fourth source class counts a parallel hand-rolled alternative,
so the ledger carries it (section "Discovery ledger");
it stays ineligible for the reason that section gives.

## Hard constraints

Each is the design's,
quoted in its own words,
and each was applied before any rating.

- H1:
  parses split BTF from a module file with its vmlinux base.
  A module's BTF extends the kernel's base:
  its type ids and string offsets cross into the base.
- H2:
  license compatible with `LGPL-3.0-or-later`,
  the wrapper's license and the binding's (`package/zfs/request-to-openzfs-kernel-device/Cargo.toml:21`).
- H3:
  builds for `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu`
  with no C build step and no load-time library from the distribution.
  The wrapper ships as two GNU release archives (`toolchain.lock.json:614-653`)
  and must start on a machine without OpenZFS userland.
- H4:
  reads a file the caller names,
  with no BPF syscall,
  no root requirement and no network.
- H5:
  exposes named struct members with their bit offsets and type sizes,
  and enum members with their values,
  at the depth the anonymous union needs.
- H6:
  source is inspectable and its provenance maps to the published artifact.

H3 is the constraint that narrows the field,
and it is the project's,
not a fact of nature.
Dropping it re-admits `libbpf-rs` and a `dlopen` of the system's `libbpf`,
which is a choice for the owner (section "Open questions for the project").
H5 is why a low-level ELF reader is not a candidate by itself:
`object`,
`goblin`,
`scroll` and `libelf` expose section bytes,
not BTF types.

## Base components and overlays

One base category,
for all four eligible finalists and for the refused candidates that got as far as screening:
inspectable open-source local technology.
Two are libraries the binding links (crates) and two are programs the gate starts;
the skill's base category does not separate them,
and criterion C2 does.

Overlays,
every one that matches:

- High-trust execution:
  the same check runs in the git wrapper and in the root ZFS event handler process.
- Native,
  Wasm,
  prebuilt binary,
  or generated-code boundary:
  every candidate parses kernel-produced bytes,
  and no candidate links native code.
  For `pahole` and `bpftool` the artifact is a distribution binary,
  so the provenance question is the package's and the release-version matrix is owed (measurement item 14).
- Multi-platform claim:
  the wrapper's two GNU release targets.

Gates marked not applicable,
with reasons:

- Managed service and SaaS gates:
  no hosted component is proposed.
- Sensitive data,
  privacy,
  compliance,
  residency,
  retention,
  geography:
  the input is kernel type metadata and the output is a verdict about struct offsets;
  no personal data crosses the boundary.
- Proprietary local technology:
  every candidate's source is readable.
- Replacement parity:
  there is no incumbent to replace (`incumbent: null` above).
  This is not a swap,
  so the parity overlay and rule `RCO` do not apply.
  The crate's test-time `pahole` call is a fixture,
  not a runtime reader,
  and it is recorded as repository precedent under "Discovery ledger" rather than as an incumbent.
- Open-source precedence:
  satisfied with no exception;
  no proprietary candidate was needed and none was admitted.

## Frozen criteria, refrozen weights

Frozen before any candidate-specific soft evidence was rated,
and applied to every finalist.
The weights below are the refrozen ones (section "Refreeze of this revision"),
which are the skill's weights and not the design writer's.

- C1,
  bounded,
  fail-closed handling of untrusted bytes,
  weight 1.
- C2,
  release and deployment fit:
  no runtime program,
  no C build,
  both GNU targets,
  weight 1.
- C3,
  privilege surface in the root handler,
  weight 1,
  higher is smaller.
- C4,
  diagnostic precision:
  member-level expected and actual values,
  weight 1.
- C5,
  maintenance and upstream responsiveness,
  weight 1.
- C6,
  integration and test cost,
  weight 1,
  higher is cheaper.
- C7,
  start cost,
  weight 1,
  low confidence because it is unmeasured.
- C8,
  human auditability of the reader surface a root process runs,
  weight 1,
  higher is leaner,
  added at the refreeze from the skill's overlay
  (section "Human auditability").

Ratings run 0 to 4,
every rating carries a confidence,
and a low-signal rating carries the evidence-supported minimum and maximum whose midpoint is the provisional value.
No criterion is removed from the denominator:
each one bears on at least one finalist.
Maximum points are 32:
the weight sum 8 times 4.
At the design writer's weights,
reported below as a sensitivity arm only,
they are 108.

The ratings for the in-house reader price code that does not exist,
and the arithmetic marks that score as priced,
not ranked.

## Refreeze of this revision

The first revision froze the design writer's weights,
5,
5,
4,
4,
3,
3 and 2,
while stating that the project had set no priority among the criteria.
The skill's rule for that case is weight 1 for every criterion whose priority is unspecified,
and the first revision never ran the one-input matrix at those weights,
so its baseline was outside the skill and its matrix described another rubric.
The gate review re-ran it at those weights and found
a program passing a library.

This revision refroze the rubric before rerunning anything,
and made three changes,
all recorded here before the rerun below was run:

1.  Weights:
    every criterion 1,
    because the project has stated no priority.
    The design writer's weights are kept as a reported sensitivity arm,
    with C8 at 1 because their text states no priority for it either.
2.  A criterion C8,
    human auditability,
    from the skill's high-trust overlay,
    which the first revision named and then measured nothing against.
    Its evidence is section "Human auditability".
3.  `btf-rs`'s C1 confidence,
    from high to medium.
    The first revision rated it high while rating `ebeepf`'s C1 medium on the same class of evidence,
    and the archive shows no bounds check of its own and no fuzz target
    (section "C1").

None of the three was chosen after seeing a winner:
C8's measurements favour `btf-rs`,
which is already the leader,
so it cannot be the reason for a different winner;
the C1 change moves against `btf-rs`;
and the weights are the skill's default rather than a reading of the evidence.
No criterion rating other than that one confidence label changed,
and no weight was chosen to make a preferred candidate win.
The complete matrix over the refrozen rubric is section "Sensitivity".

## Open questions for the project

Two of these decide the ranking.
Both are preferences the evidence cannot answer,
and the skill says to ask rather than to invent a tiebreaker.
The rest are carried so they stay visible,
and nothing here needs them.

1.  A library linked into the handler,
    or a program the handler starts.
    This is the design's Q1.
    The standing answer of 2026-11-19 is that the parser need not be ours and that no second program is required;
    it did not say that a second program is acceptable at run time in the root handler.
    The two arms are not interchangeable:
    a library runs in the handler's address space,
    where a parser bug is a panic,
    a hang or a large allocation inside the root process;
    a spawned program runs in a child,
    and both BTF files are mode `0444` on this host,
    so that child need not run as root and can be given a hard resource limit and a sandbox.
    The rubric prices the deployment and privilege criteria for this choice,
    and `C3`'s premise is examined in
    [C3](#c3-privilege-surface-in-the-root-handler-weight-1-higher-is-smaller).
2.  A priority between the criteria.
    The skill's default,
    every criterion 1,
    orders the four finalists
    `btf-rs` 27.5 > `ebeepf` 24.5 > `pahole` 18.5 > `bpftool` 17.5 of 32.
    That order is not stable under every one-input test:
    a maintenance weight of 4 ties `ebeepf` with `pahole`,
    and a maintenance weight of 5 puts `pahole` above `ebeepf`,
    so a program passes the second library.
    A project that ranks deployment fit and privilege above maintenance keeps the library arm ahead;
    one that ranks maintenance above them does not.
    Section "Sensitivity" gives the rows.
3.  The scoring's baseline pair,
    `btf-rs` against the declined in-house reader,
    now ties at the skill's weights at 27.5 each.
    That pair decides nothing while the project's answer of 2026-11-19 stands,
    and it would decide everything if the project reopened the in-house route.
4.  The disk-cache question,
    Q3 of the decision record,
    is held and stays separate.
    Its thresholds,
    a 100 ms median above the parse's baseline and a 256 MiB resident peak,
    are the design's proposal from a single probe with no control,
    and measurement item 10 is the run that replaces them.
    Nothing here needs its answer:
    the per-process cache ships either way.

## Context

The ZFS binding sends `/dev/zfs` requests whose command struct,
`zfs_cmd`,
the kernel and the module must agree on byte for byte.
A member that moves turns a destroy into something else,
and userspace cannot see the move.
So the gate reads the module's own type information,
`/sys/kernel/btf/zfs`,
with the kernel's base file `/sys/kernel/btf/vmlinux`,
and compares the declared sizes,
member offsets and enumerator values with the ones the crate's own `#[repr(C)]` types compile to.
A mismatch,
a missing file,
a short read or a parse failure disables raw requests and the wrapper takes the Git route instead (decision record,
"Decision").

The reader is the piece of that gate that is not in this worktree.
The project answered the two questions that bounded it.
It was asked whether it is a hard requirement that the parser is our own code and that the product
starts no second program,
and it answered "No,
not required" on 2026-11-19:
the conjunction is not required,
which is why the gate review called the first revision's "the product may start no second program" an inversion,
and why no crate and no program is ruled in or out by that answer.
No release string decides ("Any version,
layout decides").
The crate placement is settled:
the binding package holds the loader,
the pre-scan,
the expectations,
the comparison and the reader once it is picked (decision record,
Q4).
What is left is which reader.

Since the design was accepted at `22703f23df`,
four items Q1 listed have closed or moved.
Finalist validation ran on 2026-11-20;
discovery finished in a round on the same day with the project's check-in answer and a saturated terminal result;
the check-in artifact is written;
and this report closes the last one.
That leaves two limits,
and this report states them wherever the affected row is scored rather than in a closing caveat:
`aarch64` is built for both libraries and run for none,
and the start cost is unmeasured.

## The check-in of 2026-11-20

The skill asks one check-in per broad vetting process,
**before the first discovery query**,
stating the subject,
the forks settled so far,
the source classes,
the draft schedule,
and who runs the enumeration.
This vetting process is broad:
its names came from registries,
repository hosts and web pages,
not only from memory,
the project's picks or repository precedent.

This check-in came after the fact for most of the enumeration,
and the report says so rather than calling it pre-discovery.
The order of events was:

- the initial round,
  twelve numbered queries plus the first web search,
  ran on 2026-11-19 between 18:10:30 and 19:38:00-05:00,
  before any check-in (section "Query schedule",
  items 1 to 12);
- the check-in was put to the project on 2026-11-20 and answered the same day;
- the queries it covered are the ones run after it,
  items 13 to 15 and the three class 2 organization lists,
  which are the remaining web-class queries and list reads;
- the terminal discovery result therefore rests on a schedule whose first twelve queries
  were enumerated before the project was asked,
  and whose rest the project delegated.

The artifact stands under [the design's discovery record][design-check-in]
and is quoted here because this report is written afterwards and must carry it.

- Subject:
  the BTF reader question Q1 picks.
- Settled forks:
  a library the binding links or a program the gate starts;
  no hard requirement that the parser be our own code;
  the two GNU release targets.
- Source classes:
  the skill's four,
  crates.io,
  the repository host,
  the wider web,
  and this repository with its prior reports.
- Draft schedule:
  the three class 3 queries and the two class 2 list reads the earlier rounds had left,
  each with its literal query,
  provider,
  filters,
  page and result count.
- Options offered:
  the earlier sessions' writer;
  a model the project names;
  the project itself.
- Answer:
  "This session's writer (Recommended)",
  described as a `DeepSeek V4.1 Flash` writer through Pi running the remaining queries now.
- Date:
  2026-11-20.

The answer named the first revision's writer,
and the round's three class 3 queries and two class 2 list reads ran there afterwards,
as the answer describes;
no delegated starting brief was needed.
What the answer did not cover is the enumeration itself:
the earlier twelve queries were already run when it was asked.
Forks the skill lists among the context inputs were not put in that question either,
namely the trust boundary and the feature-breadth-versus-auditability tradeoff;
both bear on the weights,
and this revision records them as unasked rather than as settled
(section "Open questions for the project").

## Query schedule

Frozen in two rounds,
then expanded once for taxonomy terms and frozen again,
exactly as the skill's pagination and expansion rules ask.
Ordering is the provider's default,
no `sort` parameter was sent in any query,
no include or exclude filter was sent in any query,
and no result was truncated after the page rule.
The page rule is `per_page=100`,
the provider's maximum,
for every registry and repository query,
first page only;
each count is the provider's own total,
so that page held every row.

Initial round,
2026-11-19T18:10:30-05:00 to 18:53:37-05:00:

1.  crates.io `q=btf`,
    no filter:
    81 rows of 81.
    New candidates:
    `btf`,
    `btf-rs`,
    `btfparse`,
    `btfdump`,
    `aya-obj`,
    `padlock-dwarf`.
    New survivors:
    `btf-rs`,
    `btfparse`,
    `btfdump`,
    `aya-obj`.
2.  crates.io `q=split btf`:
    17 rows of 17,
    every one also in the `q=btf` page.
    None new.
3.  crates.io `q=btf kernel`:
    50 rows of 50,
    every one also in the `q=btf` page.
    None new.
4.  GitHub repository search `BTF parser in:name,description language:rust`:
    1 row (`retis-org/btf-rs`).
    None new.
5.  GitHub code search `from_split_bytes language:rust`:
    25 rows over 6 repositories.
    None new.
6.  Web search,
    the provider's own ranking,
    one call:
    recorded under source class 3.

Expansion round,
2026-11-19T19:30:00-05:00 to 19:38:00-05:00:

7.  crates.io `q=libbpf`:
    91 rows of 91.
    None new;
    the BTF-related rows are `libbpf-rs`,
    `libbpf-sys`,
    `aya-obj` and its forks.
8.  GitHub repository search `BTF parser in:name,description`,
    no language filter:
    7 rows.
    None new.
9.  GitHub repository search `btf in:name,description language:rust`:
    37 rows.
    None new.
10. GitHub repository search `topic:btf language:rust`:
    3 rows.
    None new.
11. GitHub repository search `topic:btf-parser`:
    0 rows.
    None new.
12. GitHub releases for each audited repository at `per_page=100`:
    5,
    12,
    5 and 31.
    None new.

Round of 2026-11-20,
after the check-in answer,
all on the DeepSeek provider,
each one page:

13. `Rust crate parse BTF split BTF module vmlinux base BTF parser library`:
    44 rows.
    Names the ledger lacked:
    `bpf-compatible-rs`,
    `vock`.
    New survivors:
    none.
14. `C library parse BTF split BTF module vmlinux base BTF parser`:
    46 rows.
    Names the ledger lacked:
    `libabigail`,
    `parttimenerd/btfparse`.
    New survivors:
    none.
15. `alternatives to libbpf btf__parse BTF parser library`:
    49 rows.
    Names the ledger lacked:
    `ebeepf`,
    `cilium/ebpf`,
    `libbpfgo`,
    `gbpf`,
    `coolbpf`,
    `bpf-compatible`.
    New survivors:
    one,
    `ebeepf`.

Class 2 lists of the same round,
from the repository host's API with pagination to exhaustion:
`libbpf` 15 of 15 public repositories,
`trailofbits` 298 of 298,
`eunomia-bpf` 157 of 157.
`projectsV2(first: 1) { totalCount }` answers 1 for each of the three,
and their public boards were read as HTML
because the session's token lacks `read:project`
and the REST project endpoints are sunset.
The boards are "BPF Verifier Visualizer" (no items),
"[ENG] Sigstore Transparency Log Monitoring",
and "eunomia-bpf" (no items).
None names a candidate.

What the filters can hide,
recorded because the skill asks:

- The three class 3 pages are provider-ranked,
  not registries,
  so they are not provably exhaustive.
  Each returned fewer rows than the provider's 50-row cap and offered no cursor,
  so the provider reported its listing ended;
  no page 2 exists for those queries.
- The first attempt at class 3 stopped at that provider's 10-row display cap while the provider reported 44 rows,
  and the first attempt at class 1 read 50 of 81 rows at `per_page=50`,
  a provider cap and not saturation.
  Both were re-run to exhaustion and the re-runs are the records above.
- Class 2 item 4 was ambiguous once:
  the first version's filtered run returned 7 rows
  and today's re-run returns 1 for the filtered query and 7 for the unfiltered one,
  so which filter the index applied then is not knowable now.
  Both listings were screened and neither adds a survivor.
- A release list is only as complete as the API's own total,
  and the `per_page=100` reads equal the totals 5,
  12,
  5 and 31;
  a reader trusting the default page size of 30 would undercount the last.
- Organizations outside the three read were not enumerated,
  and the three were chosen because they hold every repository the ledger audits.
  An organization that holds no candidate is not a candidate index.

Terminal result:
saturated with at least two survivors.
Screening survivors:
`btf-rs` 2.0.1,
`ebeepf` 0.1.0,
`pahole` 1.32,
`bpftool` 7.8.0.

The two numbering schemes in these documents count the same queries differently,
and a reader moving between them should know it.
This report numbers every query of every round in one sequence,
so `ebeepf`'s discovery is query 15;
the design numbers it inside its own section,
where it is the third query of the 2026-11-20 round
(design,
"Class 3 of this round").

The class 3 result is an inferred exhaustion,
not a reported one,
and this revision keeps that reading.
Its three pages ended below the provider's cap with no cursor,
which is what a finished provider listing looks like and also what a silently capped one can look like.
The gate review tested that risk directly rather than arguing it:
eight further crates.io queries the frozen schedule never used
(`bpf type format`,
`vmlinux`,
`pahole`,
`kernel btf`,
`type format kernel`,
`btf split module`,
`dwarves`,
`bpf debug info types`)
found no BTF reader with a split base that the ledger lacks
(`~/temp/agent/gate-review-of-the-btf-reader-vet-report-on-sonnet-2026-11-20/keep/probe-crates-io-extra-queries-output.txt`).
That supports the saturation;
it does not prove it,
and no claim here rests on it alone.

A corroboration this report ran,
read only,
the response written at 2026-11-20T18:08:38-05:00 by its own file modification time:
`curl --silent --user-agent '...' 'https://crates.io/api/v1/crates?q=btf&per_page=100&page=1'`,
HTTP 200,
`total` 81 and 81 rows,
and every row name matches the design's transcription in the same order.
Response kept as `keep/crates-io-q-btf-page-1-2026-11-20.json` beside this report's fingerprint script.
No new parser appeared between the round and this report.

The kept discovery artifacts,
under `/home/user/temp/agent/zfs-layout-checker-reader-discovery-finished-2026-11-20/keep/`:
`query-1-literal-deepseek-page-1.txt`,
`query-2-c-libraries-deepseek-page-1.txt`,
`query-3-alternatives-deepseek-page-1.txt`,
`org-libbpf-repos.json`,
`org-trailofbits-repos.json`,
`org-eunomia-bpf-repos.json`,
the three `org-*-projects-listing.html` pages,
the three `org-*-project-*.html` pages,
and `evidence/checksums.txt` over them and the BTF copies.
Two ecosyste.ms reads sit beside them
and one was refused for a missing descriptive User-Agent
(`ecosystems-crates-keyword-btf-page-1.json` holds that error,
and `ecosystems-topic-btf-page-1.json` reports 0 repositories for the topic).
They were attempts,
not required source classes,
and the class that mattered,
crates.io,
finished.

## Discovery ledger

Each row names the candidate,
its discovery source,
its base category and overlays,
and its screening result.
The base category is the same for every row and is stated once:
inspectable open-source local technology,
with the high-trust,
native-boundary and multi-platform overlays from section "Base components and overlays".
The exhaustive row-by-row listing of the 81 registry rows
and the 37 repository rows is in [the design's discovery record][design-discovery],
which this report does not duplicate;
what follows is complete at candidate granularity,
so every candidate the listing names is either a row here or inside the screened-together group.

### `btf-rs` 2.0.1

- Discovery:
  crates.io `q=btf`,
  first result by recent downloads among parser crates;
  also returned by the class 2 repository and code searches.
- Version:
  2.0.1,
  tag `v2.0.1`,
  commit `03fd01bd9505d0888b568cd3aaa9b232b7e3fb20`,
  published 2026-11-12;
  license `LGPL-2.1-or-later`;
  archive SHA-256 `fa34ec915ede6b7eaeebcb38fdb749fcac1538ec1b94dcfde63a034ad8f7963b`,
  equal to the index `cksum`.
- Provenance,
  measured in this revision:
  the archive's `src/`,
  `LICENSE` and `examples/` are byte-identical to the tag's
  (`diff -r --brief` printed nothing),
  so the mapping rests on the bytes and not only on the archive's own `.cargo_vcs_info.json`,
  which a publisher writes and `cargo` never verifies.
  The product's build also runs a second archive,
  `btf-rs-derive` 2.0.1,
  a proc-macro the same authors publish from the same commit with `path_in_vcs` `btf-rs-derive`;
  its SHA-256 is
  `95d548bd1301bf327418190bbe8d63bc34b5671cc01f3ecf04aceb8ac47a7c60`,
  its `src/lib.rs` is 114 lines,
  and it is byte-identical to the tag's `btf-rs-derive/src/lib.rs`.
  It is the same-author dependency the audit surface has to carry.
- Screening:
  pass,
  every screening gate either proven or pending targeted evidence.

### `ebeepf` 0.1.0

- Discovery:
  class 3 query 15 of the round of 2026-11-20,
  whose 49-row page named it.
  Record correction:
  the class 1 page also held the name,
  tagged at the time as "eBPF loader,
  reads BTF as an input" from its description alone;
  that tag predates its source audit and the row below supersedes it.
  The class 3 page is what promoted it to a candidate.
- Version:
  0.1.0,
  published 2026-10-06,
  commit `5de05465bd9445d89b13a967e198d0aeffd71ec4`;
  license `LGPL-2.1-only OR BSD-2-Clause`,
  so the BSD arm applies;
  archive SHA-256 `f5f4162021ff5dce1b6c5a657267523d7d41c6818bae2adc0adbfbf4b5b6fa96`,
  equal to the version entry's `checksum`.
- Screening:
  pass.

### `btfparse` 1.3.9

- Discovery:
  crates.io `q=btf`.
- Version:
  1.3.9,
  commit `3788c4410a6cfd64ff57aa5bca20b2ee4eed3cd5`,
  2026-01-08,
  `Apache-2.0`,
  no dependencies,
  5,876 lines under `src/`.
- Screening:
  H1 fail.
  One `TypeInformation::new` over one `Readable`,
  numbering types from 1,
  with no base file (`src/btf/type_information.rs:435-490`),
  so a module's crossed-in ids and string offsets cannot be represented.
  Exit at screening;
  no score.

### `aya-obj` 0.6.0

- Discovery:
  crates.io `q=btf`,
  by downloads.
- Version:
  0.6.0,
  `aya-rs/aya` at `44a6709164d2ac0c19c7845101e7b033e25b3f3f`,
  `MIT OR Apache-2.0`.
- Screening:
  H1 fail.
  Its reader parses one object's `.BTF` section and has no base;
  the `base` at `aya-obj/src/btf/btf.rs:355` is a byte offset (`:340-365`).
  Exit at screening.

### `btf` 0.5.1

- Discovery:
  crates.io `q=btf`.
- Version:
  0.5.1,
  `arcjustin/btf` at `3b57cdab2ea80d13c3b5abd400bb2e4bfc0d6292`,
  last commit 2023-02-08,
  `MIT`.
- Screening:
  H1 fail,
  one blob parser with a 24-byte header and no base (`src/btf.rs:15-81`);
  no commit since 2023 is noted and the gate alone is enough.
  Exit at screening.

### `btfdump` 0.0.5

- Discovery:
  crates.io `q=btf`.
- Version:
  0.0.5,
  `anakryiko/btfdump` at `e5951d4b2e2e7ce6c36001c1b48c1c239c9759dc`,
  `BSD-2-Clause`,
  with `clap`,
  `object`,
  `memmap2`,
  `scroll`,
  `scroll_derive`,
  `regex`,
  `bitflags`.
- Screening:
  H1 fail as a library:
  it dumps one BTF file and has no split base (`src/types.rs:1089-1120`,
  `src/lib.rs`).
  As a tool it would be the start-a-program route with an extra package;
  the ledger's two tool finalists already cover that route.
  Exit at screening.

### `padlock-dwarf` 0.82.0

- Discovery:
  crates.io `q=btf`.
- Version:
  0.82.0,
  `gidotencate/padlock` at `6a7d045f2c6b0cb64549ddebe5825411e25178fa`,
  `MIT OR Apache-2.0`.
- Screening:
  H1 fail,
  a 794-line BTF reader inside a DWARF and PDB analyzer,
  single blob,
  no base (`crates/padlock-dwarf/src/btf.rs:114-152`);
  H5 fails on fit,
  since its output is a whole-program struct list.
  Exit at screening.

### `libbpf-rs` 0.27.2 and `libbpf` through FFI

- Discovery:
  crates.io `q=libbpf`,
  and repository precedent (the wrapper's prior vet refused the C libraries).
- Version:
  0.27.2,
  `libbpf/libbpf-rs` at `6d771ee9835fcf517dbcf1d29e25a97480425d45`;
  license `LGPL-2.1-only OR BSD-2-Clause` from the crates.io metadata,
  so the BSD arm applies.
- Screening:
  H3 fail.
  Its default feature is `libbpf-sys/vendored-libbpf`,
  which runs libbpf's C build,
  and its manifest says it "requires other necessary libs to be present on the system" (`libbpf-rs/Cargo.toml:20-26`).
  A `dlopen` of the system's `libbpf` avoids the link
  but keeps the native provenance
  and adds a privileged library load inside the root handler,
  so H3 and H6 fail there too.
  Exit at screening on H3.
  This is the row that H3 admits if the project drops it.

### `bpftool` 7.8.0

- Discovery:
  this host (`/usr/bin/bpftool`) and the libbpf reader it carries;
  also on the `libbpf` organization's repository list as its own project.
- Version:
  7.8.0 on libbpf 1.8,
  as the binary reports itself;
  the tool's license is the kernel's (GPL-2.0 with BSD-2 parts) and it runs as a separate process,
  which is not a link.
- Provenance:
  `/usr/bin/bpftool` is the distribution's `bpf 7.3.4-1` package
  (`pacman --query --owns /usr/bin/bpftool`),
  installed 2026-11-16 with `Validated By : Signature`.
  Its self-reported version and its package's version disagree,
  and the reason is structural:
  the version string comes from `BPFTOOL_VERSION`,
  which the build sets from the kernel tree it was built in (`src/Makefile:80-81`),
  not from the package.
  So a check on the tool's self-reported version proves nothing about which package shipped it.
  The binary's SHA-256 is
  `475a53182f50231c40e40ce002f2e032dc418489d56a2f8f902aa317aed03d20`.
- Screening:
  pass on H1,
  H2 for a separate process,
  H4,
  H5;
  H6 pass on the package record above;
  H3 is not applicable as a library but the product needs the program at run time,
  which C2 prices.
  Finalist.

### `pahole` 1.32

- Discovery:
  this host;
  the design's own test-time inspection command;
  repository precedent,
  since the binding's live test already runs `/usr/bin/pahole`.
- Version:
  1.32,
  GPL-2.0-only,
  a separate process.
- Provenance:
  `/usr/bin/pahole` is the distribution's `pahole 1:1.32-2` package
  (`pacman --query --owns /usr/bin/pahole`),
  not a package named `dwarves`,
  installed 2026-11-16 with `Validated By : Signature`;
  the binary's SHA-256 is
  `00acbd859cd0cca6e71dca93d885ed2483c5a52a78d3df1a9bcc9da82881503c`.
  Its BTF parser is libbpf's,
  built into the package rather than loaded from the distribution at run time:
  `readelf --dynamic /usr/bin/pahole` lists only `libdwarves`,
  `libdwarves_emit`,
  `libdwarves_reorganize`
  and `libc`.
- Screening:
  pass on H1,
  H2 for a separate process,
  H4,
  H5,
  H6 on the package record above.
  Finalist.

### The in-house reader

- Discovery:
  repository precedent (the binding's own unsafe boundary and its "our own small binding" decision)
  and the skill's existing-tools rule,
  which counts parallel systems and hand-rolled alternatives as its fourth source class.
  It is not hypothetical code:
  `feat/git-policy-cli-zfs-start-time-compatibility-gate` at `ac59add52c`
  carries a working decoder under
  `package/zfs/request-to-openzfs-kernel-device/src/compatibility_check/`:
  `btf_header.rs` 354 lines,
  `btf_types.rs` 242,
  `btf_entry.rs` 225,
  `btf_records.rs` 154 and `btf_decode.rs` 125,
  1,100 lines before tests,
  plus 422 lines of tests in the three `*_tests.rs` files beside them.
  It is unmerged,
  so this worktree has none of it,
  and the branch's own gate review left a seam still open
  (`doc/handover/cli-git-rust-implementation.md` lines 40756 to 40757:
  emptying the three decoder files leaves 14 errors in six files).
- Version:
  none;
  the model and walker would follow `include/uapi/linux/btf.h`
  and the kernel's own validation (`kernel/bpf/btf.c:5195-5325`).
- Screening:
  H1 to H6 pass by construction,
  and it is still ineligible.
  The skill admits a custom implementation only when every existing tool fails a named hard constraint,
  and no such constraint exists:
  the project answered on 2026-11-19 that our own parser and no second program is not a hard requirement.
  Conditional candidate,
  priced in the arithmetic so the record shows what the route would have cost,
  and outside the ranking.

### Screened together, and why each is not a candidate

- `zfsdev-rs`'s layout code hard-codes a layout and parses no BTF;
  the crate already reuses its codec and layout code,
  so it is a dependency of the binding's history,
  not a reader.
- `object`,
  `goblin`,
  `scroll`,
  `libelf`:
  ELF sections and byte streams,
  the building blocks of the in-house reader,
  not BTF type models.
  H5 fails for each.
- `trailofbits/btfparse`,
  `parttimenerd/btfparse` (its fork),
  `libabigail`:
  C++ builds,
  H3 fail;
  `libabigail` would also use libbpf's parser,
  which H3 already refuses.
- `cilium/ebpf`,
  `libbpfgo`,
  `gbpf`:
  Go,
  so no link path into a Rust binary,
  H3 fail.
- `coolbpf`:
  a C platform over libbpf whose README needs `elfutils-devel`,
  `gcc`,
  `clang` and `llvm`;
  H3 fail,
  and its parser would be libbpf's.
- `bpf-compatible` and `bpf-compatible-rs`:
  helpers that resolve BTF files from the btfhub archive;
  `bpf-compatible-rs` 0.1.3's `src/lib.rs` holds archive-path helpers and no type model,
  so the category fails.
- `aquasecurity/btfhub` and `btfhub-archive`:
  distribute BTF files,
  so they are data sources.
- `trailofbits/nate`:
  a Rust DWARF parser,
  not a BTF reader.
- `vock`:
  a Debian ITP for a tracer,
  a tool rather than a parser.
- The remaining class 1 rows are eBPF loaders,
  linkers,
  profilers,
  generators,
  tools and crates whose names happen to contain "btf";
  the design's listing tags each one.
- The remaining class 2 rows are tools,
  generators,
  databases and unrelated projects;
  the design's listing tags each one.

## Hard-gate outcomes

- `btf-rs` 2.0.1:
  H1 pass (split constructors and the consumer probe),
  H2 pass,
  H3 pass:
  source,
  an `x86_64` run,
  and an `aarch64` library build,
  with that target still unrun
  ([The `aarch64` build](#the-aarch64-build-and-what-it-does-not-show)),
  H4 pass,
  H5 pass,
  H6 pass on the archive's bytes against the tag.
  Finalist,
  validated on `x86_64`.
- `ebeepf` 0.1.0:
  H1 pass (split constructor and the probe),
  H2 pass on the BSD arm,
  H3 pass on source plus the same `aarch64` build,
  run unexercised,
  H4 pass,
  H5 pass,
  H6 pass on the archive's bytes against its commit.
  Finalist,
  validated on `x86_64`.
- `pahole` 1.32:
  H1 pass,
  H2 pass as a separate process,
  H3 not applicable as a library and recorded as a runtime requirement instead,
  H4 pass,
  H5 pass,
  H6 pass on the package record:
  `pahole 1:1.32-2`,
  installed with its signature validated.
  Finalist,
  validated at its output boundary.
- `bpftool` 7.8.0:
  H1 pass,
  H2 pass as a separate process,
  H3 not applicable as a library,
  H4 pass,
  H5 pass,
  H6 pass on the package record:
  `bpf 7.3.4-1`,
  installed with its signature validated,
  whose binary reports a different version of itself.
  Finalist,
  validated at its output boundary.
- `btfparse`,
  `aya-obj`,
  `btf`,
  `btfdump`,
  `padlock-dwarf`:
  H1 fail,
  exit at screening.
- `libbpf-rs`,
  `libbpf` through FFI,
  `trailofbits/btfparse`,
  `parttimenerd/btfparse`,
  `libabigail`,
  `coolbpf`,
  `cilium/ebpf`,
  `libbpfgo`,
  `gbpf`:
  H3 fail,
  exit at screening.
- The in-house reader:
  H1 to H6 pass by construction and excluded by the skill's existing-tools rule.
  Not a gate failure,
  and not a finalist.

No candidate passed a hard gate on metadata alone:
every finalist's gate rests on a source path and,
for the two libraries,
on a run.
For the two tools the provenance gate rests on the distribution's own package database,
which is a primary record of who built the binary and which package carries it,
and on nothing that maps the binary to a source revision;
both packages build from a source archive their build scripts name,
and neither package records that archive here.
That is the provenance a distribution package can offer,
and the depth gap it leaves is what
[The two tools' provenance and suites](#the-two-tools-provenance-and-suites) records.

## Host facts

This report re-measured the host read-only at 2026-11-20T18:16:26-05:00,
so the validation evidence below describes today's module and not a file that has since been replaced:

- Kernel `7.3.5-1-cachyos`,
  architecture `x86_64` (`uname --kernel-release --machine`).
- ZFS module version `2.4.4-1`,
  `srcversion` `A3553C308C29FD4B6595A72` (`cat /sys/module/zfs/version /sys/module/zfs/srcversion`).
- `/sys/kernel/btf/vmlinux`,
  6,579,400 bytes,
  SHA-256 `dbf55cc7d4a2027c494cd7fed706ee751d26fe4f5739999d65161d3a1714f17c`,
  modification time 2026-11-20T14:34:52-05:00.
- `/sys/kernel/btf/zfs`,
  570,234 bytes,
  SHA-256 `edb8433af826ea7f5a9fdfb92aa13cd00f0e1f455a3637608e0f00c98cefd509`,
  modification time 2026-11-20T14:34:52-05:00.
- Both files are mode `0444`,
  `-r--r--r--`,
  read by every user (`stat --format='%A %a'` in this revision).
  A spawned reader therefore needs no privilege to read them,
  which is the fact C3's premise missed in the first revision
  ([C3](#c3-privilege-surface-in-the-root-handler-weight-1-higher-is-smaller)).
- Both hashes equal the ones the validation round recorded for its read-only copies
  (`keep/evidence/checksums.txt` in each round's kept folder),
  so the runs parsed the same bytes this host serves now.
- Boot time 2026-11-19T11:12:11-05:00,
  so no reload separates the runs from this reading.
- The host kernel is not the product's qualified release until step 23 of the implementation plan,
  so these facts qualify the evidence,
  not the product.

The compared set,
as every run reports it:
both files' 32-byte header,
the base's 151,181 types,
the module's 13,141 own types numbered 151,182 to 164,322,
`zfs_cmd` at 13,744 bytes with 32 members,
the anonymous union at 106,368 bits with `zc_inject_record` and its anonymous struct,
`dmu_objset_stats` at 288 bytes with `dds_flags` at 2,296 bits,
`zinject_record` at 368 bytes with 19 members,
and the 104 `zfs_ioc` enumerators with the seven values the design names.

## Execution manifests

This report ran no third-party code.
The four manifests below describe rounds run by earlier sessions in this worktree;
each is reconstructed from the kept logs' own headers,
which record the command line,
start time,
elapsed milliseconds and exit status per step.

### Manifest: `btf-rs` 2.0.1

- Candidate and revision:
  `btf-rs` 2.0.1,
  archive SHA-256 `fa34ec91...f7963b`,
  clone origin `retis-org/btf-rs` at tag `v2.0.1`,
  `HEAD` `03fd01bd9505d0888b568cd3aaa9b232b7e3fb20`.
- Clone policy:
  `gh repo clone ... -- --branch v2.0.1 --depth 1`,
  then `git remote set-url --push origin DISABLED` before any other command.
- Top-level command per step:
  `cargo` from the pinned toolchain,
  exactly as each log's `# podman:` line records it,
  for example `cargo test --locked --offline --verbose --all-features`.
- Statically reached commands beyond `cargo`:
  `rustc`,
  `rustdoc`,
  `rustfmt`,
  `clippy-driver` from the same toolchain;
  a C compiler for the `zstd` and `xz` optional dependencies when `elf-compression` is on;
  no lifecycle script,
  no network client,
  no root operation.
- Files inspected to establish the tree:
  `Cargo.toml`,
  `Cargo.lock`,
  `.github/workflows/ci.yaml`,
  `btf-rs/tests/integration_test.rs`,
  `btf-rs/tests/assets/`.
- Image:
  `docker.io/cachyos/cachyos:latest` at `sha256:ab3e1698b7ce5eaa4f6342d7a0f687ce731c443706238a49338f46239036a185`,
  `podman` 6.2.0.
- Ceilings:
  `--cpus 2`,
  `--memory 2g`,
  `--pids-limit 512`,
  `--rm`.
- Credentials,
  environment,
  home:
  no ambient credentials,
  `HOME=/work/home` inside the container with no host home mounted,
  `--userns=keep-id`,
  the BTF copies mounted read-only at `/btf`.
- Network:
  on for one fetch step (`rustup` 1.29.3 installing stable 1.100.0 with `rustfmt` and `clippy`,
  then `cargo fetch`),
  off for every build and test step with `--network none` and `CARGO_NET_OFFLINE=true`.
- Expected reads and writes:
  the mounted tree and the read-only BTF copies;
  writes confined to the scratch tree,
  the cargo home and the toolchain home.
- Success,
  failure,
  cleanup and stop:
  each step's exit status compared with an expected one;
  the container removes itself;
  the driver stops on a step failure after recording it.
- Undeclared discovery:
  the published archive ships no tests,
  which the manifest did not expect,
  so the suite's run was moved to the repository at the same commit
  and the manifest was updated in the round's own record before the suite ran.
  Nothing else appeared.

### Manifest: `ebeepf` 0.1.0

- Candidate and revision:
  `ebeepf` 0.1.0,
  archive SHA-256 `f5f41620...5b6fa96`,
  clone origin `kxxt/ebeepf` at `HEAD` `5de05465bd9445d89b13a967e198d0aeffd71ec4`,
  push URL disabled.
- Top-level command per step:
  `cargo test`,
  `cargo fmt --check`,
  `cargo clippy --lib -- -D warnings`,
  and `cargo run` for the consumer,
  each recorded in its log's `# podman:` line.
- Statically reached commands beyond `cargo`:
  `rustc`,
  `rustfmt`,
  `clippy-driver`,
  and `clang` plus `llvm` for the crate's BPF fixtures.
- Image:
  `localhost/ebeepf-validation:rust-1.100.0-clang` at
  `sha256:d201d863b813804a9bc89db027eb51ed6f9839c4d8d363a2ab77e1363bed2a05`,
  built for this round from the same CachyOS base digest plus the distribution's Rust 1.100.0 and clang.
- Ceilings,
  credentials,
  home,
  mounts and the network policy:
  the same as the `btf-rs` manifest.
  Deviation recorded:
  the `aarch64` fetch attempt ran with `--network host` because it needed the distribution host,
  and that attempt failed (section "Validation results" records the exit and the error).
- Undeclared discovery:
  the suite needs `clang` for its BPF fixtures.
  Without it 94 unit tests pass and 8 fail at `src/test_bpf.rs:53`.
  The manifest was rewritten to build a clang image before the suite ran,
  and the without-clang runs are kept as failures.

### Manifest: `pahole` 1.32 and `bpftool` 7.8.0

- Candidates and revisions:
  `/usr/bin/pahole` 1.32,
  the distribution package `pahole 1:1.32-2`,
  SHA-256 `00acbd859cd0cca6e71dca93d885ed2483c5a52a78d3df1a9bcc9da82881503c`;
  `/usr/bin/bpftool` 7.8.0 on libbpf 1.8 by its own report,
  the distribution package `bpf 7.3.4-1`,
  SHA-256 `475a53182f50231c40e40ce002f2e032dc418489d56a2f8f902aa317aed03d20`.
  Both are the distribution's packages,
  so the provenance question is the package's and the release-version matrix is owed.
- Top-level command:
  `pahole --btf_base ... -C zfs_cmd|dmu_objset_stats|zinject_record`,
  `pahole --btf_base ... -E -C zfs_ioc`,
  and `bpftool btf dump file ... --base-btf ... format raw`,
  all recorded in `keep/evidence/tool-harness-invocations.txt` with their exit status.
- Environment:
  the host,
  not a container;
  no ceiling,
  because these are the distribution's binaries reading two read-only copies and writing only their own stdout.
  The harness recorded each invocation's exit status.
- Undeclared discovery:
  none;
  the harness observed its six invocations,
  four for `pahole` and two for `bpftool`,
  plus both negative controls.
- Limits of this manifest,
  stated because the skill asks for one:
  the tool route's own product arms,
  owed measurements 13 and 14,
  and the release matrix are unrun,
  so this manifest validates the tools at their output boundary and nothing beyond it.

### Manifest: this report's own runs

- Fingerprint script:
  `node compute-the-vet-report-fingerprint.ts` in `~/temp/agent/zfs-layout-checker-reader-vet-report-2026-11-20/keep`,
  exit 0,
  printed the canonical JSON and the hash;
  the second run at 2026-11-20T18:16:26-05:00 printed the same hash.
- Registry corroboration:
  the `curl` call in section "Query schedule",
  written at 2026-11-20T18:08:38-05:00,
  HTTP 200,
  76,457 bytes kept.
- Host reads:
  `uname`,
  `cat` of the two identity files,
  `stat` and `sha256sum` of the four files,
  `uptime --since`,
  the same readings re-run at 2026-11-20T18:16:26-05:00,
  exit 0,
  output in section "Host facts".
- Report write:
  the lock script under `keep/`,
  create-new,
  exit 0,
  and its record kept.
- Verification:
  `monochromatic-lint` on the two committed files and `pandoc --from=gfm` on this report,
  both after the write,
  with the lint findings filtered to the lines `git diff main...HEAD` adds,
  and both recorded in this session's final report.
- Nothing here fetched,
  built,
  installed or ran a candidate.

### Manifest: this revision's own reads

This revision fetched,
measured and read,
and ran no candidate:

- `gh api` GET requests,
  read only,
  authenticated as this machine's user,
  against four repositories:
  `repos/retis-org/btf-rs`,
  `repos/kxxt/ebeepf`,
  `repos/acmel/dwarves` and `repos/libbpf/bpftool`,
  plus their `contributors`,
  `issues`,
  `pulls`,
  `commits` and `releases` endpoints,
  and one detail read per sampled issue or pull request with its comments and reviews.
  Responses are kept as JSON under this revision's `keep/gh/`.
- `curl` of three published crates.io archives,
  `btf-rs` 2.0.1,
  `btf-rs-derive` 2.0.1 and `ebeepf` 0.1.0,
  from `https://static.crates.io/`,
  each SHA-256 compared with the report's recorded value.
- Three read-only clones at depth 1,
  `gh repo clone ... -- --depth 1`:
  `acmel/dwarves`,
  `libbpf/bpftool` and `libbpf/libbpf`,
  plus `retis-org/btf-rs` at tag `v2.0.1` for the byte diff.
  The `dwarves` submodule `lib/bpf` was initialised to count the vendored libbpf's parser.
- Local reads only:
  `find ... | xargs wc -l`,
  `rg --count-matches '\bunsafe\b'`,
  `diff -r --brief`,
  `pacman --query --owns`,
  `pacman --query --info`,
  `readelf --dynamic`,
  `stat`.
- Ceilings:
  none needed.
  No candidate was executed,
  nothing was built,
  and the network reads are a few hundred API responses and three
  crate archives.
- Everything under
  `~/temp/agent/zfs-layout-checker-reader-vet-report-gate-review-findings-fixed-2026-11-20/scratch/`,
  with the kept reports,
  responses and measurements under its `keep/`.

## Validation results

Equal depth is the claim being checked:
every finalist got the same three layers where they apply,
namely its own upstream build and test path,
its own suite,
and the consumer boundary that the product would use.

### `btf-rs` 2.0.1, validated on `x86_64` Linux

- CI-equivalent path,
  run from the repository at tag `v2.0.1` because the archive ships no tests:
  `cargo fmt --check`;
  `cargo clippy --locked --all-features -- -D warnings` for the library and the example;
  `cargo doc --locked --all-features` under `RUSTDOCFLAGS=-D warnings`;
  six feature builds;
  `cargo test --locked --all-features`.
  Every step exit 0;
  the test step's own elapsed time is 14,982 ms (`ci-cargo-test-all-features.log:1-4`),
  and its suite reports 168 integration tests and four doctests passed in 11.16 s (`:183`,
  `:197`),
  with no `#[ignore]` and no skipped case.
- Default features:
  64 integration tests and four doctests passed in a 3,166 ms step (`default-features-cargo-test.log:1-4`,
  `:79`,
  `:93`).
  So 104 cases need the features the CI enables,
  which is a fact about the suite's reach and not a failure.
- The published archive:
  15 files,
  `autotests = false`,
  `include` holding only `examples/`,
  `src/` and `LICENSE`,
  so `cargo test` there runs four doctests that only compile (`published-crate-archive-cargo-test-all-features.log:19`).
  Its `.cargo_vcs_info.json` names `03fd01bd9505...`,
  which is the tag's commit,
  so the archive maps to the source whose suite passed:
  that is provenance under H6,
  and it is the reason the archive's missing tests are recorded rather than scored.
  This revision checked the claim on the bytes rather than on that file:
  `diff -r --brief` of the archive's `src/` against the tag's printed nothing,
  and the same holds for `LICENSE` and `examples/`
  (section "Discovery ledger").
- Consumer boundary:
  a program depending on the library by path,
  reading bounded buffers and calling `Btf::from_bytes` for the base and `Btf::from_split_bytes` for the module,
  which is the loader route the product designs.
  183 checks,
  0 failures,
  exit 0,
  731 ms (`design-consumer-check-on-host-btf-copies.log:1-4`,
  `SUMMARY passed=183 failed=0`).
  Negative controls refused with their own errors,
  no panic and no stall:
  a base truncated to 1,024 bytes,
  a module truncated the same way,
  a module whose magic byte changed,
  and a split object passed as a base.
  The first run of this check failed,
  and the failure is recorded here as unexplained rather than passed over:
  `summary.txt` in that round's kept evidence reads
  `STEP FAIL design-consumer-check-on-host-btf-copies exit=101 elapsed-ms=3690`,
  and the log was overwritten by the passing re-run above,
  so the cause is not recoverable from anything kept.
  What is known is that the consumer program or its manifest changed between the two runs,
  since the same source and the same two files give exit 0 in 731 ms on the re-run;
  the re-run's own command line is the one in the log.
  A 3,690 ms step is longer than the passing run,
  which is consistent with a first-time compile failing rather than with a parse failure,
  but that is an inference and not a record.
- Omits `aarch64` as a run:
  the gate review built the library for the target without `rustup`
  and this revision cites that build
  ([The `aarch64` build](#the-aarch64-build-and-what-it-does-not-show));
  no run happened on any machine here.
- No fuzz target exists in the crate,
  and none was run;
  the bounded pre-scan and the negative controls are what the validation has instead (source quality record,
  section "Evidence for the scores").

### `ebeepf` 0.1.0, validated on `x86_64` Linux

- Archive checked against the registry first:
  the version entry's `checksum` equals the downloaded archive's SHA-256,
  and `.cargo_vcs_info.json` names the clone's `HEAD`.
- Suite,
  on the archive:
  `cargo test` ran 102 unit tests,
  `kernel_smoke` with 1 passed and 22 ignored,
  and 1 doctest,
  all green,
  in a 6,967 ms step with the suite at 6.43 s (`archive-cargo-test-default-with-clang.log:1-4`,
  `:113`,
  `:146`,
  `:152`).
  `cargo fmt --check` clean;
  `cargo clippy --lib -- -D warnings` clean with one manifest warning about a redundant `homepage`.
  The repository's own workspace adds a `bpfcov` member whose `libbpf-rs` dependency does not resolve offline,
  so the archive is the step and the repository's workspace is recorded as unusable there.
- The 22 omitted `kernel_smoke` cases,
  recorded as the skill asks:
  exact command `cargo test`,
  run from the archive's root,
  the same step as the suite above;
  purpose,
  to load BPF programs and maps through the BPF syscall;
  each one's name carries its own requirement,
  and every one of the 22 needs root or `CAP_BPF` and a kernel feature,
  for example
  `opens_kernel_module_split_btf` ("requires root or CAP_BPF and a loaded kernel module exposing BTF")
  and `reads_program_output_streams` ("requires root or CAP_BPF and a kernel with BPF program streams"),
  as the log prints them (`archive-cargo-test-default-with-clang.log:122-143`).
  The run had neither root nor a capability set,
  so none could run there.
  None can affect the surface this report claims:
  that surface is reading two files and comparing their types,
  hard constraint H4 excludes the BPF syscall from the reader path outright,
  and the one of the 22 that touches split BTF,
  `opens_kernel_module_split_btf`,
  reaches it through the loader's syscall path rather than through the file parse.
  The consumer probe above is what covers the split parse on this host's pair.
- Consumer boundary:
  the same program shape,
  on the same copies.
  196 checks,
  195 passed,
  1 failed,
  exit 1,
  478 ms (`design-consumer-check-on-host-btf-copies-with-clang.log:1-4`,
  `:236-237`).
  The failure is a negative control:
  a module passed as the base of another split parse is accepted instead of refused,
  which is the residual the source audit found at `src/btf.rs:411-419`.
  The other controls passed.
- Omits the `aarch64` run,
  and the build is closed separately:
  the image's Rust ships no `aarch64` standard library,
  and the `rustup` route failed,
  with
  `error sending request for url`,
  the URL being `https://static.rust-lang.org/dist/channel-rust-stable.toml.sha256`,
  and `client error (Connect): tcp connect error: Connection timed out (os error 110)`
  exit 1,
  32,633 ms (`fetch-rustup-toolchain-and-aarch64-target.log:1-28`).
  In the same container `curl` reached the rustup installer host,
  so this is the distribution host's connect path and not the container's network policy.
  It happened twice.
  The gate review closed the build half without `rustup`,
  and this revision cites that
  ([The `aarch64` build](#the-aarch64-build-and-what-it-does-not-show));
  the run stays unavailable.
- No fuzz target exists in the crate,
  and none was run.

### `pahole` 1.32 and `bpftool` 7.8.0, validated at their output boundary

- The harness ran each tool on the same two copies and checked the same members and enumerators the design compares:
  323 checks,
  0 failures (`keep/evidence/tool-harness-checks.txt`).
- Negative controls:
  `pahole` on the module without a base exits 1 with libbpf's "did you forget to provide base BTF" message;
  `bpftool` on the same file exits 234 with "Invalid argument".
- Output facts that bear on C4:
  `pahole` prints absolute byte offsets at every nesting depth,
  so its anonymous containers need no path arithmetic;
  `bpftool` prints `bits_offset` relative to the containing type,
  renders an empty BTF name as `(anon)`,
  and prints no member size,
  so a consumer must resolve each `type_id` through both dumps.
  The harness does that,
  and that resolver is the cost C4 prices.
- Omitted `aarch64`:
  the host has no `aarch64` environment,
  so both binaries are `x86_64` evidence only.

### The two tools' provenance and suites

The first revision claimed equal depth for the four finalists.
They are not equal,
and this section says how far apart they are,
because the project is being asked to choose between the two arms.

Provenance:

- `pahole` is the distribution's `pahole 1:1.32-2` package,
  built 2026-11-02 and installed 2026-11-16,
  with `Validated By : Signature` in the package database.
  Its declared dependencies are `bash`,
  `glibc`,
  `libelf` and `zlib`,
  and its binary needs only its own three libraries and `libc`.
- `bpftool` is the distribution's `bpf 7.3.4-1` package,
  built 2026-11-12 and installed 2026-11-16,
  also `Validated By : Signature`,
  declaring `glibc`,
  `readline`,
  `zlib`,
  `libelf`,
  `libcap`,
  `zstd`,
  `llvm-libs`,
  `binutils` and `libsframe`;
  its binary needs `libelf`,
  `libcrypto`,
  `libz`,
  `libcap` and `libLLVM`.
  It reports itself as `bpftool v7.8.0` on `libbpf v1.8`,
  which is not its package's version,
  because that string is set at build time from the kernel tree (`src/Makefile:80-81`).
- Both are signatures a package manager verified at install,
  which is what a distribution package can offer as provenance.
  Neither maps the shipped binary to a source revision,
  and neither package records the source archive its build used here,
  so H6 for these two rests on the package record and stops there.

The BTF parser both tools run is libbpf's:

- `pahole` builds libbpf into its own libraries
  (the clone's `lib/bpf` submodule at `6d0328e7`,
  whose `src/btf.c` is 6,780 lines)
  and reads through `btf_loader.c` (863 lines).
- `bpftool` builds libbpf in as well,
  and adds its own `src/btf.c` (1,895 lines) over the same parser.

Upstream suites,
recorded with the reasons they did not run here:

- `pahole`:
  the suite is 78 shell scripts,
  18 of them BTF-focused,
  driven by `tests/tests` after `buildcmd.sh` builds the whole tool set (`build-and-test-cmd.sh`).
  It was not run here,
  and the reason is not duration:
  running it means building `pahole` and its libraries from a revision the package is not,
  which is a build the product never performs,
  and several cases need a `vmlinux`,
  kernel sources or the project's own VM harness
  (`.github/workflows/test.yml`,
  `vmtest.yml`).
  What that omits is a check of the encoder and printer over fixtures,
  which can touch the BTF printer the product's `-C` arm would consume;
  so the omission cannot be claimed harmless,
  and it is recorded here as an unclosed gap rather than as a pass.
- `bpftool`:
  the repository's CI has build,
  lint,
  docker,
  static-build and release workflows and no functional test job
  (`.github/workflows/`),
  and its functional tests are the kernel's `tools/testing/selftests/bpf`,
  which need root,
  a BPF-capable kernel and a VM harness the project's own workflow drives (`vmtest` in libbpf).
  No such run happened here,
  and the surface it would exercise is `bpftool btf dump`'s output,
  which is exactly the consumed surface,
  so this omission is a gap too.
- Read strictly,
  the skill fails a candidate whose relevant suite cannot be inspected or run.
  This report does not fail the tools,
  because each had a consumer-boundary validation with negative controls on this host's pair;
  what it does instead is refuse the "equal depth" claim,
  price the gap in the confidence of C5 and C6 for the tools,
  and carry it as a limit the project can see.

The C3 premise the first revision used is wrong in one respect:

- It rated both tools' privilege surface 1 with high confidence on the premise that the child runs as root.
  Both BTF files are mode `0444` on this host,
  so a child could read them unprivileged,
  sandboxed and under a hard resource and wall-clock limit,
  which a library in the handler's own address space cannot be.
  Nothing in the product's design requires the child to be privileged;
  the root handler can drop to an unprivileged user for the child.
  So the tools' C3 cost is a cost the program arm can shrink,
  and the report prices the unshrunken form and says so.
  The rating stays 1 for the spawned-program shape the design names
  (a program the root handler starts),
  because that is the shape being priced;
  the unprivileged-child variant is the one the project should weigh,
  and it is stated in
  [Open questions for the project](#open-questions-for-the-project).

### The `aarch64` build and what it does not show

Both libraries now build for `aarch64-unknown-linux-gnu`,
and neither has run there.
The build is the gate review's,
run after the first revision called the target unrun,
in capped containers with no home mount and the network off:
`podman run --rm --network none --cpus 2 --memory 2g --pids-limit 512 --userns=keep-id`,
`HOME` inside the container and `CARGO_NET_OFFLINE=true`.

- Toolchain:
  the validation round's stable 1.100.0 plus the `rust-std-1.100.0-aarch64-unknown-linux-gnu` archive.
  The reviewer recorded that it took that archive from the channel manifest dated 2026-11-12 and that the
  archive's SHA-256 matches the manifest;
  this revision did not repeat that fetch and does not rest a claim on the hash.
  `rustup` was not used,
  which is what the `ebeepf` round's failed fetch needed.
- `btf-rs` 2.0.1 archive:
  `cargo build --lib --locked --offline --target aarch64-unknown-linux-gnu`,
  exit 0,
  4,166 ms,
  `Finished \`dev\` profile ... in 3.96s`
  (`~/temp/agent/gate-review-of-the-btf-reader-vet-report-on-sonnet-2026-11-20/keep/aarch64-build-check-btf-rs-lib-build-aarch64-offline.log:1-4`).
- `ebeepf` 0.1.0 archive:
  the same command,
  exit 0,
  10,963 ms,
  `Finished \`dev\` profile ... in 10.75s`
  (`.../aarch64-build-check-ebeepf-lib-build-aarch64-offline.log:1-4`).
- No C compiler ran in either step;
  every dependency compiled from Rust source.
- The fetch step that needed the network is kept separately
  (`aarch64-build-check-fetch-ebeepf-dependencies-network-on.log`),
  and the driver that ran them is `aarch64-library-build-check-driver.ts` beside them.

What this closes:
H3's second target is now a build for both libraries,
not source reading alone.
What it does not close:
nothing here executed the `aarch64` artifacts,
so a target-specific miscompile would still be invisible,
and the two tools have no `aarch64` artifact at all.
The evidence is a cross-vendor reviewer's run that this revision did not repeat;
its logs and driver are cited where they are kept,
and the report does not present them as its own measurements.

## Maintenance audit

The skill asks for a specific audit,
not an issue count,
and the first revision asserted its C5 ratings without keeping the records.
This revision ran it for all four finalists,
on 2026-11-20,
through the repository hosts' API,
read only,
and kept every response.
The requests are the same shape for each repository:
`repos/<owner>/<name>`,
`.../contributors?per_page=100`,
`.../issues?since=2025-11-20T00:00:00Z&state=all&sort=updated&direction=desc&per_page=100`,
`.../pulls?state=all&sort=updated&direction=desc&per_page=10`,
`.../commits?since=2025-11-20T00:00:00Z&per_page=100`,
`.../releases?per_page=100`,
plus one detail read per sampled issue or pull request with its comments and its reviews.
Every response is at
`~/temp/agent/zfs-layout-checker-reader-vet-report-gate-review-findings-fixed-2026-11-20/scratch/keep/gh/`,
and the two scripts that collected them are kept in the same `keep/` folder,
as `collect-maintenance-records.ts` and `collect-maintainer-actions.ts`.
The sample rule is the skill's:
all issues when at most 20 exist in the window,
otherwise the 10 most recently updated.

### `btf-rs`

- Primary URL:
  `https://github.com/retis-org/btf-rs`,
  read 2026-11-20.
- Repository:
  created 2022-10-07,
  10 stars,
  4 forks,
  2 open items,
  last push 2026-11-12.
- Maintainer concentration:
  four contributors,
  118,
  31,
  18 commits and 3 for `dependabot`,
  so the lead author holds about 69 percent of them.
  Of the 73 commits in the window,
  `atenart` wrote 59,
  `amorenoz` 7,
  `vlrpl` 5 and `dependabot` 2.
- Issues in the window:
  none;
  the 14 rows the window returns are all pull requests.
  The two open items are older:
  issue 1 ("Add some BTF building capabilities") was last updated 2025-11-17,
  three days before the window opens,
  and pull request 37 ("Add kind layout support") last on 2026-10-26.
  So this is the skill's "active releases with weak public issue support" shape,
  and its issue history is too small to measure responsiveness from.
- Pull requests:
  ten of the 14 rows the window returns,
  sampled as the skill asks.
  Every merged one was reviewed and merged by a second maintainer,
  never by its author:
  `amorenoz` merged PR 42,
  35 and 34,
  `vlrpl` merged 41,
  38,
  39,
  40,
  36 and 33,
  and each carries a review by the merger.
  Two of them are external (`dependabot`,
  41 and 40),
  and both were reviewed and merged rather than merged unreviewed.
  Merge latency spans 0.5 days to 91 days,
  the long ones being the 2.0 release itself and the widened-mask work.
  Labels are used sparingly:
  the two dependency PRs carry `dependencies` and `rust`.
- Release record:
  five releases,
  `v2.0.1` on 2026-11-12 and `v2.0.0` on 2026-04-16 among them;
  the maintenance merges reach a release,
  with PR 38's widened masks the known exception,
  which is still open as PR 37.
- Hardening record:
  the window's commits include
  `obj: fix offset overflows` (2026-02-04,
  `381a38d8f1`),
  `obj: cap Vec len estimates` (2026-02-05,
  `bb8c673e01`),
  `obj: check header flags` (2026-02-05,
  `a75c5ddc67`),
  `obj: mmap: fix strings section validation` (2026-02-16,
  `529c3faa92`),
  `tests: add more invalid cases` (2026-02-05,
  `1a01ae7fd9`)
  and `section: mmap: fix oob access in resolve_name_by_offset` (2026-10-23,
  `64e2bdfa42`,
  merged as PR 42 on 2026-10-29).
  That record is maintenance evidence and also the reason C1 is not rated high:
  the crate needed overflow and out-of-bounds fixes this year.
- Classification:
  responsive maintainers with a small backlog,
  evaluated mainly through source,
  releases and tests.

### `ebeepf`

- Primary URL:
  `https://github.com/kxxt/ebeepf`,
  read 2026-11-20.
- Repository:
  created 2026-08-15,
  0 stars,
  0 forks,
  0 open items,
  last push 2026-10-06,
  license detection `NOASSERTION`,
  which matches the archive's two license files and is a detection limit,
  not a defect.
- Maintainer concentration:
  one contributor,
  36 of 36 commits.
- Issues and pull requests:
  none in the window and none older.
  Nothing was merged,
  nothing was reviewed,
  and the repository has no `.github` directory,
  which the API answers 404 for when re-read in this revision,
  so there is no CI to lean on.
- Release record:
  one release,
  2026-10-06,
  on crates.io;
  the repository host lists no release object.
  No commit after the release.
- Classification:
  effectively unmaintained since its single release,
  with no tracker activity at all;
  the skill says zero tracker activity is low-signal and not proof of health,
  and here the source record agrees with the silence rather than softening it.

### `pahole`

- Primary URL:
  `https://github.com/acmel/dwarves`,
  read 2026-11-20,
  the project's mirror;
  the package points at `https://git.kernel.org/pub/scm/devel/pahole/pahole.git` as its home.
- Repository:
  created 2011-11-05,
  286 stars,
  81 forks,
  49 open items,
  last push 2026-10-02.
- Maintainer concentration:
  62 contributors,
  of which `acmel` holds 1,595 commits and the next four hold 86,
  57,
  33 and 24.
  In the window,
  100 commits returned and `acmel` wrote 87 of them.
- Issues in the window:
  five,
  all sampled,
  all still open,
  one maintainer reply among them
  (issue 74,
  one maintainer comment out of two),
  no labels,
  no assignees,
  no milestones.
- Pull requests:
  ten sampled of eleven in the window,
  and all ten are open with no review.
  Several carry maintainer comments (`acmel` on 41 and 29,
  `alan-maguire` on 46)
  but none was merged,
  labelled or assigned through the host.
  That is what a mirror looks like:
  patches are reviewed and merged on the project's mailing list,
  and the host's pull requests are a secondary queue.
- Release record:
  no release objects on the host;
  the tool releases from the project's own site and its distribution packages,
  and the package here is `1:1.32-2`.
- Classification:
  active development in a single very experienced maintainer,
  with weak issue triage on the host and no merge path there;
  C5 rates the release cadence,
  not the host's queue.

### `bpftool`

- Primary URL:
  `https://github.com/libbpf/bpftool`,
  read 2026-11-20.
- Repository:
  created 2022-01-16,
  723 stars,
  124 forks,
  31 open items,
  last push 2026-11-09,
  license detection `NOASSERTION`,
  which is the three license files the repository carries
  (`LICENSE`,
  `LICENSE.BSD-2-Clause` and `LICENSE.GPL-2.0`).
- Maintainer concentration:
  the contributors read returned a full page of 100,
  so the count is at least 100;
  the largest are `qmonnet` 324,
  `dependabot` 68,
  `anakryiko` 33,
  `olsajiri` 32 and `borkmann` 17.
  In the window,
  100 commits,
  of which `dependabot` 22 and `qmonnet` 17;
  the rest is spread over 29 other authors,
  several of them one patch each.
- Issues in the window:
  16,
  all sampled,
  8 closed and 8 open,
  each closed one by a maintainer;
  comments run from 0 to 6 and maintainers answer most of them.
  Labels appear rarely,
  and no assignee or milestone was set.
- Pull requests:
  ten of 75 in the window,
  sampled as the skill asks.
  Nine are merged,
  all by `qmonnet`,
  and five of those are `dependabot` bumps,
  each reviewed by `qmonnet` before it merged.
  The one external code change among the merged (`#297`,
  `include: add u128 to linux/types.h`) was reviewed twice,
  and it merged in 0.7 days,
  so nothing external was merged unreviewed in this sample.
  The periodic "Bpftool sync" pull request (`#301`) brings the kernel tree's state across and carries no review,
  which is its normal shape.
  The one open external fix (`#299`,
  "fix line info offsets in CFG dump") has no review yet.
  Merge latency is short for the mechanical ones,
  hours to days.
- Release record:
  releases on the host,
  driven by tags and a release workflow;
  the binary here reports `7.8.0` while its package is `7.3.4-1`,
  which is the version-string finding
  ([The two tools' provenance and suites](#the-two-tools-provenance-and-suites)).
- Classification:
  active,
  well-staffed maintenance,
  with a real merge path and a dependence on one lead maintainer.

### What the audit changes

`btf-rs` is the only finalist whose pull requests are reviewed and merged by a second maintainer,
and it is the only one whose record shows hardening fixes landing in the window.
`ebeepf`'s C5 at 2 stands and is now backed by its whole record rather than three counts.
The two tools' C5 at 4 was asserted with no audit at all;
the audit supports a mature cadence for both,
but it also shows that `pahole` has no merge or triage path on the host at all,
so that 4 is a rating of the release record and not of the host's responsiveness.
No rating moves as a result of this audit;
what moves is that the ratings now have records behind them,
which is what the skill asks a C5 claim to have.

## Human auditability

The skill's high-trust overlay asks for measurements of the code a root process would run,
and the first revision named the overlay and then measured nothing against it.
This revision measured the four finalists from their own source,
on 2026-11-20,
with the commands kept.

- `btf-rs` 2.0.1:
  2,749 lines of Rust in 7 files under `src/`,
  of which the route the product calls,
  `from_bytes` and `from_split_bytes` through `btf.rs`,
  `cbtf.rs` and `section.rs`,
  is about 2,000;
  the rest is the optional ELF reader and the collection helpers;
  1 `unsafe` token,
  at `src/btf.rs:58`,
  in the file-mapping constructor the product's route does not call;
  5 direct runtime dependencies
  (`btf-rs-derive`,
  `byteorder`,
  `fallible-iterator`,
  `memmap2`,
  `thiserror`),
  one of them same-author (`btf-rs-derive`,
  114 lines);
  no process execution and no network anywhere in `src/`;
  five file-open or mapping sites in `btf.rs`,
  which are the path-based constructors,
  and none on the byte-slice route.
- `ebeepf` 0.1.0:
  28,769 lines of Rust in 25 files under `src/`,
  of which the BTF reader is `src/btf.rs` at 1,926 lines,
  6.7 percent of the crate,
  with the rest an eBPF loader the product would not use;
  57 `unsafe` tokens,
  none in `btf.rs`,
  concentrated in `sys.rs` (16),
  `netlink.rs` (9),
  `user_ringbuf.rs` and `ringbuf.rs` (8 each) and `map.rs` (7);
  7 direct runtime dependencies
  (`bitflags`,
  `flate2`,
  `goblin`,
  `libc`,
  `object` with `write`,
  `sha2`,
  `thiserror`);
  five `Command::new` calls in `src/skel.rs` that run `clang`,
  which the parsing route does not reach;
  no network.
- `pahole` 1.32:
  the BTF parse path is libbpf's `src/btf.c` at 6,780 lines of C plus the project's `btf_loader.c` at 863;
  there is no `unsafe` count to take,
  because all of it is unchecked by construction;
  the binary needs four shared libraries,
  all the project's own or `libc`;
  the product never links it,
  so its lines are the child's and not the handler's.
- `bpftool` 7.8.0:
  the same 6,780 lines of libbpf C plus the project's `src/btf.c` at 1,895;
  C again,
  so no `unsafe` count applies;
  the binary needs `libelf`,
  `libcrypto`,
  `libz`,
  `libcap` and `libLLVM`,
  the last a large dependency the product would load into the child.

Names of the measured commands:
`find <crate>/src -name '*.rs' -print0 | xargs --null wc -l`,
`rg --count-matches '\bunsafe\b'`,
`rg --count-matches 'std::process::Command|Command::new'`,
`rg --count-matches 'TcpStream|UdpSocket|reqwest|hyper'`,
`readelf --dynamic`,
`wc -l` over the two C sources.

What the measurements say:

the two libraries differ by a factor of ten in lines and by 57 to 1 in `unsafe` tokens,
and the product would use 7 percent of `ebeepf`'s code.
That is a real and large auditability difference,
and it is what C8 prices.
The tools are in the middle:
their parser is one 6,780-line C file reading bytes,
which is a smaller surface to read than `ebeepf`'s crate but not memory-safe,
and it does not run in the handler's address space at all,
which is an auditability advantage a line count does not show.
The overlay says not to give unused features positive weight;
the C8 ratings price the code the product would actually run or call,
not the crate's total size,
which is why `ebeepf` is not rated lower than the tools.

## Evidence for the scores

Each entry names its criterion,
weight,
rating,
confidence and score effect,
and a range's endpoints
appear where a range exists.
The design's own evidence records stand in its "Scored comparison";
these are the records this report checked,
with the run or path beside each claim.

### C1: bounded fail-closed handling of untrusted bytes, weight 1

- `btf-rs` 3 medium.
  Refuses truncated input and a wrong magic (four negative controls above),
  and its input is kernel-validated;
  it has no section check of its own and its released masks are the 16-bit count and 5-bit kind fields,
  so the pre-scan is the binding's (`btf-rs/src/section.rs:182,206`,
  `btf-rs/src/btf.rs:303-355`).
  The confidence is medium,
  not high as the first revision had it:
  the same evidence class is what `ebeepf` gets medium for,
  the crate has no fuzz target and no mutation testing,
  and its own history carries overflow and out-of-bounds fixes from 2026-02-04 and 2026-10-29
  (section "Maintenance audit").
  A high label would keep this rating out of the perturbation set,
  and one confidence label should not decide whether the library pair has one reversal or two.
  Effect:
  3 of 32.
- `ebeepf` 3 medium.
  Bounds both sections,
  validates every type reference before returning a table (`src/btf.rs:441-477`,
  `:708-751`),
  and refuses truncation and a wrong magic;
  4 is refused because its README calls the tree AI-generated code that has not been thoroughly reviewed,
  and because it accepts a split table as a base.
  Effect:
  3.
- `pahole` 3 medium and `bpftool` 3 medium.
  Each refuses through its own parser and exit status,
  and the wrapper must bound the file before handing it over.
  Effect:
  3 each.

### C2: release and deployment fit, weight 1

- Both libraries 4 high:
  pure Rust,
  default features empty,
  no C build,
  and both GNU targets build from source
  ([The `aarch64` build](#the-aarch64-build-and-what-it-does-not-show)).
  Effect:
  4 each.
- Both tools 1 high:
  each adds a program the product must find and version,
  and one of them in the root handler.
  Effect:
  1 each.

### C3: privilege surface in the root handler, weight 1, higher is smaller

- Both libraries 4 high:
  in-process code keeps the privileged handler to its own binary.
  Effect:
  4 each.
- Both tools 1 high,
  and the rating prices the shape the design names,
  a program the root handler starts:
  that shape adds a path and library resolution to the boundary.
  The premise the first revision gave,
  that the child must run as root,
  is wrong:
  both BTF files are mode `0444`,
  so the child can read them unprivileged and bounded
  ([The two tools' provenance and suites](#the-two-tools-provenance-and-suites)).
  The unprivileged-child shape is a variant the project can choose and this rubric does not price;
  it is one of the open questions.
  Effect:
  1 each.

### C4: diagnostic precision, weight 1

- `btf-rs` 4 high and `ebeepf` 4 high:
  each exposes names,
  bit offsets and sizes,
  so a refusal can name the member and both values;
  `ebeepf`'s `size_of` also multiplies array elements (`src/btf.rs:576-706`),
  which `btf-rs` leaves to the binding (`btf-rs/src/btf.rs:512` has `len()` only).
  Effect:
  4 each.
- `pahole` 3 medium:
  its `-C` printer maps directly to the compared members.
  Effect:
  3.
- `bpftool` 2 medium:
  the raw dump needs a type-id resolver in the wrapper,
  demonstrated by the harness.
  Effect:
  2.

### C5: maintenance and upstream responsiveness, weight 1

Every claim here is now backed by the sample recorded in section "Maintenance audit".

- `btf-rs` 3 medium:
  releases `v2.0.0` (2026-04-16) and `v2.0.1` (2026-11-12),
  every merged pull request in the window reviewed and merged by a second maintainer,
  merge latency from 0.5 to 91 days,
  hardening fixes landing in the window,
  two open items (PR 37,
  issue 1),
  four contributors with about 69 percent of commits from one of them,
  and the release lagging its own main at `cdf9ea46` on the widened masks.
  Effect:
  3.
- `ebeepf` 2 medium:
  one release,
  2026-10-06;
  one maintainer and 36 of 36 commits;
  no issues,
  no pull requests and no CI;
  no commit after the release;
  a README that declares the tree unreviewed.
  Effect:
  2.
- Both tools 4 high:
  both are packaged here and both follow a release cadence,
  and the audit shows a real merge path for `bpftool` and a single very active maintainer for `pahole`.
  It also shows that `pahole`'s host pull requests are all open with no review,
  because that project merges on a mailing list,
  so the rating covers the release record and not the host's queue.
  Effect:
  4 each.

### C6: integration and test cost, weight 1, higher is cheaper

- Both libraries 3 medium:
  the shared shape is the bounded loader,
  the pre-scan and its widest-encoding walk,
  the expectation table with its path grammar,
  the comparison,
  the refusal value and their tests,
  about 350 to 550 source lines and 300 to 450 test lines;
  the dependency route still owes one piece of arithmetic,
  array byte sizes,
  which `btf-rs` leaves to the binding and `ebeepf` supplies.
  `ebeepf` has no CI to lean on,
  which is why it does not rate higher than `btf-rs`.
  Effect:
  3 each.
- Both tools 3 medium:
  a spawn wrapper and an output parser each,
  plus packaging and a version matrix,
  which the unrun release matrix leaves uncertain.
  Effect:
  3 each.

### C7: start cost, weight 1, low confidence and unmeasured

- Both libraries 2.5 low,
  range 2 to 3:
  `btf-rs` parses the base's type offsets and the module's types;
  `ebeepf`'s `parse_split` copies the base's 151,181 types into the module's table,
  which is a memory cost `btf-rs` does not pay.
  No timing was taken by any round,
  so this is a structural reading,
  not a measurement.
  Effect:
  2.5 each,
  endpoints 2 and 3.
- Both tools 1.5 low,
  range 1 to 2:
  a process start and a full dump each.
  Effect:
  1.5 each,
  endpoints 1 and 2.

### C8: human auditability of the reader surface, weight 1, higher is leaner

- `btf-rs` 4 medium:
  2,749 lines,
  of which about 2,000 are the route the product calls,
  1 `unsafe` token outside the route the product calls,
  5 direct dependencies with one same-author proc-macro of 114 lines,
  no process execution and no network.
  It does not rate higher because its dependencies are five rather than none,
  and one of them runs at build time in the product's own build.
  Effect:
  4.
- `ebeepf` 2 medium:
  28,769 lines of which the reader is 1,926,
  57 `unsafe` tokens,
  7 direct dependencies,
  and five `clang` invocations in a module the parse route never calls.
  The product would link ten times the code it uses.
  Effect:
  2.
- Both tools 2 medium:
  the parser is one 6,780-line C file,
  which is smaller than `ebeepf`'s crate but unchecked by construction,
  and `bpftool`'s child also loads `libLLVM`.
  They do not rate higher because the child's code is not memory-safe either,
  and do not rate lower because none of it runs in the handler's address space.
  Effect:
  2 each.

The numbers behind every entry are section "Human auditability".

The in-house reader's ratings price code that does not exist:
C1 4,
C2 4,
C3 4,
C4 4,
C5 3,
C6 2,
C7 2.5,
C8 4.
Its 27.5 of 32 is a ceiling,
not a measurement,
and the design says so.

## Score arithmetic

Earned is the sum of weight times rating.
Two baselines are printed:
the frozen one,
which is the skill's weights and the one the ranking above uses,
and the design writer's weights,
reported as a sensitivity arm and never used to rank.

### The frozen baseline

Every weight 1,
maximum 32,
normalized to one decimal place.
Every figure below was recomputed by
`~/temp/agent/zfs-layout-checker-reader-vet-report-gate-review-findings-fixed-2026-11-20/scratch/recompute-refrozen-rubric-matrix.ts`,
whose output is kept beside it,
from the ratings in section "Evidence for the scores".

- `btf-rs` 2.0.1:
  `1*3 + 1*4 + 1*4 + 1*4 + 1*3 + 1*3 + 1*2.5 + 1*4` = 27.5 of 32,
  85.9.
- `ebeepf` 0.1.0:
  `1*3 + 1*4 + 1*4 + 1*4 + 1*2 + 1*3 + 1*2.5 + 1*2` = 24.5 of 32,
  76.6.
- `pahole` 1.32:
  `1*3 + 1*1 + 1*1 + 1*3 + 1*4 + 1*3 + 1*1.5 + 1*2` = 18.5 of 32,
  57.8.
- `bpftool` 7.8.0:
  `1*3 + 1*1 + 1*1 + 1*2 + 1*4 + 1*3 + 1*1.5 + 1*2` = 17.5 of 32,
  54.7.
- In-house reader,
  priced and not ranked:
  `1*4 + 1*4 + 1*4 + 1*4 + 1*3 + 1*2 + 1*2.5 + 1*4` = 27.5 of 32,
  85.9.

Low-signal endpoint totals,
over the two endpoints of C7's range:

- `btf-rs`:
  27 to 28,
  which is 84.4 to 87.5.
- `ebeepf`:
  24 to 25,
  which is 75.0 to 78.1.
- `pahole`:
  18 to 19,
  which is 56.3 to 59.4.
- `bpftool`:
  17 to 18,
  which is 53.1 to 56.3.
- In-house:
  27 to 28,
  which is 84.4 to 87.5.

### The design writer's weights, a reported sensitivity

5,
5,
4,
4,
3,
3,
2 and 1,
the last because that text states no priority for C8 either.
Maximum 108.

- `btf-rs`:
  94 of 108,
  87.0.
- `ebeepf`:
  89 of 108,
  82.4.
- `pahole`:
  62 of 108,
  57.4.
- `bpftool`:
  58 of 108,
  53.7.
- In-house:
  96 of 108,
  88.9.

Endpoint totals are 93 to 95,
88 to 90,
61 to 63,
57 to 59 and 95 to 97,
which is 86.1 to 88.0,
81.5 to 83.3,
56.5 to 58.3,
52.8 to 54.6 and 88.0 to 89.8.

At the seven criteria the first revision used,
the same ratings give 90,
87,
60 and 56 of 104 and 92 for the in-house reader.
The first revision printed 24 of 28 for the in-house reader at equal weights on those seven,
and called it "between" the two libraries;
the sum is 23.5,
which ties `btf-rs`'s 23.5,
as the design's own arithmetic says.
It changed no ranking then and it changes none now,
because that row is priced and never ranked.

Both baselines place `btf-rs` ahead of `ebeepf`,
and the writer's weights widen that gap from 3 points of 32 to 5 points of 108.
Neither baseline is a finding:
the frozen one is the skill's and the other is a preference nobody has stated.

## Sensitivity

Method,
one input at a time,
over the refrozen rubric:
every weight from 1 through 5 for each of the eight criteria,
every medium-confidence and low-confidence exact rating one step down and up within 0 through 4,
and both endpoints of every low-signal range.
The ranges' endpoints are the plausible values,
so no step is taken outside a range.
That defines 71 tests over the four eligible finalists,
and every one of them was run at both baselines.
Ranking is by the unrounded fraction,
and equal totals are an exact tie.

### At the frozen baseline

Baseline:
`btf-rs` 27.5 > `ebeepf` 24.5 > `pahole` 18.5 > `bpftool` 17.5 of 32.

- `btf-rs` against `ebeepf`,
  51 tests:
  all 51 keep the order.
  No rating step,
  no weight step and no range endpoint closes the 3-point gap,
  because `btf-rs`'s rating is at least `ebeepf`'s on every criterion,
  so no weight increase on any criterion can favour `ebeepf` either.
- `ebeepf` against `pahole`,
  52 tests:
  50 keep,
  1 ties and 1 reverses.
  - weight C5 from 1 to 4 ties at 30.5 each.
  - weight C5 from 1 to 5 reverses:
    `pahole` 34.5 over `ebeepf` 32.5.
    This is the test that moves a program past a library,
    and it is the most decision-relevant row in the matrix,
    because the pick the project is asked to make is exactly library against program.
- `pahole` against `bpftool`,
  52 tests:
  44 keep and 8 tie,
  with no reversal.
  - `pahole` C1 from 3 to 2 ties at 17.5 each.
  - `pahole` C4 from 3 to 2 ties at 17.5 each.
  - `pahole` C6 from 3 to 2 ties at 17.5 each.
  - `pahole` C8 from 2 to 1 ties at 17.5 each.
  - `bpftool` C1 from 3 to 4 ties at 18.5 each.
  - `bpftool` C4 from 2 to 3 ties at 18.5 each.
  - `bpftool` C6 from 3 to 4 ties at 18.5 each.
  - `bpftool` C8 from 2 to 3 ties at 18.5 each.

Ten of those 71 tests tie or reverse an adjacent order.
Under the skill's rule,
a tie asks for the controlling preference and invents no tiebreaker,
and a reversal means the order is not stable;
so no ranking at this baseline can be published as the report's order.

Only one of the eight weights moves the whole order,
and only above 3:

- weight C5 at 2:
  `btf-rs` 30.5 > `ebeepf` 26.5 > `pahole` 22.5 > `bpftool` 21.5,
  the baseline order;
- weight C5 at 3:
  `btf-rs` 33.5 > `ebeepf` 28.5 > `pahole` 26.5 > `bpftool` 25.5,
  the baseline order;
- weight C5 at 4:
  `btf-rs` 36.5 > `ebeepf` 30.5 = `pahole` 30.5 > `bpftool` 29.5,
  an exact tie;
- weight C5 at 5:
  `btf-rs` 39.5 > `pahole` 34.5 > `bpftool` 33.5 > `ebeepf` 32.5,
  so both programs pass the second library.

Every other criterion's weight,
at every value from 2 through 5,
leaves the order exactly as the baseline has it.
That is the sharpest statement this matrix makes about the decision:
the only single input that changes what the project would pick
is how much maintenance and upstream responsiveness should count.

The in-house pair is priced and not ranked,
and its matrix is recorded because the project could reopen it:
at the frozen baseline it ties at 27.5 each,
and 35 of its 47 tests move it,
24 to a tie and 11 to a reversal.
The reverses are the ones where `btf-rs` gains a step or the in-house ratings fall,
for example `btf-rs` C1 3 to 4 at 28.5 against 27.5,
and every C6 weight above 1,
because the in-house reader rates 2 there against `btf-rs`'s 3.

### At the design writer's weights

Baseline:
`btf-rs` 94 > `ebeepf` 89 > `pahole` 62 > `bpftool` 58 of 108.

- `btf-rs` against `ebeepf`,
  51 tests:
  49 keep and 2 tie.
  - `btf-rs` C1 from 3 to 2 ties at 89 each.
  - `ebeepf` C1 from 3 to 4 ties at 94 each.
  Both are the C1 confidence change this revision made visible:
  at weight 5 that rating is worth 5 points per step,
  which is exactly the pair's gap.
- `ebeepf` against `pahole`,
  52 tests:
  all keep.
  At these weights maintenance is 3 and the gap is 27 points,
  so nothing single moves it.
- `pahole` against `bpftool`,
  52 tests:
  48 keep,
  2 tie and 2 reverse.
  - `pahole` C1 from 3 to 2 reverses:
    `bpftool` 58 over `pahole` 57.
  - `pahole` C4 from 3 to 2 ties at 58 each.
  - `bpftool` C1 from 3 to 4 reverses:
    `bpftool` 63 over `pahole` 62.
  - `bpftool` C4 from 2 to 3 ties at 62 each.

Six of those 71 tests tie or reverse an adjacent order.
So the writer's weights are unstable on two pairs and stable on one,
the frozen weights are unstable on two pairs and stable on the library pair,
and no single pair is stable at both baselines.

### What the evidence could and could not settle

The skill's first remedy for a move is to gather decisive evidence,
refreeze and rerun.
This revision did the evidence half for three inputs:
the maintenance audit for C5 (section "Maintenance audit"),
the auditability measurements for C8 (section "Human auditability"),
and the C1 confidence label for `btf-rs` (section "C1").
It refroze and reran,
and the rerun still moves:

- the `ebeepf` and `pahole` pair still reverses,
  and the input that moves it is a weight,
  not a rating.
  A weight is a preference.
  No further source reading tells the project whether maintenance should count five times as much as deployment fit;
  the skill says to ask,
  and section "Open questions for the project" asks.
- the `pahole` and `bpftool` pair still ties on eight one-input steps.
  The C4 evidence orders `pahole` ahead when both are read carefully,
  because `pahole` prints the compared members and `bpftool` needs a resolver;
  but the skill does not let a report convert a tie into an order by argument,
  so the tie stands until a preference settles it.
- the library pair's two ties exist only at the writer's weights,
  which are not the baseline.

The aggregate ranges are 27 to 28,
24 to 25,
18 to 19 and 17 to 18 at the frozen baseline.
Stability here covers one input at a time and nothing else;
simultaneous multi-input changes are not claimed.

## Pros and cons for every finalist

### `btf-rs` 2.0.1

- Pros:
  split BTF and the type kinds the check needs;
  pure Rust with the default features off,
  no C build and no runtime program;
  a documented API and 168 integration tests plus four doctests on the CI path,
  64 under the default features;
  the consumer probe parses this host's pair and walks the anonymous union,
  183 checks with four negative controls refused;
  the archive maps to the tested commit on a byte diff;
  every merged pull request in the last twelve months was reviewed and merged by a second maintainer;
  2,749 lines,
  one `unsafe` token and no process execution or network in the crate.
- Cons:
  `LGPL-2.1-or-later` adds a copyleft-family dependency;
  no section validation in the crate,
  so the binding keeps the pre-scan;
  the release's masks lag its own main branch on the widened fields;
  overflow and out-of-bounds fixes landed as recently as 2026-10-29,
  which is why C1 is medium and not high;
  a build-time proc-macro from the same authors joins the audit surface;
  the published archive ships no tests,
  so its suite runs from the repository at the tag;
  no fuzz target.
- Cost:
  the shared shape,
  about 350 to 550 source lines and 300 to 450 test lines,
  plus the array-size shim and one fuzz target.
- Its score:
  27.5 of 32 at the frozen baseline,
  the highest of the eligible candidates,
  and 94 of 108 at the design writer's weights.
- Why it leads when the stability test is set aside:
  no other candidate's rating beats its own on any criterion,
  and a library needs no program the product must find and version.

### `ebeepf` 0.1.0

- Pros:
  a split parser with a byte-level API and a path constructor;
  `size_of` already multiplies array elements,
  so no array-size shim;
  pure Rust with the default features off;
  its archive suite and the consumer probe both ran.
- Cons:
  a dual `LGPL-2.1-only` and `BSD-2-Clause` crate,
  so it has to be taken under the BSD arm;
  one release,
  one maintainer and no CI,
  and no commit after the release;
  its README calls the tree unreviewed AI-generated code;
  released masks are the same 5-bit kind and 16-bit count fields as `btf-rs`'s;
  it accepts a split table as its own base,
  which the consumer probe showed;
  `parse_split` copies the base's types into the module's table;
  28,769 lines of which the reader is 1,926,
  with 57 `unsafe` tokens and seven direct dependencies;
  no fuzz target.
- Cost:
  the shared shape without the array-size shim.
- Its score:
  24.5 of 32 at the frozen baseline,
  three points behind on maintenance and auditability,
  and 89 of 108 at the design writer's weights.
- Why it is second among the libraries:
  the pair separates on maintenance,
  auditability and the C1 confidence the archive supports,
  and no single rated input at the frozen baseline moves it.

### `pahole` 1.32

- Pros:
  the command the design already names for its test-time comparison and the binding's live test already runs;
  `-C` prints the compared members;
  no new Rust dependency,
  and nothing of it runs in the handler's address space;
  a mature upstream with a release cadence and one very experienced maintainer;
  its BTF files are mode `0444`,
  so a child needs no privilege.
- Cons:
  a runtime package;
  a program the root handler starts;
  a text format;
  an invocation each for the struct,
  the nested type and the enum;
  the product must find and version it;
  its release matrix is unrun,
  its upstream suite was not run here,
  and its host pull requests get no review because the project merges on a mailing list;
  its parser is libbpf's 6,780 lines of C.
- Its score:
  18.5 of 32 at the frozen baseline,
  the runtime program costing the deployment and privilege points the libraries do not.
- Why it is third at the baseline and second at a maintenance weight of 5:
  the whole of its deficit is C2 and C3,
  and both are criteria a project can rank below maintenance.

### `bpftool` 7.8.0

- Pros:
  libbpf's reader;
  a documented base option and a raw type listing;
  a mature upstream with a real merge path and many contributors,
  and nothing of it in the handler's address space.
- Cons:
  the same runtime-program and root-path costs;
  the raw listing needs a type-id resolver before a member can be compared;
  member sizes are absent from the dump;
  its self-reported version and its package's version disagree,
  so a version check on the tool proves nothing;
  its child loads `libLLVM` beside `libelf`,
  `libcrypto`,
  `libz` and `libcap`;
  its functional suite was not run here.
- Its score:
  17.5 of 32 at the frozen baseline,
  the lowest of the four,
  for the resolver its output needs and its medium diagnostic rating.

### The in-house reader, priced and not ranked

- Pros:
  it can enforce every bound in one place;
  no dependency,
  no runtime program and no new license;
  it can read lazily,
  so C7 would be its to win once a reader exists;
  a partial decoder already exists on an unmerged branch
  (section "Discovery ledger").
- Cons:
  about 350 to 450 source lines plus 100 to 150 test lines,
  a fuzz target and a review of all of it;
  the branch's own decoder is 1,100 lines plus 422 of tests already,
  and its seam does not hold;
  every existing tool would still pass every hard gate,
  which is the skill's bar for writing one.
- Why unranked:
  the project answered on 2026-11-19 that our own parser is not a hard requirement,
  so the existing-tools rule keeps it ineligible while its 27.5 of 32 stays on record as what the route would have cost.

## Conditional rankings

No ranking here is the report's own.
The skill's rule is to rank by the provisional baseline only when every defined one-input test preserves order,
and at the frozen baseline ten of 71 tests do not.
What follows is therefore conditional in the skill's sense:
the order on each side of every input that moves it,
with the reason each adjacent pair sits that way.

### With every criterion at weight 1, the frozen baseline

`btf-rs` 27.5 > `ebeepf` 24.5 > `pahole` 18.5 > `bpftool` 17.5 of 32.

- `btf-rs` leads `ebeepf` because both are libraries in the wrapper's address space,
  and the pair separates on maintenance and on auditability:
  reviewed and merged pull requests,
  hardening fixes landing,
  2,749 lines and one `unsafe` token,
  against one release,
  one maintainer,
  no CI and ten times the code for 7 percent of it used.
  All 51 one-input tests on that pair leave the order,
  which is the only adjacent pair the matrix does not move at these weights.
- `ebeepf` leads `pahole` only while maintenance counts no more than three times deployment fit.
  The reason it leads at all is that a library needs no program the product must find and version
  and no binary the handler starts;
  at a maintenance weight of 4 the pair ties,
  and at 5 it reverses.
- `pahole` leads `bpftool` because its `-C` output carries the compared members
  while `bpftool`'s raw dump needs a type-id resolver,
  and because `bpftool`'s parser adds 1,895 lines over the same libbpf.
  Eight one-input steps tie that pair and none reverses it,
  so that order holds at the frozen baseline and its margin is one point of 32.

### If maintenance and responsiveness weigh 4

`ebeepf` 30.5 ties `pahole` 30.5.
No order between the second library and the first program is supportable at that weight,
and the skill says to ask which the project prefers rather than to break the tie.
That question is the second of [Open questions for the project](#open-questions-for-the-project).

### If maintenance and responsiveness weigh 5

`btf-rs` 39.5 > `pahole` 34.5 > `bpftool` 33.5 > `ebeepf` 32.5.
Both programs pass the second library,
and the program arm becomes the rational pick under that priority.
This is the row the first revision's matrix lacked and the gate review named as the most decision-relevant one.

### If the project reopens the in-house reader

That row is priced and not ranked because it is ineligible,
not because it lost.
At the frozen baseline it ties `btf-rs` at 27.5 of 32,
and 35 of its 47 tests move it,
24 to ties and 11 to reversals,
so a reopened in-house route would begin with an unresolved comparison rather than an answer.

## Confidence and evidence limits

Stated beside the rows they qualify,
gathered here so nothing is buried:

- `aarch64` builds and does not run,
  for both libraries.
  H3's second target has a build for each and no execution,
  and the two tools have no `aarch64` artifact at all.
  The build is the gate review's run,
  cited from its kept logs.
- The start cost is unmeasured,
  so C7 is a low-confidence range in every row.
  Measurement item 10 is the run that would settle it,
  together with the disk-cache thresholds.
- The weights are the skill's default because the project stated no priority.
  The design writer's weights are reported as a sensitivity arm and never ranked,
  and both arms are unstable on some pair.
- The tools' validation is one layer,
  not three:
  their output boundary was exercised with negative controls on this host's pair,
  and neither upstream suite was run.
  Their C5 and C6 rest on the records in section "Maintenance audit" and on nothing they executed.
- The tools' provenance is a distribution package and a signature,
  not a binary-to-source mapping;
  `bpftool`'s self-reported version differs from its package's,
  which is the same cost C2 prices,
  seen in a measurement.
- `ebeepf`'s provenance is its weakest point:
  one release,
  one maintainer,
  no CI,
  a README that declares the tree unreviewed,
  and a split base accepted as a base.
  Its 24.5 is a rating on that record,
  not a forecast.
- `btf-rs`'s published archive ships no tests and the suite ran from the repository at the tag;
  its release lags its own main on the BTF masks,
  so a future kernel's widened fields are refused by the binding's pre-scan until a release catches up.
- Neither library has a fuzz target,
  and no mutation testing exists for either;
  the negative controls are the only adversarial evidence.
- CI was reproduced on the CachyOS image with stable 1.100.0,
  not on GitHub's `ubuntu-latest`,
  so the two differ in distribution and in what else is installed.
- The class 3 web class is an inferred exhaustion and not a reported one,
  though each query ended below the provider's own cap,
  none sent a filter,
  and the review's eight extra queries found nothing new.
- The host kernel `7.3.5-1-cachyos` is not the product's qualified release until step 23,
  so the passing runs qualify the readers on this host's pair,
  not the product on every kernel.
- The 32-byte BTF header shape is what these files carry;
  the 24-byte shape a 6.8 guest's tools emit is owed measurement item 6 and no run here covers it.
- The independent review this revision answers was written by a different vendor from the authoring session,
  so it counts under rule `IRW` as an independent read.
  This revision's own edits are not independently reviewed:
  the same session made them,
  and a fresh cross-vendor read,
  or the project's own reading,
  is what would make them independent.

## Terminal result: sensitivity unresolved

The skill's terminal outcome for this evidence is "sensitivity unresolved":
record conditional rankings for each outcome-changing input and recommend none.
That is this report's result.

Why the recommendation the first revision made is withdrawn:

- The skill allows a ranking only when every defined one-at-a-time test preserves order.
  At the frozen baseline ten of 71 do not:
  a weight of 4 ties the second library with the first program,
  a weight of 5 reverses them and moves the other program ahead too,
  and eight rating steps tie the two programs.
- The first revision's grounds do not survive the refreeze.
  Its refusal of one reversal left that rating's confidence untouched,
  so the matrix never changed;
  two of the four ties it explained away as maintenance ties are C6 steps that no maintenance evidence touches;
  and its separation of the libraries rested on evidence it had not recorded.
- The evidence the skill's loop asks for was gathered and applied,
  and the rerun still moves.
  What is left is a preference:
  how much maintenance should weigh against deployment fit and privilege.
  That is the project's to state,
  and section "Open questions" asks it.

What the report therefore carries:

- the frozen baseline's order,
  `btf-rs` > `ebeepf` > `pahole` > `bpftool`,
  as the order at those weights and under no claim of stability;
- the conditions that change it,
  spelled out in section "Conditional rankings";
- the two open questions the project has to answer for any order to become a recommendation.

What does not change:
`btf-rs` 2.0.1 is the leading option at both baselines,
it is the only finalist whose pull requests are reviewed and merged by a second maintainer,
and it is the leanest reader any of the four offers.
Reading the evidence strictly,
it is the option this report would recommend if a priority had to be assumed;
the skill does not let a report assume one,
so it recommends none.

Beside the result,
in the same breath:

- `aarch64` is built and unrun for both libraries;
- the start cost is unmeasured;
- the tool arm's validation is one layer deep and its suites are unrun;
- and this is not adoption:
  no crate is added to any manifest,
  and the decision record's Q1 section still says the pick is the project's.

## What this report did not do

- It adopted no candidate,
  added no crate to any manifest,
  edited no decision record,
  and wrote nothing under `doc/handover/`.
- It ran no third-party code.
  It read source,
  archives,
  package records and repository-host responses,
  and every execution record above is a kept log of a round run earlier in this worktree or by the gate review.
- It ran nothing on `aarch64` and took no timing,
  and its `aarch64` build evidence is the gate review's,
  cited where it is kept.
- It did not re-run the finalist suites,
  the two tools' upstream suites,
  or the class 3 queries;
  it corroborated the crates.io count only.
- It did not open an issue,
  a pull request or a comment on any candidate's repository,
  and it sent no query carrying this repository's name or this machine's addresses.
- It is not final and not approved;
  the project declares that.

## Sources

Internal:

- [`doc/decision/cli-git-zfs-runtime-layout-checker.md`][record],
  read whole,
  including its Q1 section and the answers of 2026-11-19.
- [`doc/planning/cli-git-zfs-runtime-layout-checker-design.md`][design],
  "Candidate ledger",
  "Discovery record",
  "Finalist validation",
  "Scored comparison",
  "Sensitivity",
  "Ranking and pros and cons",
  "The vet artifacts this brief could not write",
  "Owed measurements" and Q1.
- [`doc/audit/tech-rust-crate-for-openzfs-vet-2026-11-18.md`](tech-rust-crate-for-openzfs-vet-2026-11-18.md),
  for the wrapper's release model and the struct difference between OpenZFS releases.
- `package/zfs/request-to-openzfs-kernel-device/Cargo.toml`,
  `README.md` and `src/kernel_device/live_btf_tests.rs`,
  read for the incumbent question.
- `toolchain.lock.json:614-653`,
  for the two GNU release archives.

Kept evidence:

- `/home/user/temp/agent/zfs-layout-checker-reader-vet-report-gate-review-findings-fixed-2026-11-20/scratch/keep/`,
  this revision's own records:
  the recomputed matrix and its output,
  the repository-host responses under `gh/`,
  the per-sample maintainer records under `gh/detail/`,
  the fingerprint and lock scripts,
  and the lint mapping script.
- `/home/user/temp/agent/gate-review-of-the-btf-reader-vet-report-on-sonnet-2026-11-20/keep/`,
  the independent review this revision answers,
  its recomputation of the first revision's matrix,
  the two `aarch64` build logs and their driver,
  its extra crates.io probe,
  and its consumer re-run.
- `/home/user/temp/agent/zfs-layout-checker-reader-finalist-validation-2026-11-20/keep/`,
  its `README.md`,
  `report-from-summary.md`,
  `evidence/btf-rs-2.0.1/`,
  `evidence/checksums.txt`,
  `evidence/tool-harness-checks.txt` and `evidence/tool-harness-invocations.txt`.
- `/home/user/temp/agent/zfs-layout-checker-reader-discovery-finished-2026-11-20/keep/`,
  its `README.md`,
  `report-from-summary.md`,
  the three query listings,
  the organization response files and the board pages,
  and `evidence/ebeepf-0.1.0/`.
- `/home/user/temp/agent/zfs-layout-checker-reader-vet-report-2026-11-20/keep/`,
  the first revision's fingerprint script,
  registry response and lock record.

External,
all read 2026-11-20:

- crates.io [crate pages][crates-io],
  the sparse index for `btf-rs`,
  the `q=btf`,
  `q=split btf`,
  `q=btf kernel` and `q=libbpf` search endpoints,
  and the three archives this revision downloaded and hashed.
- [docs.rs][docs-rs] for both library crates.
- the repository hosts' records for
  [`retis-org/btf-rs`](https://github.com/retis-org/btf-rs),
  [`kxxt/ebeepf`](https://github.com/kxxt/ebeepf),
  [acmel/dwarves][pahole-repo],
  [`libbpf/bpftool`](https://github.com/libbpf/bpftool) and
  [`libbpf/libbpf`](https://github.com/libbpf/libbpf),
  through their API,
  and four read-only clones at depth 1.
- the distribution's package database through `pacman`,
  for both tools' packages,
  their versions,
  their declared dependencies and their signature validation.
- [bpftool documentation][bpftool-docs] for `btf dump` and `--base-btf`,
  and the [pahole repository][pahole-repo] for `--btf_base` and `-C`.
- [Linux kernel BTF documentation][kernel-btf] for the header and the split-file format,
  at tags `v6.8` and `v7.3` as the design records.

[skill]: ../../.agents/skills/choosing-technology/SKILL.md
[record]: ../decision/cli-git-zfs-runtime-layout-checker.md
[design]: ../planning/cli-git-zfs-runtime-layout-checker-design.md
[design-discovery]: ../planning/cli-git-zfs-runtime-layout-checker-design.md#discovery-record
[design-check-in]: ../planning/cli-git-zfs-runtime-layout-checker-design.md#the-check-in-of-2026-11-20
[crates-io]: https://crates.io/crates/btf-rs
[docs-rs]: https://docs.rs/btf-rs
[bpftool-docs]: https://docs.kernel.org/bpf/bpftool-btf.html
[pahole-repo]: https://github.com/acmel/dwarves
[kernel-btf]: https://docs.kernel.org/bpf/btf.html
