# Rust file-enforcer rewrite handover

## Purpose

Carry the Rust + clap + JSONC file-enforcer rewrite across the user's reboot
and into implementation.
The design is accepted:
[`doc/decision/file-enforcer-rust-rewrite.md`](../decision/file-enforcer-rust-rewrite.md).
Every question,
answer,
and its history is in
[`doc/planning/file-enforcer-rust-rewrite.md`](../planning/file-enforcer-rust-rewrite.md).
Implementation is tracked in issue #1818.

## State on 2026-10-29

- The decision record is accepted and committed,
  with the dependency approvals (`toml_edit`,
  `browserslist-rs`,
  `schemars`,
  `serde_json`) and the in-repo XML edit crate.
- All three gate probes finished
  ([`doc/research/file-enforcer-rust-rewrite-gate-probes.md`](../research/file-enforcer-rust-rewrite-gate-probes.md)),
  and the user then chose direct registry lookups for every npm and PyPI version.
- The four halted vet drafts are committed whole under `doc/audit/tech-file-enforcer-*-vet-2026-10-29.md`.
  Their scratch (crate downloads,
  clones,
  and extractions) stays under `${HOME}/temp/agent/`.
- No code exists yet.

## Update 2026-10-30
in-repo browserslist and parallel implementation

The oxc look is done
([`doc/research/file-enforcer-rust-rewrite-oxc-browserslist.md`](../research/file-enforcer-rust-rewrite-oxc-browserslist.md)),
and the user replaced both crates:
"Change of plans,
for browserslist,
we retrieve the data and write something conforming to the original browserslist ourselves,
only supporting the queries we use now,
throw on everything non-conforming with \"not implemented\"."
The decision record carries the change.
The user also said "Don't serialize aggressively paralleling work.",
so implementation areas now run as parallel worktree agents (limit five at once),
sharing the brief `${HOME}/temp/agent/fe-rust/brief-common.md`.

State (updated as agents finish;
each finished branch is reviewed and merged into `main`):

- Done and merged:
  pnpr owns `config.yaml` (task 24,
  merge `668311d986`;
  CI advisory drift step `93a95f4259`);
  browserslist subset specification
  (`doc/research/browserslist-subset-semantics.md`,
  `f1778271d5`);
  dependency-catalog Rust library `package/dependency-catalog/core-native` (task 21,
  merges `6aae0afe1c` and `b7798a4da5` for the closure fixes);
  XML edit crate `package/xml-edit/core-rust` (task 26,
  `032366654f`);
  rules list with `?` (task 23,
  `2ebe1dd93a`,
  now owned by session monochromatic-7a);
  in-repo browserslist `package/browserslist/core-rust` (task 28,
  `c60687b1d8`,
  then the Node pre-release filter with node-releases 2.0.62 and
  `doc/troubleshooting/browserslist-node-major-prerelease.md`);
  file-enforcer `core-rust` and `cli` (task 20,
  `71d0ad760b`);
  mpm-shadow toolchain commands (task 22,
  first pass,
  then merge `bca46afa8b`:
  `request` is `<origin>@<range>`,
  the lock is derived from the plan alone,
  every resolver is ported,
  and the crate's lint debt is cleared);
  the linter's Dylint driver skip (`package/linter/monochromatic-lint`,
  after the owner-approved toolchain sync and driver warm on 2026-10-30).
- Merged 2026-10-31 through the core-hardening branch:
  branch `worktree-agent-a23c3740c67ba36d9` wires the `rust-catalog`,
  `browserslist`,
  and `jetbrains-harper` rule kinds in the cli crate and moves the browserslist lock to the root.
  `git merge` had refused because the main checkout held an uncommitted `pnpm-lock.yaml` of unknown authorship
  (13 catalog specifiers raised to their resolved versions);
  the owner chose "Commit it, then merge",
  so it is `add4429c97` on its own.
  monochromatic-c0 answered afterwards that the change was not theirs
  (it was already modified when that session started).
  Its scope deviations to keep in mind:
  `core-rust/src/plan.rs` no longer reports `rule-order` between two composing rules,
  so "rust-catalog after toml-keys" is a config convention;
  `package-licenses` moved after `rust-catalog` in `file-enforcer.config.jsonc`;
  the jetbrains-harper policy constants are hardcoded from the TypeScript tool;
  the browserslist `environment` field selects nothing because sections are refused.
- Merged 2026-10-31:
  branch `worktree-agent-ad604356a8bd46267` hardens the core and contains the wiring branch.
  It adds `parse_jsonc_with_spans` to `monochromatic-jsonc-edit` without a version bump (0.1.1 is published),
  so jsonc-edit needs a bump and a Trusted Publishing release before file-enforcer publishes.
  Every config decode error now reads `file:line:col: /pointer: message`.
  No `user watch` command exists (the decision lists only `user write` and `user check`).
- Running (worktree agents):
  loose ends
  (browserslist `bin/*.ts` oxlint errors with a real rerun of the scripts,
  `XDG_CONFIG_HOME` in JetBrains discovery,
  closure lints in the cli crate and the pre-existing ones in jsonc-edit and browserslist `fetch.rs`,
  clap-derived closures in `args.rs`,
  and the watch test that timed out once under load);
  first-publication prep for the five new crates
  (`cargo-publish.yml` jobs,
  package contents,
  dry runs and container builds of the unpacked crates,
  runbook sections;
  the owner token step is asked only once the dry runs pass);
  the cutover inventory `doc/planning/file-enforcer-rust-cutover.md`
  with a rehearsed toolchain plan conversion in scratch;
  version-keyed cargo-binstall installs in mpm-shadow
  (decided 2026-10-31,
  "Fix in mpm-shadow before cutover",
  after a sync in `/var/home/user/worktrees/tr-g245` with an older lock replaced the git wrapper in the shared `~/.cargo/bin`
  and broke git in every checkout;
  repaired the same night by an owner-approved `toolchain:sync` in the main checkout;
  see `doc/troubleshooting/toolchain-sync-older-lock-downgrades-shared-cargo-installs.md`).
  The cutover waits for this agent too.
  The user then ruled "No older repo-maintained tools should ever exist when newer is present.":
  repository-published tools are never downgraded and keep one newest copy
  (decision record,
  "Toolchain (mpm-shadow)");
  that agent was told,
  and a separate agent adds the same guard to the TypeScript sync in `package/file-enforcer/core/src/toolchain/`,
  which every tree uses until cutover.
  The user also wants a warning,
  with the upgrade remedy,
  on every run of such a tool in such a tree ("noisy is fine"):
  the mpm-shadow agent builds it as a launcher behind the exposed links,
  so it lands at cutover;
  until then the TypeScript sync warns only from `toolchain:sync` and `toolchain:status`
  (the user chose "Wait for the cutover" over releasing the launcher early).
- Merged 2026-10-31:
  loose ends (`9a0012f498`:
  browserslist scripts lint-clean with a real conformance rerun byte-identical,
  `XDG_CONFIG_HOME` in JetBrains discovery,
  closure lints cleared in the cli,
  browserslist and jsonc-edit,
  the watch race fixed by rearming instead of a longer wait);
  the TypeScript sync guard (`aa74110914`:
  a `monochromatic-` cargo-binstall tool installed newer than the lock is kept with a warning and the upgrade remedy);
  the cutover inventory (`5229c9fc69`,
  `doc/planning/file-enforcer-rust-cutover.md`);
  publish prep (jsonc-edit 0.1.2 and the five crates' metadata,
  publish tasks,
  `cargo-publish.yml` jobs,
  runbook "The five file-enforcer crates 0.1.0").
- Incident 2026-10-31:
  two agent runs of `mise run file-enforcer` (23:52:20 and 00:09:55) applied the TypeScript tool's user-level rule and rewrote
  `~/.config/JetBrains/IntelliJIdea2026.3/options/LanguageServersSettings.xml`.
  The user chose "Leave it as written",
  and for the cause (AGENTS.md `WC2` says to run the tool,
  which has no project-only mode) chose "Wait for the cutover";
  the shared agent brief now forbids agents to run it.
