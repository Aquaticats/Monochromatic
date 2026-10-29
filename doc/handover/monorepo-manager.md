# Monorepo manager handover

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/desktop-app/file-manager-qt` is now `package/file-manager/variant-qt`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/dev-script/file-enforcer` is now `package/file-enforcer/core`;
> `package/module/jsonc-edit` is now `package/jsonc-edit/core-ts`.

## Status

Design research;
no product code.
Current discussion:
[software model exploration](../planning/monorepo-manager-software-model.md).
The user rejected Claude's task-centric framing,
then clarified a coherent continuous-maintenance model across build,
correctness,
and publication.
The agent followed a publication-policy detour;
the user explicitly returned the discussion to the model of non-tasks.
Publication is deferred entirely to 1.x,
with push to `main` recorded for that future subsystem.
The user accepted resource-aware package maintenance and requested continued grilling.
Expected session result:
a plan concrete enough to implement 0.x,
not product code.
Current work:
resolve the concrete contracts in the implementation plan's decision frontier.
The user accepted a from-scratch,
single-file,
all-Rust tool with file-enforcer rewritten in Rust on 2026-09-16
([`doc/decision/monorepo-manager-all-rust.md`](../decision/monorepo-manager-all-rust.md)).
Probe installs are authorized except `rpm-ostree install`.
The initial scope was meow 0.x only;
1.x and later may revisit those decisions (user,
2026-09-17).
The user later explicitly deferred the publishing subsystem to 1.x.
That exception does not move the rest of the design out of 0.x.
Research prompts and new decision records carry the applicable scope.

Keep this handover current when requirements,
decisions,
evidence,
rejected designs,
in-flight work,
or the next action change.

## Where things live

- Requirements,
   design,
   decisions,
   risks,
   and open questions for the tool:
   [`doc/planning/monorepo-manager-from-scratch-design.md`](../planning/monorepo-manager-from-scratch-design.md).
- Hard requirements for any manager and the Mise removal ledger:
   [`doc/planning/mise-removal-coverage.md`](../planning/mise-removal-coverage.md).
- Market vet with no recommendable candidate:
   [`doc/audit/tech-monorepo-manager-vet-2026-09-16.md`](../audit/tech-monorepo-manager-vet-2026-09-16.md).
- Bazel versus from-scratch comparison:
   [`doc/planning/monorepo-manager-build-routes.md`](../planning/monorepo-manager-build-routes.md).
- Bazel route design,
   not viable:
   [`doc/planning/monorepo-manager-bazel-route-design.md`](../planning/monorepo-manager-bazel-route-design.md).
- Original Bazel developer-experience question:
   [`doc/research/bazel-migration-dx.md`](../research/bazel-migration-dx.md).
- Research appendices with probes and citations:
   [`doc/planning/monorepo-manager-route-research/`](../planning/monorepo-manager-route-research/).

## How the choice narrowed

Each step's evidence is in the linked documents.

1.  Mise is to be replaced because its documentation is insufficient,
    with no ETA.
2.  The market vet screened 732 entries;
    none passed,
    including after the user counted external wrappers as plugging in.
3.  The Bazel route hit disqualifying problems in design;
    the from-scratch route was recommended.
4.  Stack deep dives covered TypeScript on Node,
    Bun,
    and Deno,
    Rust core with TypeScript children,
    all Rust,
    the Kotlin shapes,
    Python,
    OCaml,
    and .NET.
    Go and Zig were excluded by the user.
5.  The user judged the Kotlin shapes,
    Bun,
    Deno,
    Python,
    OCaml,
    and .NET not worth further research.
6.  The user made single-file `./meow` shipping a hard requirement,
    then killed the TypeScript route because Node single executables are too big;
    a minimal probe measured 144 MiB.
    The Rust core with TypeScript children is out by the same reason,
    closed as settled because it needs Node at run time.

## In-flight work

- Finished on 2026-09-16:
   the all-Rust research,
   copied to `doc/planning/monorepo-manager-route-research/stack-all-rust-rewrite.md`
   and merged into "All-Rust tool" in the from-scratch design.
  Its configuration-hosting ranking is withdrawn:
   the user dropped the Turing-complete configuration requirement
   ("Configuration" and "Correction on 2026-09-16" in the design).
- Finished on 2026-09-17:
   declarative configuration research,
   copied to `doc/planning/monorepo-manager-route-research/stack-declarative-config.md`
   and merged into "Declarative configuration" in the design.
  It exposed a recording error:
   the docs said the configuration is "not Turing-complete",
   while the user only removed the requirement;
   corrected in `6a8ea0651`.
