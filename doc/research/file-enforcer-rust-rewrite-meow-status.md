# meow and file-enforcer Rust rewrite: current state (read-only research, 2026-10-29)

Research for [`doc/planning/file-enforcer-rust-rewrite.md`](../planning/file-enforcer-rust-rewrite.md),
gathered 2026-10-29 by a read-only research agent;
file reads and source citations only,
nothing was executed.

All paths are repo-relative to /var/home/user/Monochromatic. "Fact" = read in a file; "Inference" = mine.

## 1. Is meow live, paused, abandoned, or superseded?

### Short answer

- Fact: meow is the accepted endpoint, and its file-enforcer slice is still assigned to it. No doc reassigns file
  enforcement to Pants or to anything else.
- Fact: meow work is paused. The last substantive design commit is 2026-09-21 (`e71ccfc80a`, `f0464c174e`,
  `628b458aa0`). Later commits touching those docs are mechanical (2026-09-29 code retirement pointers, 2026-10-18
  rewrap, 2026-10-29 package-category moves).
- Fact: the task-execution slice (not file enforcement) was carved out of meow and given to Pants as an interim owner
  on 2026-10-29. The carve is "temporary" per the vet.
- Inference: "paused, plan-only, endpoint still meow" is the best label. Not abandoned; not superseded for file
  enforcement.

### Evidence

- `doc/decision/monorepo-manager-all-rust.md:3-9`: "Accepted 2026-09-16 by the user, answering 'Do you accept the
  all-Rust stack for meow 0.x (Rust approved for this tool, file-enforcer rewritten in Rust)?' with 'Accept
  all-Rust'. `meow` is a placeholder name."
- `doc/handover/monorepo-manager.md:16-17`: "Design research; no product code."
- `doc/handover/monorepo-manager.md:792-841` ("Next action"): the last recorded state is mid-interview (Q13 about
  failed source gates propagating to consumers); "Implementation is unstarted and unrequested (rule `VRB`)."
- `doc/planning/monorepo-manager-implementation-plan.md:5-6`: "Plan only, initially written 2026-09-17 ... no code
  exists and none is authorized yet (rule `VRB`)." Milestones M1 to M9; M6 is "file enforcement parity"
  (lines 290-298); M9 is cutover (lines 380-398).
- `doc/planning/monorepo-manager-implementation-plan.md:419-421`: "M4 and M6 remain provisional where configuration
  schema and file-enforcement scheduling assume the unaccepted task model." So even the file-enforcement milestone
  is flagged provisional because "what is a task" is unsettled.
- `doc/planning/mise-removal-coverage.md:3-8`: "the replacement is chosen, its design is complete, and no code
  exists yet." Same file, lines 37-40: sections that call the TypeScript file-enforcer the canonical compiler
  "describe today's TypeScript implementation, which keeps running until meow replaces it." (The same file still
  contains older text at lines 140-160 and 907-915 proposing file-enforcer stays the TypeScript compiler; the
  status block at the top says that is superseded. Mixed record.)
- GitHub issue #1799 (open, 2026-10-29, filed by Aquaticat): "We would be very happy **not** to build an entire
  monorepo manager if a well-maintained, well-documented existing tool can meet the needs" ... "not an approval to
  migrate to a particular tool or to implement meow yet". Its filters include: cover "natively or by small
  documented extensions, the remaining functions of file-enforcer (including safe, comment-preserving
  TOML/JSONC/XML changes)". meow goal 5: "One integrated owner for managed-file enforcement and freshness: replace
  the existing file-enforcer machinery without losing protected destinations, dependency/read tracking, and
  comment-preserving structured edits."

### What "task-slice" is

- Fact, `doc/audit/tech-task-slice-owner-vet-2026-10-29.md:41-47`: "The task slice is one slice of the monorepo
  manager." Decision scope: "select the owner for Mise's task-execution slice: task discovery, per-package task
  addressing, dependency edges, task bodies, watch, failure tolerance, and machine-readable listing, at 222
  packages and 1,811 tasks."
