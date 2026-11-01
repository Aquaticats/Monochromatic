# Every doc file two layers deep

This is a plan, not an execution.
Nothing under `doc/` has been moved by this document.
It prepares the migration that makes every tracked file under `doc/` sit at
 `doc/<layer1>/<layer2>/<file>`,
 records the assumptions the request leaves open,
 and carries a decision tree that resolves to a complete execution plan
 whichever branch is picked at review time.

The request, in its own words:
 move `doc/troubleshooting/optique-readonly-argument-effects.md` to
 `doc/troubleshooting/optique/readonly-argument-effects.md`,
 and do the same for every file under `doc/`.
Acceptance of this plan is what unlocks the migration (rule `DRR`).

## What the invariant says

The example fixes three things:

1.  Depth: the file ends up two directories under `doc/`, shape
     `doc/<layer1>/<layer2>/<file>`.
2.  Layer 1 keeps the existing family, `troubleshooting` here.
3.  Layer 2 comes from the filename's subject prefix, and the prefix leaves the
     leaf name: `optique-` becomes the directory `optique/`, the leaf keeps
     `readonly-argument-effects.md`.

The invariant this plan targets, stated once:
 every tracked file under `doc/` matches `doc/*/*/file`, with no file deeper.

## Measured inventory

Counts from `git ls-files doc/` on 2026-10-19, 2,333 tracked files.

Depth under `doc/`, measured as `git ls-files doc/ | awk -F/ '{print NF-1}'`:

-  0 directories: 2 files.
     `doc/dependency-blocklist.md`, `doc/secret-management-caveman.html`.
-  1 directory: 2,154 files at `doc/<family>/<file>`.
     These need one new layer.
-  2 directories: 176 files at `doc/<family>/<dir>/<file>`.
     These already conform.
-  3 directories: 1 file,
     `doc/audit/resharp-fuzz-2026-06-19/tool/anchor_denot.rs`.
     This is over-deep under a strict reading.

Families at one directory layer
 (`git ls-files doc/ | awk -F/ 'NF-1==2 {print $2}' | sort | uniq -c`):

-  troubleshooting 727, planning 436, audit 387, handover 341, decision 155.
-  runbook 37, todo 29, research 10, philosophy 10, limitation 7, agent 6.
-  artifact 3, tech-debt 2, migration 2, misc 1, howto 1, adr 1.

File kinds among the 2,154 at one layer:

-  1,780 Markdown files.
-  374 others: evidence `.json`, `.json.gz`, `.zip` archives, 60 `.patch`
     sidecars, `.mjs` and `.ts` scripts, `.html`, `.jsonl`, `.spvasm`, `.toml`,
     `.rs`, `tsconfig.json`.
-  2,096 hyphenated names, 59 bare names (`bash.md`, `glm.md`, `README.md`,
     `tsconfig.json`, `performance.build.md`).
-  153 stems carry two or more files, so sidecar groups must move together.

Hubs and stray layers:

-  4 family hubs at `doc/<family>/README.md`: audit (501 lines), troubleshooting
     (430), todo (133), philosophy (42).
-  5 campaign hubs already two layers deep, e.g.
     `doc/audit/resharp-fuzz-2026-06-04/README.md`.
-  `doc/audit/README.md` is titled "Dependency Security Report"; it is content,
     not an index, and root `README.md` links it as `AUDIT.md`.
-  `doc/handover/tsconfig.json` is tooling config living beside the
     `ts-index-imports-build-all.*.ts` scripts.

Reference surface, `rg -o 'doc/[A-Za-z0-9._/-]+'` counts:

-  13,226 occurrences repo-wide: 4,054 in `.md`, 4,200 in code and config,
     4,972 inside evidence archives.
-  1,229 distinct referenced paths in Markdown scope; 894 name tracked files.
-  3622 relative Markdown links inside `doc/` (`](bash.md)` style).

Reference hazards measured, not assumed:

-  335 referenced `doc/...` paths are not ours: Node.js `doc/api/*` prose,
     package-local `package/translation-repair/doc/*` reached as
     `doc/configuration.md` from package docs. A blind `doc/` string rewrite
     corrupts these.
-  `.github/workflows/admission-gate.yml` reads
     `doc/planning/depot-admission-ledger.json` at run time.