- Done on 2026-09-16 with the user's authorization:
   `rustup target add x86_64-unknown-linux-musl`;
   static musl daemon skeletons with XXH3 and with `gxhash` both built and ran.
- Accepted 2026-09-16:
   [`doc/decision/monorepo-manager-all-rust.md`](../decision/monorepo-manager-all-rust.md).
- Finished on 2026-09-17:
   the choosing-technology vet for meow's cache key hash,
   [`doc/audit/tech-meow-cache-key-hash-vet-2026-09-17.md`](../audit/tech-meow-cache-key-hash-vet-2026-09-17.md),
   merged into "Cache key hash vet result" in the design.
  It recommends `twox-hash` 2.1.4 XXH3-128 under the SIMD reading of "hardware acceleration";
   under the dedicated-instruction reading no non-cryptographic library passes.
  The user answered "SIMD counts",
   and `twox-hash` XXH3-128 was recorded in `doc/decision/monorepo-manager-cache-key-hash.md`
   before the user saw the brief.
  Issue #545 got a comment pointing at that result,
   since its body named this vet as the blocker.
- Withdrawn and reopened on 2026-09-17,
   after the briefs:
   that decision record was deleted (`git log` keeps it),
   because the user said "non-crypto isn't a requirement.
   I only said we don't necessarily need a crypto hash."
   and "Shipping x86-64-v3 and x86-64-v4 as separate builds is fine."
  Both change the vet's premises
   ("Cache key hash vet result" in the design).
  Before re-running the selection,
   the user was asked how much collision resistance weighs against throughput ("A",
   speed first)
   and which x86-64 levels block publishing ("v4 block only").
  The re-run vet started the same day.
  Also answered:
   "Yes to HCL" (recorded in the all-Rust decision record)
   and "Merge BFQ into DRR instead" (rule `DRR` in `AGENTS.md`).
  Issue #545 got its follow-up comment on 2026-09-17,
   superseding the two earlier ones,
   with the fingerprint-sized numbers that bear on the music player's own choice.
  The vet agent reported that the Write tool refused its Markdown drafts,
   so it wrote them through Bash,
   and that some of its Bash commands broke rule `1CB`.
- Stopped unfinished on 2026-09-17,
   with no files written:
   troubleshooting docs for `gxhash` defects,
   no longer useful once meow and the music player leave `gxhash`;
   the evidence stays in `gxhash-owned.md` and issue #545.
- Finished and merged on 2026-09-17:
   `probe-platforms.md` ("Platform probes"),
   `rust-structured-edits.md` ("Managed file editing"),
   and `gxhash-owned.md` ("Repository-owned gxhash").
- Probe artifacts removed on 2026-09-17:
   the QEMU source volume,
   six probe images,
   the Rust 1.84.0 toolchain,
   the custom-target sysroot symlink,
   finished probes' Cargo build directories in the scratchpad,
   and `~/temp/agent/structured-edits-target-2026-09-16/`.
  The aarch64 Rust targets stay.
- Opened issues #546 and #547 on 2026-09-17 for the user to file the prepared `xot` issue
   and post the prepared `kornelski/xml-rs#88` comment personally
   (drafts in `doc/troubleshooting/xml-attribute-whitespace-serialization.md`).
- Opened issue #545 on 2026-09-17,
   "music player must switch away from gxhash";
   the user said this session must not touch music player work.
- `ssh m1` was powered on by the user on 2026-09-17 for the hash measurement:
   Darwin arm64,
   16 GiB,
   8 cores,
   macOS 27.0,
   114 GiB free on the internal disk and 216 GiB on `/Volumes/MacData`,
   with a Rust toolchain already in `/Users/user/.cargo/bin`.
  Rule `HRM` still applies:
   write-heavy work goes on `/Volumes/MacData`.
- Opened issues #550,
   #551,
   and #552 on 2026-09-17 for the user to file the two `hcl-edit` defects
   and the `hcl-rs` evaluator divergences personally.
- Stopped unfinished on 2026-09-17:
   `btrfs-pinned-bytes.md` research,
   because the user dropped reflinks and the cache now records output pointers only.
- Also decided 2026-09-16 and recorded in the design:
   no pty opt-in,
   `gxhash128` keys,
   cache contents and failure caching,
   pointer-only outputs,
   eviction by a size cap plus 30-day age,
   flaky retry reporting,
   and rule `QGR` in `AGENTS.md`.