- Outcome: eight candidates piloted 2026-10-29 (pnpm, Nadle, Poe, Buck2, Pants, projen, Please, Tilt)
  (`doc/planning/task-slice-comparison.md:1-30`). Pants 2.33.1 recommended
  (`doc/audit/tech-task-slice-owner-vet-2026-10-29.md`, "Recommendation"). meow "as designed: not scorable,
  plan-only" (same file, "meow as designed" bullet).
- User quote (`doc/handover/task-slice-adoption.md:104-105`, answer 2026-10-29): "2, and we're still going to
  migrate off pants to meow later when we have capacity." Option 2 = adopt owner but write migration plan first.
  Adoption acceptance is still pending in that doc ("Remaining gate before migration: explicit acceptance"); no
  `doc/decision/task-slice-owner-pants.md` exists (`ls doc/decision` found none).
- Fact, `doc/planning/task-slice-pants-migration.md`: proposal only; "No `doc/decision/` record yet, no task file
  changed."

### Who owns file enforcement now

- Fact, `doc/planning/task-slice-pants-migration.md:226` lists what Pants does not take: "file-enforcer and
  root-file generation; toolchain provisioning (`toolchain:sync`); package publishing, secrets ...; the `meow` build
  itself."
- Fact, same doc lines 252-262 and 430-434: the Pants BUILD generator is proposed to extend
  `file-enforcer.config.ts:691` `generateMiseToml` ("extending it is what `RCI` asks"). That means the current
  TypeScript file-enforcer gains a duty (emit Pants config) and is the live owner in the interim.
- Fact, `doc/handover/task-slice-plan-revision.md:36-38` repeats: the generator moves "to
  `file-enforcer.config.ts:691` `generateMiseToml`".
- Fact: no doc says "Rust file-enforcer as a standalone product". Every Rust-file-enforcer statement is "inside the
  meow binary" (`doc/decision/monorepo-manager-all-rust.md:50-52`: "file-enforcer's functionality is rewritten in
  Rust inside the same binary").
- Fact, `package/file-enforcer/core/DECISION.rust-migration.md:3-8`: "Superseded 2026-09-16 by
  `doc/decision/monorepo-manager-all-rust.md` ... the TypeScript implementation keeps running until the Rust tool
  replaces it."

### New file-enforcer duties after the meow design was frozen (inference, with facts)

- Fact: the meow design inventoried the root config at 2,329 lines and 27 logic units
  (`doc/planning/monorepo-manager-from-scratch-design.md:1554-1571`; `stack-declarative-config.md:286-296`). The
  root config is now 2,592 lines (`wc -l file-enforcer.config.ts`).
- Fact: `package/file-enforcer/core/src/toolchain/` (plan, providers, lock, exposure) did not exist in the inventory.
  `doc/decision/mpm-toolchain-adoption.md:203-205`: "`file-enforcer` owns the typed toolchain policy".
- Fact, `doc/decision/rust-dependency-catalog.md` (Accepted 2026-10-29, `:49-65`): "`file-enforcer` renders a
  generated `[workspace.dependencies]` table in each crate", adopts `cargo add` literals into `rust-catalog.toml`,
  has a check-only mode. This is the "Cargo.toml workspace deps" duty. Also `doc/planning/cargo-toml-file-enforcer.md`.
- Inference: a ground-up Rust rewrite now has to cover toolchain lock generation, the Rust dependency catalog sync,
  and a possible Pants BUILD generator, none of which the meow design's 27-unit inventory counted. The Rust
  rewrite would also render its own `Cargo.toml` from that catalog (self-hosting).
- Fact: the dirty working tree already has `?? .projenrc.ts` (a projen pilot leftover) and modified
  `package/file-enforcer/core/src/toolchain/plan.artifacts.data.ts` and `providers.parse.ts`. Concurrent work
  exists in file-enforcer's TypeScript (rule EC1).