-  `package/translation-repair/source-scan/src/doc-line-limit.unit.test.ts`
     (`readGuardedDocs`) does `readdir('doc', family)` and matches stems on
     bare filenames; nested topics make it silently read nothing.

## Assumptions

Flagged per the request; each names what breaks if it is wrong.

1.  "2 layers deep" means exactly two directory components under `doc/`.
     If it means "at least two", the over-deep `.rs` file and any future
     `tool/` sub-bundles stay put and the depth check in
     `Verification` becomes `NF < 4`.
2.  Layer 1 stays the existing family, as the example keeps `troubleshooting`.
3.  Layer 2 is a subject prefix of the filename and the prefix leaves the leaf
     name, as `optique-` leaves `readonly-argument-effects.md`.
     The alternative (keep the full stem as leaf) is a decision-tree branch,
     not an axiom.
4.  "Every file" includes non-Markdown evidence, archives, patches, and
     scripts; they move with their subject doc (`SBS`).
5.  Package docs stay beside code (`DPL`), so `package/*/doc/` is out of scope.
6.  Files already two layers deep stay, names untouched, unless
     `Q7` normalizes them.
7.  Moves use `git mv` so `git log --follow` keeps working.
8.  Historical evidence archives (`.json`, `.json.gz`, `.zip`) are frozen
     bytes; their inner path strings are quotes of the past and are not
     rewritten.
9.  This plan file lives at `doc/planning/doc-depth-two-layers.md` under the
     current `DL1` shape and migrates with everything else; see
     `Self-application`.
10.  A Markdown file holds at most 1,000 lines (`doc-line-limit`), so the full
     2,333-row mapping is generated at execution time, not inlined here.

## Recorded decisions this revises

Executing this plan reverses or rewrites recorded rationale. Say so at review.

-  `doc/philosophy/documentation.md`, section "Stripping the prefix and
     nesting one level", records: "Deeper nesting was rejected because the
     segments after the family are ad-hoc topic tags, not a stable taxonomy...
     The family is the only division that earns a directory."
     This plan promotes topic tags to directories anyway.
-  The same section records "A second dotted segment stays flat in the
     filename", which `Q6` reopens.
-  `doc/philosophy/documentation.md` says a hub "indexes spoke files in the
     same directory"; after the move spokes live in subdirectories, so hub
     prose must change or hubs must move (`Q4`).
-  `AGENTS.md` `DL1`, `DL3`, `RBK` name the current path shapes;
     `doc/troubleshooting/<topic>.md`, `doc/runbook/<topic>.md`,
     `doc/handover/<topic>.md`. All three need replacement text.
-  `.agents/skills/troubleshooting-doc/SKILL.md` and
     `.agents/skills/runbook/SKILL.md` teach the same shapes.
-  The "Patches live beside their doc" rule in
     `doc/philosophy/documentation.md` keeps working, but its example path
     `doc/troubleshooting/<topic>.patch` becomes
     `doc/troubleshooting/<topic>/<name>.patch`.

## Naming options for layer 2

The request's example is ambiguous in one place: `optique` is a single-word
 subject, so "first hyphen segment" and "longest subject prefix" agree on it.
They disagree on 2,095 other hyphenated names. Worked examples use real files.

### Option A, mechanical first segment

Split at the first hyphen; left becomes the directory.

-  `doc/planning/translation-repair-package-split.md`
     to `doc/planning/translation/repair-package-split.md`.
-  `doc/decision/cli-git-rust-rewrite.md`
     to `doc/decision/cli/git-rust-rewrite.md`.
-  `doc/audit/the-critics-are-shown-the-wrong-paragraph.md`
     to `doc/audit/the/critics-are-shown-the-wrong-paragraph.md`.
-  `doc/decision/prefer-readonly-member-channel-authority.md`
     to `doc/decision/prefer/readonly-member-channel-authority.md`.

Pros: zero judgment, one scriptable rule, matches the example literally.
Cons: mangles multi-word subjects (`translation-repair` 433 files,
`cli-git` 58, `music-player` 36, `prefer-readonly` 28, `forbidden-strings` 21),
and 151 files land in stop-word directories (`the/`, `two/`, `to/`, `file/`,
`module/`, `issue/`). A dry run over the 2,154 one-layer files
yields 553 target directories, 388 holding exactly one file.

