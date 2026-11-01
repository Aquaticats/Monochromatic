# Can committed .config/mise/conf.d replace the generated root mise.toml?

Research for [`doc/planning/file-enforcer-rust-rewrite.md`](../planning/file-enforcer-rust-rewrite.md),
gathered 2026-10-29 against mise 2026.10.20 source (tag `v2026.10.20`) and disposable fixtures with positive controls,
under `${HOME}/temp/agent/fe-rust/confd/`; F marks a fact, I an inference.

Note 2026-10-30: `package/forbidden-strings/cli/bin/container-snapshot.mjs` and `verify-evidence.mjs`, whose line
numbers this document cites, are TypeScript files now (`container-snapshot.ts` and `verify-evidence.ts`, with their
helpers split into sibling modules); the cited lines no longer exist at those numbers.


Date 2026-10-29. Installed mise: `/home/user/.local/bin/mise` 2026.10.20 (linux-x64, 2026-10-26).
Source: tag `v2026.10.20` (commit 8aceb19, "chore: release 2026.10.20") checked out as a worktree at
`/var/home/user/temp/agent/mise-src-v2026.10.20` (the older clone `mise-2026-10-29` is 2026.10.23-dev, so it was not used).
All source paths below are relative to that worktree.
Fixtures and scripts: `/var/home/user/temp/agent/fe-rust/confd/` (`lib.ts`, `s1.ts` to `s11.ts`, fixtures under `fx/`).
Every fixture run used `env -i`-style scrubbed env, a disposable `HOME`, disposable `MISE_{DATA,STATE,CACHE,CONFIG,SYSTEM_CONFIG}_DIR`
and `MISE_CEILING_PATHS` at the fixture root. The real repo's mise state, trust records and files were never touched.
Marks: **F** = fact (source line or fixture output), **I** = inference.

## Overall answer

**Works** for the proposed layout on mise 2026.10.20, with the required edits in section 7 and the caveats at the end.

The decisive test (`s3.ts`, `s3j.ts`): the repo's real `mise.no-env.toml`, the real generated `mise.toml` [env] block and all 249
real `package/*/*/mise.toml` were copied into two disposable fixtures.
Layout A is today's generated single `mise.toml`.
Layout B is `mise.toml` (= verbatim `mise.no-env.toml`) + `.config/mise/conf.d/env.toml` + `.config/mise/conf.d/bin.local.toml`.
Results A versus B, all byte-identical (sha1 equal):
- `mise env --json` at the root and inside `package/logger/core` (3510 bytes, 60 PATH entries in the same order).
- `mise tasks ls --all` (2887 lines) and the full `mise tasks --all --hidden --json` (2914 tasks, 3,200,681 bytes, even `config_source`).
- `mise run probe-root` (prints PATH), `mise run //package/logger/core:zz-probe` (prints `FORBIDDEN_STRINGS_RULES`), the same task run from the package dir,
  `mise x -- printenv PATH`, `mise exec -- printenv ...` (`SLINT_ENABLE_EXPERIMENTAL_FEATURES=1`, `SLINT_EMIT_DEBUG_INFO=1`, rules path).
- `mise trust --all` output (identical output in A and B).
- `mise hook-env -s bash`: every line identical except the opaque `__MISE_DIFF` and `__MISE_SESSION` blobs (they encode the config file list; **I**).
- The only other DIFFERENT field in `s3.ts` output is `mise config ls`, expected: B lists three files at the root (`mise.toml`, `env.toml`, `bin.local.toml`) where A lists one.

Positive control (layout C): same files but the committed file named `a-env.toml` (sorts before `bin.local.toml`).
`envRoot`, `envPkg`, `runRoot` and `x` all came out DIFFERENT, with the 60 package bins now ahead of `.cache/*` bins.
So the comparison can detect a PATH order change, and the identical A/B result is not a blind spot.

## 1. Which files load, and at what precedence