## 2. Does any meow code exist?

- Fact: no. `ls package/` shows no `meow*`, `monorepo-manager*`, `task-slice*` or `pants*` package; the only
  monorepo-named dir is `package/monorepo-task` (`cli`, `core-exec`), the TypeScript `task-run` stopgap the user
  called "an extremely ugly hack" (`doc/planning/task-slice-pants-migration.md:15-20`).
- Fact: `rg --files --glob Cargo.toml package` lists 26 manifests; none is meow or file-enforcer. `git branch -a`
  and `git worktree list` show no meow branch or worktree (the design requires "its own git worktree",
  `doc/planning/monorepo-manager-implementation-plan.md:183-186`; none was created).
- Fact: `rg -w meow` outside `doc/` hits only unrelated words in translation-repair and oxlint test data, plus
  `package/oxlint-plugin/no-restricted-syntax` (low-information symbol-description fixture); none is the tool.
- Fact: probes only. The handover records skeleton probes in the session scratchpad (static musl daemon skeletons,
  2,098,120 bytes with XXH3) that are "not durable" (`doc/handover/monorepo-manager.md:256-257`). A differential
  edit harness is cited in M6 evidence (`...implementation-plan.md:296-298`) but no repo path exists for it.
- Existing Rust file-enforcer-adjacent building blocks that DO exist:
  - `package/jsonc-edit/core-rust` (crate `monochromatic-jsonc-edit` 0.1.1, std-only, no `unsafe`).
  - No Rust TOML or XML editor crate exists: `package/toml-edit/core` is TypeScript only (no `Cargo.toml` there; the
    only `package/toml-edit` Cargo manifest is the fuzz sidecar `core.fuzz`).
  - `package/forbidden-strings/cli` (clap derive CLI, 0.5.0), `package/mpm-shadow/cli`, `package/search-shadow/cli`.

## 3. What the meow design decided or proposed about the Rust file-enforcer port

Legend: DECIDED = user quote and date in a repo doc. PROPOSED = research or design recommendation with no user
acceptance. All scoped to "meow 0.x" (user, 2026-09-17).

### Config model (templates, cat, map over packages, logic without TypeScript)

- DECIDED 2026-09-16: Turing-completeness is no longer required. User: "eliminate the turing-complete requirement
  of the config language. File-enforcer was written under the constraints of time."
  (`doc/planning/monorepo-manager-from-scratch-design.md:576-590`).
- DECIDED 2026-09-17: syntax is OpenTofu-shaped HCL. User: "Yes to HCL" (`doc/decision/monorepo-manager-all-rust.md:
  87-95`); later "HCL syntax is fine. This is only a light endorsement." and expression power "Full language
  allowed" (design `:1668-1690`).
- DECIDED 2026-09-17: `hcl-edit` 0.9.7 with two local patches under meow's own evaluator; unpredictable-result
  functions allowed but make evaluation uncacheable, with warnings; `meow::` namespace; "Ship no formatter";
  `lsp-server` for a language server (`doc/decision/monorepo-manager-hcl-front-end.md:1-47`; handover `:331-353`).
- PROPOSED (research, declarative-config appendix): how logic is expressed. Block kinds `absent`, `file`, `mirror`,
  `toml_keys`, `lsp4ij_server`, `task`, `check`; OpenTofu functions `file`, `fileset`, `jsondecode`, `tomldecode`,
  `join`, `concat`, `distinct`, `sort`, `lookup`, `try`, `can`, `dirname`, `startswith`, `strcontains` plus proposed
  `spdx_ids()` and `lockfile()`; variables `path.root`, `each.key/value` under `for_each`, per-match iterator
  (`monorepo-manager-route-research/stack-declarative-config.md:306-366`). "Every block name is a proposal, not an
  existing API" (`:312-313`). Examples LI01 to LI22 in the same file.