### Option B, subject table with stem fallback

Split at the longest known subject prefix; bare or sentence-style names fall
 back to whole-stem directory with a `README.md` leaf.

-  `doc/planning/translation-repair-package-split.md`
     to `doc/planning/translation-repair/package-split.md`.
-  `doc/decision/cli-git-rust-rewrite.md`
     to `doc/decision/cli-git/rust-rewrite.md`.
-  `doc/troubleshooting/optique-readonly-argument-effects.md`
     to `doc/troubleshooting/optique/readonly-argument-effects.md`,
     identical to the request.
-  `doc/audit/the-critics-are-shown-the-wrong-paragraph.md`
     to `doc/audit/the-critics-are-shown-the-wrong-paragraph/README.md`.

Pros: keeps subjects whole, matches the request on its example, one curated
 table covers the 20 dominant prefixes.
Cons: the table needs curation and a maintenance rule for new subjects;
 sentence-style audit names still become one-file directories.

### Option C, whole-stem directory for everything

Every file becomes `doc/<family>/<stem>/README.md`.

-  `doc/troubleshooting/optique-readonly-argument-effects.md`
     to `doc/troubleshooting/optique-readonly-argument-effects/README.md`.

Pros: fully mechanical, no judgment, collision-free.
Cons: contradicts the request's leaf shape, turns all 2,154 files into
 directories, and re-creates exactly the "directory holding one file" outcome
 `doc/philosophy/documentation.md` objected to.

### Option D, subject table with first-segment fallback

Option B's table, but unmatched names fall back to Option A rather than to a
 stem directory.

Pros: complete without inventing subjects for the tail.
Cons: keeps Option A's stop-word directories for 151 files.

Ranking: B > D > A > C, because B and D agree on the 20 dominant subjects and
 part on the tail, where `the/critics-...` is worse than a stem directory,
 so B beats D; D beats A because the subject table fixes
 `translation-repair` and `cli-git` for 491 files where A alone garbles them;
 A beats C because C contradicts the request's example outright.

### Leaf naming for bare stems

Once layer 2 exists, `bash.md` has no prefix to split. Options, with
 `doc/troubleshooting/bash.md` and its future sibling `bash-history.md`:

-  `README.md` leaf: `doc/troubleshooting/bash/README.md`, later
     `doc/troubleshooting/bash/history.md`. The topic directory grows a hub
     exactly like families do today.
-  Self-named leaf: `doc/troubleshooting/bash/bash.md`, later
     `doc/troubleshooting/bash/history.md`.
-  `index.md` leaf: `doc/troubleshooting/bash/index.md`.
-  Curated parent topic: `doc/troubleshooting/shells/bash.md`.

Ranking: `README.md` > self-named > `index.md` > curated parent, because
 `README.md` is this repo's established hub name and GitHub renders it when
 browsing the topic directory; self-named keeps `rg` basenames unique and
 keeps patch sidecars sharing their doc's stem; `index.md` has no precedent
 here; curated parent reintroduces judgment the split rule already answered.

## Decision tree

```text
Q1 depth          exactly 2          | at least 2
Q2 layer-2 name   A first-segment | B subject+stem | C stem-dir | D subject+A
Q3 bare stems     README | self-named | index | curated parent
Q4 family hubs    exempt | relocate | dissolve
Q5 root-level     attach to sibling topic | doc/misc/ | new family
Q6 dotted names   split at first separator | stem-dir | keep flat
Q7 deep dirs      keep names | normalize
Q8 references     rewrite-all | WR6 hybrid | rewrite + central note
Q9 batching       atomic commit | per-family batches
Q10 enforcement   guard test | rule only
```

Each question in `Decision tree` lists options, consequences, and the exact
 execution delta so any leaf resolves without re-planning.

### Q1, depth semantics

-  Exactly 2: the invariant check rejects any `NF != 4`; the one over-deep
     file flattens to `doc/audit/resharp-fuzz-2026-06-19/anchor_denot.rs`
     or its `tool/` layer becomes a leaf-name prefix.
-  At least 2: the file stays, and future `tool/` bundles may nest deeper.

