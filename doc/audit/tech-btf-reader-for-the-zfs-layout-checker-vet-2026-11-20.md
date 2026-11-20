# BTF reader for the ZFS layout checker: technology vet (2026-11-20)

Not final and not approved.
This report recommends a reader and adopts nothing (rule `SGO`).
Only the project declares the work complete.

Status:
complete as a report,
at lifecycle phase 8,
Recommended.
Phase 9,
Adopted,
waits for an action request that names the reader.
The report recommends `btf-rs` 2.0.1,
with the `aarch64` target unrun and the start cost unmeasured stated beside it,
and it records how the sensitivity rule was read,
so the project can read that rule strictly and land on "recommend none" instead
(section "Recommendation").
Checked by this session:
every evidence record below was read against the kept log it names;
the score arithmetic and every sensitivity row this report lists,
recomputed from the published ratings;
the fingerprint recomputed from its printed input;
and the host facts re-read.
Not checked:
the suite runs themselves,
which this report did not repeat,
and the sensitivity test counts stated as totals,
which are the design's own counts.

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
  2026-11-20T18:17:40-05:00,
  the write time of this revision.
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
  run twice,
  the second at 2026-11-20T18:16:26-05:00,
  both exit 0 and both printing that value.
- Lock:
  `~/temp/agent/technology-vet-locks/` + the SHA-256 of the absolute report path + `.lock`.
  This report is new,
  so the lock records `preEditSha256: null` and the report was created with create-new semantics instead of a rename.
  The record of that lock,
  its holder and its release is kept at
  `~/temp/agent/zfs-layout-checker-reader-vet-report-2026-11-20/keep/lock-record.json`.
- Active audit owner:
  this session,
  a Pi session in the worktree `.claude/worktrees/zfs-layout-checker-reader-validation`,
  on branch `docs/planning-cli-git-zfs-layout-checker-reader-validation`.
  No second agent wrote any line of this report.
  The discovery enumeration was delegated to this session's writer by the project's answer of 2026-11-20;
  the finalist validation and the discovery round were run by earlier sessions in the same worktree,
  and this report is their synthesis,
  not a re-run.
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

## Fingerprint input

Every field the skill names,
with its value.
The script sorts the two set-valued arrays and the deployment keys by raw UTF-16 code-unit order,
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
The step that would have held a reader,
step 6 of the implementation plan,
is unbuilt.

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
which is a choice for the owner (section "Unresolved preferences").
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

## Frozen criteria and weights

Frozen before any candidate-specific soft evidence was rated,
and applied to every finalist.

- C1,
  bounded,
  fail-closed handling of untrusted bytes,
  weight 5.
- C2,
  release and deployment fit:
  no runtime program,
  no C build,
  both GNU targets,
  weight 5.
- C3,
  privilege surface in the root handler,
  weight 4,
  higher is smaller.
- C4,
  diagnostic precision:
  member-level expected and actual values,
  weight 4.
- C5,
  maintenance and upstream responsiveness,
  weight 3.
- C6,
  integration and test cost,
  weight 3,
  higher is cheaper.
- C7,
  start cost,
  weight 2,
  low confidence because it is unmeasured.

Ratings run 0 to 4,
every rating carries a confidence,
and a low-signal rating carries the evidence-supported minimum and maximum whose midpoint is the provisional value.
No criterion is removed from the denominator:
each one bears on at least one finalist.
Maximum points are 104:
the weight sum 26 times 4.

Two things about the rubric are the writer's and not the project's,
and they are stated here rather than buried.
The weights are the design writer's;
the project stated no priority among the criteria,
and the skill's default for an unspecified priority is weight 1 for every criterion.
The equal-weight order is reported beside the weighted one
(section "Score arithmetic") so the project can read the same evidence under its own default.
The ratings for the in-house reader price code that does not exist,
and the arithmetic marks that score as priced,
not ranked.

## Unresolved preferences

- The pick.
  This report recommends;
  the project decides.
  The decision record's Q1 section says the pick is a decision the project still owes,
  and this report is its input,
  not its answer.