- Fact (research): the 27 units split 13 built-in features, 3 repository tasks (forbidden-strings compile, pnpr
  config, Browserslist), 5 plain data, 6 retired (`stack-declarative-config.md:1121-1137`; design `:1554-1571`).
  Awkward: per-manifest Cargo derivations, writes outside the repo (JetBrains settings), `mise.toml` while Mise
  stays.
- DECIDED 2026-09-17: rules that write outside the repository (JetBrains settings) live in a per-user meow config
  (`doc/decision/monorepo-manager-per-user-config.md`; design `:1668-1690`). Trust model follows cli-git.
- DECIDED 2026-09-17: one root `meow.hcl` with tags per package, argv commands with no shell, `task "x"` one-label
  blocks (handover `:520-527`). Task/ordering schema itself is OPEN and `depends_on` was retracted by the user
  (design `:1622-1645`; implementation plan `:419-421`).
- Consequence for the new request: the 2026-09-17 JSONC option was ranked fourth. The ranking was
  "A (OpenTofu-shaped HCL) > B (TOML) > F (TOML with CEL) > E (JSONC) > D (YAML) > C (KDL) > I > J > G > H"
  (`stack-declarative-config.md:3150-3152`; design `:1603`). Option E cons: "No multi-line strings, so every long
  text is a separate file. Quoting and bracket noise ... B's placeholder and dedicated-built-in cons"
  (`stack-declarative-config.md:2878-2920`). Pros: schema completion in editors, +180,224 bytes, repo incumbents.
  "Disqualifying problems: None found."
- Inference: JSONC data cannot express the 2,592-line TypeScript config's per-file derivations (map over
  `package/*/Cargo.toml`, template text, `cat` of AGENTS.md plus preamble, SPDX mapping) without a built-in
  placeholder syntax or expression language; the research's own cons say so. "Config in jsonc" therefore needs
  either dedicated built-ins per derivation or logic in Rust. This is the key design question for the user.

### Structured-edit crates

- DECIDED by user answers 2026-09-16/17 (requirements): "we must still handle comment-preserving jsonc editing and
  toml editing properly"; "XML comments must also be properly preserved"; malformed managed XML fails with a
  diagnostic; "'untouched bytes identical' rule and preserve CRLF per file - no need"; generated files may change
  once in a reviewed commit (design `:632-640`, `:2520-2540`).
- PROPOSED/recorded as "Chosen editors" (design `:2495-2518`, from `rust-structured-edits.md`): TOML = `toml_edit`
  behind a repository wrapper (21 of 21 cases; bare `toml_edit` 12); JSONC = `jsonc-parser` 0.33.2 CST behind a
  wrapper (16 of 16); XML = `roxmltree` with byte-range splicing (9 of 9). Troubleshooting docs back each wrapper rule
  (`doc/troubleshooting/toml-edit-comment-loss.md`, `jsonc-parser-json5-defaults.md`, etc.).