Execution delta: one `git mv` versus none; one line in the depth check
 (`NF != 4` versus `NF < 4`). Recommendation: exactly 2, matching the
 request's wording.

### Q2, layer-2 naming

Options and ranking live in `Naming options for layer 2`.

Execution delta: Option B needs a curated subject table (20 to 40 rows,
 derived from the measured prefix counts) checked into the mapping step;
 A and D run pure regex; C needs no table but rewrites every leaf to
 `README.md`. Recommendation: B.

### Q3, bare-stem leaves

Options and ranking live in `Leaf naming for bare stems`.

Execution delta: only the 59 bare names change target; 5 stems are
 sidecar groups (`oxlint.md` plus `oxlint.patch`), and under `README.md`
 leaves both rename to `README.md` and `README.patch`, so the sibling rule
 in `doc/philosophy/documentation.md` survives either way. Recommendation:
 `README.md`.

### Q4, family hubs

The request says "every file", which literally includes the four
 `doc/<family>/README.md` hubs.

-  Exempt hubs: hubs stay at `doc/<family>/README.md`, invariant text becomes
     "every spoke file two layers deep".
-  Relocate hubs: `doc/troubleshooting/README.md` to
     `doc/troubleshooting/index/README.md` (or `README/README.md` under a
     mechanical rule).
-  Dissolve hubs: hub prose spreads into per-topic `README.md` files.

Execution delta: exempt means `DL1` keeps its hub clause and only spoke text
 changes; relocate means `DL1` and root `README.md` links move and GitHub
 stops rendering a family landing page; dissolve is a rewriting project of its
 own for 1,066 hub lines. Ranking: exempt > relocate > dissolve, because the
 hub is the map of the family, not a spoke, and `DL1` already carves it out.

### Q5, root-level files

Two files sit directly in `doc/`.

-  `doc/secret-management-caveman.html` has a named sibling topic,
     `doc/philosophy/secrets.md`; attaching gives
     `doc/philosophy/secrets/secret-management-caveman.html`.
-  `doc/dependency-blocklist.md` is referenced 10+ times as
     `doc/dependency-blocklist.md` from `pnpm-workspace.yaml` and shim
     package READMEs. Candidates: `doc/misc/dependency-blocklist/README.md`,
     a new `doc/policy/dependency-blocklist/README.md`, or attach to
     `doc/troubleshooting/dependencies.md` as
     `doc/troubleshooting/dependencies/dependency-blocklist.md`.

Execution delta: two target rows in the mapping plus their rewrite entries.
 Recommendation: attach to the sibling topic where one names the subject
 (secrets), `doc/misc/<stem>/README.md` otherwise.

### Q6, dotted compound names

`performance.build.md` sits beside `performance.md` in both `doc/todo/` and
 `doc/troubleshooting/`; `performance.logging.bench.mjs` is a script sidecar.

-  Split at the first separator, hyphen or dot:
     `performance.md` to `performance/README.md`,
     `performance.build.md` to `performance/build.md`,
     `performance.logging.bench.mjs` to `performance/logging.bench.mjs`.
-  Stem directory: `performance.build/README.md`.
-  Keep dotted names flat inside the topic directory:
     `performance/performance.build.md`.

Execution delta: pure mapping rule text. Ranking: split at first separator >
 stem directory > keep flat, because the first option turns the `performance.*`
 cluster into a topic directory with a hub, which is the hub-and-spoke model
 already in use; and it revises the "second dotted segment stays flat" line in
 `doc/philosophy/documentation.md` explicitly.

### Q7, existing topic directories

13 directories hold the 176 conforming files, e.g.
 `doc/audit/tech-monorepo-manager-vet-2026-09-16/` (17 files),
 `doc/decision/ios-iphone-x-vet-report/` (19).

-  Keep names: they are already `doc/<family>/<topic>/<file>`.
-  Normalize: fold `tech-*` campaigns into `doc/audit/tech/` with prefixed
     leaves, merging 48 loose `tech-*.md` files into the same directory.

Execution delta: zero moves versus roughly 200 more moves and one more
 collision surface. Ranking: keep > normalize, because normalization buys
 naming taste and costs link churn in finished records.

### Q8, reference rewriting

`AGENTS.md` `WR6` governs moved identifiers in historical records: original
 text stays, an in-place note gives the current value and the move date.

