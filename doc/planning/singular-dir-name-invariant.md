# Singular dir names and the package name path invariant

Note 2026-11-04T22:23:58-05:00:
the program `cli-git-native` is renamed `monochromatic-git-policy-cli`,
the command word `git cli-git` becomes `git monochromatic-git-policy-cli`,
the settings file `cli-git.config.jsonc` becomes `monochromatic-git-policy-cli.config.jsonc`,
and the folders,
logs and service units that carry `cli-git` carry `monochromatic-git-policy-cli`
(the list is in `doc/planning/git-policy-cli-program-rename.md`).
The new names arrive with the first release after this date;
this repository switches its installed program and its settings file when it installs that release.
This record keeps the names of its time.

Status:
 executed 2026-07-15,
 all eleven batches landed.
The decisions under the `Decisions` heading were made by the user during a grilling session on 2026-07-15.
Deviations and residue are recorded at the end of this doc.

## Motivation and reframing

The original request was "remove all trailing `s` from dir names".
Grilling surfaced the real motivation:
`package.json` names should equal the package path with `/` replaced by `-`.

Measured state at decision time:
135 workspace packages,
 28 name/path mismatches.
Only 6 mismatches were trailing-`s` problems;
the other 22 had names whose category segment differed from the dir entirely
(`pi-*` under `pi-plugins/`,
`config-oxlint-*` under `oxlint-plugins/`,
`cli-git` under `git-policies/`).
A literal strip would also have created roughly 30 new mismatches
by breaking currently-matching plural pairs such as `claude-code-plugins-*`.

The plan is therefore two goals,
 not one:

- Invariant:
   every npm workspace package satisfies
  `name === '@monochromatic-dev/' + <path relative to the workspace packages root>.replaceAll('/', '-')`.
- Full retroactive depluralization of repo-controlled dir names,
  with dir and package name renamed in lockstep so the invariant never breaks.

## Decisions

- Rule formulation is "repo-chosen dir segments are singular nouns",
  never a literal "no trailing `s`":
  `harness` is singular yet ends in `s`,
  `css`/`rss` are acronyms,
  `throws` is a verb.
- Terms of art keep their plural,
   external and repo-coined alike:
  `dotfiles`,
  `import-attributes`,
  `no-exports` (the `package.json` `exports` field),
  `forbidden-strings` (tool identity:
   CLI,
   packages,
  `forbidden-strings.append.local.txt`),
  `jetbrains` (brand).
- Skill directory names are skill identities (`testing-practices`,
  `css`) and stay unchanged.