- DECIDED 2026-09-17: new TOML inline tables use TOML 1.1 trailing comma `{ a = 1, }` (design `:2605-2612`).
- Fact, OVERTAKEN by later decision: 2026-09-24 the repo accepted its own JSONC crate with no third-party parser
  (`doc/decision/jsonc-edit-rust-foundation.md:3-60`: "No third-party parser, syntax-tree, decimal or bigint
  dependency; the standard library only"; accepts only "JSONC container roots: comments and trailing commas, never
  JSON5-only forms"). `doc/planning/jsonc-edit-rust-port.md:196-198,358-359` says that port "does not silently
  change" meow's `jsonc-parser` choice and "do not replace meow's editor as part of this port". So the JSONC editor
  for a Rust file-enforcer is now an open reconciliation: `monochromatic-jsonc-edit` (in-repo, already used by
  cli-git and the linter) versus meow's `jsonc-parser` wrapper. Inference: the in-repo crate is the obvious reuse
  target; no doc decides it.

### Output format

- DECIDED 2026-09-17: user "Emit every line as a json, error or warning or not." Every stdout/stderr line is a JSON
  object, no human renderer, no `miette`/`ariadne` etc.; `meow run` forwards task bytes raw (design `:187-224`;
  all-Rust decision `:96-103`).
- Fact: "file-enforcer emits typed events, because today's log records carry no structured fields" (design `:3355-3372`).

### Staleness cache

- PROPOSED (research only): FE09 port the SHA-256 manifest and `mkdir`-style lock for interoperability while TS and
  Rust coexist (`stack-all-rust-rewrite.md:FE09`, lines ~396-407). Design: 20 staleness modules, 1,420 code lines,
  in the port scope (design `:1280-1285`).
- DECIDED 2026-09-17 (user): "No need" to preserve byte identity or interoperability; separate worktree means the
  TS and Rust enforcers never run on the same tree (design `:1696-1724`). So the lock-interop motive is gone;
  inference: the staleness format is free to change.
- Fact: design says file-enforcer's staleness manifest "is not reused as the task cache, because it checks size and
  modification time for sources" (design `:3249-3250`). Task cache freshness = hash of declared reads (DECIDED
  2026-09-17 while grilling, design `:1655-1660`).

### Watch mode

- PROPOSED/adopted by design: FE21 to FE23 watch mode "is replaced by the daemon watcher" (9 modules, 1,027 code
  lines; design `:1265-1272`); the daemon watches the whole repository (design `:3173-3187`). Hard requirement from
  the user 2026-09-16: "watch plus RPC pattern" (`mise-removal-coverage.md:65-80`). Inference: a standalone
  clap CLI with `--watch` would need its own watcher (notify crate is in `rust-catalog.toml`); the design assigns
  watch to the meow daemon.

### Hashing

- DECIDED 2026-09-17: "Accept twox-hash" -- cache keys and content hashes use XXH3-128 via `twox-hash` 2.1.4
  (`doc/decision/monorepo-manager-cache-key-hash.md:1-50`). Rejected `gxhash` after collisions. Scope is meow's
  task cache; whether file-enforcer's staleness manifest also uses it is not stated (inference: probably, but the
  FE09 research proposed SHA-256 for compatibility).
- Platforms (DECIDED 2026-09-16/17): Linux x86_64 and aarch64, each glibc and static musl; only `x86-64-v4` and
  aarch64 block releases (all-Rust decision `:54-86`). Applies to a meow binary; no statement for a separate tool.

### CLI parser

- Fact: no meow doc names a parser for meow itself. `rg -w clap` in the design, implementation plan and decision
  records finds nothing about meow's CLI. The only proposal is the research FE02 line: "CLI parsing with `clap` 4
  (an incumbent in `CARGO_SHARED_DEPENDENCIES`, `file-enforcer.config.ts:1498`)"
  (`stack-all-rust-rewrite.md:361`). PROPOSED, never accepted.
- Command surface DECIDED 2026-09-17 for meow: `watch`, `run <target>`, `status`, `stop`, `pause`, `resume`, `end`,
  `priority`, `trust`/`untrust`/`status`, `doctor`, hidden `lsp` (design `:322-454`, implementation plan `:338-372`).
  There is no `meow sync`/`meow enforce` command spelled out; file enforcement is a maintained concern run by the
  daemon, not a verb. The TS CLI today is `file-enforcer [--watch] [--check]` (`package/file-enforcer/core/src/cli.ts:
  29-39`).

### JSONC mention in meow docs

- Only as option E above and as the editor requirement for managed files. No doc proposes JSONC as meow's own
  configuration format after the "Yes to HCL" answer.

## 4. Other repo precedent: jsonc, config crates, clap, Rust approval

### JSONC as Rust tool configuration (strong precedent, user-directed)