**F** Every pattern loads; there is no stop at the first match.
Source `src/config/mod.rs:2508-2545` (`shared_config_filenames`) lists, per directory, lowest precedence first:
`.config/mise/conf.d/*.toml` (2527), `.config/mise/conf.d/*/mise.toml`, `.config/mise/config.toml`, `.config/mise/mise.toml`,
`.config/mise.toml`, `.mise/conf.d/*.toml`, `.mise/config.toml`, `mise/conf.d/*.toml`, `mise/config.toml`, `mise.toml`, `.mise.toml`.
Then the `.local` family (`mod.rs:2553-2567`): `.config/mise/conf.d/*/mise.local.toml`, ..., `mise.local.toml`, `.mise.local.toml`.
`layer_config_filenames` (`mod.rs:2608-2625`) concatenates shared, env, local, env-local.
`config_paths_in_dir_with_filenames` (`mod.rs:2877-2897`) runs every pattern through `flat_map`.
Docs agree: `docs/configuration.md:147-148` (table), `:174-185` (conf.d prose: "alphabetical order, before the directory's own config.toml").

**F** Fixture `s1.ts` (13 candidate files, each setting `WHO`, a unique `TAG_*`, and one `_.path` entry): all 13 loaded.
`mise config ls` order, lowest first:
`conf.d/a.toml`, `conf.d/m.local.toml`, `conf.d/z.toml`, `conf.d/b/mise.toml` (folder), `.config/mise/config.toml`, `.config/mise/mise.toml`,
`.config/mise.toml`, `.mise/conf.d/r.toml`, `mise/conf.d/q.toml`, `mise.toml`, `.mise.toml`, `conf.d/b/mise.local.toml`, `mise.local.toml`.
`WHO` resolved to `mise.local.toml`.
So: conf.d files are alphabetical (a, m.local, z), rank below every `mise.toml`/`mise.local.toml`/`.mise.toml`,
and `.config/mise/conf.d` is the lowest of the three project conf.d dirs.

**F** A `*.local.toml` inside conf.d is NOT local-precedence.
`m.local.toml` sorted between `a.toml` and `z.toml` (fixture), and the local list at `mod.rs:2553-2567` contains only `conf.d/*/mise.local.toml` (folder fragments), no `conf.d/*.local.toml`.
It loads through the shared pattern `conf.d/*.toml`. Gitignore status is independent: root `.gitignore:11` `*.local.*` ignores it (`git check-ignore -v` on the real repo printed that rule for `.config/mise/conf.d/bin.local.toml`; `env.toml` is not ignored).

**F** Dotted name caveat (`mod.rs:2767-2777`, `:2793-2802`, `:2804-2824`; `src/env.rs:107-115`; docs `configuration.md:183`, `configuration/environments.md:146-170`).
`node.tools.toml` is deprecated (warning today, env-only selector from 2027.8.10).
`name.local.toml` is explicitly unconditional: `conf_d_file_environment` strips `.local` and then needs another dot.
Fixture `s10.ts`: default run: `bin.local.toml` loads, no warning; `node.tools.toml` loads WITH the `dotted_conf_d_filename` warning.
`MISE_ENV_CONF_D=true` (the future default): `bin.local.toml` still loads, `node.tools.toml` is dropped (positive control that the future mode was exercised).
So `bin.local.toml` and `env.toml` are safe names across the 2027.8.10 flip.

**F** Hidden files in conf.d are skipped (`mod.rs:2735-2755`). Non-regular files are skipped.
**F** Write target: `mise set` / `mise use` never write into conf.d (`mod.rs:2915-2935` `first_config_file`; fixture `s9.ts`: `mise set NEWVAR=x` went to `mise.toml`, both conf.d files unchanged).

Verdict: proven.

## 2. `{{config_root}}`, relative `_.path`, relative `_.file` inside `conf.d/*.toml`

