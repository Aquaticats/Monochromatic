# Gate probes for the Rust file-enforcer rewrite

Research for [`doc/decision/file-enforcer-rust-rewrite.md`](../decision/file-enforcer-rust-rewrite.md) ("Consequences", gates before implementation), 2026-10-29,
run in podman with disposable homes and caches; scratch under `${HOME}/temp/agent/fe-rust/gates/`,
mpm source clone at `${HOME}/temp/agent/mpm-8.2.0-20261029`.

Facts are marked F, inferences I.

## State of the run

All three probes finished. Nothing is running: container `wizardly_dubinsky` exited, no `fe-gate-*` unit is loaded, no image was built or pulled.

## Things created

- Scratch root (mode 700): `/var/home/user/temp/agent/fe-rust/gates/` with `p1/` (scanner build + fixtures), `p2/` (browserslist), `p3/` (mpm sandbox).
- Clone: `/var/home/user/temp/agent/mpm-8.2.0-20261029` (meta-package-manager tag v8.2.0, commit 99eb596b75f6cff8f6a7319af893c2beb8d4e90c).
- systemd unit: `fe-gate-p1build` (finished; unit was `--collect`ed). The first launch of this unit failed offline and was relaunched.
- Containers: all `--rm`, none left (the last one, `wizardly_dubinsky`, exited). No `fe-gate-*` unit remains.
- Images: none built or pulled. Reused pre-existing `localhost/cli-git-test-rust:git-2.56.0` and `docker.io/library/node:26.12.0-trixie-slim` (owner's; keep).
- Downloaded artifacts in `p3/dl/` (uv 0.13.6, pnpm 12.16.1, mpm 8.2.0 wheel); sha256 of all three match `toolchain.lock.json`.

## Probe 1: forbidden-strings rules input

Build (F): copy of `package/forbidden-strings/{cli,core}` into `gates/p1/src`, built in container
`localhost/cli-git-test-rust:git-2.56.0` (rustc 1.97.0), `--memory=2g --cpus=2`, `cargo build --release --locked`, network on
(the image sets `CARGO_NET_OFFLINE`; overridden), under `systemd-run --user --unit=fe-gate-p1build`, 3m14s. Scanner version v0.5.0.
Fixtures and script: `gates/p1/fx`, `gates/p1/probe1.sh`; full output `gates/p1/probe1.out`. Run with `--network=none`,
`FORBIDDEN_STRINGS_CACHE_DIR`, `HOME`, `XDG_CACHE_HOME` all inside the fixture; `/root/.cache` did not exist afterwards (F).
Rules use tail-format sections, planted literals longer than 8 bytes: v1 has only `oldrule`, v2 only `newrule`; one target holds both strings.

### (a) absent compiled cache compiles from text (F)

```
$ forbidden-strings --rules rules.txt target.txt      # cache dir empty
exit=1
stderr: {"type":"forbidden-strings/cache-warning","schemaVersion":1,"reason":"missing","recovery":"compile-from-text"}
        target.txt:2 rule=oldrule
cache afterwards: cache/forbidden-strings/v0.5.0/linux-x86_64/b0d42a09...5402/rules.bin
```

The planted string is found; findings and warnings go to stderr, stdout is empty (F; `src/lib.rs:796-797` prints warnings with `eprintln!`).
Rescan with unchanged text: same finding, no warning (warm path, `src/runtime_cache/mod.rs:75-76`).

### (b) changed rule with warm cache (F)

```
cp rules-v2.txt rules.txt    # cache warm for v1
exit=1  stderr: {"...","reason":"missing","recovery":"compile-from-text"}  target.txt:3 rule=newrule
cache afterwards: two digest dirs (b0d42a09... and 1dc2b421...)
rescan v2: target.txt:3 rule=newrule, no warning
revert to v1 text: target.txt:2 rule=oldrule, no warning (old artifact still warm)
```

Positive control satisfied: v1 reports only `oldrule`, v2 only `newrule`; the new-only string appears after the change and the old-only string stops appearing.
Mechanism (F, source): the cache path is `<root>/forbidden-strings/v<version>/<os>-<arch>/<sha256 of exact rules text>/rules.bin`
(`src/runtime_cache/path.rs:176-215`, digest at `:72`, `mod.rs:56`). A changed rule is therefore a cache miss under a new digest, never a stale hit.
`compile-rules --rules rules.txt` (`src/cli.rs:139-151`, `src/runtime_cache/mod.rs:147-177`) eagerly published the v2 artifact in a fresh cache dir
(digest 1dc2b421..., identical to the digest the scan produced), and the following scan printed only `rule=newrule` with no warning.
Cache root override must be absolute (`path.rs:116-122`, message at `:233`).

### (c) baseline for missing path and comma-joined value (F)

```
--rules missing.txt target.txt                 exit=2  forbidden-strings: read rules missing.txt: No such file or directory (os error 2)
--rules rules.txt missing-target.txt           exit=1  missing-target.txt: read error: No such file or directory (os error 2)
--rules rules-v1.txt,rules-v2.txt target.txt   exit=2  forbidden-strings: read rules rules-v1.txt,rules-v2.txt: No such file or directory (os error 2)
(after creating a file literally named "rules-v1.txt,rules-v2.txt")   exit=1  target.txt:2 rule=oldrule   -> the value is one filename
FORBIDDEN_STRINGS_RULES=missing.txt target.txt                       exit=2  same error as the flag
FORBIDDEN_STRINGS_RULES=rules-v1.txt,rules-v2.txt target.txt         exit=2  read rules rules-v1.txt,rules-v2.txt: No such file...
target.txt (no flag, no env, no default file)                        exit=2  read rules forbidden-strings.local.txt: No such file...
--builtin-rules clean.txt (no rules file anywhere)                   exit=0  baseline alone, no error
compile-rules --rules missing.txt / comma value                      exit=2  same read error
```

Source (F): `--rules` is one `Option<String>` (`src/cli.rs:209-215`); an explicit flag or env sets `explicit` (`src/lib.rs:662-663`);
`src/frx_load.rs:191-206` errors on any read failure except a missing implicit default under `--builtin-rules`.
There is no concept of an optional entry today: the single tolerated absence is the implicit default file combined with `--builtin-rules`
(`frx_load.rs:198-204`, help text `cli.rs:77-82`). A missing positional scan file is a finding-style line with exit 1, not exit 2.
The new comma-list-with-`?` syntax will change: comma value (one filename today), missing-entry behavior (hard error today),
and the exit code for a missing path (2 for rules, 1 for scan operands).

### Verdict, probe 1

Gate (live run on throwaway rules files, incl. a missing optional entry): PASS for (a) and (b); (c) is "baseline recorded", no pass/fail.
Limit: the scanner has no optional-entry feature, so "missing optional entry" can only be recorded as today's hard error until the new syntax exists (I).

## Probe 2: browserslist-rs on the real config

Real config (F): `.browserslistrc` has no section headers; its seven lines are default queries. A request for the "production section" therefore
falls through to the defaults in every arm (JS `selectBrowserslistQueries` does the same, `file-enforcer.config.ts:659-663`).
file-enforcer calls `browserslist(queries, {path: false, stats: {}})` (`file-enforcer.config.ts:944-957`).

Versions (F): crates.io `browserslist-rs` max version 0.21.2 (published 2026-09-16); its data crate `browserslist-data` 0.3.1 (published 2026-09-16, not on 0.3.x newer).
JS side from `pnpm-lock.yaml`: browserslist 4.29.5, caniuse-lite 1.0.30001819 (npm publish 2026-10-22; latest on npm is 1.0.30001820),
node-releases 2.0.60, electron-to-chromium 1.5.467, installed in `node:26.12.0-trixie-slim` (node v26.12.0) from `gates/p2/js/package.json`.
Rust: container `cli-git-test-rust`, `Cargo.toml` pins `browserslist-rs = "=0.21.2"`, `resolve` (low-level, no config or stats lookup) and `execute` arms,
files `gates/p2/proj/src/main.rs`, output `gates/p2/rust.out`.

Outputs (F):

```
Rust A resolve(7 explicit queries)         ["and_chr 151","and_ff 153","android 151","chrome 151","firefox 140","node 26.8.0" ... "node 26.0.0"]   (14 items)
Rust B execute(config file, env production)   identical to A
Rust C execute(path dir copy, env production) identical to A
JS task form browserslist(undefined,{path,env:'production'})   ["and_chr 156","and_ff 158","android 156","chrome 156","firefox 153","node 26.12.0" ... "node 26.0.0"] (18 items)
JS file-enforcer form (parseConfig, path:false, stats:{})        identical to the task form
main checkout .browserslistrc.resolved.local.json (read only copy)  identical to both JS arms (checked equal programmatically)
```

Differences, reported separately:

1. Browser data version (F): and_chr 151 vs 156, android 151 vs 156, chrome 151 vs 156, and_ff 153 vs 158. Cause (I): the Rust data crate snapshot (2026-09-16)
   is five weeks older than the locked caniuse-lite (2026-10-22). The embedded data has no recorded provenance file (`src/generated/info.txt` is 0 bytes).
2. Firefox ESR (F): Rust 140 (hard-coded in `browserslist-rs-0.21.2/src/queries/firefox_esr.rs:4`), JS 153 (hard-coded `FIREFOX_ESR_VERSION = '153'`, browserslist `index.js:15`). Different constants, not data lag alone.
3. Node data (F): Rust latest 26.8.0, JS and generated JSON 26.12.0, so `last 1 node major versions` yields 9 vs 13 versions. Lowest is 26.0.0 in both.
4. Query handling (F): all seven queries resolve in Rust, same browser families, same output shape and ordering (name, then version descending). No query-handling difference found.
   A standalone `not dead` errors in both ("cannot start with 'not'" in Rust, "Write any browsers query ... before `not dead`" in JS); inside the list it works in both.
5. Config handling (F): `browserslist-rs` keeps its config parser private (`mod config`); only `execute` reads files, and it also honors the `BROWSERSLIST` and `BROWSERSLIST_CONFIG` environment variables
   (`src/config/mod.rs`). `resolve` ignores them. A file-enforcer port should use `resolve` with its own line parsing to match the current `path:false, stats:{}` behavior (I).

TS conversion (F): `package/config/rolldown/src/browserslist-targets.ts` and `generated-file-exists.ts` copied to `gates/p2/ts`, run with node v26.12.0 type stripping, `exists` and `importJson` injected:

```
rust array  browser -> ["chrome151","firefox140"]   node -> ["node26.0.0"]
js array    browser -> ["chrome156","firefox153"]   node -> ["node26.0.0"]
control ['ie 11'] -> TypeError: Browserslist target is unsupported by tsdown target mapping: ie 11   (probe able to fail)
```

Verdict, probe 2: PASS on mechanics (the crate resolves the real config, JSON array format matches, the TS consumer produces rolldown-style targets).
FAIL on data parity today: output differs from the JS result and the committed generated JSON in four values (items 1 to 3), so a Rust port changes emitted build targets
(chrome151/firefox140 instead of chrome156/firefox153) until the data crate is refreshed or the difference is accepted.

## Probe 3: mpm outdated reading

Setup (F): sandbox `gates/p3` in container `node:26.12.0-trixie-slim` (no repo state): uv 0.13.6, pnpm 12.16.1 (both from lock URLs, sha256 verified),
mpm 8.2.0 wheel (lock sha256 verified) installed into a venv `p3/mpm-venv` on uv-managed CPython 3.13.18; its Python dependencies were resolved from PyPI now, not from a lock (I: may differ from the repo's `uvx` environment).
Env file `p3/env.sh` (HOME, PNPM_HOME, UV_TOOL_DIR, caches all under `/w` = `gates/p3`). Installed: pnpm global `cowsay@1.5.0` (latest 1.6.0), `mkdirp@3.0.1` (latest 3.0.1);
uv tools `pycowsay==0.0.0.1` (latest 0.0.0.2), `cowsay==6.1` (latest 6.1). `rimraf` is not installed.
The clone `mpm-8.2.0-20261029` at tag v8.2.0 has commit 99eb596b...; `doc/decision/mpm-toolchain-adoption.md` cites v8.2.0 as f095931a... (F: the two differ; I: tag moved or doc cites a pre-tag commit; not investigated further).

CLI shape (F): there is no `--output-format` option in 8.2.0 (`Error: No such option '--output-format'`); the JSON switch is the global `--table-format json` (`cli.py` option group, `mpm --help`).
Manager selectors `--pnpm`, `--uvx` precede the subcommand. Command forms used: `mpm --pnpm --uvx --table-format json outdated|installed [QUERY] [--exact]`.

Fields (F, source `cli_explore.py` `installed` ~431 and `outdated` ~517):

- Per manager key: `{"id", "name", "packages": [...], "errors": [...]}`.
- `installed` package: `id`, `name` (null), `installed_version`.
- `outdated` package: `id`, `name` (null), `installed_version`, `latest_version`. `installed` rows have no latest field.
- pnpm adapter maps `pnpm outdated --global --json` `current` to `installed_version` (may be None via `.get`) and `latest` to `latest_version` (`managers/pnpm.py`); uvx parses `uv tool list --outdated` lines `name vX [latest: Y]` (`managers/uv.py`, regexes at class UVX).
- `refiltered_outdated` (`manager.py`) additionally drops rows whose installed and latest compare equal.

Results (F):

```
installed older than latest:   outdated -> pnpm cowsay installed_version 1.5.0 latest_version 1.6.0 ; uvx pycowsay installed 0.0.0.1 latest 0.0.0.2
installed equal to latest:     outdated -> NO row for mkdirp (3.0.1) or uvx cowsay (6.1); installed -> mkdirp installed_version 3.0.1, cowsay 6.1
not installed (rimraf):        installed rimraf and outdated rimraf -> "packages": [] for both managers, errors []
pnpm binary absent from PATH (selected with --pnpm --uvx): exit 0, JSON has only the "uvx" key; the "pnpm" key is missing, no error entry; stderr empty at default verbosity
   (--verbosity INFO: "info:pnpm: Skipped: no executable named 'pnpm' found in PATH."); --stop-on-error: same, exit 0 (source pool.py:539-545)
only --pnpm selected and pnpm absent: exit 2, stderr "critical: No manager selected." (pool.py:585)
wrong/empty PNPM_HOME: pnpm packages [] errors [] for both installed and outdated, exit 0 (pnpm itself reports "No global packages found")
registry unreachable (container --network=none): outdated exit 0, packages [] for both managers, errors non-empty:
   pnpm "Timed out after 120s." ; uvx "error: Request failed after 3 retries ... dns error"; stderr warnings "Could not list outdated packages."
   installed still works offline with the correct versions. --stop-on-error did not change exit code (0) or output.
```

Extra cases (F), run last:

```
installed NEWER than registry latest: pnpm global yaml@3.0.0-2 (npm latest 2.9.1; dist-tag next is 3.0.0-2)
  mpm installed --exact yaml -> installed_version "3.0.0-2"
  mpm outdated  --exact yaml -> packages [] errors []        (no row)
  pnpm outdated --global --json (direct) lists only cowsay   (exit 1 because cowsay is outdated)
direct pnpm outdated while offline: answered from pnpm's local metadata cache (cowsay 1.5.0 -> 1.6.0, exit 1),
  while the same call under mpm timed out after 120s (I: cause not isolated; warns that "latest" can be a cached value)
```

yaml was removed from the sandbox afterwards.

## Candidate rule evaluation

Rule: "latest = outdated row's latest when present, otherwise the installed version mpm reports; compare with the lock; an absent or failed manager result is an error."

- Installed older than latest: HOLDS. `latest_version` comes from the row (cowsay 1.6.0, pycowsay 0.0.0.2).
- Installed equal to latest: HOLDS. No row, so latest = installed (mkdirp 3.0.1, cowsay 6.1).
- Tool not installed: DOES NOT HOLD as written. mpm returns `packages: []`, `errors: []`, exit 0 in both commands; the rule has no installed version to fall back to,
  so the tool must be treated as an error or sync trigger by file-enforcer, not as "equal".
- Manager missing (pnpm absent from PATH): DOES NOT HOLD unless file-enforcer checks the JSON itself. mpm exits 0, drops the manager key, writes nothing to stderr at default verbosity
  (also with `--stop-on-error`). Only a selection of that single manager exits 2 ("No manager selected."). So "absent manager result" must mean "key missing from the JSON".
- Manager failure (registry unreachable): DOES NOT HOLD as written. mpm exits 0, `packages` is empty, and the failure is only in the `errors` array and stderr warnings;
  the fallback "latest = installed" would call every tool up to date. "Failed" must mean non-empty `errors`.
- Wrong PNPM_HOME: silent empty `installed` and `outdated`, no error (F). Indistinguishable from nothing installed; same handling as not installed.
- Installed newer than registry latest (yaml 3.0.0-2 vs 2.9.1): FAILS. No row, so the rule reports latest = 3.0.0-2, which is not the registry's latest (2.9.1). mpm exposes no registry latest for rows it omits.
- Lock-relative answer when installed differs from the lock: derivable only when a row exists or installed equals the real latest (I, from the data):
  - installed < lock = latest: row gives latest, lock is current, machine is stale.
  - lock < installed < latest: row gives latest, lock is behind.
  - lock < installed = latest: no row, latest = installed, lock is behind (correct).
  - installed > latest (pre-release or moved dist-tag): not derivable from mpm; the fallback invents a latest.
  Because mpm lists only installed packages, a lock-relative answer is never derivable for a locked tool that is not installed.

Sufficient amended rule (I, untested end to end): latest = row `latest_version` when a row exists; otherwise, only if the manager key is present, its `errors` is empty,
and the tool appears in `installed`, treat installed as latest with a documented blind spot for installed-newer-than-latest; any other combination is an error. Where that blind spot matters, first versions come from the registry directly, as the decision already says for new tools.

## Verdicts

- Gate 1 (forbidden-strings live run on throwaway rules, incl. missing optional entry): PASS for cache behavior (a), (b); (c) baseline recorded. The optional-entry syntax does not exist yet.
- Gate 2 (browserslist-rs on the real config feeding the rolldown consumer): PASS on mechanics; data differs from the JS result in browser versions, Firefox ESR constant and node list (items 1 to 3), so build targets would change.
- Gate 3 (mpm outdated reading): the candidate rule FAILS as stated (not-installed, missing manager, failed manager, installed-newer-than-latest); it holds for older and equal. The amended rule above is the minimum that covers the observed cases.

## Cleanup

Container, units, processes: none left. No image built or pulled. Scratch is kept under `/var/home/user/temp/agent/fe-rust/gates/` and the clone at `/var/home/user/temp/agent/mpm-8.2.0-20261029` for the coordinator to inspect or delete.