-  Rewrite-all: fix every `doc/...` string and relative link everywhere.
-  `WR6` hybrid: rewrite living docs (root files, `AGENTS.md`, `CLAUDE.md`,
     hubs, `doc/philosophy/`, `doc/agent/`, `doc/todo/`, `doc/runbook/`,
     `doc/troubleshooting/`, `doc/limitation/`, skills, package docs, code
     comments, configs); in historical records (`doc/handover/`, `doc/audit/`,
     `doc/decision/`, `doc/artifact/`, `doc/research/`) keep the old path text
     and add an in-place note with the new path and move date.
-  Rewrite plus one central moved-paths note per historical record.

Execution delta: rewrite-all is one script pass; `WR6` hybrid adds a note
 pass over the historical families; the central-note variant bounds note
 volume. Ranking: WR6 hybrid > rewrite-all, because `WR6` is binding and
 rewrite-all silently deletes the record of where things lived; central note
 beats per-mention notes only where a record cites dozens of moved paths.
 One sub-question rides here: in historical records a Markdown link cannot
 both keep its old href and resolve. Options: keep old href and add a note
 with the new path (`WR6` literal), or fix the href and note the old path.
 Flagged as open question `OQ-1`.

Evidence archives stay frozen under every branch (assumption 8). The pinned
 GitHub blob URLs inside them (e.g.
 `blob/main/doc/audit/depot-source-identity-controls-2026-10-17.evidence.json`)
 will 404 after the move; the companion `.md` carries the note.

### Q9, batching

-  Atomic: one commit with moves, rewrites, tooling, and rule text, like
     precedent `bd19d57e20`, subject "refactor(*): rename docs/ to doc/ and
     singularize doc families per SGD".
-  Per-family batches: one commit per family, each carrying that family's
     moves plus its reference updates.
-  Moves first, rewrites later: two waves.

Execution delta: commit shape only, but it decides whether any commit leaves
 dangling cross-family links. Ranking: atomic > per-family > two-wave, because
 atomic keeps every commit's link graph whole and matches the recorded
 precedent; two-wave leaves dangling links between waves.

### Q10, enforcement

-  Guard test: a depth assertion (`git ls-files doc/` must yield `NF == 4`)
     added to `source-scans` or a root mise task.
-  Rule only: `DL1` text plus review, matching
     `doc/philosophy/documentation.md`, "The rule is the enforcement".

Execution delta: one test file or none. Ranking: guard test > rule only,
 because depth is mechanically checkable while the philosophy doc's deferral
 covered placement taste, not arithmetic.

## Execution plan

Phases are numbered for reference from the tree deltas; run them in order
 after acceptance.

1.  Freeze the picks. Fill one row per question `Q1` to `Q10` in this doc's
     decision table and commit that as part of the plan revision.

2.  Build the mapping table. A scratch script in
     `${HOME}/temp/agent` (rule `ATH`) walks `git ls-files doc/` and emits
     `old<TAB>new` rows implementing `Q2`, `Q3`, `Q5`, `Q6`; sidecar groups
     share one topic directory; `Q7` rows are added or omitted. Spot-check at
     least the thirteen worked examples in `Naming options for layer 2` and
     `Leaf naming for bare stems`. Collisions abort the run.

3.  Update rules and philosophy in the same change as the moves: `DL1`, `DL3`,
     `RBK` replacement text (each stays under 50 words, `RLM`), the
     `doc/philosophy/documentation.md` sections named in
     `Recorded decisions this revises`, and the two skill files. Regenerate
     `CLAUDE.md` via file-enforcer (`WC2`).

4.  Move files: `git mv` per mapping row, sidecar groups in one command.

5.  Rewrite references per `Q8`: living docs first, then historical notes;
     never rewrite strings inside `.json`, `.json.gz`, `.zip` evidence; skip
     335 foreign `doc/...` paths (Node.js docs, `package/*/doc/`) by
     membership test against the mapping table, not by pattern.

6.  Update tooling: `readGuardedDocs` in
     `package/translation-repair/source-scan/src/doc-line-limit.unit.test.ts`
     becomes two-level and matches `translation-repair` on topic directory
     names; `file-enforcer.config.ts` string blocks that cite `doc/...` paths
     are edited at source and its outputs regenerated;
     `.github/workflows/admission-gate.yml` ledger path follows its file;
     `monochromatic-lint.config.jsonc`, `slopo.conf.yaml`, `slopo.ignore.txt`,
     `pnpm-workspace.yaml`, `mise.toml`, `cli-git.config.jsonc` comments.