- Fact, `doc/decision/cli-git-rust-rewrite.md:3-17` (Accepted 2026-10-04): "The user selected a Rust cli-git
  replacement, JSONC configuration through the repository's existing Rust package ... The same implementing agent
  owns the unified Rust/Markdown/MDX linter. It also uses JSONC, replacing the earlier HCL choice."
- Fact, `doc/handover/unified-linter.md:48-54`: "Both tools use JSONC through `package/rust-module/jsonc-edit`
  [now `package/jsonc-edit/core-rust`]. The linter uses `monochromatic-lint.config.jsonc`, replacing
  `monochromatic-lint.config.hcl` on 2026-10-04." The user changed the linter from HCL (which "follows meow",
  `unified-linter.md:323`) to JSONC.
- Fact: two live root files: `cli-git.config.jsonc`, `monochromatic-lint.config.jsonc`. Loaders:
  `package/git-policy/cli/src/native/config_parse.rs:38` and `package/linter/monochromatic-lint/src/` (`run_check.rs`,
  `config_*.rs`) both `use monochromatic_jsonc_edit::{JsoncValue, parse_jsonc}` and walk the value by hand. Neither
  uses `config`, `figment`, or serde derive over the config (git-policy and linter list `serde_json` for other
  purposes). Layering = "simple handwritten configuration merging inside the linter" (decision `:12-15`).
- Fact: the `figment` crate was vetted and rejected for the linter (`doc/audit/tech-unified-linter-config-deep-merge-vet-
  2026-09-23.md`: "every crate failed HC1 at validation, and the only survivor is the in-linter baseline function").
- Fact: no doc mentions `config-rs`/"config crate"/JSON5 as a config choice for a repo Rust tool. JSON5 hits are all
  "reject JSON5 forms" (`jsonc-edit-rust-foundation.md:68`, `troubleshooting/jsonc-parser-json5-defaults.md`).
- Inference: for a jsonc config the established repo path is `monochromatic-jsonc-edit` + manual typed extraction,
  not serde/config-rs. Its comment-preserving edit API is overkill for read-only config but it is the in-repo
  parser, std-only, 512-depth bound.

### clap precedent