- Weights.
  No priority was stated,
  so the writer's weights and the equal-weight default both appear.
  A priority the project states can move the two library rows against each other only,
  and the evidence for that pair is in section "Sensitivity".
- The disk-cache question,
  Q3 of the decision record,
  is held and stays separate.
  Its thresholds,
  a 100 ms median above the parse's baseline and a 256 MiB resident peak,
  are the design's proposal from a single probe with no control,
  and measurement item 10 is the run that replaces them.
  Nothing here needs its answer:
  the per-process cache ships either way.
- The one preference the skill's own procedure would ask for,
  if the project reads its sensitivity rule strictly,
  is the live pair:
  `btf-rs` against `ebeepf`.
  Section "Recommendation" states both readings and their outcomes,
  so the question is visible rather than implied.

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

The reader is the one unbuilt piece of that gate.
The project answered the two questions that bounded it:
the parser need not be our own code and the product may start no second program (2026-11-19,
"No,
not required"),
and no release string decides ("Any version,
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
`aarch64` is unrun for every finalist,
and the start cost is unmeasured.

## The pre-discovery check-in of 2026-11-20

The skill asks one check-in per broad vetting process,
before the first discovery query,
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

The question was put to the project on 2026-11-20 and answered the same day.
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
  this session;
  a model the project names;
  the project itself.
- Answer:
  "This session's writer (Recommended)",
  described as a `DeepSeek V4.1 Flash` writer through Pi running the remaining queries now.
- Date:
  2026-11-20.

The answer named this session's writer,
and the round's three class 3 queries and two class 2 list reads ran here afterwards,
as the answer describes;
no delegated starting brief was needed.

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
  7.8.0 on libbpf 1.8;
  the tool's license is the kernel's (GPL-2.0 with BSD-2 parts) and it runs as a separate process,
  which is not a link.
- Screening:
  pass on H1,
  H2 for a separate process,
  H4,
  H5,
  H6;
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
  GPL-2.0 for the dwarves package,
  a separate process.
- Screening:
  pass on H1,
  H2 for a separate process,
  H4,
  H5,
  H6.
  Finalist.

### The in-house reader

- Discovery:
  repository precedent (the binding's own unsafe boundary and its "our own small binding" decision)
  and the skill's existing-tools rule.
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
  H3 pass on source with the `aarch64` target unrun,
  H4 pass,
  H5 pass,
  H6 pass.
  Finalist,
  validated on `x86_64`.
- `ebeepf` 0.1.0:
  H1 pass (split constructor and the probe),
  H2 pass on the BSD arm,
  H3 pass on source with `aarch64` unrun,
  H4 pass,
  H5 pass,
  H6 pass.
  Finalist,
  validated on `x86_64`.
- `pahole` 1.32:
  H1 pass,
  H2 pass as a separate process,
  H3 not applicable as a library and recorded as a runtime requirement instead,
  H4 pass,
  H5 pass,
  H6 pass.
  Finalist,
  validated at its output boundary.
- `bpftool` 7.8.0:
  H1 pass,
  H2 pass as a separate process,
  H3 not applicable as a library,
  H4 pass,
  H5 pass,
  H6 pass.
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
  SHA-256 `00acbd859cd0cca6e71dca93d885ed2483c5a52a78d3df1a9bcc9da82881503c`;
  `/usr/bin/bpftool` 7.8.0 on libbpf 1.8,
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
- Omits `aarch64`:
  no build and no run happened,
  so H3's second target is source evidence only,
  as it was before the round.
  That is the one platform omission and it is not a suite the round chose to skip.
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
- Omits `aarch64`,
  for a recorded reason:
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
  It happened twice and is recorded as an unavailable platform,
  not as a pass.
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
- Omitted,
  with reasons:
  neither tool's own upstream test suite was run,
  because the product would start the packaged program and never link or build it,
  so its upstream suite cannot affect a claimed or consumed surface;
  the product's own tool arms are owed measurements 13 and 14,
  and the release-version matrix they name is unrun,
  which is why C2 and C7 keep their low ratings for the tools.
- Omitted `aarch64`:
  the host has no `aarch64` environment,
  so both binaries are `x86_64` evidence only.

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

### C1: bounded fail-closed handling of untrusted bytes, weight 5

- `btf-rs` 3 high.
  Refuses truncated input and a wrong magic (four negative controls above),
  and its input is kernel-validated;
  it has no section check of its own and its released masks are the 16-bit count and 5-bit kind fields,
  so the pre-scan is the binding's (`btf-rs/src/section.rs:182,206`,
  `btf-rs/src/btf.rs:303-355`).
  Effect:
  15 of 104.
- `ebeepf` 3 medium.
  Bounds both sections,
  validates every type reference before returning a table (`src/btf.rs:441-477`,
  `:708-751`),
  and refuses truncation and a wrong magic;
  4 is refused because its README calls the tree AI-generated code that has not been thoroughly reviewed,
  and because it accepts a split table as a base.
  Effect:
  15.
- `pahole` 3 medium and `bpftool` 3 medium.
  Each refuses through its own parser and exit status,
  and the wrapper must bound the file before handing it over.
  Effect:
  15 each.

### C2: release and deployment fit, weight 5

- Both libraries 4 high:
  pure Rust,
  default features empty,
  no C build,
  both GNU targets on source.
  Effect:
  20 each.
- Both tools 1 high:
  each adds a program the product must find and version,
  and one of them in the root handler.
  Effect:
  5 each.

### C3: privilege surface in the root handler, weight 4, higher is smaller

- Both libraries 4 high:
  in-process code keeps the privileged handler to its own binary.
  Effect:
  16 each.
- Both tools 1 high:
  a root-spawned program adds a path and library resolution to that boundary.
  Effect:
  4 each.

### C4: diagnostic precision, weight 4

- `btf-rs` 4 high and `ebeepf` 4 high:
  each exposes names,
  bit offsets and sizes,
  so a refusal can name the member and both values;
  `ebeepf`'s `size_of` also multiplies array elements (`src/btf.rs:576-706`),
  which `btf-rs` leaves to the binding (`btf-rs/src/btf.rs:512` has `len()` only).
  Effect:
  16 each.
- `pahole` 3 medium:
  its `-C` printer maps directly to the compared members.
  Effect:
  12.
- `bpftool` 2 medium:
  the raw dump needs a type-id resolver in the wrapper,
  demonstrated by the harness.
  Effect:
  8.

### C5: maintenance and upstream responsiveness, weight 3

- `btf-rs` 3 medium:
  releases `v2.0.0` (2026-04-16) and `v2.0.1` (2026-11-12),
  commits through 2026-10-29,
  one merged out-of-bounds fix (PR 42,
  2026-10-29,
  in 2.0.1),
  two open items (PR 37,
  issue 1),
  4 forks,
  a small maintainer group,
  and the release lagging its own main at `cdf9ea46` on the widened masks.
  Effect:
  9.
- `ebeepf` 2 medium:
  one release,
  2026-10-06;
  one maintainer;
  0 stars,
  0 forks,
  0 open issues;
  no CI (`.github` answers 404);
  no commit after the release;
  a README that declares the tree unreviewed.
  Effect:
  6.
- Both tools 4 high:
  each follows its project's release cadence,
  and both are packaged here.
  Effect:
  12 each.

### C6: integration and test cost, weight 3, higher is cheaper

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
  9 each.
- Both tools 3 medium:
  a spawn wrapper and an output parser each,
  plus packaging and a version matrix.
  Effect:
  9 each.

### C7: start cost, weight 2, low confidence and unmeasured

- Both libraries 2.5 low,
  range 2 to 3:
  `btf-rs` parses the base's type offsets and the module's types;
  `ebeepf`'s `parse_split` copies the base's 151,181 types into the module's table,
  which is a memory cost `btf-rs` does not pay.
  No timing was taken by any round,
  so this is a structural reading,
  not a measurement.
  Effect:
  5 each,
  endpoints 4 and 6.
- Both tools 1.5 low,
  range 1 to 2:
  a process start and a full dump each.
  Effect:
  3 each,
  endpoints 2 and 4.

The in-house reader's ratings price code that does not exist:
C1 4,
C2 4,
C3 4,
C4 4,
C5 3,
C6 2,
C7 2.5.
Its 92 of 104 is a ceiling,
not a measurement,
and the design says so.

## Score arithmetic

Earned is the sum of weight times rating;
maximum is 104;
normalized is earned over maximum times 100,
shown to one decimal place.

- `btf-rs` 2.0.1:
  `5*3 + 5*4 + 4*4 + 4*4 + 3*3 + 3*3 + 2*2.5` = 90 of 104,
  86.5.
- `ebeepf` 0.1.0:
  `5*3 + 5*4 + 4*4 + 4*4 + 3*2 + 3*3 + 2*2.5` = 87 of 104,
  83.7.
- `pahole` 1.32:
  `5*3 + 5*1 + 4*1 + 4*3 + 3*4 + 3*3 + 2*1.5` = 60 of 104,
  57.7.
- `bpftool` 7.8.0:
  `5*3 + 5*1 + 4*1 + 4*2 + 3*4 + 3*3 + 2*1.5` = 56 of 104,
  53.8.
- In-house reader,
  priced and not ranked:
  `5*4 + 5*4 + 4*4 + 4*4 + 3*3 + 3*2 + 2*2.5` = 92 of 104,
  88.5.

Low-signal endpoint totals,
over the ranges' endpoints:

- `btf-rs`:
  89 to 91,
  which is 85.6 to 87.5.
- `ebeepf`:
  86 to 88,
  which is 82.7 to 84.6.
- `pahole`:
  59 to 61,
  which is 56.7 to 58.7.
- `bpftool`:
  55 to 57,
  which is 52.9 to 54.8.
- In-house:
  91 to 93,
  which is 87.5 to 89.4.

At the skill's default of weight 1 for every criterion whose priority is unspecified,
the same ratings give:

- `btf-rs` 23.5 of 28.
- `ebeepf` 22.5 of 28.
- `pahole` 16.5 of 28.
- `bpftool` 15.5 of 28.
- In-house 24 between `btf-rs` and `ebeepf`,
  priced and not ranked.

The order is the same at equal weights,
and the gap between the two libraries narrows to one point,
which is where the sensitivity question lives.

## Sensitivity

Method,
one input at a time:
every weight from 1 to 5,
every medium-confidence and low-confidence exact rating one step down and up within 0 through 4,
and both endpoints of every low-signal range.
The ranges' endpoints are the plausible values,
so no step is taken outside a range.
Ranking is by the unrounded fraction.

The two earlier pairs,
`btf-rs` against the in-house reader and `pahole` against `bpftool`,
changed order on twelve inputs:
eight reverses and four exact ties.
The design's list is reproduced in its "Sensitivity",
and this report checked its arithmetic on every row it names:

- `btf-rs` against in-house,
  90 against 92:
  C1 weight 1 reverses at 78 against 76;
  C1 weight 2 reverses at 81 against 80;
  C1 weight 3 ties at 84 each;
  C6 weight 5 ties at 96 each;
  `btf-rs` C5 to 4 reverses at 93 against 92;
  `btf-rs` C6 to 4 reverses at 93 against 92;
  in-house C5 to 2 reverses at 89 against 90;
  in-house C6 to 1 reverses at 89 against 90.
- `pahole` against `bpftool`,
  60 against 56:
  `bpftool` C1 to 4 reverses at 61 against 60;
  `pahole` C1 to 2 reverses at 55 against 56;
  `bpftool` C4 to 3 ties at 60 each;
  `pahole` C4 to 2 ties at 56 each.
- No test moves a tool past a library;
  more than twenty points of 104 separate them.

Six of those eight reverses are the in-house pair,
which no longer decides anything:
the project answered on 2026-11-19 that the in-house reader is not offered,
so those tests record what the route would have cost.
The two `pahole` and `bpftool` reverses and two ties are decided by the C4 evidence:
`pahole`'s `-C` output carries the compared members while `bpftool`'s needs a resolver,
so `pahole` ranks ahead when that rating moves.

The live pair,
`btf-rs` against `ebeepf`,
90 against 87,
ran 42 one-input tests:
37 leave the order,
1 reverses it,
and 4 tie it.

- `ebeepf` C1 from 3 to 4 reverses to 92 against 90.
- `ebeepf` C5 from 2 to 3 ties at 90 each.
- `ebeepf` C6 from 3 to 4 ties at 90 each.
- `btf-rs` C5 from 3 to 2 ties at 87 each.
- `btf-rs` C6 from 3 to 2 ties at 87 each.
- The rest leave it:
  C1 weight 1 at 78 against 75,
  C1 weight 2 at 81 against 78,
  C5 weight 5 at 96 against 91,
  C6 weight 5 at 96 against 93,
  C7 weight 5 at 97.5 against 94.5.
- Both C7 endpoints leave it:
  `btf-rs` at 89 to 91 against `ebeepf`'s 87,
  and `ebeepf` at 86 to 88 against `btf-rs`'s 90.

Resolution,
as the skill's first remedy asks:
gather decisive evidence,
then read the order on it.

- The one reversal rests on a step the evidence refuses.
  `ebeepf`'s C1 at 4 would claim a trust the crate's own README denies and the accepted-split-base residual contradicts,
  so 3 is the rating its bounds and provenance support.
  With the step refused,
  the reversal does not stand.
- The four ties are C5 and C6 steps on either side,
  and the maintenance and provenance evidence orders that pair without arithmetic:
  two releases,
  a merged out-of-bounds fix and a workflow on every push,
  against one release,
  one maintainer,
  no CI,
  and a self-declared unreviewed tree.
- The equal-weight run keeps the order at 23.5 against 22.5,
  so the pair's order does not rest on the writer's weights either.

What the matrix therefore supports:
the order `btf-rs` > `ebeepf` holds under every defined one-input test
once the reversal is refused and the ties are read on evidence.
What it does not support:
a claim that the pair is separated by arithmetic.
The narrowest gap between a library and a tool is 27 points of 104,
and no defined one-input test closes it.

Stability covers one input at a time and nothing else;
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
  the archive maps to the tested commit.
- Cons:
  `LGPL-2.1-or-later` adds a copyleft-family dependency;
  no section validation in the crate,
  so the binding keeps the pre-scan;
  the release's masks lag its own main branch on the widened fields;
  a small maintainer group;
  the published archive ships no tests,
  so its suite runs from the repository at the tag;
  no fuzz target.
- Cost:
  the shared shape,
  about 350 to 550 source lines and 300 to 450 test lines,
  plus the array-size shim and one fuzz target.
- Why here:
  90 of 104,
  the highest of the eligible candidates,
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
  one maintainer,
  no CI;
  its README calls the tree unreviewed AI-generated code;
  released masks are the same 5-bit kind and 16-bit count fields as `btf-rs`'s;
  it accepts a split table as its own base,
  which the consumer probe showed;
  `parse_split` copies the base's types into the module's table;
  no fuzz target.
- Cost:
  the shared shape without the array-size shim.
- Why here:
  87 of 104,
  three points behind on maintenance and provenance,
  and ahead of the tools because a library needs no second program.

### `pahole` 1.32

- Pros:
  the command the design already names for its test-time comparison and the binding's live test already runs;
  `-C` prints the compared members;
  no new Rust dependency;
  a mature upstream with a release cadence.
- Cons:
  a runtime package;
  a root-spawned program in the handler;
  a text format;
  an invocation each for the struct,
  the nested type and the enum;
  the product must find and version it;
  its release matrix is unrun.
- Why here:
  60 of 104,
  the runtime program costing the deployment and privilege points the libraries do not.

### `bpftool` 7.8.0

- Pros:
  libbpf's reader;
  a documented base option and a raw type listing;
  a mature upstream.
- Cons:
  the same runtime-program and root-path costs;
  the raw listing needs a type-id resolver before a member can be compared;
  member sizes are absent from the dump;
  the tool's version tracks the kernel's tools rather than the distribution's.
- Why here:
  56 of 104,
  the lowest of the four,
  for the resolver its output needs and its medium diagnostic rating.

### The in-house reader, priced and not ranked

- Pros:
  it can enforce every bound in one place;
  no dependency,
  no runtime program and no new license;
  it can read lazily,
  so C7 would be its to win once a reader exists.
- Cons:
  about 350 to 450 source lines plus 100 to 150 test lines,
  a fuzz target and a review of all of it;
  every existing tool would still pass every hard gate,
  which is the skill's bar for writing one.
- Why unranked:
  the project answered on 2026-11-19 that our own parser is not a hard requirement,
  so the existing-tools rule keeps it ineligible while its 92 of 104 stays on record as what the route would have cost.

## Ranking

Eligible candidates,
best first:
`btf-rs` 2.0.1 > `ebeepf` 0.1.0 > `pahole` 1.32 > `bpftool` 7.8.0.
The in-house reader stands outside the ranking,
ineligible and priced at 92 of 104.

- `btf-rs` beats `ebeepf` because both are libraries in the wrapper's address space
  and the pair separates on maintenance and provenance:
  two releases,
  a merged out-of-bounds fix and a workflow on every push,
  against one release,
  one maintainer,
  no CI and a self-declared unreviewed tree.
  The arithmetic agrees at 90 against 87,
  and the pair's single reversal and four ties are the C1 and C5 steps the evidence settles.
- `ebeepf` beats `pahole` because it needs no program the product must find and version
  and no binary the root handler starts;
  the arithmetic agrees at 87 against 60.
- `pahole` beats `bpftool` because its `-C` output carries the compared members
  while `bpftool`'s raw dump needs a type-id resolver;
  the arithmetic agrees at 60 against 56,
  and a C4 step alone ties that pair while a C1 step reverses it.
- The in-house reader is not ranked:
  the project's answer left it ineligible.

## Confidence and evidence limits

Stated beside the rows they qualify,
gathered here so nothing is buried:

- `aarch64` is unrun for every finalist.
  H3's second target is source evidence for both libraries,
  and the two tools are `x86_64` binaries here with no `aarch64` environment to run in.
  A build or a run on that target is the check that would close it.
- The start cost is unmeasured,
  so C7 rests on a low-confidence range and the recommendation does not claim a startup budget.
  Measurement item 10 is the run that would settle it,
  together with the disk-cache thresholds.
- The weights are the design writer's because the project stated no priority;
  the equal-weight order is published above and agrees.
- The baseline is provisional in the skill's sense:
  three of the four rows are validated only at the depth their category allows,
  and the fourth is priced code that does not exist.
- The tools are validated at their output boundary only;
  their own upstream suites were not run and their release-version matrix is unrun (owed measurements 13 and 14).
- `ebeepf`'s provenance is its weakest point:
  one release,
  one maintainer,
  no CI,
  a README that declares the tree unreviewed,
  and a split base accepted as a base.
  Its 87 is a rating on that record,
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
- The class 3 web class is provider-ranked and not provably exhaustive,
  though each query ended below the provider's own cap and none sent a filter.
- The host kernel `7.3.5-1-cachyos` is not the product's qualified release until step 23,
  so the passing runs qualify the readers on this host's pair,
  not the product on every kernel.
- The 32-byte BTF header shape is what these files carry;
  the 24-byte shape a 6.8 guest's tools emit is owed measurement item 6 and no run here covers it.
- This is not an independent review under rule `IRW`:
  the same session wrote the report and checked it.
  A cross-vendor review,
  or the project's own reading,
  is what would make it independent.

## Recommendation

The skill allows a recommendation when the report is complete and the fully sorted order is stable.
Three of its four grounds hold plainly:
discovery is saturated with four survivors rather than the two the skill's minimum asks;
every surviving finalist got the equal-depth validation its category allows;
and criteria,
weights and ratings were frozen before candidate-specific evidence was rated.

The fourth is the sensitivity rule,
which says to rank by the provisional baseline only when every defined one-at-a-time test preserves order.
Literally,
it does not:
one reversal and four ties showed up on the live pair.
The skill's own remedy for that is to gather decisive evidence or ask only the controlling preference,
then to refreeze and rerun.
The evidence was gathered and it decides both:
the reversal rests on a rating the crate's own record refuses,
and the four ties are steps on the maintenance criterion,
which is exactly the criterion the pair separates on.
No rating moved,
so a refreeze would reproduce the same numbers;
what the evidence changes is whether the baseline's order is trustworthy,
and on this pair it is.

So this report recommends,
and it recommends `btf-rs` 2.0.1:

- it is the highest-scoring eligible candidate at 90 of 104,
  and the same order holds at the skill's equal-weight default at 23.5 against 22.5;
- it is the only eligible library whose upstream record shows releases,
  a merged security-relevant fix and CI on every push,
  and the pair's separation from `ebeepf` is that record rather than arithmetic;
- its own CI path,
  its suite,
  and the consumer boundary on this host's pair all ran and passed,
  with four negative controls refused;
- it needs no program the product must find and version,
  and starts nothing in the root handler,
  which is the cost the two tool routes carry.

Beside the recommendation,
in the same breath:

- `aarch64` is unrun.
  H3's second target is source evidence for `btf-rs`,
  and a build and a run there is the one platform check still owed before the reader is put in the release path.
- The start cost is unmeasured.
  C7 is a structural reading with a low-confidence range whose endpoints change no order;
  measurement item 10 would turn it into a number.
- The order of the two libraries rests on maintenance and provenance evidence,
  not on the arithmetic margin of three points.
- The recommendation is not adoption.
  No crate is added to any manifest,
  and the decision record's Q1 section still says the pick is the project's.

The stricter reading,
recorded so the project can take it without redoing the work:
if the skill's sensitivity sentence is read as admitting no evidence resolution at all,
the matching terminal result is "sensitivity unresolved",
and the report's output would be conditional rankings with no recommendation,
the live pair's condition being `ebeepf`'s C1 and C5 ratings.
The recommendation above is made on the first reading,
and the project may decline it for the second.

## What this report did not do

- It adopted no candidate,
  added no crate to any manifest,
  edited no decision record,
  and wrote nothing under `doc/handover/`.
- It ran no third-party code:
  no fetch,
  build,
  install or test of a candidate.
  Every execution record above is a kept log of a round run earlier in this worktree.
- It ran no `aarch64` build or run,
  and took no timing.
- It did not re-run the finalist suites;
  it read their logs and the design's records of them.
- It did not re-run the three class 3 queries or the organization lists;
  it corroborated only the crates.io count.
- It is not an independent review under rule `IRW`.
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
  this report's own fingerprint script,
  registry response and lock record.

External,
all read 2026-11-20:

- crates.io [crate pages][crates-io],
  the sparse index for `btf-rs`,
  and the `q=btf`,
  `q=split btf`,
  `q=btf kernel` and `q=libbpf` search endpoints.
- [docs.rs][docs-rs] for both library crates.
- [bpftool documentation][bpftool-docs] for `btf dump` and `--base-btf`,
  and the [pahole repository][pahole-repo] for `--btf_base` and `-C`.
- [Linux kernel BTF documentation][kernel-btf] for the header and the split-file format,
  at tags `v6.8` and `v7.3` as the design records.
- the repository host's [API and project pages][github] for releases,
  organization lists and boards.

[skill]: ../../.agents/skills/choosing-technology/SKILL.md
[record]: ../decision/cli-git-zfs-runtime-layout-checker.md
[design]: ../planning/cli-git-zfs-runtime-layout-checker-design.md
[design-discovery]: ../planning/cli-git-zfs-runtime-layout-checker-design.md#discovery-record
[design-check-in]: ../planning/cli-git-zfs-runtime-layout-checker-design.md#the-pre-discovery-check-in-of-2026-11-20
[crates-io]: https://crates.io/crates/btf-rs
[docs-rs]: https://docs.rs/btf-rs
[bpftool-docs]: https://docs.kernel.org/bpf/bpftool-btf.html
[pahole-repo]: https://github.com/acmel/dwarves
[kernel-btf]: https://docs.kernel.org/bpf/btf.html
[github]: https://github.com