- Decided 2026-09-17 and recorded in "Declarative configuration" in the design:
   the configuration may be Turing-complete,
   HCL syntax is lightly endorsed,
   rules writing outside a repository live in a per-user configuration,
   the forbidden-strings rules compile with the published scanner,
   meow never generates `mise.toml` while file-enforcer keeps generating it for now,
   and meow is built in its own git worktree.
- Also decided 2026-09-17:
   meow replaces Mise only once it supports the full platform matrix,
   0.x guarantees only Linux,
   meow does not own Meta Package Manager's lifecycle,
   aarch64 musl ships static-pie from a custom target,
   local glibc builds use the host while releases follow `cargo-publish.yml`,
   new TOML inline tables use the TOML 1.1 trailing comma,
   and edits need not keep untouched bytes or CRLF.
- Finished under the `troubleshooting-doc` skill on 2026-09-17,
   committed and indexed under "Comment-preserving structured edits in Rust" in `doc/troubleshooting/README.md`:
   `toml-edit-comment-loss.md`,
   `taplo-toml-1-1-inline-table-trailing-comma.md`,
   `xml-attribute-whitespace-serialization.md`,
   `json-five-unterminated-block-comment.md`,
   `jsonc-parser-json5-defaults.md`,
   and `biome-json-crates-exact-pins.md`.
  Source citations for the `json-five`,
   `jsonc-parser`,
   and Biome policy findings were re-read in the agent's clones.
- Opened issues #548 and #549 on 2026-09-17,
   following the user's reminder-issue answers for #546 and #547,
   for the user to file the prepared `json-five` block comment issue
   and the `jsonc-parser` `allow_comments` issue personally.
  No Biome filing:
   biomejs/biome#5151 already covers the exact pins,
   and Biome's `CONTRIBUTING.md` asks that contributor communication not be AI-written.
  The `jsonc-parser` loose defaults are documented upstream behavior and stay unfiled.
- Installs are authorized for this work,
   except `rpm-ostree install`.
- Stopped unfinished on 2026-09-16:
   the design of a TypeScript daemon with a Rust native addon;
   no appendix was written.

## Process notes

- Research agents contacting registries or documentation sites send a generic User-Agent with no personal identifiers.
  On 2026-09-16 one crates.io request carried the user's email address as a contact;
   the user was told,
   and later prompts required a generic User-Agent.
- Probe paths cited as "in the session scratchpad" are not durable;
   the recorded commands and outputs are the evidence.
- Runtime and library documentation problems are recorded but do not cull options for building the tool;
   monorepo manager candidates were culled on any documentation confusion.
- Incumbent shapes are not requirements:
   on 2026-09-16 the configuration-hosting research treated `file-enforcer.config.ts`'s Turing-complete shape,
   built under time constraints,
   as something to preserve,
   and the user corrected it.

- Corrected by the user on 2026-09-17:
   "I haven't been presented with the brief on how you recommend XXH3-128 from twox-hash,
   nor HCL."
  The `twox-hash` adoption followed only a short summary above the reading question,
   treating the "SIMD counts" answer as acceptance,
   and the HCL evaluator research started without an HCL brief.
  Both briefs were then presented,
   with questions on keeping the `twox-hash` adoption,
   keeping HCL with its running research,
   and a proposed `AGENTS.md` rule `BFQ`.

## Commits

Milestones,
all 2026-09-16;
`git log --oneline -- doc/planning/monorepo-manager-from-scratch-design.md doc/planning/monorepo-manager-route-research`
lists the rest.

- `5f524b08d`:
   vet finished with no recommendable candidate.
- `647a2ccbb`:
   build routes proposed and ranked.
- `185aa6493`:
   Bazel and from-scratch route designs written.
- `4daaae8e9`:
   first ranking after designing every stack option.
- `cdf341565`:
   crate-level Rust research corrected the `unsafe` conclusion.
- `b4d6e1823`:
   leading stacks re-ranked after library-level research.
- `36754f0ff`:
   single-file shipping made a hard requirement.
- `d3af113d4`:
   TypeScript route killed over single-executable size.
- `bcf373479` and `4241732d5`:
   documents brought up to the all-Rust route.
- `652c171c0`:
   all-Rust configuration-hosting research merged.
- `2ae2c67b5`:
   declarative configuration,
   `gxhash`,
   and the CPU-capability warning recorded.
- `1e55e3335` (2026-09-17):
   JSON crate troubleshooting docs landed.
