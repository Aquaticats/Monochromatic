# MPM global toolchain migration implementation spec

Status:
 implementation plan for the machine-global slice of
 [`doc/decision/mpm-toolchain-adoption.md`](../decision/mpm-toolchain-adoption.md),
 requested by the maintainer 2026-10-28
 (`Migrate my global toolchain management off mise to mpm.run too`).
 Read that decision and
 [`doc/planning/mpm-toolchain-migration.md`](mpm-toolchain-migration.md)
 first;
 this file carries only what the global scope adds or changes.

## What moves and what stays

Moves off Mise (the `~/.config/mise/config.toml` `[tools]` slice,
58 declarations):

- Tool declarations,
 install mechanics,
 version resolution,
 and upgrade coordination,
 to the same "MPM plus focused native providers" composition the decision adopts.
- The tool-backend settings `[settings.pipx]` and `[settings.npm]`,
 which exist only to route `pipx:` and `npm:` installs.

Stays with Mise:

- `[env]` (`GH_PATH`,
 the `_.path` exposure entry),
 `[settings]` (`experimental`,
 `auto_update`),
 and `mise activate bash`.
- Mise runs tasks;
 it manages no tools.

There is no global `mise.lock`;
 the locked versions are the ones the Mise installs recorded
 (the `mise ls` active rows of the declarations),
 carried into `toolchain.lock.json` at seeding time.

## Placement decisions (maintainer 2026-10-28)

- Plan,
 lock,
 and mpm manifests live machine-locally in
 `${XDG_CONFIG_HOME:-$HOME/.config}/monochromatic-dev/toolchain/`.
 Nothing personal is committed to the repository;
 the repository owns the tooling
 (providers,
 resolvers,
 sync/status/update code) instead.
- Exposure is a dedicated farm directory,
 `<toolchainRoot>/exposure`,
 rebuilt wholesale by the sync,
 on `PATH` through one `_.path` entry in the global Mise config.
- Old Mise installs stay as the rollback
 (maintainer precedent 2026-10-26),
 pruned in a later decision.

## Paths

- Config stack (declarations and generated records):
 `${XDG_CONFIG_HOME:-$HOME/.config}/monochromatic-dev/toolchain/`
 holding `plan.tools.ts`,
 `plan.artifacts.ts`
 (renamed `plan.tools.data.ts` and `plan.artifacts.data.ts` on 2026-10-29,
 matching the `**/*.data.ts` data-module lint exemption;
 `doc/planning/rust-dependency-catalog.md`),
 `toolchain.lock.json`,
 `mpm.packages.toml`,
 and `mpm.managers.toml`.