7.  Verify per `Verification`, then commit per `Q9`.

## Verification

All commands are cheap and mechanical; a failure blocks the merge.

-  Depth invariant, exact-2 form:

```sh
# no tracked file deviates from doc/<a>/<b>/<file>
git ls-files doc/ | awk -F/ 'NF != 4 {print}'
```

-  Stale path check: every old path from the mapping must be absent from
     living scope and present only where `Q8` allows:

```sh
# living scope must be clean
rg -n -F 'doc/troubleshooting/optique-readonly-argument-effects.md' \
  AGENTS.md CLAUDE.md README.md doc/philosophy doc/troubleshooting
```

-  Guard suites:
     `mise run //package/translation-repair/source-scan:test:unit`,
     `mise run //package/translation-repair/source-scan:source-scans`,
     `mise run //package/translation-repair/source-scan:lint`.
-  Runtime consumer: the path in `.github/workflows/admission-gate.yml`
     exists (`test -f`), and the JSON parses.
-  Link integrity: `living-doc-links` covers living docs; spot-check three
     historical records for `WR6` notes.
-  History: `git log --follow` on three moved files returns pre-move commits.

## Risks

-  Review surface: 2,333 renames plus thousands of link edits in one commit;
     the precedent commit is the only prior art at this scale.
-  Guard blindness: `readGuardedDocs` reads nothing after nesting until step 6
     lands; a green run before that proves no coverage (`QIV`).
-  External breakage: GitHub issues, pull requests, and the blob URLs frozen
     in evidence keep pointing at old paths forever; no in-repo fix exists.
-  `WR6` note volume: 4,054 Markdown mentions, of which the historical share
     may want one note each; `Q8` bounds it.
-  Mapping surprises: the 59 bare stems, 151 stop-word prefixes, and 153
     sidecar groups are the collision-prone tail; the spot-check in step 2
     is the defense.

## Open questions

Mirrors the tree; each is answered by picking a branch.

-  `OQ-1`: `WR6` semantics for Markdown links in historical records.
     Old href plus note, or fixed href plus note of the old path.
-  `OQ-2`: whether "every file" includes the four family hubs (`Q4`).
-  `OQ-3`: exactly-2 versus at-least-2 depth (`Q1`).
-  `OQ-4`: whether the leaf keeps its subject prefix after the split
     (assumption 3). Stripping matches the request's example but breaks
     grepping by the old basename; keeping it preserves basenames and leaves
     `optique/optique-readonly-argument-effects.md`.
-  `OQ-5`: subject-table size and who maintains it (`Q2`).
-  `OQ-6`: where `doc/dependency-blocklist.md` lands (`Q5`).

## Self-application

This file, `doc/planning/doc-depth-two-layers.md`, is itself one layer deep
 and migrates under its own plan:

-  Option B: `doc/planning/doc-depth/two-layers.md`.
-  Option A: `doc/planning/doc/depth-two-layers.md`, which reads as a family
     named `doc` and is a worked argument against Option A.
-  Option C: `doc/planning/doc-depth-two-layers/README.md`.

## Precedent consulted

-  In-repo: `bd19d57e20` renamed `docs/` to `doc/` and singularized families
     in one commit; `doc/planning/singular-dir-name-invariant.md` batched its
     plan and `AGENTS.md` rule text first, then leaf renames.
-  Umbraco documentation keeps each topic in its own directory with a
     `README.md` landing page and the directory named for that page
     (<https://docs.umbraco.com/contributing/documentation/style-guide/structure>).
-  A docs-structure kit promotes a domain to `docs/<area>/<domain>/README.md`
     once it grows past one file
     (<https://github.com/dotdotgod/dotdotgod-kit/blob/main/docs/arch/DOCS_STRUCTURE.md>).
-  OpenStack guidance groups topical filenames into topic subdirectories and
     keeps hyphenated descriptive names
     (<https://docs.openstack.org/doc-contrib-guide/rst-conv/file-naming.html>).