- Publishing,
  as the user chose ("Merge now, then token"):
  the publish-prep merge lets CI release jsonc-edit 0.1.2,
  with one expected red run for the five new crates;
  then the owner creates one token (publish-new,
  publish-update,
  trusted-publishing) and runs `cargo login` in their own terminal;
  then this session publishes xml-edit,
  dependency-catalog,
  browserslist,
  file-enforcer,
  and file-enforcer-cli in that order,
  creates the five Trusted Publishing configs,
  and the owner revokes the token.
  mpm-shadow needs its own bump past 0.1.1 after the version-keyed install agent merges.
- Adopted from the decision record and recorded unasked (veto open):
  the CLI binaries reach CI and containers the way the decision's bootstrap says
  (a prebuilt `mpm-shadow` archive with its SHA-256 in the lock,
  then `cargo:monochromatic-file-enforcer-cli` through cargo-binstall from its release archives,
  not a tag-pinned `github:` request,
  which AGENTS.md `PIN` forbids for tools this repository publishes);
  the IDE launcher's explicit path is `<worktree>/.cache/toolchain-bin/mpm-shadow`.
- Merged 2026-10-31 as `75d2316005`:
  version-keyed cargo-binstall roots (`--root <toolchain root>/cargo/<crate>/<version>`),
  newest-only `current` links for `monochromatic-` crates,
  older-lock warnings on `sync`,
  `status`,
  `lock`,
  `update`,
  `bin-paths`,
  and on every run through per-tree launchers (`.cache/toolchain-launchers/`,
  about +0.57 ms per call),
  and `mpm-shadow prune --toolchain`;
  289 tests on `main`.
  Its open questions:
  trees git does not list are protected from `prune --toolchain` only by `--lock`
  (a per-sync consumer registry was not asked for);
  archive and `cargo-git` version directories also accumulate;
  no OS lock around the `current` swap.
- OpenSnitch,
  2026-10-31:
  the user's OpenSnitch auto-rejects unanswered prompts and saves `reject-always-simple-<path>` rules,
  which is how agent worktree builds and old toolchain paths got blocked.
  At the user's request the rustup rule is now `allow-always-simple-var-home-user-cargo-bin-rustup.json`
  (its action was already `allow`; only the name said reject).
- Running:
  a `package/wg` agent adds OpenSnitch allow rules for toolchain update paths and routes update traffic outside the tunnel
  ("We control the package/wg/* category. Make our wg auto make update traffics go through clearnet.");
  it stops before any live change and hands the apply commands to this session.
- Running:
  an mpm-shadow agent for the next item and for the registry prune that still reads `$CARGO_HOME/.crates.toml`.
- Formerly queued,
  now running:
  `mpm-shadow update <tool>...` and resolving a lock-less plan entry alone,
  without moving other rows
  (decision record,
  "Toolchain (mpm-shadow)",
  adopted 2026-10-31 under "follow everything pnpm offers").
- Global toolchain change 2026-10-31,
  at the user's request "Install Mistral Vibe globally for me managed by our mpm setup.":
  `pipx:mistral-vibe` 3.0.4 (`mpm-uvx`,
  bins `vibe`,
  `vibe-acp`,
  `vibe-app-server`) added to `~/.config/monochromatic-dev/toolchain/plan.tools.data.ts`;
  `toolchain:update:global` also moved 7 tools,
  and the user chose "Install all 8";
  pre-update copies of the plan and lock are in `${HOME}/temp/agent/vibe-install/`.
- Global plan conversion:
  the user delegated it on 2026-10-31
  ("Use your own judgement. I trust you'll do this step good enough autonomously."),
  so this session converts the real plan under `~/.config/monochromatic-dev/toolchain/` at cutover without asking,
  keeping the decision record's 0600 backup beside the original until the converted global sync is verified.
  A rehearsal on a copy (`${HOME}/temp/agent/fe-rust/global-rehearsal/input/`) runs first,
  including the `asset` hints the synthetic rehearsal could not settle
  (`llamacpp-rocm`,
  `openinterpreter`).
- Running 2026-10-31 (cutover pieces the inventory found unbuilt):
  a real-global-plan conversion rehearsal on the copy;
  `package/vm-builder/cli` gets its own process runner (inventory precondition 7);
  `file-enforcer check` gains a committed-versus-local outputs selector,
  replacing the inventory's `jq` stopgap for the four gitignored rules.
  Still unbuilt after these:
  CI and container delivery of the two binaries,
  the `20-env.toml` move,
  and the repository plan conversion in the switch commit.
- Published 2026-10-31:
  `monochromatic-xml-edit`,
  `-dependency-catalog`,
  `-browserslist`,
  `-file-enforcer` and `-file-enforcer-cli` 0.1.0
  (dependency-catalog first needed `44a00d1f99`:
  crates.io rejects keywords of 20 or more characters,
  which the dry run does not check),
  Trusted Publishing configs 29956 to 29960,
  the owner revoked the token,
  and this session ran `cargo logout`.
  CLI release run `41171845290` dispatched.