- Install root (payloads):
 `${XDG_DATA_HOME:-$HOME/.local/share}/monochromatic-dev/toolchain-global/`
 with `<id>/<version>/` install trees,
 `bin/` as the `bin` CLI download directory
 (the repository scope's `toolchainBinDir` role),
 and `exposure/` as the exposure farm.
- `bin` CLI configuration:
 per-scope,
 `<toolchainRoot>/bin.config.json`,
 selected with `BIN_CONFIG` during sync,
 so the two scopes never flip one shared `default_path`
 (a small fix the repository scope adopts too).
- Postinstall `git` link directory:
 `$HOME/.local/bin`,
 replacing the Mise `postinstall` that linked `git` there
 (`doc/planning/cli-git-global-install.md`).

The path algebra in
`package/file-enforcer/core/src/toolchain/paths.ts`
gains a global variant
(`globalToolchainPaths`),
built from `HOME`/`XDG_CONFIG_HOME`/`XDG_DATA_HOME`
and reusable by every entry point with `--global`.

## Global plan

- `plan.tools.ts` (`plan.tools.data.ts` since 2026-10-29) in the config stack is a TypeScript data module:
 comments preserved,
 no imports,
 validated at load into `ToolPlanEntry[]`
 (the typed plan is the loaded shape;
 the file is its editable form).
- Ids follow the old declaration keys one to one,
 so the migration is auditable against the old config,
 except `aqua:rust-lang/rustup` which becomes `rustup`
 (the `aqua:` prefix named a Mise backend that no longer exists).
- Request shapes:
 `{track: 'latest'}` for the floating entries,
 `{pinned: '1.3.14'}` for `bun`,
 `{pinned: '0.1.32-aquaticat.0319832'}` for `npm:typeslayer`,
 `{pinned: 'ref:85e3eb76819762cdcaa732fa87533ff896546bac'}` for
 `cargo:https://github.com/slint-ui/slint`,
 `{channel: 'next'}` for `npm:@deepseek-ai/dsh`,
 `{channel: 'stable'}` for `rust`.
 Commented-out declarations in the old config
 (`cargo:xremap`,
 `github:BtbN/FFmpeg-Builds`,
 `crush`)
 carry over as comments.
- Seeding writes each lock row at the version the old install recorded,
 then `toolchain:update:global --check` reports upstream drift without writing
 (the same "floating requests with committed resolutions" semantics the
 repository scope keeps).

## Routing table

Family mapping (routes and precedents from the repository lock):

- `bin-sourced`:
 `rg`,
 `gh`,
 `git-lfs`,
 `tree-sitter`,
 `npm:tweakcc` is npm-family (below),
 `usage`,
 `typst`,
 `pandoc` (aliases `pandoc-lua` and `pandoc-server` link the one inode),
 `glow`,
 `lsd`,
 `kopia`,
 `rclone`,
 `starship`,
 `depot`,
 `yt-dlp` (`rename_exe` becomes `bin install`'s name argument),
 `deno`,
 `opentofu`,
 `hyperfine`,
 `cargo-binstall`,
 `pnpm`,
 `bun`,
 `github:openinterpreter/openinterpreter`
 (`matching` is resolver-side asset selection;
 three binaries in one asset).
- `archive`:
 `node`,
 `go`,
 `uv`,
 `age`,
 `neovim` (runtime tree),
 `aws` (`symlink_bins` becomes `bins: ['aws', 'aws_completer']`),
 `rustup`
 (rust-lang/rustup publishes no GitHub release assets,
 probed 2026-10-28;
 the versioned channel archive on `static.rust-lang.org` is the real source
 and the `bin` CLI cannot fetch it,
 so the install copies `rustup-init` as `rustup`),
 `github:lemonade-sdk/llamacpp-rocm`
 (`asset_pattern` is resolver-side asset selection).
- `cargo-binstall`:
 `cargo:fd-find`,
 `cargo:coreutils` (`features = "unix"` becomes `cargoFeatures`),
 `cargo:llmfit`,
 `cargo:procs`,
 `cargo:cargo-bloat`,
 `tokei`
 (its GitHub releases publish no assets since `v13.0.0-alpha.1`,
 probed 2026-10-28,
 and the retired Mise backend installed it from crates.io),
 `cargo:monochromatic-git-policy-cli`
 (postinstall link `git` into `$HOME/.local/bin`).
- `cargo-git` (new route):
 `cargo:https://github.com/slint-ui/slint`
 at the pinned ref,
 building the `slint-lsp` package
 (probe-verified against the Mise install's `bin/` listing).
- `mpm-pnpm`:
 the 17 `npm:` entries
 (`npm:@opencode/cli`'s `allow_builds` becomes a pnpm build-script
 allowance,
 probe-verified before cutover).
- `mpm-uvx`:
 `pipx:huggingface_hub`,
 `pipx:serena-agent`
 (the old `[settings.pipx] uvx = true` already routed them through
 `uv tool install`).
- `go-install` (new route):
 `go:github.com/charmbracelet/crush`
 (`go install <pkg>@v<version>` into a scoped `GOBIN`,
 preserving the declaration's build-from-source semantics).
- `rustup-toolchain`:
 `rust`
 at the `stable` channel
 (resolves `1.98.1`,
 the version the old install recorded;
 `rustup` itself is the `rustup` entry above,
 so `rust` does not re-declare it).
- Bootstrap entries:
 `bin`
 (standalone binary,
 the repository scope's `0.29.4` pin)
 and `mpm`
 (PyPI wheel `8.2.0`),
 both idempotent probes first.

## Provider extensions

Bounded additions,
each replacing something the old config expressed in Mise option syntax:

- `cargo-git` route:
 `cargo install --git <url> --rev <ref> --root <installDir> <package>`.
- `go-install` route:
 `GOBIN=<installDir>/bin go install <packageId>@v<version>`
 after the `go` tool is staged.
- Archive route hardening:
 restore the executable bit on every declared bin in an extracted tree
 before linking
 (this replaces llamacpp-rocm's `postinstall` `chmod`,
 which existed only because the release zip stores mode `0644`).
- Resolver-side asset selection:
 `options.assetPattern`
 (llamacpp-rocm's `asset_pattern`,
 openinterpreter's `matching`)
 filters release assets when the lock row is resolved.
- `options.allowBuilds`:
 the `mpm-pnpm` provider passes the pnpm build-script allowance for
 `@opencode/cli`.
- `PostinstallLink.link.linkDir` accepts an absolute directory
 (the `$HOME/.local/bin` link)
 as well as the worktree-relative spelling.

Exposure names must be unique across one plan;
 the global plan declares `rustup` only on the `rustup` entry.

## Update resolution

Existing resolvers cover npm,
 crates.io,
 PyPI,
 GitHub releases,
 and the node SHASUMS files.
 New resolution families:

- `go` runtime:
 `https://go.dev/dl/?mode=json` latest linux-amd64 release.
- `go-install` tools:
 `https://proxy.golang.org/<module>/@latest`.
- `rustup`:
 the `rust-lang/rustup` release tags.
- `rust` `stable` channel:
 `channel-rust-stable.toml`
 (the nightly-date resolver's sibling).

npm policy is unchanged:
 the registry `latest` dist-tag wins,
 so a publisher rollback can move a tool down in the update diff;
 `npm:@deepseek-ai/dsh` follows its declared `next` tag.

## Cutover

Hard switch after a successful full global sync,
with the Mise installs left in place as rollback:

1. Seed the config stack from the old declarations
 and provision with `toolchain:sync:global`.
2. Edit `~/.config/mise/config.toml`:
 delete `[tools]`,
 `[settings.pipx]`,
 `[settings.npm]`,
 and the two commented-out backend blocks;
 keep `[settings] experimental`,
 `auto_update`,
 and `[env]`;
 add the `_.path` entry pointing at the farm;
 repoint `GH_PATH` at the farm's `gh` link.
3. Verify the user boundary:
 every declared tool resolves through the farm,
 `git` still runs the policy wrapper,
 `gh` honors `GH_PATH`,
 and the old shims do not shadow the farm
 (PATH order measured,
 not assumed).
4. Record the cutover in the decision,
 the planning records,
 and `doc/planning/cli-git-global-install.md`'s rollback wording.

## Interplay with the repository scope

- Manager-native installs are machine singletons
 (`~/.cargo/bin`,
 `~/.local/bin`,
 pnpm's global prefix,
 `~/.rustup`).
 Where both scopes lock one package
 (`cargo:fd-find`,
 `cargo:coreutils`,
 `cargo:monochromatic-git-policy-cli`,
 `npm:typescript-language-server`)
 the seeds agree today,
 but a later update of one scope alone moves the shared copy and the other
 scope's `toolchain:status` reports drift.
 Rule:
 run both updates together,
 and let status report any drift
 (recorded as a known limit,
 not papered over).
- The `git` wrapper links from both scopes
 (`.cache/cli-git-bin/git` repository-wide,
 `$HOME/.local/bin/git` global)
 point at the same `~/.cargo/bin/cli-git-native`
 and therefore move together.
- `rustup` home is shared;
 the repository scope's `nightly-2026-10-28` and the global scope's `stable`
 toolchain coexist,
 exactly as the old Mise `rust` entry did beside the repository pin.

## Verification

1. `mise run toolchain:status:global` clean after the sync on this machine.
2. Disposable home (`mktemp -d`,
 `HOME=`,
 `XDG_DATA_HOME=`,
 `XDG_CONFIG_HOME=`,
 container with 2 GiB / 2 CPUs):
 sync the Linux x64 slice and run one representative tool per route
 (`node --version`,
 `cargo binstall` output probe,
 `uv tool list`,
 `rustup toolchain list`,
 `go version`,
 one `bin`-sourced binary).
3. User boundary on this machine:
 `command -v` for a sample of declared tools resolves to the farm,
 `git` wrapper policies still fire,
 and `mise run` tasks still work.
4. No synthetic host load;
 every probe reports the command,
 working directory and exit status.

## Known limits to record, not paper over

- macOS and Windows provisioning parity stays open
 (the tool acquisition parity gate).
- The lock is machine-local,
 so it has no git history;
 the review surface is the `toolchain:update:global` output plus the file
 diff the maintainer reads in place.
- Manager-native singletons can flip between scopes
 (the interplay rule above).
- `mpm installed` sees `bin`-sourced tools only through the `bin` manager
 reading `bin`'s configuration,
 so inventory runs carry the scope's `BIN_CONFIG`
 (the sync does;
 `toolchain:status:global` reports the true state regardless).

## Implementation record (2026-10-28)

What the implementation needed beyond the shapes above.

- The lock seeds at the versions the retired Mise installs recorded
 (58 declarations,
 plus the `bin` and `mpm` bootstraps),
 with artifact rows materialized from each upstream at those exact versions
 (`linux` rows only,
 as the repository scope keeps).
- Probe-corrected routing:
 `tokei` moved to `cargo-binstall`
 (its GitHub releases publish no assets since `v13.0.0-alpha.1`),
 `rustup` moved to `archive`
 (rust-lang/rustup publishes no GitHub release assets;
 the versioned `static.rust-lang.org` channel archive is the real source),
 and `tree-sitter` moved to `archive`.
- The `tree-sitter` incident:
 its `.gz` asset stores the payload name `bin` in the gzip header,
 which `bin install` honored as the destination name,
 renaming the whole toolchain `bin/` directory aside to `.bin.old`
 and leaving a file where the directory was
 (first sync,
 2026-10-28).
 The directory was restored from `.bin.old`,
 the archive route gunzips to the declared name,
 and `installBinSourced` now fails loudly when an install replaces the bin
 directory.
- `uv tool install` passes `--force`,
 so tools whose executables survive from a retired Mise install
 (`~/.local/bin`)
 converge instead of failing
 (the `pipx:huggingface_hub` case).
- The `mpm-uvx` status probe matches names across registry spelling
 (`huggingface-hub` versus `huggingface_hub`).
- Sync probes no longer crash the run when their own staging prerequisite is
 missing;
 the install converges instead.
- `bin`'s install database moved per scope
 (`<toolchainRoot>/bin.config.json` through `BIN_CONFIG`),
 because two scopes sharing one database flip each other's recorded versions
 (the `pnpm` name collides across scopes at different versions).
 The repository scope's existing database was copied to its new location,
 and `toolchain:status` stayed `61 tools in sync`.
- New update resolvers:
 `assetPattern` asset selection,
 npm declared dist-tags,
 the Go module proxy,
 go.dev's download index,
 and the rustup channel archive
 (`resolveRustup`,
 since the GitHub family cannot see `static.rust-lang.org` rows).
- End state on the maintainer's machine:
 `59 of 59` in sync with the global lock,
 every declared tool resolving through the farm,
 `mise self-update` clean,
 and the `linked-worktree-only` guard still rejecting in main worktrees
 through the repointed `~/.local/bin/git`.
 The seeding-time `npm:typeslayer` pin
 (`0.1.32-aquaticat.0319832`)
 was dropped from the plan at the maintainer's decision
 ("Remove it entirely"),
 after probes showed the public registry no longer carries that build
 (`npm view typeslayer versions` lists 33 versions and none of them);
 its retired Mise install stays under the standing
 "leave the old installs" decision.