**F** `config_root` for `<repo>/.config/mise/conf.d/env.toml` is `<repo>`.
`src/config/config_file/config_root.rs:115-120`: `parent == "conf.d"` and grandparent `mise` and great-grandparent `.config` gives the great-great-grandparent, which is the repo root.
(The folder-fragment branch at `:70` applies only to `conf.d/<dir>/mise*.toml`, `mod.rs:2951`.)
Docs: `configuration.md:534` ("`~/src/app/.config/mise/conf.d/tools.toml` -> `~/src/app`").
**F** `{{config_root}}` is injected per directive from that function: `src/config/env_directive/mod.rs:291-293`.
**F** Relative `_.path`: `env_directive/mod.rs:206-212` (`normalize_path` joins a relative path onto the config root, strips `./`, expands `~/`), applied with the source's root at `:719-728`.
**F** Relative `_.file` (non-`find_up`): `env_directive/file.rs:207` `ctx.normalize_path(pattern)`, so also the config root. The context's `config_root` is set at `env_directive/mod.rs:546` from the same function.

**F** Fixture `s2.ts` (candidates that differ): `.config/mise/conf.d/env.toml` with
`CR="{{config_root}}"`, `_.file=[".env.plain.json", {path=".env.local.json", redact=true}]`, `_.path=["rel-bin","./dot-bin","{{config_root}}/tpl-bin"]`.
Decoys: a different plain JSON and a different sops-encrypted `.env.local.json` (real `sops` 3.13.3 + age key at the fixture's `~/.config/mise/age.txt`) in each of repo root, `.config`, `.config/mise`, `.config/mise/conf.d`.
Result from the repo root and from `repo/sub/deeper`: `CR=<repo>`, `CS=<repo>/.config/mise/conf.d/env.toml`, `PLAIN_WHICH=root`, `SOPS_WHICH=root` (decrypted),
PATH = `<repo>/rel-bin:<repo>/dot-bin:<repo>/tpl-bin`. Only the root candidate matched; the other three decoys did not.
Controls that the other candidate roots are distinguishable (`s1.ts`): a folder fragment `conf.d/b/mise.toml` resolves to `conf.d/b`, `.config/mise/mise.toml` resolves to `.config/mise`.

Entry-by-entry for the current generated `[env]` block (`mise.toml:1236-1350`, generator `file-enforcer.config.ts:691-781`):
- `_.file = [{ path = ".env.local.json", redact = true }]`: keeps working (root-relative; sops decrypt proven in `s2`).
- every `_.path` entry (`.cache/search-shadow-bin`, `.cache/mpm-shadow-bin`, `.cache/toolchain-bin`, `.cache/cli-git-bin`, `node_modules/.bin`, 55 `package/*/*/node_modules/.bin`): keeps working, root-relative (proven in `s3`, full 60-entry PATH identical).
- `SLINT_ENABLE_EXPERIMENTAL_FEATURES`, `SLINT_EMIT_DEBUG_INFO`: plain strings, no paths (`s3k.ts`: visible under `exec` and `x`).
- `FORBIDDEN_STRINGS_RULES = "{{config_root}}/.cache/forbidden-strings.rules.txt"`: keeps working, resolves to the absolute `<repo>/.cache/...` (`s3k.ts`).

Verdict: proven.

## 3. Merge of `[env]` and `_.path` across files; final PATH order

**F** Plain keys: the higher-precedence file wins (`WHO` in `s1`).
**F** `_.path` arrays from different files are concatenated, not replaced: each directive becomes an entry tagged with its source file
(`env_directive/mod.rs:550` `paths.push((path, source))`), then consecutive same-source groups are processed with the later source prepended
(`env_directive/mod.rs:719-733`, comment "paths should prepend r.env_paths"), so a higher-precedence file's entries come FIRST in PATH, and a file's own entries keep their written order.
Then `env_paths` go to the front of the inherited PATH: `src/toolset/toolset_paths.rs:134-135` and `:151-153`.
**F** Fixture `s1`: PATH order is exact reverse of `mise config ls` order between files.
Fixture `s4` layout B: `env.toml` (`bin1`) before `bin.local.toml` (`bin2`).
**F** `_.file` arrays in two fragments both apply (`s8.ts` second half: `ONE` and `TWO` both set).

So the split works only if `.cache/*` and `node_modules/.bin` live in the file that sorts LATER alphabetically (higher precedence), and the machine-specific package bins in the file that sorts EARLIER.
`bin.local.toml` < `env.toml` satisfies that. Numeric prefixes make the dependency explicit; fixture `s11.ts` with `20-env.toml` + `10-bin.local.toml` gave
`.cache/search-shadow-bin`, `node_modules/.bin`, `package/a/b/node_modules/.bin`, `package/c/d/node_modules/.bin`, `/usr/bin`, `/bin`, with empty stderr.

**F** Maintenance trap: `mise.no-env.toml` has no top-level `[env]` today (only per-task `[tasks."lint".env]` at `:767`).
If anyone later adds `[env]` `_.path` to `mise.toml` or `mise.local.toml`, that file outranks every conf.d fragment, so its entries land EARLIER in PATH than `.cache/search-shadow-bin` (`s1`: mise.toml-sourced entries precede conf.d-sourced ones). Keep PATH entries only in conf.d.
**F** Nonexistent PATH dirs are kept (`s1`: fixture dirs did not exist and were in PATH), so a stale `bin.local.toml` is harmless.
**F** Full 60-entry order reproduced exactly in `s3` (A equals B), and reversed in control C.

Verdict: proven (order depends on file name sort; document it next to the files).

## 4. Trust

**F** Trust is per config root, not per file (default mode).
`src/config/config_file/mod.rs:629-640` `config_trust_root` returns `config_root(path)`, which for conf.d files is the repo root (section 2).
`trust_check` (`:739-...`) and `is_path_trusted` use that root. `load_all_config_files` groups files by trust root (`src/config/mod.rs:4172-4190`).
**F** Fixture `s4.ts`:
- No trust record: layout A (`mise.toml`) and B (`conf.d`) both fail with "Config files in <first file> are not trusted. Trust them with `mise trust`" when non-interactive. Same behavior, only the named file differs.
- `mise trust` with no argument trusted `<repo>` in both layouts; afterwards a NEW conf.d file added later loaded without any prompt.
- Linked git worktree of a trusted main checkout loaded with no extra trust in both layouts (`config_file/mod.rs:966` `main_checkout_equivalent`).
  This is newer than the behavior recorded in `doc/troubleshooting/mise-disposable-worktree-trust.md` (mise 2026.7.0). **F** only for linked worktrees; **I** the old docs may be stale for this case.
- Fresh `git clone` in another path: untrusted in both layouts (needs `mise trust`, `mise trust --all`, or `MISE_TRUSTED_CONFIG_PATHS`).
- GOTCHA **F** (`s4.ts` 6a, `s4b.ts`): `MISE_TRUSTED_CONFIG_PATHS=<clone>/mise.toml` (the file spelling that `doc/troubleshooting/mise-untrusted-temporary-worktree-shell.md` recommends) trusts a plain `mise.toml` (control passes) but does NOT cover `.config/mise/conf.d/*.toml`: fails "conf.d/env.toml is not trusted".
  The directory spelling (`=<clone>`) works. Existing code already uses directories: `package/monorepo-task/core-exec/src/build-and-test-fixture.ts:197` (`root`), `package/env-local-json/core/src/env-local-json.unit.matrix.test.ts:79,91,239,269` (`REPO_ROOT`), `package/test-harness/core-matrix/README.md:138,144` (`/workspace`).
  Only that troubleshooting doc's file-path advice would break once conf.d exists.
- `Dockerfile:22` runs `mise trust --all`; unaffected.

**F** How the repo handles trust today: no rule in `AGENTS.md` (`rg -i trust AGENTS.md` finds only unrelated lines); `mise.no-env.toml:9-12` tells readers to run `mise trust` after cloning and that root trust covers `package/**/mise.toml`; the troubleshooting docs above; tests use `MISE_TRUSTED_CONFIG_PATHS`. Neither `mise.no-env.toml`, the user's `~/.config/mise/config.toml`, nor any package `mise.toml` sets `paranoid` (`rg -i paranoid`).

**F** Gating of `_.file` and templates: in default mode `_.file` is gated only by the config's own trust (`env_directive/file.rs:213-217` calls `env_file_trust_check_from`, which returns true unless `paranoid.env_files()`, `config_file/mod.rs:825-832`).
Templates are not separately gated; an untrusted file does not load at all. A config whose body is "safe" (tools and tasks only, `mise_toml.rs:751-776`, `is_safe_config_body` at `:2655`) loads without trust; `env.toml` has `[env]` so it needs trust, exactly as the old generated `mise.toml` did.
**F** If paranoid mode were enabled (`s8.ts`): trust becomes per file and content-bound (`config_trust_root` returns the file path), so a bare `mise trust` trusted only `mise.toml`, `mise trust --all` trusted each conf.d file and each `_.file` target, and rewriting the generated `bin.local.toml` would invalidate its trust each time. Not enabled in the repo.

Verdict: proven-with-caveat (same prompts as today; fix the file-spelled `MISE_TRUSTED_CONFIG_PATHS` doc advice).

## 5. Tasks and monorepo addressing

**F** Tasks are NOT limited to `mise.toml`: tasks defined in `conf.d/*.toml` are discovered and run (`s6.ts`: `//:fromconfd` listed and ran with the fragment's env). Source: PR #6792 (2025-10-28, `configs_at_root` expands conf.d globs). The proposed layout keeps tasks in `mise.toml`, so this only matters if tasks are later moved.
**F** Monorepo keys also work from a fragment (`s6.ts` second fixture: `monorepo_root = true`, `[monorepo]`, `[settings]` in `conf.d/mono.toml`; `mise run //package/a/b:pt` ran). `src/config/mod.rs:2221-2240` `find_monorepo_config` / `resolve_monorepo_config_at_root` merge `[monorepo]` across all configs at the root. Not needed for the proposal, since `mise.toml` keeps them.
**F** Package-level configs still see the root env: `s6` `config ls` in `package/a/b` lists `<repo>/.config/mise/conf.d/env.toml`, `<repo>/mise.toml`, `package/a/b/mise.toml`; the package task printed `FROM_CONFD=yes PKGVAR=pkg` and the conf.d PATH entry. With the real 249 packages (`s3`): `envPkg` identical to layout A.
**F** Task env vars used by `mise.no-env.toml:233-238` (`MISE_MONOREPO_ROOT`, `MISE_CONFIG_ROOT`) are unchanged (`s7.ts`): root task and conf.d task both `MONOREPO_ROOT=<repo> CONFIG_ROOT=<repo>`; package task `CONFIG_ROOT=<repo>/package/a/b MONOREPO_ROOT=<repo>`; source `src/task/task_executor.rs:2832-2882`.
But the fallback in that script, `existsSync(join(configRoot, 'mise.no-env.toml'))` (`mise.no-env.toml:238`), becomes false at the root after the rename and falls to `resolve(configRoot,'..','..','..')`. It is only reached when `MISE_MONOREPO_ROOT` is unset (non-monorepo invocation), but it must be edited.

Verdict: proven.

## 6. `mise env`, `exec`, `run`, `x`, IDE, shell hooks

**F** `mise env --json`, `mise exec`, `mise x`, `mise run` (root task, package task by `//` address, package task from package dir): identical A versus B (section "Overall answer").
IDE path: JetBrains launches `mise exec -- slint-lsp` (comment at `mise.toml:1326-1333`); the equivalent `mise exec -- printenv SLINT_ENABLE_EXPERIMENTAL_FEATURES` printed `1` in B. **I** JetBrains itself was not run; the plugin's own parsing of `mise.toml` (if any) is unproven.
**F** Shell hook (`s5.ts`, one bash process re-evaluating `mise hook-env -s bash` as a prompt would): a NEW conf.d file, an in-place edit, an atomic replace (dotfile + `mv`), and a delete were each picked up on the next hook-env; control edit of root `mise.toml` too.
Source: `src/hook_env.rs:626-660` `config_search_dir_mtimes` watches the mtimes of the conf.d dirs among the `DEFAULT_CONFIG_FILENAMES` parents; editing a loaded file is covered by the watch-file mtimes.
**F** `mise hook-env` output equal to A except the opaque session blobs (section "Overall answer").

Verdict: proven (IDE plugin internals unproven).

## 7. Things in the repo that read or name `mise.toml` / `mise.no-env.toml` and what breaks

Break (must edit):
- `file-enforcer.config.ts:686-781` `generateMiseToml`, and the call at `:2505-2511` (`manageMiseManifests` glob `./mise.no-env.toml`): remove or repoint (generate only `bin.local.toml`; glob becomes `./mise.toml`).
- `package/monorepo-task/core-exec/src/build-and-test-fixture-config.ts:8-28` reads both `../../../../mise.toml` (`generatedConfig`) and `../../../../mise.no-env.toml` (`canonicalConfig`): the second `readFile` rejects (ENOENT) at import; and `package/monorepo-task/core-exec/src/build-and-test.unit.test.ts:38-43` asserts `generatedConfig.startsWith("# Generated from mise.no-env.toml ...")`. Both must change. The slicing at `build-and-test-fixture-config.ts:105-115` reads task text from `generatedConfig` and keeps working if it points at the new `mise.toml`.
- `mise.no-env.toml:238` (`existsSync(join(configRoot, 'mise.no-env.toml'))`): see section 5.
- `doc/troubleshooting/mise-untrusted-temporary-worktree-shell.md` (file-spelled `MISE_TRUSTED_CONFIG_PATHS`): see section 4.

Stale text only (no behavior): `mise.toml` header lines 1-4 and `mise.no-env.toml:3-4`; comments in `task.ts:379`; `package/music-player/android-app/mise.toml:8,51`, `package/jsonc-edit/core-rust.fuzz/mise.toml:2`, `package/forbidden-strings/{core.fuzz,cli.fuzz}/mise.toml:2`, `package/linter/kotlin/mise.toml:3`; READMEs `package/forbidden-strings/cli.fuzz/README.md:30,37`, `package/linter/monochromatic-lint-closure/README.md:186`, `package/linter/kotlin/README.md:54`, `package/file-enforcer/core/DECISION.rust-migration.md:88`, `package/file-enforcer/core/src/toolchain/{plan.tools.data.ts:2,5,18,1031; types.ts:198; lock.ts:46}`; `.idea/jsonSchemas.xml:26,32` maps a schema to `mise.toml` and `mise.no-env.toml` and has no mapping for conf.d files (cosmetic). 114 files in total mention `mise.no-env` (`rg -l`, includes `doc/`).

No code matches the untrusted-config error text, which now names `<repo>/.config/mise/conf.d/env.toml` instead of `<repo>/mise.toml` (`s4`):
`rg -n 'are not trusted|Config files in|not trusted' package .claude .agents` over ts, rs, mjs, sh, json, jsonc (excluding node_modules, dist, bundle) ran and returned only unrelated comments plus the generic fixture string `mise ERROR not trusted` in `package/ide/app/src/language/notice_tests.rs:157,169` (no path in it).

Unaffected (read package-level `mise.toml`, or only the root `[monorepo]` header, or only comments):
- `package/fs-path/core/src/root-marker.ts:81-109` reads `${dir}/mise.toml` for a `[monorepo]` table header: still true, since `mise.toml` keeps `[monorepo]` (`mise.no-env.toml:19`).
- `package/config/pnpr/src/publish-steps.ts:54`, `package/forbidden-strings/cli/bin/{container-snapshot.mjs:189,219; verify-evidence.mjs:123,250,258}`: package-level `mise.toml`.
- `package/test-harness/core-diagnostic/src/format-error.ts:50,84`: comment about `[monorepo]`.
- `cli-git.config.jsonc:18`, `file-enforcer.config.ts:857`, `package/git-policy/cli/src/native/config_parse_acceptance_tests.rs:114`: comments; the code reads the env var `FORBIDDEN_STRINGS_RULES` or the `.cache` path, not the file.
- `.github/workflows/*` and `.github/actions/task-cli-setup/action.yml`: no mise config reads (the actions are "Mise-free"; `rg -i mise .github` finds only unrelated words).
- `Dockerfile:22` (`mise trust --all`), `docker-compose.yml:18,26` (`mise run ...`): unaffected. `package/mutation-test/cli/runtime/containerignore:10` excludes `mise.local.toml` from a build context; **I** add `.config/mise/conf.d/bin.local.toml` there if the context should not carry machine-specific bin dirs (harmless if left).
- Root `.mise/` exists but holds only `.mise/locks/...`; no `.mise/conf.d`, `.mise/config.toml`, `mise/` or `.config/` at the repo root, so no existing file collides with the new conf.d dir (`ls -a`, `find .mise`).

## 8. Version sensitivity, status, upstream issues

**F** Project `.config/mise/conf.d/*.toml` is in the discovery list at tag v2024.12.0 and absent at v2024.11.0 (GitHub raw `src/config/mod.rs`, `rg conf\.d`). Global conf.d: PR #3439 (2024.12.5, `CHANGELOG.md:15979`).
Project `.mise/conf.d` and `mise/conf.d` came later (#12061 2026.8.7; #12395 2026.8.13); `.config/mise/conf.d` needs none of that.
**F** conf.d is documented, not marked experimental: `docs/configuration.md:174-185`, listed in the file table at `:147-148`. Only two migration notes: dotted names (section 1) and folder fragments (a different, newer feature).
**F** Task discovery from `.config/mise/conf.d` fixed in 2025.10.20 (#6792), irrelevant here since tasks stay in `mise.toml`.
**F** `gh search issues --repo jdx/mise` for `conf.d` combined with `config_root`, `_.path`, `monorepo`, `trust`, `hook-env`, `worktree`, `precedence`, `relative path`, `env`, `fragment`: no open issues; closed hits (#1444 feature request, #3429, #15129) are unrelated to this layout.
Discussions (GraphQL search `repo:jdx/mise conf.d`): #12276 (closed) "loaded in reverse alphabetical order": answered as load order alphabetical, hook execution order reversed; relevant to `_.path` only in that later file = higher precedence = earlier in PATH (matches our fixtures);
#12564 (open) "conf.d env resolution problem": a conf.d `[env]` template that references `{{ env.X }}` defined in `mise.toml` fails because conf.d is evaluated before the higher-precedence `mise.toml`. Our `env.toml` references only `{{config_root}}`, so unaffected, but keep any env-referencing template out of conf.d if it depends on `mise.toml` values.
#13652 (open) concerns folder fragments in global tasks only.

Verdict: proven-with-caveat (feature has existed since 2024.12.0, is documented stable; version drift risks are the dotted-name migration and no more).

## Verdict per behavior

- conf.d loaded for a project, all files load, alphabetical, below `mise.toml`/`mise.local.toml`/`.mise.toml`: proven.
- `conf.d/*.local.toml` loads but has no local precedence: proven (name it `bin.local.toml` only for the `*.local.*` gitignore).
- `{{config_root}}`, relative `_.path`, relative `_.file` (plain and sops) resolve to the repo root: proven.
- `_.path` concatenation and final order (cache bins, `node_modules/.bin`, package bins): proven; depends on file name sort.
- Trust: proven-with-caveat (per config root; file-spelled `MISE_TRUSTED_CONFIG_PATHS` does not cover conf.d; paranoid mode would change it, not enabled).
- Tasks and `mise run //package/...:task` with root config split: proven (A versus B full task JSON identical).
- `env`, `exec`, `x`, `run`, hook-env incl. new/edited/deleted fragments: proven. JetBrains plugin internals: unproven (only the CLI path it uses was run).
- Repo readers: two breakages (core-exec fixture config + test; `mise.no-env.toml:238` fallback) plus generator; others comments.
- Version: proven-with-caveat.

## What the files must look like

`mise.toml`: `mise.no-env.toml` renamed, unchanged (keeps `monorepo_root = true`, `[monorepo]`, `[settings]`, `[vars]`, `[hooks]`, tasks).
Edit its `:238` fallback and drop the "Generated" header.

`.config/mise/conf.d/20-env.toml` (committed; name must sort AFTER the generated file; `env.toml` also works, `bin.local.toml` < `env.toml`):

```toml
# .config/mise/conf.d/20-env.toml
[env]
_.file = [{ path = ".env.local.json", redact = true }]
_.path = [
  ".cache/search-shadow-bin",
  ".cache/mpm-shadow-bin",
  ".cache/toolchain-bin",
  ".cache/cli-git-bin",
  "node_modules/.bin",
]
SLINT_ENABLE_EXPERIMENTAL_FEATURES = "1"
SLINT_EMIT_DEBUG_INFO = "1"
FORBIDDEN_STRINGS_RULES = "{{config_root}}/.cache/forbidden-strings.rules.txt"
```

`.config/mise/conf.d/10-bin.local.toml` (generated, gitignored by `.gitignore:11`; sorts BEFORE the committed file so its entries land later in PATH):

```toml
# .config/mise/conf.d/10-bin.local.toml
[env]
_.path = [
  "package/agent-harness-shared/advisor/node_modules/.bin",
  # ... every existing non-empty package/*/*/node_modules/.bin, sorted as today
]
```

Both names are unconditional under today's and the 2027.8.10 `env_conf_d` rules (`s10`, `s11`).

## Caveats and unknowns

- **F** A fresh clone or worktree has no `bin.local.toml` until the generator (or an install hook) writes it, so package bins are missing from PATH until then; today they are committed. Nonexistent dirs are harmless, an absent file is also harmless (loads fine, `s4` B worktree).
- **F** Linked worktrees inherit trust from the main checkout (2026.10.20); fresh clones and `/tmp`-style disposable copies do not. Use the directory spelling of `MISE_TRUSTED_CONFIG_PATHS`.
- **I** Not tested: the generator writing `bin.local.toml` non-atomically while another mise process reads it (write to a dotfile or temp name then rename; hidden conf.d files are skipped, `mod.rs:2735-2755`, and `s5` step 4 showed that rename pattern is picked up).
- **I** Not tested: Windows path handling, `mise.lock` interaction (no `[tools]` in this layout; lockfile for conf.d would sit at `.config/mise/mise.lock`, `src/lockfile.rs:2532-2561`), JetBrains' own TOML parsing.
- Behavior was verified on 2026.10.20 only. The dotted-name warning and `env_conf_d` flip (2027.8.10) are the scheduled changes that touch conf.d filenames.
- The earlier fixture note in `doc/research/file-enforcer-rust-rewrite-replacements.md:114` ("Probe: `.config/mise/conf.d/env.toml` `[env]` value was visible") is now backed by source and the A/B equivalence runs above.
- Housekeeping: no background process was started; fixtures remain under `/var/home/user/temp/agent/fe-rust/confd/fx/`; a git worktree of the mise clone was added at `/var/home/user/temp/agent/mise-src-v2026.10.20`.