- `4fdf58b84` (2026-09-17):
   cache key hash vet merged into the design.
- `d96af77a6`,
   `b1736b8bb`,
   and `1e2f55e2d` (2026-09-17):
   the pending question set,
   the plan's command surface and language blockers,
   and the Mise ledger's task-discovery owner.
- `84100d255` and `d237f3ce0` (2026-09-17):
   the final doc sweep.
  The Bazel route design is marked not taken,
   `doc/research/bazel-migration-dx.md` now flags the mid-flight glob abort as unfounded (issue #555),
   and the from-scratch design's status no longer calls itself a draft with the configuration design open.

## Research results on 2026-09-17

Finished on 2026-09-17:
 design research for the HCL evaluator,
 function library,
 formatter,
 and language server,
 copied to `doc/planning/monorepo-manager-route-research/hcl-tooling.md`
 and merged into "HCL tooling" in the design,
 with vet reports `tech-meow-hcl-front-end-vet-2026-09-17.md`
 and `tech-meow-language-server-framework-vet-2026-09-17.md`
 and three troubleshooting docs committed.
It recommends a patched `hcl-edit` under meow's own evaluator,
 a curated OpenTofu-named function set,
 meow's own formatter,
 and `lsp-server`.
Its brief went to the user with five questions.
Answered on 2026-09-17:
 patched `hcl-edit` under meow's own evaluator,
 and functions with unpredictable results allowed but making their evaluation uncacheable with detailed warnings,
 both recorded in
 [`doc/decision/monorepo-manager-hcl-front-end.md`](../decision/monorepo-manager-hcl-front-end.md);
 all three upstream defects get reminder issues.
Answered the same day:
 the `meow::` namespace for meow-only functions,
 "Ship no formatter",
 `lsp-server` for the language server,
 and "Emit every line as a json,
 error or warning or not.",
 which dissolved the diagnostic-renderer question and added "Output format" to the design.
Nothing from the HCL research is left open.
Corrected before merging:
 the research called a C toolchain forbidden by the all-Rust decision record,
 which only says the tool is written in Rust;
 whether a dependency may compile C is an open user question,
 also raised by the hash vet.
The re-run cache key hash vet finished on 2026-09-17:
 [`tech-meow-cache-key-hash-vet-2026-09-17-600031ed.md`](../audit/tech-meow-cache-key-hash-vet-2026-09-17-600031ed.md),
 merged into "Cache key hash re-run vet result" in the design,
 with `doc/troubleshooting/rust-hash-crate-build-and-test-quirks.md` committed.
It recommends `twox-hash` 2.1.4 XXH3-128 again,
 now over a new finalist `rscrypto` by 1.5 points,
 with cryptographic candidates 8 to 15 times slower on the weight-5 criteria.
Its one user question was whether to power on the m1 to decide `twox-hash` against `rscrypto` by measurement.
The user answered on 2026-09-17:
 "I just powered on my m1 mac.
 Please measure.",
 and the measurement ran there the same day
 (`c4f5f0d3d` and `95143e348` update the report in place).
`twox-hash` stays first at 85 of 92,
 `rscrypto` second at 83.5,
 while `hashcrew` and `xxhash-rust` swapped third and fourth;
 no sensitivity test changes the winner any more.
The remaining proxy is Darwin against Linux aarch64,
 which a rented Arm Linux run would close;
 the user declined that and answered "Accept twox-hash",
 recorded in
 [`doc/decision/monorepo-manager-cache-key-hash.md`](../decision/monorepo-manager-cache-key-hash.md).
m1 state after the run,
 verified from this session:
 only the machine's own toolchain is installed
 and no probe processes are left;
 the agent's scratch tree stays on `/Volumes/MacData`,
 and three macOS probe quirks are documented.
Spot-checked before merging:
 `rscrypto` was published 2026-05-02 with about 1,100 recent downloads (crates.io API,
 2026-09-17).
The C toolchain question the HCL research raised is closed by this vet's measured bound,
 so it was withdrawn from the user's question set.

Finished on 2026-09-17:
 design research for the per-user `meow` configuration,
 copied to `doc/planning/monorepo-manager-route-research/per-user-config.md`
 and merged into "Per-user configuration" in the design.
It recommends an XDG path with an explicit override,
 per-attribute declared scope for layering,
 repository proposals accepted by the per-user file for outside writes,
 a restricted mode until a repository root is trusted,
 two-level cache keys,
 and one daemon per repository root.
Its brief went to the user with four questions,
 all answered on 2026-09-17 and recorded in
 [`doc/decision/monorepo-manager-per-user-config.md`](../decision/monorepo-manager-per-user-config.md):
 repository proposals accepted by the per-user file,
 trust "Go with the logic in our cli-git",
 per-user tasks in a separate non-shadowing kind,
 and a switch that defaults to reading the file everywhere.
The trust answer replaced the research's restricted mode:
 `doc/decision/cli-git-policies-platform.md` and
 `package/git-policy/cli/src/allowed-worktree-dirs.ts` were read,
 so meow gets explicit `trust`,
 `untrust`,
 and `status` subcommands,
 exact-byte snapshots rather than a content hash,
 a registry under the account home rather than an XDG path,
 and blocking of every configuration-loading command until trust exists.
The research used no clones and wrote no troubleshooting docs;
 it reported three rule `1CB` slips,
 and it noted that `git config --list --show-origin` printed a credential URL that it did not reproduce anywhere.

Finished on 2026-09-17:
 design research for how `vm-builder` replaces its `exec` import from file-enforcer's `/ts` subpath,
 copied to `doc/planning/monorepo-manager-route-research/vm-builder-exec.md`
 and merged into "vm-builder migration" in the design.
The user answered all five of its questions the same day:
 no shared process owner,
 the replacement lands with the rewrite,
 no speed gate so the performance fixture retires unmeasured,
 a new fixture package for the `prefer-readonly-parameter-type` read,
 and "Correct the record" for the root `mise.toml`,
 which stays generated until Mise is removed.
That research also found two defects worth separate work:
 `package/dev-script/file-enforcer/README.md:134-197` documents `exec` forms the implementation rejects
 and `README.md:290` imports an unexported subpath,
 and `workspace-source-effect.unit.test.ts:67-70` finds functions by searching source text,
 so an unrelated rename breaks it silently today.

Both defects were filed as issue #554 on 2026-09-17,
 with the exact locations and the correct `exec` shapes,
 rather than fixed here:
 the README fix is mechanical,
 but the unexported subpath needs an API choice,
 and this session's scope is meow's design.

At that point the research queue was closed.
The subsequent UX discussion reopened the software model;
see "Software-model correction" for the current frontier.

## UX alignment on 2026-09-17

Done on 2026-09-17,
answering "Fix/update all docs first then write the implementation plan":
 the route comparison,
 the Mise removal ledger,
 the market vet,
 the superseded 2026-06-02 audit,
 the `gxhash` troubleshooting doc,
 and two research appendices now state the accepted decisions,
 the file-enforcer README documents the `exec` and package APIs that exist,
 and
 [`doc/planning/monorepo-manager-implementation-plan.md`](../planning/monorepo-manager-implementation-plan.md)
 lays out milestones M1 to M9 with the evidence each must produce.

Asked on 2026-09-17 whether to start M1,
 the user stopped the question:
 "the existence of this question means we are not clear/aligned on what meow's UI/UX is yet".
Four proposed output models were all rejected,
 and the user stated the model instead,
 now recorded as "User interface" in the design:
 `meow watch` runs in a second terminal as the JSON stream,
 kept open but minimized and not meant to be read;
 `meow run <target>` runs in the working terminal,
 queues at interactive priority,
 holds the terminal,
 forwards the task's bytes as-is on a miss,
 prints the stored result on a hit,
 and prompts for a `meow watch` terminal when no daemon is live.
The rejected comparison page is at `meow-ux/meow-ux-options.html` in the session scratchpad.
Answered the same day,
 all recorded in "User interface",
 "RPC",
 "Scheduler",
 and "Modularity":
 `meow status` for inspection;
 `meow pause`,
 `meow resume`,
 `meow end`,
 and `meow priority` as commands now,
 because "We don't need specifically a socket or a separate client,
 ever";
 the socket demoted to private plumbing;
 the TUI redefined as "just `meow watch` displayed differently while adding some interactivity";
 queue position shown while `meow run` waits;
 nothing added on failure beyond the task's exit code;
 Ctrl+C ending the task;
 `meow run` attaching to work already running;
 unbuilt dependencies ruled out because the daemon keeps everything current;
 and every modular piece,
 the logger included,
 becoming a reusable package.
Then answered too:
 `meow status` shows failed,
 paused,
 blocked,
 flaky,
 queued and running work but never successes,
 with its layout left for the TUI;
 `meow doctor` prints problems only;
 the trust review shows every file that would be evaluated,
 unpaged,
 AUR-style;
 `meow stop` joins Ctrl+C and tasks never outlive the daemon;
 `meow run` takes one target or a shell-style glob,
 warns about quoting,
 prefixes each line with its target on a multi-match run,
 and exits with the first failure's code.
Configuration authoring,
 answered the same day:
 one root `meow.hcl` with tags per package and inheritance,
 tags inferred then adjusted by glob-selecting `package` blocks,
 one-label `task "test"` blocks,
 argv commands with no shell,
 most-specific-tag-wins specialization,
 and mandatory `overridden` and `override` meta-arguments.
- Finished on 2026-09-17:
   the Mise task usage study the user asked for,
   copied to `doc/planning/monorepo-manager-route-research/mise-task-usage.md`
   and merged into "What using Mise for tasks is actually like" in the design.
  It confirms tag selection (2,632 lines of bare `extends` removed),
   the override markers (96 silent shadows),
   argv commands,
   and a real expression language (63.1% of tasks interpolate `{{vars.`).
  It also costs the one-root-file decision:
   about 7,000 lines in one file,
   no answer for what narrows a listing when standing in a directory,
   and inline logic with no owner.
  Three defects it found are filed as issue #555,
   each re-verified here:
   root `mise run test` skips 15 packages,
   three stale `mise watch ... -- node ...` tasks remain,
   and `mise.toml:308-312` cites evidence that `doc/handover/lint-fix-2026-06.md` does not contain.
  Of its four questions,
   three are answered:
   the working directory scopes every command ("Infer everywhere"),
   the inline logic becomes meow built-ins,
   and an agent in a disposable worktree uses `meow trust --yes`.
- Retracted on 2026-09-17:
   "Ordering comes from declared reads,
   writes,
   and explicit `depends_on`",
   which this session had recorded under "Settled without asking" and the user never approved:
   "I never approved 'depends' or 'depends on'.
   Let's align on that.
   Grill me."
  Task ordering is open again.
  Answered while grilling:
   cross-package edges come from the native manifests,
   and freshness comes from hashing a task's declared reads.
- Session interrupted on 2026-09-17 for a Claude Code update,
   with one question set drafted but unasked.
  The user's framing:
   `build` and `test` should be built into meow rather than mixed in with lesser tasks,
   while still being reached through `meow run //package/cow:test` rather than dedicated commands,
   because "some tasks are more than others".
  The drafted options are now withdrawn:
   they assumed the special-task model the user subsequently challenged.

## Software-model correction

The user resumed with an explicit instruction not to accept Claude's framing of a task:
stages of software development need not be treated as build tasks and similar commands.

- Retracted:
  the inference that build and test are privileged task names,
  and the built-in-name,
  override,
  and namespace questionnaire derived from it.
- Proposal only:
  [software model exploration](../planning/monorepo-manager-software-model.md)
  distinguishes products,
  evidence,
  readiness for a use,
  and executions.
  It recommends software state as the primary experience over development stages,
  without forbidding a stage view or changing accepted CLI behavior.
- Direct evidence:
  `package/module/jsonc-edit/package.json` exports source and built bundles;
  its `src/parse.unit.test.ts` consumes the neutral bundle;
  `AGENTS.md` rule `ST3` directs workspace consumers to source.
  Consumer requirements therefore need more precision than a package-wide built label.
- Independent review:
  Advisor emphasized separate product freshness and production outcome,
  distinct unavailable versus failing evidence,
  per-record input provenance,
  and imperative operations outside the product/evidence model.
- Preserved:
  automatic affected builds and default tests,
  failure caching,
  native-manifest relationships,
  content hashing,
  the working-terminal contract,
  and attention-only status.
  No new auto-publish,
  retry-until-green,
  or dependency mechanism is proposed.
- Process-rule proposal:
  extend `AGENTS.md` rule `QPM` to question inherited nouns as well as mechanisms.
  Exact proposed wording is in the exploration;
  `AGENTS.md` is not changed.
- Precedent research:
  source-verified OpenTofu declarative goals and Kubernetes conditions and phase summaries
  are cited in the exploration.
  These are conceptual precedents,
  not technologies being adopted or runtime-tested guarantees.
  The research agent's scratch-file write was denied;
  its inline findings were checked against the cited primary sources before inclusion.
- Commits:
  `ab3455894` records the initial proposal;
  `bddf3a3df` withdraws the questionnaire and updates plan blockers;
  `4d4b81f48` fixes the proposal's semantic line break;
  `bd32a02cd` records the primary-source precedents.
- Verification:
  `mise run lint:markdown --` with the exploration,
  design,
  implementation plan,
  and this handover passed after the line-break fix and source additions.
  `git diff --check` also passed.
  No product code changed.

## Continuous-maintenance clarification

The user found the state-versus-stages question difficult to follow,
said "I would say A",
and emphasized the coherent model of "auto keeping everything up to date,
build correctness publish everything".
The tentative A answer is not blanket acceptance of the proposed ontology.
The artificial choice is withdrawn as the driving question.

Working interpretation:
meow intrinsically maintains building,
correctness checks,
and publication-related work rather than presenting privileged generic tasks.
Independent review recommended clarifying publication authority before asking about trigger timing.
The user then answered:
"Publication is automatic too.
On version bump + target registry already has it".
The user confirmed that the registry condition refers to an existing package:
automatically publish its new version,
do nothing if that version is already there,
and do not automatically publish a never-published package.
This is recorded in "Automatic publication" in the from-scratch design.
The agent then asked about the source and trigger boundary,
citing `.github/workflows/cargo-publish.yml` as incumbent evidence.
The user corrected the scope:
"Do not get off-tracked.
The publishing subsystem won't exist until 1.x.
We're still talking about the model of 'non-tasks'.
Answer:
push to main."

Publication is absent from 0.x,
and push to `main` is recorded for 1.x only.
No publication question blocks the current design.
Independent review of the correction recommended a concrete explanation before more menus:
meow maintains products,
current check results,
and managed-file requirements;
executions implement those responsibilities.
The rewritten exploration preserves the source evidence and prior corrections without treating its glossary as adopted.
A proposed `VR2` amendment records that mentioning a feature does not authorize a subsystem-design detour;
`AGENTS.md` remains unchanged.

## Linting clarification

The user found the package-maintenance explanation almost right and asked where linting fits.
The exploration now explicitly groups linting,
type checking,
and tests as intrinsic maintained checks.
The previous word correctness hid repository policy and style checks inside an unspecified umbrella.
Current build outputs and current lint findings can coexist;
linting is not a lesser arbitrary operation.

Lint checking maintains findings;
lint autofixing changes source.
Including automatic lint checks does not settle automatic source-rewrite policy.
The incumbent distinction is visible in `mise.toml:590-612`:
`lint:oxlint`,
`format:oxlint` with `--fix`,
and `lint:types` are separate definitions.
A proposed amendment to `RCO` extends its responsibility-coverage rule to model redesign;
`AGENTS.md` remains unchanged.

## Resource-saving clarification

The user asked whether a package should proceed to building when linting fails.
The accepted answer is no for automatic downstream work blocked by an applicable source check.
The model describes intrinsic execution eligibility,
not merely independently refreshed results.
The user accepted it with "Okay,
I'm satisfied with what we have here",
then asked to continue grilling until the plan is concrete enough to implement 0.x.

Checks may require generated or built prerequisites.
The current `ensureOxlintConfig` in `mise.toml:350-420` builds linter configuration before `lint:oxlint`;
a global all-lint-before-all-build rule would therefore be insufficient.
The distinction is necessary prerequisite production versus work blocked by the check's result.
No relative speed claim was made,
and no manual bypass was chosen.
Independent review confirmed those limits and the tradeoff:
gating skips downstream work after failure but delays downstream diagnostics.

## First implementation-plan interview

User answers:
Q1 A,
Q2 A,
Q3 A,
Q4 A,
Q6 A.
For Q5 the user rejected both end-lifetime options:
"Let's forbid `meow end` for automatic work for 0.x."

Accepted:
finish independent source checks after a failure;
foreground requests honor gates;
cancel superseded ordinary background work;
foreground maintenance follows current inputs;
autofixing ordinary source requires explicit intent;
`meow end` cannot end automatic work.
"Maintenance and foreground lifecycle" in the design is the authoritative record.

The read-only audits returned:

- `meow-schema-frontier` reached its turn limit and returned a partial investigation.
  Important evidence pointers:
  nested Gradle/Cargo identities,
  cross-ecosystem output relationships absent from native manifests,
  HCL selector conflicts,
  managed metadata changing discovery,
  and explicit operations whose effects cannot be replaced by cached logs.
- `meow-runtime-frontier` returned runtime contract gaps:
  distinct argument vectors under one label,
  trust revocation,
  environment ownership,
  log-storage failure,
  output validity,
  working-terminal output contradictions,
  and the stale shipped-build count.
- Findings about unaccepted progression were already stale when the reports arrived.
  Q3 and Q4 resolve their input-supersession questions;
  the user's Q5 overrides their suggested end-lifetime policy.
- Verified locally before use:
  `package/desktop-app/file-manager-qt/mise.toml:40-53` builds and launches a GUI,
  illustrating why repeated explicit intent cannot be defined only as output replay.
  The plan's shipped-build enumeration now includes the accepted x86-64-v2 variant and drops the incorrect count.
  Other evidence pointers still need direct verification before their dependent questions.

## Second implementation-plan interview

- Q7 A:
  Ctrl+C detaches a foreground request from automatic maintenance;
  the work continues.
- Q8 A:
  a currently blocked foreground request reports the blocker and returns nonzero.
- Q9 A,
  then clarified after rejecting the proposed per-definition reuse toggle:
  "Every execution has a defined set of inputs and outputs.
  If something must always re-execute,
  they can add a input of currentDateTime to the set."
  Always-on execution caching remains intact.
  An intentionally varying input changes computation identity;
  there is no cache-off policy or blanket fresh-execution rule for explicit operations.
- Q10 rejected the proposed target-wide scheduling lane:
  "Follow priority and queue everything";
  invocations may run simultaneously or pause under the existing priority model.
  The earlier same-target-duplicate wording is not a mutex by label.

Independent correction review confirmed the distinction:
recording an attempt is separate from reusing its outcome.
Keep coalescing equivalent reusable computations,
not arbitrary invocations sharing a label.
Actual overlapping writes still need correctness protection.
The user's subsequent clarification rejected per-definition reuse configuration.
The normal cache key includes every resolved declared input,
including an intentionally varying value such as `currentDateTime`.
Input binding and volatile sampling need concrete engineering contracts,
not a cache bypass.

## Next action

1.  Q9 is settled;
    do not reopen cache-off choices or target-wide scheduling lanes.
    The native-identity example is now verified:
    Android's `settings.gradle.kts` declares `:app`,
    its `rust/Cargo.toml` declares a standalone crate,
    and its `mise.toml` separates Android Lint from Rust checks while naming the JNI output destination.
    Q11 A is accepted:
    package-wide concerns cover all applicable owned native parts;
    narrower addresses remain possible.
    `doc/planning/monorepo-manager-design-workpad.md` now contains an ownership sketch,
    the verified nested pseudo-package counterexample from `oxlint-test-import`,
    a revised discovery/membership/applicability distinction,
    and the next membership-default question.
    Q12 A is accepted:
    require the missing declaration rather than enroll a nested project by default.
    The user clarified "Both":
    the Android Gradle application and Rust engine should be separate packages,
    and the fixture cases should also be separate packages.
    Do not preserve those incumbent groupings or move source files during this design session.
    The workpad now separates ownership from cross-package consumption.
    Ask Q13 about propagation of a dependency's failed source gate to a source-consuming package's production.
    Keep dependency runtime-test gating separate from that question.
    The user's correction is about using writing to solve the problem,
    not merely preserving conclusions or session history.
    A `DCK` amendment is proposed in the workpad;
    `AGENTS.md` remains unchanged.
    Task #8 is active;
    #9 remains incomplete and is blocked on #8,
    rather than being marked finished.
2.  Verify remaining audit evidence before asking its dependent native-identity,
    authoring,
    trust,
    logging,
    and environment questions.
    Do not ask further publication questions or resume withdrawn model questionnaires.
3.  Implementation is unstarted and unrequested (rule `VRB`).
    The requested outcome is a concrete 0.x implementation plan,
    with package ownership,
    interfaces,
    state transitions,
    configuration examples,
    migration coverage,
    and acceptance tests.
    The implementation plan's completion contract is the stop condition.
4.  Repository follow-ups this design produced:
    issues #545 through #552,
    #554,
    and #555,
    all for the user to act on personally.
## Working rules this session followed

- Copy each research result into `doc/planning/monorepo-manager-route-research/`,
   verify its load-bearing claims,
   merge it into the from-scratch design,
   and commit.
- Ask every flagged question in the same turn,
   with pros,
   cons,
   and a ranking;
   the user asked on 2026-09-16 that flagged items never wait in documents.
- Record accepted choices in `doc/decision/` only after the user accepts them,
   and present a brief first (rule `DRR`).
- Lint every Markdown change with `mise run lint:markdown -- --fix <paths>` before committing.