- Fact: crates depending on clap (from `package/*/*/Cargo.toml`, catalog floor `clap = ">=4"` in
  `rust-catalog.toml`): `forbidden-strings/cli` (derive, `Parser`+`Subcommand`; comment at `Cargo.toml:64-67`: "`clap`
  owns argv parsing, generated help/version output, and validation ... derive feature lets `src/cli.rs` declare the
  command shape as a typed struct"), `ide/app` (derive), `nested-wayland-session/cli` (derive), `music-player/
  desktop-app` (derive), `mpm-shadow/cli` (builder `clap::Command`), `search-shadow/cli` (builder `Command`/`Arg`).
- Counter-precedent: `package/git-policy/cli` (cli-git native) does NOT use clap; it hand-ports Git 2.56.0
  `parse-options` because it must tokenize Git's own arguments
  (`doc/handover/cli-git-native-command-parser.md:44-60`). Not applicable to a normal CLI.
- Fact, `doc/decision/mpm-toolchain-adoption.md:289`: "The maintainer then chose a shadow of `mpm` written in Rust
  with clap, following the `git` wrapper precedent" (2026-10-29). So clap for a new Rust CLI is the current
  maintainer-chosen pattern.
- Config loading in those clap crates: `forbidden-strings/cli` reads rule files given by flags; `music-player/
  desktop-app` uses `directories`; `ide/app` uses `toml`+`serde_json`; shadows read none. None loads a layered
  jsonc config; only cli-git and the linter do, via `monochromatic-jsonc-edit`.

### Rust-approval rules that would apply to a new Rust package

- Fact: `AGENTS.md` has no approved-language rule. `rg -i 'approved language'` finds nothing. `SCR` (bash/powershell
  prohibition) exists; `RNQ` ("Rust packages use Rust tests through nextest, never TypeScript tests. Use nextest's
  default build and test concurrency") applies; `PKG` (README.md, zero lint errors, passing tests covering every
  exported code path), `AP1`/`AP2`/`SGD`/`SBS` (package placement and naming), `NLG` (no synthetic load) apply.
  `MXL` governs max-lines. The implementation plan cites `MXR`/`RDC` (300 code lines per `.rs` file, rustdoc on every
  item, `monorepo-manager-implementation-plan.md:188-190`), but `rg -w 'MXR|RDC' AGENTS.md` finds nothing (those
  codes appear only in planning docs such as `doc/planning/unified-linter.md`), so treat them as retired or
  unconfirmed.
- Fact, `doc/planning/load-bearing-code-languages.md` (129 lines): a proposal ("Present the revised wording for
  acceptance"), not yet applied. It would require "Repo-authored logic uses approved languages only"; "The word
  'approved' refers to an explicit approval applicable to the work, not the presence of a language in existing
  files". Proposed text mentions TypeScript default for automation. Rust approval today is by per-work user
  decision: `monorepo-manager-all-rust.md:52-53`: "Rust is approved for this tool, for the approval question in
  `doc/planning/load-bearing-code-languages.md`" (scope = meow), and `cli-git-rust-rewrite.md` (scope = cli-git +
  linter). Inference: a standalone Rust file-enforcer is outside both scopes; the user's current request would be
  the approval.
- Fact, `doc/decision/rust-dependency-catalog.md`: every Rust crate is its own workspace root, no root Cargo
  workspace; every registry dep comes from `rust-catalog.toml` with `workspace = true`; internal crates are path deps
  with `version = "=x.y.z"`; file-enforcer (TS) renders and checks it. Adding clap/serde etc. to a new crate goes
  through this; new deps under `choosing-technology` vets (e.g. `monochromatic-jsonc-edit/Cargo.toml:36-39`: "Anything
  added here needs a fresh choosing-technology run").
- Fact, `doc/decision/monorepo-manager-all-rust.md:136-139,64-68`: shipped builds are glibc + static musl, Linux
  first; the root `mise.toml` "stays generated by file-enforcer from `mise.no-env.toml` until Mise is removed, and
  meow never generates it." The user's list (CLAUDE.md, mise.toml, Cargo.toml, toolchain lock) includes `mise.toml`,
  which the meow decision excluded from meow but the TS enforcer still generates. A standalone Rust file-enforcer
  would have to either keep generating it (consistent with Mise staying until removal) or not (needs a decision).

## Conflicts and open questions the user should be asked (inference)

1. Standalone or slice of meow? The request (clap CLI, jsonc config) contradicts three meow-0.x decisions: HCL config
   (2026-09-17), inside the single meow binary (2026-09-16), and the daemon-owned watch/RPC (user requirement). No
   doc says the user withdrew them. Needed: is meow's file-enforcement slice being extracted early into a standalone
   `file-enforcer` binary (and meow later links it), or do the HCL/meow decisions change?
2. JSONC cannot carry the logic (see section 3 Inference). Which approach: dedicated built-ins in Rust with JSONC
   as data, logic compiled into Rust, or an expression sub-language? The user's own 2026-09-16 premise ("logic in
   today's file-enforcer.config.ts moves into code") fits "logic in Rust, JSONC for data".
3. JSONC editor/parser crate: in-repo `monochromatic-jsonc-edit` (2026-09-24 decision, std-only) versus the meow
   design's `jsonc-parser` wrapper.
4. Scope of the ground-up rewrite given new duties (toolchain lock, Rust catalog sync, Pants BUILD generation, Mise
   still generated) and the "keep TS file-enforcer running until replaced" decision, plus concurrent TS edits in
   `package/file-enforcer/core/src/toolchain/`.
5. Watch mode and staleness: keep `--watch`/`--check` in the standalone CLI, or leave watch to meow's daemon.
6. Rust approval scope (no AGENTS.md rule; per-work decision).