- Tool-mandated dir names stay unchanged:
  `.github/workflows`,
  `.claude/skills`,
  `.agents/skills`,
  `.factory/skills`,
  `.crush/*`,
  `agents/` (a Claude Code plugin's agent folder,
   the folder name Claude Code reads by default,
   and the name a plugin manifest's `agents` key replaces),
  Cargo `tests/`,
   cargo-fuzz `fuzz_targets/`,
  `corpus/`,
  `artifacts/`,
  `LICENSES/` (REUSE),
  `share/applications` (XDG),
   KWin `contents/`,
  `.idea/`,
  `node_modules/`,
   build outputs.
- Plugin families use a uniform singular `-plugin` suffix:
  `pi-plugin/`,
  `claude-code-plugin/`,
  `intellij-plugin/`,
  `rolldown-plugin/`,
  `oxlint-plugin/`.
- `oxlint-plugins/*` packages are renamed `config-oxlint-*` to `oxlint-plugin-*`;
  `packages/config/oxlint` keeps its name and updates its dependencies and re-exports.
- `figma-parsers/` becomes `figma/`;
  leaf `penpot` becomes `to-penpot` so `figma-kiwi` and `figma-to-penpot` need no name change.
  The old category name was already wrong for the Penpot converter,
   which is not a parser.
- `git-policies/` becomes `git-policy/`.
  The guard CLI stays at `git-policy/cli` and its name changes from `cli-git` to `git-policy-cli`;
  the bin name `git` is untouched.
  `AGENTS.md` rules GCW and CLG update their `cli-git` mentions and the cited source path
  (GCW:
  retired from `AGENTS.md` 2026-09-29;
  the README section that held it was dropped 2026-10-13).
- `agent-harnesses-shared/` becomes `agent-harness-shared/`;
  all six member packages take `agent-harness-shared-*` names,
  including the two previously named `module-current-time-context` and `module-terminal-title`.
- Rust crate names are out of scope.
  Crate names are binary identities (`file-manager`,
  `terminal`),
  none are published,
   and path-derived names would worsen user-facing binaries.
- Enforcement is documentation only:
  two new tagged `AGENTS.md` rules,
   no file-enforcer check.
  (Recommendation during grilling was a file-enforcer check for the mechanical invariant;
  the user chose rules only.)
- Execution is per-family batches,
   each verified and committed before the next;
  auto-push is on,
   so every batch must be green.

## Rename inventory

Counts are from the decision-time measurement;
 re-enumerate before each batch.

### Package name renames (38)

- `pi-plugins/*` to `pi-plugin/*`:
   12 names `pi-*` to `pi-plugin-*`;
  leaf `thinking-defaults` becomes `thinking-default`.
- `claude-code-plugins/*` to `claude-code-plugin/*`:
   11 names;
  leaves `stop-reminders` and `hook-types` become `stop-reminder` and `hook-type`.
- `oxlint-plugins/*` to `oxlint-plugin/*`:
   6 names `config-oxlint-*` to `oxlint-plugin-*`.
- `agent-harnesses-shared/*` to `agent-harness-shared/*`:
   6 names.
- `git-policies/cli`:
  `cli-git` to `git-policy-cli`.
- `module/llm-types` to `module/llm-type`:
  `module-llm-types` to `module-llm-type`.
- `test-fixture/data-sequences` to `data-sequence`.

### Dir moves with no name change

- `figma-parsers/kiwi` to `figma/kiwi`;
  `figma-parsers/penpot` to `figma/to-penpot`.
- `rolldown-plugins/` to `rolldown-plugin/`.
- `intellij-plugins/` to `intellij-plugin/`.
- `ownership-markers/` to `ownership-marker/`.
- `git-policies/` to `git-policy/` (`api`,
  `repository`,
  `forbidden-strings` already named `git-policy-*`).

### Root and non-package dirs

- `packages/` to `package/` (final batch;
   see risks).
- `packages-deprecated/` to `package-deprecated/`.
- `docs/` to `doc/`;
  plural families `agents`,
  `decisions`,
  `limitations` singularize;
  nested plurals (`*-vet-reports`,
  `tools`) singularize.
- 50 `src`-internal plural dirs singularize:
  `rules`,
  `fixtures`,
  `handlers`,
  `parsers`,
  `operators`,
  `backends`,
  `filters`,
  `stubs`,
  `scripts`,
  `errors`,
  `types`,
  `components`,
  `styles`,
  `sessions`,
  `screenshots`,
  `pages`,
  `assets`,
  `seeds`,
  `dictionaries`,
   and peers.
  Line-item vetoes (for example `assets`,
  `pages`) are welcome before that batch runs.

### Explicitly unchanged

Everything in the exemption decisions:
tool-mandated dirs,
 terms of art,
 skill identities,
 acronyms,
 non-plural `s`-enders,
and all Rust crate names.

## New AGENTS.md rule

One merged rule,
`SGD` (the user folded the planned `PNP` into it):
dir segments singular;
package name equals `@monochromatic-dev/` plus the package path with `/` replaced by `-`;
rename dir,
 name,
 and consumers together.
This doc is the rule's exemption inventory:
tool-mandated names,
 acronyms,
 skill identities,
established terms of art (`dotfiles`,
`forbidden-strings`,
`import-attributes`).

## Batch order

1.   This plan doc plus the two `AGENTS.md` rules;
      regenerate `CLAUDE.md` via file-enforcer.
2.   `src`-internal singularization,
      one commit per `packages/<category>`.
3.   Small families:
     `figma/`,
     `ownership-marker/`,
     `rolldown-plugin/`,
     `intellij-plugin/`.
4.   `git-policy/` including the `git-policy-cli` rename and GCW/CLG updates (GCW:
     retired from `AGENTS.md` 2026-09-29;
     the README section that held it was dropped 2026-10-13).
5.   `oxlint-plugin/` including `config/oxlint` consumers and the TSD rule mention.
6.   `agent-harness-shared/`.
7.   Leaf renames:
     `module/llm-type`,
     `test-fixture/data-sequence`.
8.   `pi-plugin/`.
9.   `claude-code-plugin/`,
      updating hook paths in harness config in the same commit;
     restart live sessions afterwards.
10.  `docs/` to `doc/`,
      updating `AGENTS.md` rules DPL,
      RBK,
      SK1 to SK3,
      DL1 to DL3
     and every skill file referencing `docs/` paths.
11.  `packages/` to `package/` and `packages-deprecated/` to `package-deprecated/`:
     `pnpm-workspace.yaml`,
      every `//packages/` mise address,
     `file-enforcer.config.ts` globs,
     workflows,
     `DEFAULT_ALLOWED_WORKTREE_DIRS` in the git guard,
     forbidden-strings config,
     `slopo.conf.yaml`.

## Per-batch procedure

- `git mv`;
   sweep old identifiers and paths with uncapped `rg` and sanity-check the sweep ran.
- Check new names against `forbidden-strings.append.local.txt` before adopting them.
- Update `package.json` names,
  `workspace:*` consumers,
   imports,
   tsconfig references.
- `pnpm install` to regenerate the lockfile;
   never hand-edit it.
- `mise run //package-path:lint:types` and tests for every touched package.
- Exercise user-facing artifacts at the boundary:
  hooks through the host harness,
  `git-policy-cli` via its real bin.
- Commit with explicit scoped pathspecs in the multi-package message format.

Proposed handling of renamed-away names:
add forbidden-strings entries for the ten retired category-level identifiers
(`pi-plugins`,
`claude-code-plugins`,
`oxlint-plugins`,
`agent-harnesses-shared`,
`figma`,
`git-policies`,
`intellij-plugin`,
`rolldown-plugin`,
`ownership-marker`,
`cli-git`)
rather than one entry per package name.

## Risks

- Hook configs reference `claude-code-plugins` build outputs by path;
  the batch that moves them breaks live hooks until config updates land,
  and running sessions need a restart.
- User-global harness config outside the repo may reference `packages/` paths;
  report anything found rather than editing silently.
- Historical handover and decision docs mention old names heavily;
  the sweep updates them so docs stay canonical,
   with this doc recording the mapping.
- The final `packages/` rename changes every mise task address;
  open terminals and muscle memory go stale at once.

## Execution deviations and residue

- The guard CLI's runtime identifiers stay `cli-git`
  (`cli-git.config.ts` discovery name,
  `.git/cli-git-transaction`,
  JSONL prefixes,
   logger tags,
   the `git cli-git` management subcommand):
  renaming them would invalidate user-global trust registries.
- Marketplace plugin `name` fields in `.claude-plugin/marketplace.json` stay:
  they are installed-plugin identities,
   like skill names.
- The rust linter's implementations dir became `src/builtin`
  (not `src/rule`) because `rule.rs` already owned the singular name.
- The `seeds` to `seed` rename initially bypassed the hardcoded final-newline
  exclusion prefix and the policy appended newlines to 3374 fuzz seed files;
  exclusions were updated and the corpus bytes restored,
   verified byte-identical.
- `package-paused/` internals keep their plural dirs:
   the tree is
  workspace-inert and frozen;
   only stale renamed-package references were swept.
- Forbidden-strings entries for retired identifiers were skipped by user decision.
- The `dev-script-file-enforcer` generated-policy-contracts test failed for a
  preexisting reason unrelated to the renames:
  commit b979593d5 (readonly-rule findings sweep,
   earlier the same day)
  removed the lazy-provider mutation contract from canonical policy sources
  while the test still asserted it.
  Fixed in f35655481:
   the dead generator strip is gone and the test now
  asserts both canonical and generated sources stay contract-free.
- User-global configs updated:
  `~/.pi/agent/settings.json` plugin paths and
  `~/.claude/settings.json` statusline path point at `package/...`.