- Merged 2026-10-31:
  the vm-builder runner (`472bb6f502`),
  the global rehearsal with the GPU-build asset fix (`fc3da5e25d`),
  and `file-enforcer check|write --outputs committed|local|all` derived from `.gitignore` files.
  `yt-dlp` moves to `pipx:yt-dlp` in the global conversion (decided under the owner's delegation).
- Running:
  mpm-shadow pnpm-style `sync` (resolves new or changed entries, frozen in CI) on its update-by-tool branch,
  which also adds the README rule "pnpm answers first"
  (the owner wants that rule in package docs, not AGENTS.md);
  the wg agent.
- mpm-shadow pnpm-style `sync` built on branch `worktree-agent-a32535bbf12b77b95` (328 tests):
  resolves new or re-edited entries alone,
  frozen under `--frozen-lock` or a CI environment detected the way pnpm does (a port of ci-info's `isCI`, `CI=false` turns it off),
  `--no-frozen-lock` kept (pnpm's `--no-frozen-lockfile`).
  Settled by "pnpm answers first" without asking:
  `sync` with no lock creates it, frozen refuses;
  the same agent fixes the two line-break lint errors from `7c7ca78478`.
  Merge,
  then bump and release mpm-shadow,
  before the switch branch regenerates the repository lock.
- Renaming to pnpm verbs (decision record,
  "Toolchain (mpm-shadow)"):
  the mpm-shadow agent renames the crate's commands,
  the switch-branch agent renames the mise tasks and AGENTS.md `CM7`'s `toolchain:sync`.
- Release fix 2026-10-31:
  run `41172212354` failed packaging the cli against published core 0.1.0,
  which lacks `output_scope` (merged after publication);
  core and cli are bumped to 0.1.1 with the internal pin `=0.1.1`,
  and the push publishes both and creates `monochromatic-file-enforcer-cli-v0.1.1`.
  Done (owner approved the push after the auto-mode check blocked it as a public surface):
  `e570fd802d`,
  run `41172650724` green,
  the release has both Linux archives and the checksums file.
- Released 2026-10-31 (owner approved the push):
  `monochromatic-mpm-shadow` 0.2.0 (`8846e03d5c`,
  run `41173602862`,
  release `monochromatic-mpm-shadow-v0.2.0` with both Linux archives),
  carrying the pnpm names,
  pnpm-style install with offline-fixable edits applied,
  named update,
  version-keyed installs,
  launchers,
  and the `mpm --version`/`--help` shadow notice.
  Every release the cutover needs now exists;
  the switch branch regenerates the repository lock with it.
- Live 2026-10-31 (wg branch merged):
  `wg-quicker opensnitch-rules apply` created `000-allow-monochromatic-toolchain-install-roots`,
  `-exposure-links` and `-rust-tools` (precedence allows,
  daemon reloaded each at 05:57:12 with no compile warning);
  the exemption watcher was restarted on the new build (`watch-stop`/`watch-start fr-par-fr1 100 1000`),
  it lists `app-monochromatic\x2dupdates.slice`,
  and a request from that slice left via the direct address (`2607:f2c0:...`) while a normal one left via the tunnel (`2a02:6ea0:...`).
  The TypeScript sync and update now re-exec into that slice;
  mpm-shadow's slice re-exec (for `install`,
  `update`,
  `outdated`) is on `main` but not in the released 0.2.0.
  (Note 2026-10-31: both re-execs were removed from `main` by the merge `bc49bb812f`;
  the toolchain no longer knows about the slice.)
  Not done:
  the two auto-saved rejects for the removed worktree `agent-a23c3740c67ba36d9`'s `target/debug` binaries stay
  (harmless now that worktree is gone);
  the agent's broader proposals (system updates,
  general pnpm installs,
  checkout build outputs) were not asked for.
- Corrected 2026-10-31,
  user:
  "mpm-shadow should never determine itself goes through clearnet. Our wg set of pkgs should determine that!"
  The self-placement above (TypeScript sync/update and mpm-shadow re-executing into the slice) is wrong and is being removed;
  the wg packages detect update traffic alone,
  by the union of all three methods the user chose:
  a process watch that moves update entry points into the exempt slice,
  an update-host list routed around the tunnel for every program,
  and processes whose executable sits under the toolchain install roots.
  The wg agent rebuilds it;
  mpm-shadow 0.2.1 (approved) is held until the self-placement is gone from mpm-shadow,
  so it never ships.
  The OpenSnitch rules stay.
  Policy for the wg tools,
  user:
  "Anyone using our wg set of tools should be warned on start and the behavior shouldn't have a toggle.
  It should always enforce our policies, even clearnet."
  So no switch disables any bypass,
  every start warns exactly what leaves the tunnel,
  and a policy that cannot be enforced is reported loudly rather than degraded silently.
  This replaces an opt-in,
  fail-closed instruction this session had given the wg agent on its own.
- wg rework reported 2026-10-31 (branch `worktree-agent-a6f97f5da6f22de4d`, tip `66c06d6dcc`):
  the removal `8423cf1922` was merged alone into `main` as `bc49bb812f`,
  so the TypeScript sync/update and mpm-shadow no longer re-execute into the slice
  (`rg` over `package/file-enforcer` and `package/mpm-shadow` finds no slice,
  guard variable or tunnel reference,
  only unrelated package names such as `6tunnel` in `packages.generated.ts`).
  The four wg commits (`fdfed418ce`,
  `b9e6e1b67b`,
  `a2b8a0005a`,
  `66c06d6dcc`) wait for a separate reviewer
  (running on the fixed range `8dbd3e47da..worktree-agent-a6f97f5da6f22de4d`)
  and for the owner's answers.
  Measured 2026-10-31 with `${HOME}/temp/agent/fe-rust/host-overlap.ts`:
  the update-host list (21 hosts) resolves to 86 addresses
  and includes `github.com` and `api.github.com`,
  which the option text the owner chose did not name,
  so every program's GitHub traffic (browser,
  `gh`,
  `git push`) would leave the tunnel;
  of 31 everyday sites sampled,
  `docs.python.org` shares all 8 of its addresses with `pypi.org`,
  and `raw.githubusercontent.com` and `avatars.githubusercontent.com` share 4 of 8 with the release-asset hosts.
  Processes on the toolchain's `node` at that moment,
  which the toolchain-path method would move:
  `@deepseek-ai/dsh`,
  `@alibaba-group/open-code-review`,
  five `mvm` MCP servers,
  the `bash-output-filter` hooks,
  and a test fixture.
  `wg-quicker down` removes the nft kill switch (`package/wg/cli-quicker/src/tunnel-cleanup.ts`),
  so a down/up restart leaves the time between the two commands (not measured) unprotected;
  restarting only the exemption watcher has no gap.
  mpm-shadow 0.2.1 is dropped:
  since 0.2.0 mpm-shadow only gained and lost the self-placement,
  plus a style refactor (`8dbd3e47da`) and a docs edit (`c8655d6152`),
  so its version stays 0.2.0.
  Asked the owner:
  the host-list scope and how to apply live.
  Answered 2026-10-31:
  "Keep the list as built" (all 21 hosts,
  `github.com` and `api.github.com` included,
  shared-address sites accepted),
  and "Full tunnel restart now" (`wg-quicker down fr-par-fr1`,
  then `up`,
  accepting the unprotected moment between them),
  once the separate reviewer finishes with no blocking findings.
  The separate review finished 2026-10-31 with one blocker,
  so nothing went live and the four wg commits stay unmerged.
  Its findings,
  most severe first:
  1.  Blocker unless the resolver path is inside the tunnel:
      DNS answers are trusted without bound
      (no cap per host or in total,
      one concurrent `ip` spawn per address,
      additions logged only at debug).
      `setDns` (`package/wg/cli-quicker/src/tunnel.ts`) points resolution at the tunnel only when the config has `DNS =`;
      without it the physical link's resolver picks the answers.
      Sharpest case:
      an answer naming the tunnel's own DNS server (for example 1.1.1.1) sends all DNS out of the tunnel.
      The reserved-range list also misses TEST-NET-1/2/3,
      `192.0.0.0/24`,
      `198.18.0.0/15`,
      `::ffff:0:0/96` in hex form,
      `64:ff9b::/96` and `2002::/16`.
  2.  The start notice does not say rules carry whole shared CDN addresses.
  3.  `WG_QUICKER_EXEMPT_COMMAND` and `WG_QUICKER_EXEMPT_UID` cross sudo and act as a toggle
      (`/bin/true` as the companion leaves `up` succeeding with no process watch).
  4.  The process watch trusts names and paths any same-uid process controls
      (`exec -a mpm curl`, a binary copied under the toolchain root);
      a Flatpak or bwrap process without user-bus access might reach the slice this way (not exercised).
  5.  The toolchain-root rule covers every installed tool (node, bun, pnpm, java, android-sdk, llama.cpp, and more),
      which the notice does not say.
  6.  Failures after startup reach no person:
      the Rust watcher's log is swallowed by `up`,
      and the TypeScript refresh loop runs detached with its output ignored.
  7.  A failed adoption is never retried,
      and a process that exits mid-move is reported as not moved after a 2 s stall.
  8.  Minor ones:
      the `setpriv` gid,
      no start-time recheck for descendants,
      full scans counted by wakeups rather than time,
      the loop exiting on an error with no report,
      EACCES read as vanished,
      `CStr::from_ptr` on a possibly NULL home,
      adoption dropping the origin's CPU and memory limits,
      and the notice still claiming bypass with `Table = off`.
  Verified clean:
  netlink length and sender checks,
  constant `busctl`/`setpriv` arguments with a cleared environment,
  other users and root excluded,
  no panics in the new modules,
  host-rule ownership on `down`.
  Measured on this machine 2026-10-31:
  `fr-par-fr1` carries DNS servers (Cloudflare and Quad9,
  DNS over TLS) with routing domain `~.` and the default route;
  the Wi-Fi link only owns `lan`;
  `/etc/resolv.conf` is the `127.0.0.53` stub with `search lan`;
  `setDns` runs before the first host refresh;
  so here the answers come through the tunnel,
  but the tool must also hold for configs without `DNS =`.
  The wg agent was resumed with every finding except the resolver-trust choice,
  which is asked of the owner,
  plus a coordinator suspicion to settle from source:
  whether Node's `resolve4`/`resolve6` ever applies the `lan` search domain,
  which the LAN router answers.
  Owner's answer 2026-10-31 on resolver trust:
  "Tunnel DNS only":
  host rules use only answers from the tunnel's own `DNS =` servers,
  queried through the tunnel;
  a config with no `DNS =` gets no host rules and a loud POLICY NOT IN FORCE at every start and refresh,
  while the process watch and toolchain paths still run.
  Every option shown included refusing rules for the tunnel's DNS servers and endpoint,
  a cap with a loud report,
  and the missing reserved ranges.
  Sent to the wg agent.
  Next:
  review its fixes (separate reviewer again),
  merge,
  build both wg packages in `main`,
  then the full restart the owner chose and the live checks from the agent's first report.
  Two items were this session's own inferences,
  so under the new `BRQ` they were asked rather than briefed;
  owner's answers 2026-10-31:
  runtime failures "Notification + journal" (journal at error level plus a rate-limited desktop notification),
  and moved processes "Keep origin caps" (the new scope copies the origin unit's CPU and memory limits).
  The wg agent had built both as separate commits (`4295964a4d` and `023bf20581`;
  `01baa1f667` and `085e7acf48`),
  so both stay.
  wg fixes reported 2026-10-31,
  tip `0174850d6d`:
  host answers come only from systemd-resolved's varlink `ResolveRecord` on the tunnel link
  (keeps that link's DNS over TLS;
  Node's resolver is no longer used);
  caps 64 per host and 384 in total with a loud cut report
  (measured 132 answers,
  92 distinct,
  at most 24 for `registry.npmjs.org`);
  the tunnel's DNS servers and endpoint are protected
  (a late fix reads the second line `resolvectl dns` wraps onto when piped);
  `WG_QUICKER_EXEMPT_COMMAND` and `WG_QUICKER_EXEMPT_UID` removed,
  with a class test over every environment access form;
  the process watch matches the executable file name only,
  and skips and reports processes in another pid namespace
  (a `bwrap --unshare-pid` probe showed the old argv0 match could be forged from a sandbox);
  failed moves retry with backoff;
  startup failures are relayed into `up`.
  Not exercised:
  the root paths,
  notification and journal delivery end to end.
  Open from the agent:
  `WG_QUICKER_CALLER_PATH` still crosses sudo and,
  on a packaged install with no sibling build or secure-path binary,
  lets the caller's PATH choose the watcher binary,
  the same class as the removed overrides;
  sent back to the agent as following the owner's "shouldn't have a toggle".
- mpm-shadow 0.2.1 branch `mpm-shadow-0-2-1` (worktree `.claude/worktrees/mpm-shadow-0-2-1`,
  cut from `main` at `4998b72f2c`,
  tip `98541bde6c`,
  pushed as a branch,
  no tag):
  `options.tagPrefix` for `github:` requests (`eb202865fc`),
  the pooled-connection fix with `doc/troubleshooting/mpm-shadow-download-fails-with-sendrequest-after-a-long-install-step.md` (`30f1019822`),
  the archive route now covered by the newest-only rule (`current` link,
  older versions and their archives deleted,
  older-lock warnings on `install`,
  `list`,
  `update`,
  `outdated`,
  `bin` and every launcher run,
  `prune --toolchain`),
  and a self-upgrade fix (the launcher is copied through `/proc/self/exe`) (`9a81e783f5`),
  then the bump (`98541bde6c`).
  369 tests passed;
  an end-to-end run in a disposable home upgraded 0.1.1 to 0.2.0 and left only `current -> 0.2.0`.
  Merging the bump into `main` starts the release,
  so it waits for a separate review and the owner's approval.
- Dockerfile on the switch branch (tip `e792cbaa10`):
  the pre-branch Dockerfile (`048459c73d`) failed earlier,
  at the TypeScript sync (20 of 63 tools,
  no rustup or cargo);
  the branch installs all 64 tools,
  then fails at `mise run prepare` in `prepare:playwright`,
  a nested `podman build` that an unprivileged image build refuses
  (measured five ways:
  missing `newuidmap`,
  subuid range outside the build's IDs,
  `devpts` mount,
  chroot cgroup remount,
  `/proc` remount).
  The agent's options,
  ranked A > C > D > B:
  A replaces `RUN mise run prepare` with `mise run prepare:pnpm`;
  C builds the Playwright image at container start;
  D publishes it to a registry;
  B a privileged build.
  Asked of the owner.
  Also measured:
  inside the image the tree's own `mpm-shadow` link is the lock's 0.2.0,
  which cannot read `tagPrefix`,
  so the final build needs the released 0.2.1 in the lock.
- Owner's answers 2026-10-31 (question tool):
  the Dockerfile's Playwright step:
  "Publish to a registry",
  with GitHub Container Registry (`ghcr.io/aquaticats/monochromatic-playwright`,
  pushed by a GitHub Actions job with the workflow's `GITHUB_TOKEN`),
  "Public" (no login),
  "Tag by content hash" (`sha256(playwright.Dockerfile)[:16]`;
  CI pushes when that file changes;
  `prepare:playwright` pulls the same tag),
  and "Fall back to local build" when the pull fails.
  mpm-shadow 0.2.1:
  "Approve after clean review"
  (if the separate review finds no blockers and any should-fix items are fixed and re-checked,
  merge and let the release run,
  then verify the published binary).
  Sent to the switch agent:
  the publish workflow on its own branch `playwright-image-publish` from `main` first,
  the pull-then-fallback `prepare:playwright` on the switch branch,
  a measurement that `podman pull` works inside the unprivileged image build,
  and GitHub's docs on a new package's default visibility.
- From monochromatic-7a 2026-10-31:
  git wrapper 0.4.5 is published and installed as the production git
  (`git --version` appends `monochromatic-git-policy-cli 0.4.5, shadowing git at /usr/bin/git`),
  locked in `75ea3adee5`,
  which also moved mpm-shadow to 0.2.0,
  search-shadow to 0.2.1,
  binstall and slopo;
  a worktree with an older lock downgrades the shared installs on its next sync.
- Separate review of `mpm-shadow-0-2-1` (2026-10-31) found two blockers,
  so 0.2.1 is not released:
  tool ids and lock versions are never checked as one safe path segment
  (executed:
  a lock tool id `..` made `prune --toolchain` delete the whole scope root),
  and a half-extracted or broken version counts as finished
  (GNU tar gives the file its final mode before writing it;
  executed:
  a garbage newer bin made prune delete the working 0.2.0 and point `current` at the garbage).
  Should-fix:
  the `/proc/self/exe` launcher wiring untested and not gated off Linux;
  repository-published decided by tag prefix alone,
  not by `github:Aquaticats/Monochromatic`;
  tag matching takes the first listed match rather than the highest version,
  and a prefix not ending in `v` can match a sibling;
  stale cargo-only help text;
  the pooling doc lacks its hyper-util `pool.rs` citation.
  A fresh agent is fixing all of them on the same branch (version stays 0.2.1);
  the same reviewer re-checks the delta before the merge.
  Fixed 2026-11-01,
  tip `8e017174ea` (12 commits on `98541bde6c`;
  403 tests passed;
  each new guard shown to fail a named test when removed):
  names checked as one plain segment at decode,
  in the cleanup and in the path builders;
  archive installs unpack into a staging directory,
  are checked,
  then marked finished with `.mpm-shadow-installed`;
  repository-published means `github:Aquaticats/Monochromatic` plus the `monochromatic-` prefix;
  the highest whole version wins,
  with a message when the 10-page cap stops the search;
  downloads kept per version.
  Behaviour changes the author named:
  an unmarked archive directory from 0.2.0 or earlier is reinstalled once;
  `bin` and `node` stay usable as tool ids.
  The same reviewer is re-checking `98541bde6c..8e017174ea`,
  plus the crate's lint gate and what `cargo-publish.yml` runs before publishing.
  Re-check 2026-11-01:
  no blocker,
  F1 to F8 closed (F1 and F2 executed in disposable homes).
  Should-fix:
  `monochromatic-lint package/mpm-shadow/cli` exits 1 with 33 `markdown/semantic-line-breaks` errors the range introduced
  (the release workflow runs no lint,
  test or clippy step,
  only `cargo package`,
  `cargo publish --no-verify` and the two-architecture build with a `--version` smoke run).
  Minor:
  `publish_staged` keeps a marked directory whose program is gone,
  and the repository comparison is case-sensitive.
  All three sent back to the fix agent;
  then a short re-check,
  then the merge that releases 0.2.1.
  Fixed 2026-11-01,
  tip `5e301c751d` (lint task exit 0 with no findings,
  `lint:clippy` exit 0,
  406 tests passed);
  final re-check of `8e017174ea..5e301c751d` running.
  Final re-check clean (no blocker,
  no should-fix;
  lint exit 0 confirmed on an export with a positive control).
  Merged into `main` as `f345b6c271` and pushed by hand
  (a merge does not trigger the wrapper's auto-push;
  `main` was 21 commits ahead until `git push origin main`);
  release run `41262166385` started 2026-11-01T01:42:03Z.
  Released 2026-11-01:
  run `41262166385` succeeded (crate published,
  both architectures built,
  release created);
  crates.io `max_version` is 0.2.1;
  release `monochromatic-mpm-shadow-v0.2.1` has the x86_64 and aarch64 linux-gnu archives and `monochromatic-mpm-shadow-0.2.1-sha256sums.txt`;
  the x86_64 archive passed `sha256sum --check`,
  and its binary printed `mpm-shadow 0.2.1` in a throwaway home.
  The switch agent was told to run item 3 (merge `main` with `package/mpm-shadow` resolved to `main`'s,
  the lock regenerated with 0.2.1,
  the final no-overlay Dockerfile build).
- Second separate wg review (2026-10-31,
  range `8dbd3e47da..0174850d6d`):
  no blocker.
  Should-fix,
  sent to the wg agent:
  journal at warning instead of error level;
  `eprintln!` panics once the unbounded `/run` log fills,
  killing the watcher unreported;
  the notification rate limit keyed on text that embeds cgroup names,
  an unbounded map,
  and notifications that never expire;
  an unbounded exec-event drain;
  no total deadline or size cap on the resolved varlink reply;
  unescaped cgroup names in root-built lines;
  stalls of up to about 12 s;
  and minors (root `read_dir` following symlinks in user-writable roots,
  uncanonicalized roots,
  a fail-open pid-namespace skip,
  scope-name reuse,
  the per-host cap applied before filtering,
  five more reserved IPv6 ranges,
  a hardcoded tool list in the TypeScript notice,
  no report of the watcher's own death).
  The full restart waits for these fixes,
  so the owner's tunnel goes unprotected once rather than twice.
- Owner's answers 2026-10-31 on two config cases the reviewer raised:
  no `ExemptMark` "Keep as is",
  owner's note:
  "Warn if ExemptMark is unset but continue to up, because sometimes for some configs we want no exemptions.";
  `Table = off` "Keep loud not-in-force".
  Sent to the wg agent to record in the README so later reviews don't flag it.
- Playwright image (switch agent,
  2026-10-31):
  branch `playwright-image-publish` (worktree `.claude/worktrees/playwright-image-publish`,
  from `658b372099`,
  tip `590a8a5b4a`) adds `.github/workflows/playwright-image.yml` and `doc/runbook/publish-playwright-image.md`;
  the tag for today's `playwright.Dockerfile` is `e802ff364dc61464`.
  On the switch branch (tip `994cac6f4d`),
  `prepare:playwright` calls a new `task-playwright-image` bin in `package/monorepo-task/core-exec`:
  keep a local image of the tag,
  else pull from GHCR,
  else build locally with the printed reason.
  A new GHCR package is private by default,
  and only someone with admin rights can make it public in its settings,
  which GitHub documents as one-way;
  the REST API has no endpoint for it,
  so that step is the owner's after the first push.
  Measured:
  `podman pull` works inside the unprivileged image build only with empty subuid files plus `ignore_chown_errors`,
  and it added 8.4 GB that nothing in the image can run.
  Owner's answer:
  "Skip in repo image"
  (the Dockerfile runs `mise run prepare:pnpm` and drops the podman workaround;
  developers and CI still pull from GHCR);
  sent to the switch agent.
  A separate reviewer is checking the workflow branch before the merge.
- Overlap noted 2026-10-31:
  `doc/handover/repo-review-2026-10-31.md` (another session) already tracks `docker-compose.yml` running a nonexistent `ai-proxy:start` task
  (since the Moon-to-Mise migration),
  `.dockerignore` baking the local secrets appendix into images,
  and the Dockerfile exporting a nonexistent `.env.local`;
  left to that session.
- Switch branch 2026-11-01:
  the owner's "Skip in repo image" landed as `cc96e34a54`
  (`RUN mise run prepare:pnpm`;
  the vfs,
  subuid and `ignore_chown_errors` setup removed,
  and the `podman` apt package too,
  since nothing else in that image uses it:
  its only other user,
  the inference canary,
  is in `package-paused/`;
  owner confirmed "Remove podman" 2026-11-01).
  The image now skips `prepare:playwright` and `prepare:android`
  (the latter is `mpm-shadow install`,
  which the Dockerfile already runs).
  A separate reviewer started on the whole switch branch at the fixed range `82bc287a8c..cc96e34a54`
  (378 files,
  +3,209 / -87,302),
  so only item 3's delta (the lock regenerated with 0.2.1,
  `main` merged,
  the final build) needs a short check later.
- wg fixes for the second review reported 2026-11-01,
  tip `e1ef91dbb1` (12 commits on `0174850d6d`):
  journal at error level;
  non-panicking writes with a log budget (256 KiB an hour,
  4 MiB in total);
  rate limits keyed by failure kind,
  notifications that replace the previous one and expire after 10 minutes;
  a bounded exec-event drain;
  a 10 s total deadline and 1 MiB cap on the resolved varlink reply;
  one escape function for root-built lines;
  shorter stalls;
  the watcher binary resolved only from root-controlled locations (`WG_QUICKER_CALLER_PATH` no longer chooses it);
  sentinels that report a watcher's death;
  TypeScript runtime failures reaching the journal and notification;
  and a test that `up` continues after the `ExemptMark` and `Table = off` warnings,
  with the owner's words in the README.
  Both packages' `buildAndTest` passed (126 Rust tests;
  82 TypeScript suites).
  Not exercised:
  root paths,
  real journal and desktop delivery.
  The same reviewer is re-checking the delta `0174850d6d..e1ef91dbb1`;
  then merge,
  build both packages in `main`,
  the full restart,
  and the live checks
  (rules with pref 50 protocol 201,
  the slice,
  the watcher log,
  `journalctl --identifier=wg-quicker-exempt` and `--identifier=wg-quicker`,
  one sentinel per watcher).
- Delta re-check of `0174850d6d..e1ef91dbb1` (2026-11-01) found a blocker:
  `install_roots` accepts the canonical target of a linked install root with no home check,
  so a link from `.../monochromatic-dev/toolchain` to `/` made `classify(/usr/bin/curl)` return `ToolchainPath`
  (the reviewer compiled the real matcher),
  which would move every adoptable process of the user out of the tunnel.
  Should-fix:
  the per-kind repeat limit hides distinct failures of one kind,
  including startup outcomes in `up`'s output.
  Minor:
  the notification worker thread starts before the signals are blocked;
  sentinels share the watcher's cgroup;
  a short escape set;
  a directory swap window;
  the log budget silencing success lines.
  All sent to the wg agent;
  the restart waits for its fixes and one more delta check.
  Owner's answer 2026-11-01 on a missing watcher binary with `ExemptMark` set:
  "Abort up as before"
  (`up` fails and rolls back,
  as before this branch;
  the branch had made it continue with a loud error,
  which the owner never decided).
  Fixed 2026-11-01,
  tip `557ea24e24` (12 commits on `e1ef91dbb1`;
  156 Rust tests and 89 TypeScript suites passed;
  `monochromatic-lint` clean for both packages):
  linked install roots accepted only strictly below the resolved home,
  others reported as not in force;
  `WatcherMissingError` aborts and rolls back `up`;
  the reporter built after the signal descriptor;
  four distinct causes per kind per interval (global cap 32),
  startup reports unlimited in `up`'s output;
  wider escapes;
  sentinels as transient system services with `ManagedOOMPreference=omit`,
  confirmed by a marker.
  The author notes the sentinels run user-writable executables as root in long-lived services.
  The same reviewer is re-checking `e1ef91dbb1..557ea24e24`.
  Re-check 2026-11-01:
  no blocker.
  Should-fix (sent to the wg agent):
  the repeat limit's cause is cut at 200 characters after a fixed prefix of about 128,
  so different errors of one kind collapse.
  Minor:
  sentinels report "watcher died" at logout and reboot;
  `systemctl status` hint for a collected unit;
  the TS marker write follows symlinks;
  no timeout on `systemd-run`.
  Held for the owner:
  a link target anywhere strictly under the home is still accepted
  (a link to `~/.local/share` makes everything under it match).
- Live 2026-11-01 (the owner's full restart):
  `557ea24e24` merged into `main` as `3b3020c65a` and pushed;
  both packages built in `main`;
  the previous `dist` and watcher binary backed up to `${HOME}/temp/agent/wg-golive/backup`;
  `wg-quicker down fr-par-fr1` started 22:24:14 EDT,
  `wg-quicker up fr-par-fr1` exited 0 at 22:24:45
  (output in `${HOME}/temp/agent/wg-golive/up.log`).
  Verified:
  the POLICY NOTICE printed on start;
  86 update-host rules added
  (`ip rule` then showed 47 IPv4 and 49 IPv6 lines for table 52000,
  each family including the fwmark rule);
  `ip route get 151.101.2.137` (crates.io) leaves via `wlp9s0` table 52000,
  while 1.1.1.1 and 9.9.9.9 (the tunnel's DNS servers) stay on `fr-par-fr1`;
  toolchain `node` processes (`open-code-review`,
  the `mvm` MCP servers and others) now sit in `wg-quicker-update-<pid>-<start>.scope` under the slice;
  a request from the slice left from `2607:f2c0:...` (physical) and a normal one from `2a02:6ea0:...` (tunnel).
  Reported loudly at start,
  by design:
  about 18 matched processes not moved because each is the main process of another session's service
  (`cli-git-mutpar-*`,
  `rr-cent...`).
  Defect found live:
  both sentinels were started as services but did not confirm within 2000 ms,
  and afterwards no `wg-quicker*` unit or sentinel process exists,
  so a watcher's death would go unreported.
  System journal:
  both units failed at exec with "Unable to locate executable '<path under /var/home/user>': Permission denied",
  `status=203/EXEC`,
  started by PID 1 in `init_t`;
  SELinux refusing system services the execution of these files:
  `getenforce` is Enforcing,
  the node is labelled `data_home_t`,
  and the watcher binary `container_file_t:s0:c311,c738`
  (a container mount relabelled its `target/` directory);
  root's discretionary permissions cannot deny an exec,
  and no AVC record was logged for that minute (a dontaudit rule could explain it;
  unverified).
  Sent to the wg agent to confirm,
  write a troubleshooting doc,
  and find a placement that runs on this host,
  ideally applied without another tunnel gap).
- Separate review of the switch branch (2026-11-01,
  `82bc287a8c..cc96e34a54`):
  no new blocker.
  Verified clean:
  `file-enforcer 0.1.1 check --outputs committed` on an export exits 0,
  every old generator has a disposition,
  all 63 old tools carried with nothing this repository publishes pinned,
  no secret echoed in CI,
  the lock's archive checksums match the releases.
  Should-fix,
  sent to the switch agent:
  an existing checkout's `.cache/toolchain-bin/mpm-shadow` links to `~/.cargo/bin/mpm-shadow` 0.2.0,
  which exits 2 on the plan's `tagPrefix` even after the lock names 0.2.1,
  so the README needs an existing-checkout path;
  the branch's own tagPrefix and pooling commits duplicate the reviewed 0.2.1 code,
  so its `package/mpm-shadow` resolves to `main`'s in the item 3 merge;
  references to the Playwright workflow and runbook require `playwright-image-publish` to merge first;
  stale instruction-bearing docs;
  drift steps are `continue-on-error`,
  although the decision calls them a gate;
  `FILE_ENFORCER_IDLE_SECONDS` unset in CI and the Dockerfile.
  Also noted:
  `config.yaml` changes three lines (header,
  package count 170 to 169,
  the deleted `file-enforcer-core` leaves the trust list),
  so the merge redeploys pnpr and starts its publish run.
  Post-merge steps this session first listed
  (run 0.2.1 by absolute path for the first `install`,
  then remove `~/.cargo/bin/mpm-shadow` 0.2.0 by hand)
  are withdrawn.
  Corrected 2026-11-01,
  owner:
  "Shouldn't the old copies be removed automatically and deterministically?"
  Measured:
  `~/.cargo/bin` holds `cli-git-native` (production git,
  reached through `~/.local/bin/git` and `.cache/cli-git-bin/git`),
  `mpm-shadow` 0.2.0,
  `search-shadow`,
  `monochromatic-lint` and `monochromatic-nested-wayland-session`,
  placed by the retired TypeScript sync;
  `~/.cargo/.crates2.json` is stale (git wrapper listed as 0.2.13,
  no mpm-shadow entry).
  Owner's answers:
  bootstrap "Run the lock's version"
  (`toolchain:install` makes sure the exact mpm-shadow the lock names is installed,
  fetching the lock's archive and checking its sha256 if missing,
  then runs that copy,
  never the one on PATH;
  sent to the switch agent);
  old copies "Re-point, then remove"
  (after the managed newest copy is installed,
  links in `~/.local/bin`,
  every checkout's `.cache/*-bin` and the global farm that target an old copy are re-pointed to it,
  then the old copy and its cargo record are removed,
  each action printed);
  timing "Ship 0.2.1, add in 0.2.2"
  (0.2.1 releases after its re-check;
  a new agent builds 0.2.2 on branch `mpm-shadow-0-2-2` from `8e017174ea`;
  the switch merges only after its lock names 0.2.2).
  Lesson:
  an invariant the owner states ("should ever exist") is enforced by the owning tool,
  never by a manual step in a plan;
  to be recorded in `package/mpm-shadow/cli/README.md` with the 0.2.2 work,
  not AGENTS.md (the owner's earlier preference for lessons of this session).
  Still to check after merge:
  whether the Coolify app for `docker-compose.yml` auto-deploys on push.
- Separate review of `playwright-image-publish` (2026-11-01,
  `658b372099..590a8a5b4a`):
  no blocker.
  Should-fix,
  sent to the switch agent:
  any non-200 answer on an existing tag rebuilds and overwrites it;
  an arm64 machine pulls the single amd64 image with exit 0 and a warning,
  then fails at exec instead of reaching the owner's local-build fallback;
  `workflow_dispatch` has no ref guard and one concurrency group for every ref.
  Minor:
  curl timeouts and the fail-open comment,
  the minted token on curl's command line,
  the warning's "HTTP 000",
  zizmor cosmetics,
  provenance attestation (repo precedent),
  and runbook fixes (an MD8 split bold span,
  GitHub's delete and restore limits,
  check 3's preconditions).
- Queued 2026-11-01,
  owner:
  "gh issue #2576. But prioritize cutover first."
  Issue #2576 asks for a global CPU-only `xiaomi-ocr` command through mpm:
  Python selection,
  torch backend and extra requirements on `mpm-uvx`,
  an owned wrapper package,
  a local-source representation in the toolchain,
  then the global entry and install.
  It starts after the cutover,
  which deletes the TypeScript provider the issue also names
  and settles the global plan owner through the global conversion.
  Meanwhile a read-only agent writes `${HOME}/temp/agent/issue-2576/design-research.md`
  (dependencies first,
  then the open decisions with pnpm and uv precedent);
  written 2026-11-01.
  Its main points:
  `--from` is hidden in uv 0.13.6's `uv tool install` (a local wheel goes in the positional argument,
  as mpm-shadow's wheel bootstrap already does);
  uv's tool receipt cannot record a torch backend,
  and mpm-shadow's `changed_fields` and `observe_mpm_uvx` would miss a changed Python,
  backend or source,
  so drift needs an mpm-shadow marker;
  suggested options `uvPython` and `uvTorchBackend` (flat,
  like the existing ones),
  with `uvWith` deferred;
  for the wrapper source it ranks a local wheel with its sha256 in the lock first,
  then PyPI,
  a GitHub release asset,
  and a directory source
  (PyPI and release assets are publishing,
  an owner question);
  the repository has no Python precedent,
  so the test runner,
  linter,
  formatter and type checker are unpicked;
  the change lands after 0.2.1 and needs another mpm-shadow release.
  It also found the cutover procedure in `doc/planning/file-enforcer-rust-cutover.md` still uses the old verbs
  (`sync`,
  `status`,
  `lock --check`);
  sent to the switch agent to update before the global conversion runs from it.
  after the cutover,
  grill the owner on the dependency picks,
  then the decisions,
  then implement (it needs another mpm-shadow release).
- Order from here:
  separate reviews of the wg fixes and of `mpm-shadow-0-2-1`;
  owner's approval;
  merge and release 0.2.1;
  the switch agent regenerates the lock with the released 0.2.1,
  merges `main`,
  and runs one final Dockerfile build without the verification overlay;
  separate review of the switch branch;
  merge;
  `mise run toolchain:install` in `main`;
  the global plan conversion.
- AGENTS.md rule for concerns leaking out of their owning component:
  two drafts (`CWN`,
  `LKF`) were not approved;
  the owner will write the wording from issue #1819.
  Settled 2026-10-31 through an interview with the owner and landed in `33a7fce914` (closes #1819):
  `OWK` in "Architecture decisions" covers this failure,
  and `BRQ` in a new section "Before briefing an agent" covers the second failure and the pin note;
  record:
  `doc/planning/agents-md-ownership-and-brief-rules.md`.
- Second failure the owner named:
  this session turned the owner's remark about severity into a policy (opt-in,
  off by default,
  fail closed) and briefed the wg agent with it as a requirement.
  Proposed fix:
  tighten `ASK` to "ask before acting on them or briefing an agent with them" (197 characters);
  the owner answered "Not yet".
  The owner asked which AGENTS.md rules pushed toward it;
  found:
  `QGR` ("Settled decisions ... adopt + record unasked") with no definition of "settled",
  `GAP` ("do expected action") read as permission to guess the action,
  `VR2` ("do the narrower") with a policy choice mistaken for a scope choice,
  `PX1`/`PXQ` as a general lean to act,
  and `EPR` plus the pnpm-first rule as an "adopt unasked" reflex;
  `PX2`,
  `ASK` and `AUT` should have stopped it.
  Tightened `QGR`,
  `GAP` and `VR2` drafts measured 207,
  205 and 201 characters (over the 200 limit),
  so fixes need trimming or splitting;
  wording left to the owner.
- Released 2026-10-31:
  `monochromatic-search-shadow` 0.2.1 (run `41174514524`)
  after two CI fixes to its exact `--version` checks (`6a2d944949`,
  `44bf57cc84`).
- Shadows identify themselves (owner chose "All three shadows", 2026-10-31):
  invoked under the shadowed name,
  a top-level `--version` prints the real tool's output then `<shadow crate> <version>, shadowing <tool> at <real path>`
  (the `hub` precedent),
  and a top-level `--help` appends a short shadow section;
  subcommand help passes through.
  The owner noticed `mpm --version`/`--help` never mention the shadow,
  and reports the git wrapper's silence as a regression introduced some time ago.
  mpm-shadow: the update-by-tool agent's branch;
  search-shadow: its own agent;
  git wrapper: reported to monochromatic-7a with the convention.
  7a's finding:
  no earlier banner existed to restore
  (the Rust wrapper has printed byte-identical Git output for `--version` since its first commit `414ab5ff76`,
  2026-10-05,
  pinned by `version_is_identical_to_native_git`;
  the retired TypeScript wrapper handled only `git cli-git --help`),
  so the "regression" reading is not supported by history;
  7a's agent implements the convention for the next wrapper release.
  The owner also confirmed mpm-shadow's separate toolchain command set "is fine".
- Switch branch `worktree-agent-acfa05abe2fe1d3fa` reported done (tip `a7b0c9f42c`):
  plan converted,
  `mise.toml` and `20-env.toml` per the decision,
  tasks renamed to pnpm verbs,
  CI with committed-then-local drift steps,
  the `repo-tools-setup` action,
  Dockerfile,
  IDE launcher on `bin --json`,
  `package/file-enforcer/core` and `file-enforcer.config.ts` deleted,
  pnpr header reworded (merging redeploys pnpr on Coolify and starts a publish run),
  committed outputs clean,
  TypeScript comparison differences all explained,
  clean-container check passed.
  Before merge it is fixing two things:
  this session's note made it pin `github:Aquaticats/Monochromatic@monochromatic-mpm-shadow-v0.2.0`,
  which AGENTS.md `PIN` forbids (now: float `@latest` with a tag-prefix option in the `github:` resolver,
  lock still recording the archive SHA-256),
  and the Dockerfile lacks `cargo` for cargo-binstall tools.
  Issue #1819 records the pin as a third instance of a session-made requirement.
- Running:
  the cutover switch branch (repoints,
  `20-env.toml`,
  repository plan conversion,
  deletion of `package/file-enforcer/core`,
  the TypeScript comparison,
  a clean-container check of the published cli).
- Session preference 2026-10-31,
  user:
  "For this session: Tell me your estimation of time until cutover whenever your estimation or confidence changes."
  Last estimate given:
  3.5 to 5.5 hours after the old-copy answers (about 35% confidence;
  mpm-shadow 0.2.2 joined the critical path);
  before that,
  3 to 4.5 hours when the owner asked again after the image-size answer (about 40% confidence;
  each separate review has found something to fix,
  so the date keeps slipping);
  before that,
  3 to 5 hours from the owner's Playwright answers at about 40% confidence,
  later than the earlier 2.5 to 5.5 hours because the wg blocker,
  the mpm-shadow 0.2.1 release and the Playwright image publishing joined the path;
  a judgement from today's agent run times (14 to 87 minutes each),
  not a measured rate.
- Fixed 2026-10-31:
  `fec-create-release` was skipped in run `41171845290`
  because its implicit `success()` sees the skipped library publish jobs upstream of `fec-publish-crate`;
  it now uses `!cancelled()` with explicit results.
- Lesson 2026-10-31 (this session, owner annoyed):
  the clearnet self-placement came from this session's own brief,
  which suggested the toolchain re-execute itself into the slice,
  though the request ("Make our wg auto make update traffics go through clearnet") named wg as the owner;
  the merge review checked that it worked, not which component owned the decision.
  Before briefing or merging a design,
  check which component the request names as the owner,
  and reject any design that makes another component aware of that concern.
- Lesson 2026-10-31 (this session):
  run every step this session can run itself instead of handing it to the owner;
  the owner pointed out that `cargo logout` was such a step.
- Found 2026-10-31:
  the `timeout` on PATH is `~/.cargo/bin/timeout`,
  and `timeout 60 vibe --version` printed `timeout 0.1.0` instead of running `vibe`;
  not yet investigated.
- Then:
  the cutover (task 25),
  dropping the rules-file generator (7a's config switch landed in `ad9fe75d05`).

Decided 2026-10-30 and recorded in the decision record:
the HTTP stack (`hyper`,
`hyper-util`,
`tokio`,
`http-body-util`,
`hyper-rustls`),
browserslist data by our own registry fetch with a lock (`flate2` and `tar`),
Firefox ESR from Mozilla product-details,
`jsonschema` for the schema corpus check,
and the browserslist grammar (exact text,
`#` comments and blank lines,
count 1 only).

Cutover reminders from finished agents:

- `package/config/pnpr/config.yaml` still starts with "Generated by file-enforcer (generatePnprConfig in file-enforcer.config.ts); edit that source."
  for byte identity;
  reword it at cutover through the constant `PNPR_CONFIG_HEADER_LINE` in `package/config/pnpr/src/generated-config-settings.ts`,
  knowing a content change redeploys pnpr on Coolify and starts a publish run.
- `file-enforcer.config.ts` renders pnpr's config through pnpr's `renderPnprConfigForWorkspace` but writes through its own `overwrite`,
  so `--check` reports drift instead of writing;
  the Rust engine needs the same split.
- Delegated 2026-10-30: the user wrote "I'm handing the forbidden-strings and git-policy work to another session. If you have more delegate them to it too."
  The forbidden-strings release, the cli-git publish, and their cutover config switch belong to that session;
  monochromatic-7a (the cli-git Rust wrapper session) confirmed it holds them and received the items on 2026-10-31
  (release order, the merged rules-list feature, the config switch);
  this session only drops the rules-file generator, and only after 7a reports the config switch done.
  Gate from 7a (2026-10-31): `cli-git.config.jsonc` cannot change until a wrapper that reads a `rulesFile` array
  (0.4.4, after forbidden-strings 0.6.0) is published, the toolchain lock names it, and it is installed as the production git;
  an older wrapper cannot parse the array. So the cutover keeps the generator until 7a says the switch landed.
  Progress seen on `main` 2026-10-31:
  forbidden-strings 0.6.0 (`5ec9a377e2`) and git-policy/cli 0.4.4 (`7ce408f679`) are released;
  the lock entry,
  install,
  and config switch are still 7a's to report.
  Gate cleared 2026-10-31:
  7a reported the switch done in `ad9fe75d05`
  (`FORBIDDEN_STRINGS_RULES` in `mise.toml`,
  generated from `file-enforcer.config.ts`,
  is now `{{config_root}}/forbidden-strings.append.txt,{{config_root}}/forbidden-strings.append.local.txt?`,
  and `cli-git.config.jsonc` `rulesFile` is the matching array),
  with wrapper 0.4.4 installed as the production git.
  The cutover commit drops the rules-file generator (the eager compile-rules step) and the seeded local appendix step,
  and the root `mise.toml` stops being generated at cutover
  (decision record,
  "Consequences":
  "the root `mise.toml` is no longer generated"),
  so the value from `ad9fe75d05` must survive the move the decision record specifies
  ("Outputs that stop being generated":
  `mise.no-env.toml` becomes the hand-written `mise.toml`,
  and the static environment moves to the committed `.config/mise/conf.d/20-env.toml`).
  Corrected 2026-10-31:
  an earlier version of this note said `mise.toml` "keeps its committed content",
  which contradicted that section.
- mpm-shadow at cutover
  (from `package/mpm-shadow/cli/README.md`):
  the converter writes each tool's `request` as `<origin>@<range>`,
  writes `{version}` for rust `packageId` and `RUSTUP_TOOLCHAIN`,
  `JAVA_HOME`,
  `ANDROID_HOME`,
  and `ANDROID_SDK_ROOT`,
  and gives `harper-cli` and `tombi` an `options.asset` glob
  (`tombi`'s need is untested);
  `assetPattern` and `cargoGit` are now unknown fields.
  There is no adopt step:
  the lock is rebuilt with `update`,
  and the rehearsal against the converted plan changed six tools
  (musl builds for `ripgrep`,
  `harper-cli`,
  `hyperfine`,
  `marksman`,
  and `bin`,
  and `superhtml` 0.7.0 to 0.7.1),
  while the global lock moves `aws` 2.39.2 to 2.39.3.
  Only the four Linux platforms are written
  (`doc/decision/mise-lock-linux-only-stopgap.md`),
  so `bin`'s macOS and Windows rows and `bun`'s baseline rows go away.
  Show that lock diff to the user as the review surface.
- forbidden-strings must be released before cli-git is published:
  cli-git now uses the rules-list API (task 23; see the planning doc, "Cutover dependency").

## Superseded next action
look at oxc-browserslist

The user said the next step is to look at `oxc-browserslist`,
before building on `browserslist-rs`.
Why it matters:
the gate probe measured `browserslist-rs` 0.21.2 on the repository's `.browserslistrc` against npm `browserslist`
and found older data
(14 targets against 18,
`chrome151` against `chrome156`,
a hard-coded `firefox140` ESR against `firefox153`,
Node up to 26.8.0 against 26.12.0).

What is already known,
from
[`doc/audit/tech-file-enforcer-browserslist-engine-vet-2026-10-29.md`](../audit/tech-file-enforcer-browserslist-engine-vet-2026-10-29.md)
(sections "oxc-browserslist 5.0.1" and "E7" to "E9"):

- `oxc-browserslist` 5.0.1 (2026-08-12,
  MIT) is the oxc project's fork of `browserslist-rs`,
  passed screening,
  and its published crate matches its tag byte for byte.
- It has no config-file loading,
  no `BROWSERSLIST*` environment variables,
  and no `extends`,
  so file-enforcer parses `.browserslistrc` itself either way.
- Its data inputs at 5.0.1 (`caniuse-lite 1.0.30001809`,
  `browserslist 4.28.8`) are older than `browserslist-rs` 0.21.2's,
  so the published crate alone does not close the data gap;
  its `cargo codegen` data regeneration has not been run.
- Unread:
  how it resolves `Firefox ESR`,
  `not dead`,
  and the `ChromeAndroid` aliases.
- Single crates.io owner (`Boshen`).

The user picked this candidate and it is already in a repository vet report,
so this is not broad vetting,
and the check-in that `.agents/skills/choosing-technology/SKILL.md` requires before enumerating candidates does not apply.
If looking at it means searching for candidates beyond memory,
the user's picks,
and repository precedent,
it becomes broad:
ask the user who runs that vet first.
The probe scratch for `browserslist-rs` is under `${HOME}/temp/agent/fe-rust/gates/`,
and an oxc clone is at `${HOME}/temp/agent/oxc-browserslist-2026-10-29`.

## After that

Start the implementation areas tracked in issue #1818:

1.  `package/file-enforcer/core-rust` and `package/file-enforcer/cli`.
2.  The dependency-catalog Rust library.
3.  mpm-shadow:
    `sync`,
    `status`,
    `update [--check]`,
    `lock`,
    `lock --check`,
    `bin-paths`,
    `--global`,
    the lock digest,
    and bootstrap.
4.  forbidden-strings and git-policy:
    the comma-separated rules list with a trailing `?` for optional entries.
5.  pnpr generates and checks its own `config.yaml`.
6.  The in-repo XML edit crate.
7.  Conversions and the one-switch cutover.

Open facts for implementation are listed in the decision record's "Consequences":
`schemars` derive without serde `Deserialize` is unproven,
the golden test's JSON Schema validator crate is unnamed,
and tag `v8.2.0` of mpm resolves to `99eb596b` while `doc/decision/mpm-toolchain-adoption.md` cites `f095931a`.

## Lesson recorded from this work

The grilling settled the design first and left its dependencies as vet gates after acceptance;
the user then approved them without vets.
The user asked that grilling enumerate and pick every dependency first,
now `GXD` in `AGENTS.md`,
and that each broad vet check in with the user before enumerating candidates,
now "Check in before broad discovery" in `.agents/skills/choosing-technology/SKILL.md`.

## Task 20 status on 2026-10-30
Built on branch `worktree-agent-a9e487e6d38ec954c`.

`package/file-enforcer/core-rust` (library) and `package/file-enforcer/cli` (binary `file-enforcer`) exist,
with `file-enforcer.config.jsonc`,
`file-enforcer.schema.json`,
the content files under `.config/file-enforcer/`,
and the schema mapping in `.idea/jsonSchemas.xml`.
Run against a throwaway copy of the repository,
the Rust tool reports zero drift for `CLAUDE.md`,
`LICENSE`,
every
`LICENSES/` directory and all 26 `Cargo.toml` key sets,
which the TypeScript tool wrote.
The rule-kind interface for the seams (rust-catalog,
browserslist,
jetbrains-harper) is in
[`package/file-enforcer/core-rust/README.md`](../../package/file-enforcer/core-rust/README.md);
until their builders are registered those kinds warn with `kind-not-wired` and plan nothing.
Not switched:
no task,
CI,
Dockerfile or config uses the new binary.
Next for this area:
register the three builders when their crates land,
then the cutover (task 25).

## Rule kinds wired on 2026-10-31
Branch `worktree-agent-a23c3740c67ba36d9`.

The `file-enforcer` binary now implements `rust-catalog`,
`browserslist` and `jetbrains-harper`
(`package/file-enforcer/cli/src/kinds.rs`;
details in the CLI README,
section "Rule kinds linked here").
The browserslist data lock moved to `browserslist-data.lock.json` at the repository root,
and the rule names it in its required `lock` field.
Against a throwaway copy of this repository the Rust tool reports no drift for `rust-catalog.toml`
and every manifest the globs match (32 files) as the TypeScript tool wrote them,
detects and repairs a changed `[workspace.dependencies]` table,
writes the same `.browserslistrc.resolved.local.json` as npm `browserslist` 4.29.5 on the same `caniuse-lite`,
and writes byte-identical JetBrains settings files to the retired TypeScript policy in a disposable home.
Engine change:
the read-order check skips pairs where both rules compose,
and `package-licenses` moved after `rust-catalog` in `file-enforcer.config.jsonc`
because it reads manifests and does not compose.
Sandbox note:
a Rust binary's direct connection to `registry.npmjs.org` is aborted inside the agent sandbox
(`Software caused connection abort`) while `curl` works,
so the cold-cache fetch was not exercised there;
the cache was seeded with `curl` and verified against the lock.
Open:
the `bin/*.ts` scripts of the browserslist package now belong to a workspace package
and still have oxlint findings
(sync Node APIs,
arrow functions,
try/finally,
missing TSDoc).

## How to respond

Reply in the session that resumes this work;
it reads this file,
then the decision record,
then issue #1818.
