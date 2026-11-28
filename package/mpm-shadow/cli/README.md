# mpm-shadow

EXTREMELY UNSTABLE AND UNFINISHED DO NOT USE

Owns this repository's toolchain plan and lock,
and shadows `mpm` ([Meta Package Manager](https://mpm.run)).

- As `mpm-shadow` it runs the toolchain commands,
  named after pnpm's
  (see "Command names follow pnpm"):
  `install` (alias `i`),
  `update`,
  `outdated`,
  `list` (alias `ls`),
  `bin`,
  `prune`,
  and `verify-archive`.
- As `mpm` (a link the toolchain plan creates) it takes pnpm's verbs too:
  a command line whose first word is one of the commands named here or one of their aliases
  (`mpm i -g`,
  `mpm up`,
  `mpm ls`)
  runs that command,
  exactly as `mpm-shadow` would.
  The owner's words (2026-11-03,
  after `mpm i -g` had reached mpm 8.2.0 and failed with "No such command 'i'"):
  "I want pnpm style verbs.
  We talked about this!"
- Every other command line typed to `mpm`,
  including one that starts with one of mpm's own options
  (`mpm --verbosity DEBUG outdated`),
  goes to the real `mpm` found on `PATH`:
  the shadow runs it with the arguments untouched and the standard streams inherited,
  waits for it,
  deletes the superseded cached versions of this repository's own crates from cargo's registry,
  and exits with mpm's own status.
  mpm 8.2.0's own `install`,
  `outdated`,
  `upgrade` (and its alias `update`)
  and the `list` alias of `installed` are therefore not reachable through `mpm`:
  type `mpm installed` for the list,
  and run the real `mpm` by its path for the others.

## Design rule: pnpm answers first

Where pnpm's behaviour answers a toolchain question
(the plan,
the lock,
`update`,
`install`,
`install`,
or what happens in CI and when frozen),
mpm-shadow follows it,
and the change is recorded,
even when it overrides an earlier decision line.
Only what pnpm leaves open goes to the owner.
The user's words:
"Please go with pnpm style instead of blocking questions that can be resolved by it on me."
The decisions themselves are in `doc/decision/file-enforcer-rust-rewrite.md`,
section "Toolchain (mpm-shadow)".

## Command names follow pnpm

The commands are pnpm's,
with the meaning pnpm gives them,
checked against the pages on <https://pnpm.io> on 2026-10-31
and against pnpm 12.16.1 in disposable directories.
The old names (`sync`,
`lock`,
`lock --check`,
`update --check`,
`status`,
`bin-paths`,
`--frozen-lock`
and `--no-frozen-lock`)
no longer exist and have no aliases,
because pnpm has none.

- `install`:
  <https://pnpm.io/cli/install>,
  "Aliases:
  `i`",
  `--frozen-lockfile`,
  `--no-frozen-lockfile`,
  `--lockfile-only` ("only updates `pnpm-lock.yaml`...
  Nothing gets written to the `node_modules` directory")
  and `--offline`.
  It replaces `sync`.
  It also takes tools,
  as `pnpm i -g <package>` does
  (read with pnpm 12.18.0 on 2026-11-13:
  `pnpm i -g is-odd` printed `+ is-odd 3.0.1`),
  which is the only way a new tool is declared by this command
  (see "Adding a tool with install").
- `install --lockfile-only --offline`:
  replaces `lock`.
  pnpm's `--offline` resolves from its metadata cache;
  mpm-shadow has no such cache,
  so an offline run rewrites only the declaration fields of rows the lock already holds
  and refuses an entry that needs resolving.
- `install --lockfile-only --frozen-lockfile`:
  replaces `lock --check`.
  The pnpm page says nothing about this combination;
  measured with pnpm 12.16.1,
  it exits 0 when the lockfile matches the manifest and 1 (`ERR_PNPM_OUTDATED_LOCKFILE`) when it does not,
  and writes nothing,
  which is the check this command did.
- `outdated`:
  <https://pnpm.io/cli/outdated>,
  "exits with code 1 when any dependency is outdated",
  and takes patterns the way `update` does.
  It replaces `update --check` and exits 1 when any selected tool would move or a lookup fails.
  pnpm's `--json`,
  `--format`
  and `--long` are not offered:
  the report is the same text `update` prints.
- `list`:
  <https://pnpm.io/cli/list>,
  "Aliases:
  `ls`".
  It replaces `status`.
  The pnpm page leaves exit codes open,
  so `list` keeps exiting 1 on drift.
  pnpm's `--json`,
  `--parseable`
  and `--depth` are not offered.
- `bin`:
  <https://pnpm.io/cli/bin>,
  "Prints the directory into which the executables of dependencies are linked",
  with `--global`.
  Plain `bin` prints the exposure directory.
  The page has no `--json`;
  `bin --json` prints the document the IDE launcher reads (`exposureDir`,
  `pathDirs`
  and `bins`),
  following the `--json` that pnpm's `list` and `outdated` pages document.
  It replaces `bin-paths`.
- `--global` also has pnpm's short form `-g`.
- `update`:
  <https://pnpm.io/cli/update>,
  "Aliases:
  up,
  upgrade"
  (read 2026-11-03),
  so `up` and `upgrade` are `update`.
- Unchanged:
  `prune`,
  which keeps its name,
  and `verify-archive`,
  which has no pnpm counterpart.

## Toolchain plan and lock

People edit `toolchain.plan.jsonc` (JSONC:
comments and trailing commas are allowed).
The one exception is `install <tool>`,
which appends the entry a request describes and writes nothing else in the file
(see "Adding a tool with install").
mpm-shadow is the only writer and the only checker of `toolchain.lock.json`,
which stays strict JSON because `jq` readers in CI and Dockerfiles select
`.tools.<id>.version`,
`.tools.<id>.artifacts.<platform>.{url,sha256}`
and `.tools.<id>.options.androidPackages`.

The machine-global toolchain has the same two files in `${XDG_CONFIG_HOME:-$HOME/.config}/monochromatic-dev/toolchain`;
every command takes `--global` to act on them.
The repository scope is found from the working directory by the nearest `toolchain.plan.jsonc`,
or from `MONOCHROMATIC_WORKTREE_ROOT`.

### The plan

Every entry has a stable id (an alias,
like a `package.json` key)
and a `request`,
like a `package.json` value:
`<origin>@<range>`.
The origin says where versions come from,
the range says which are wanted
(`latest`,
an exact version or tag,
a channel such as `nightly`,
or a Java major line such as `21`).

```jsonc
// toolchain.plan.jsonc
{
  "schemaVersion": 1,
  "tools": {
    // Why this pin exists belongs here, beside it.
    "bin": { "request": "github:marcosnils/bin@0.29.4", "route": "archive", "bins": ["bin"] },
    "age": { "request": "github:FiloSottile/age@latest", "route": "archive", "bins": ["age", "age-keygen"] },
    "zig": { "request": "zig@0.15.2", "route": "archive", "bins": ["zig"] },
    "cargo:fd-find": { "request": "cargo:fd-find@latest", "route": "cargo-binstall", "bins": ["fd"] },
    // `{version}` stands for the resolved version in `packageId` and in `env` values.
    "rust": {
      "request": "rust@nightly", "route": "rustup-toolchain", "packageId": "{version}",
      "bins": ["rustc"], "env": { "RUSTUP_TOOLCHAIN": "{version}" }
    }
  }
}
```

- A tool is `request`,
  `route`,
  optional `manager`,
  `packageId`
  and `os`,
  `bins`,
  `env`
  and `options`.
- The origins are
  `github:<owner>/<repo>` (a generic release resolver),
  `npm:`,
  `cargo:`,
  `pipx:`,
  `conda:`,
  `go:<module>`,
  `git:<url>` (the range is the revision),
  and the named resolvers
  `node`,
  `go`,
  `zig`,
  `rustup`,
  `aws`,
  `rust`,
  `android-sdk`
  and `java:temurin`.
- The `@` that separates the range is the last one not preceded by `:` or `/`,
  so `npm:@actions/languageserver@0.3.61` reads as expected.
- The routes are
  `bin-sourced`,
  `archive`,
  `cargo-binstall`,
  `cargo-git`,
  `mpm-pnpm`,
  `mpm-uvx`,
  `go-install`,
  `rustup-toolchain`,
  `rustup-components`
  and `android-sdk`.
  The route is how a tool installs;
  the request is where its versions come from.
- A `rustup-toolchain` row's `bins` are the programs the toolchain the row's channel names holds:
  the `bin` directory of the toolchain `rustup which --toolchain <channel> rustc` prints a path in,
  and never another installed toolchain's.
  Which installed toolchain a channel is belongs to `rustup`:
  its host triple is the kernel's own answer plus its build's C library,
  or the `default_host_tuple` of its settings,
  and a custom linked toolchain's directory is the link's own name,
  so `mpm-shadow` asks rather than reading the directory names.
  The names in `~/.rustup/toolchains` sorted like anything else are not read at all
  (a second toolchain beside the pin answered for the farm's `rustc`,
  `cargo` and `rustfmt` before issue 6634,
  a neighbour whose name extends the channel answered for the programs the channel's own toolchain
  lacked,
  and so did a custom link whose name extends the pin and the same channel installed for a second
  host triple).
  The row also searches rustup's own proxies in the cargo bin directory where `rustup` itself was
  installed there.
  `rustup` is never one of them:
  it is the manager,
  which `rustup-init` or the distribution's package installs before the first `mpm-shadow` run,
  and `mpm-shadow` never does (see "Bootstrap"),
  so the farm links `rustup` only where a `rustup` proxy of that name already sits in the cargo bin
  directory,
  and a row that declared it names an unresolved bin wherever none does.
  On 2026-11-17 this machine's `rustup` became the distribution's package,
  and the proxies in the cargo home went with the `rustup-init` installation it replaced,
  so no `rustup` proxy sat there any more,
  and the row that declared it named it an unresolved bin.
  A program a component provides is in the toolchain only while that component is installed,
  so such a program resolves through the farm once `rustComponents` declares it or
  `rustup component add` has put it there.
  An install adds a declared component the toolchain lacks,
  which `0.2.7-alpha.17` showed against a real `rustup` in its second pass:
  on a pinned toolchain at the `minimal` profile holding no `rustfmt` it asked
  `rustup component add rustfmt --toolchain nightly-2026-11-16`,
  the two programs were in the toolchain's `bin` afterwards and ran,
  and the report named no unresolved bin,
  where `0.2.7-alpha.16` in the same state asked nothing and named three unresolved bins.
  The controls around this answer every `rustup` call from a stand-in runner.
  `list` reads the same declarations:
  it asks `rustup` which toolchain the row's channel is,
  then `rustup component list --installed` for what that toolchain holds,
  and a declared component or target whose line is missing is drift,
  named in the `drift` line,
  which is what the next `install` would add.
  A target is a `rust-std-<triple>` line of that same listing,
  which is the listing `rustup target list --installed` filters for
  (`list_targets` in rustup 1.29.3's `src/cli/rustup_mode.rs`),
  so the two questions are one.
  When the listing itself fails,
  the row is drift too and the line carries the failure's words
  (`its components and targets could not be read: ...`),
  which is not what the next `install` would add:
  nothing is known about the parts until a listing answers.
  Every program a row lists that no component it declares and no profile installs is not in
  the toolchain at all,
  so such a program is named as an unresolved bin on a machine whose rustup keeps no proxy of
  that name in the cargo bin directory;
  `src/toolchain/the_rust_rows_own_toolchain_holds_every_bin_it_declares_tests.rs`
  is the control of that against a toolchain holding exactly what this repository's row declares.
- The example's `rust` row above shortens that list to one bin:
  this repository's own row lists the programs the toolchain holds
  (`cargo`,
  `cargo-clippy`,
  `cargo-fmt`,
  `clippy-driver`,
  `rust-gdb`,
  `rust-gdbgui`,
  `rust-lldb`,
  `rustc`,
  `rustdoc`
  and `rustfmt`),
  and none of them is `rustup`.
  Which component puts each of them there follows from the same names the plan uses:
  `cargo` comes from the `cargo` component,
  `rustc`,
  `rustdoc`,
  `rust-gdb`,
  `rust-gdbgui` and `rust-lldb` from `rustc`,
  `cargo-clippy` and `clippy-driver` from `clippy`,
  and `cargo-fmt` and `rustfmt` from `rustfmt`.
  The `cargo` and `rustc` components are not in `rustComponents`:
  they are what rustup's profile installs before any declared component,
  which on a machine set to the `minimal` profile
  (the setting this repository's measurements were taken under)
  is `rustc`,
  `cargo` and `rust-std`,
  so six of those ten programs arrive that way
  and no formatter does;
  this repository's own tasks run both
  (`cargo fmt`,
  and `format:processors` in `package/linter/monochromatic-lint/mise.toml` runs a bare `rustfmt`),
  which is why the row declares `rustfmt` (issue 6630).
  The row's `rust-src`,
  `llvm-tools` and `rustc-dev` components put no program in the toolchain's `bin` directory.
- An unknown field or origin is an error naming its path.
  `version` and `artifacts` are not plan fields:
  only `update` writes them.
- Tools install in the order written,
  except that `bin`,
  `uv`,
  `mpm`,
  `rustup`,
  `rust`,
  `cargo-binstall`,
  `go`,
  `node`
  and `pnpm` come first.

#### The `github:` resolver

- It takes the newest release (or the release of the range as a tag,
  with and without a leading `v`),
  and for each of the four Linux platforms
  (`linux-x64`,
  `linux-x64-musl`,
  `linux-arm64`,
  `linux-arm64-musl`)
  the asset whose name says Linux and the platform's CPU
  (`x86_64`,
  `amd64`,
  `x64`,
  `aarch64`,
  `arm64`).
- It skips checksums,
  signatures,
  installer packages,
  scripts,
  other operating systems and CPUs,
  baseline,
  debug,
  source,
  `.full.`
  and GPU (`vulkan`,
  `rocm`,
  `cuda`)
  builds;
  an `options.asset` glob that matches a GPU build's name lets that build through,
  and is the only way to ask for one.
- It prefers the platform's libc (a `musl` name for a musl platform,
  otherwise one without),
  then `.tar.gz`,
  `.tar.xz`,
  `.zip`,
  `.gz`,
  and a bare file.
- Two equally good assets are an error that names both.
  `options.asset` (a glob with `*`) settles it,
  for example `"asset": "harper-cli-*"` for a release that ships two programs.
- A platform with no matching asset is left out of the lock
  and fails at `install` with a message naming it.
- The version is the tag without its `v` or `name-v` prefix.
- `options.tagPrefix` picks one program's releases in a repository that tags several
  (this repository tags `monochromatic-mpm-shadow-v0.2.0`,
  `monochromatic-lint-v0.4.0`,
  and so on).
  With `"request": "github:Aquaticats/Monochromatic@latest"`
  and `"tagPrefix": "monochromatic-mpm-shadow-v"`,
  the resolver reads the release list
  (100 per page,
  newest first)
  until a short page or the tenth page,
  and takes the highest version among the releases that are neither a draft nor a prerelease
  and whose tag is the prefix followed by one whole version
  (`1.2.3` or `1.2.3-rc.1`;
  a `v` in front of it only when the prefix does not end in `v`);
  the version is that rest of the tag.
  The list is ordered by creation,
  not by version:
  a backport released later sits ahead of the newer line,
  so the first match is not necessarily the highest version.
  A tag such as `monochromatic-foo-2-v1.0.0` is a release of another program and never matches
  the prefix `monochromatic-foo-`.
  A range names a tag:
  the prefix and the range as written,
  then (unless the prefix ends in `v`) the prefix,
  `v` and the range;
  a tag that does not continue the prefix with a version is refused.
  A list with no such release is an error that names the prefix,
  and says so when the search stopped at its cap of ten pages.
  Taking the highest version means reading every page until a short one (up to ten),
  so one lookup can cost ten API requests,
  each on a new connection because the client keeps none open.
  GitHub's primary limit for unauthenticated requests is 60 an hour
  (see
  [the REST API rate limits](https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api)),
  and the resolver sends the token of `GITHUB_TOKEN`,
  or of a logged-in `gh`,
  when there is one.
  Changing the prefix is a resolution-affecting edit:
  it needs `mpm-shadow update`.
  Characters other than letters,
  digits,
  `.`,
  `_`
  and `-` are refused.

#### Lock rows

- A downloaded tool is recorded for the four Linux platforms only
  (the repository's Linux-only stopgap).
- Zig,
  Go,
  Node,
  rustup,
  conda
  and GitHub publish checksums,
  which the lock records.
  Java uses the release asset's digest.
  The Android command-line tools and the AWS CLI publish none,
  so `update` downloads each once and records the hash of what it got.

### The lock

- The lock is derived from the plan and nothing else.
  Deleting `toolchain.lock.json` and running `update` rebuilds it
  (the same versions for exact requests,
  the newest for `latest`).
  No resolver reads a previous lock row to learn a repository,
  a tag shape,
  an asset
  or a platform list.
- Tool entries keep the key order `request, version, route, manager, packageId, bins, os, env, options, artifacts`.
- A top-level `digest` (`sha256:` and the hash of the compact JSON of everything else) covers the whole content.
  `install --lockfile-only --frozen-lockfile` recomputes it,
  so a hand edit of a resolved version,
  a URL or a hash fails the gate.
  A reformatted file keeps its digest.

### What the cutover converter must write

For every entry the converter writes a `request` string.
Package-manager entries keep their specifier:
`cargo:<crate>@latest`,
`npm:<name>@latest`,
`pipx:<name>@latest`
(`@<version>` where the old request was `pinned`,
for example `cargo:cargo-fuzz@0.13.2`).
`manager` and `packageId` become optional because the request names the package;
`routes`,
`bins`,
`os`
and every `options` field except `cargoGit` (replaced by `git:<url>@<revision>`)
and `assetPattern` (renamed `asset`) carry over unchanged.

**The `{version}` placeholder is required.**
Where a field must equal the resolved version,
the converter must write `{version}`,
never today's literal:

- `rust`:
  `"packageId": "{version}"` and `"env": { "RUSTUP_TOOLCHAIN": "{version}" }`.
- `java`:
  `"env": { "JAVA_HOME": "java/{version}" }`.
- `android-sdk`:
  `"env": { "ANDROID_HOME": "android-sdk/{version}", "ANDROID_SDK_ROOT": "android-sdk/{version}" }`.

A literal such as `nightly-2026-10-30` there would make the first `update` move `version`
while `packageId` keeps the old date,
and `rustup toolchain install` would install the stale channel.

Requests for today's downloaded tools,
each checked by a live `update` in a disposable home against the current lock
(the four Linux platforms resolve to the same files,
except where a musl platform now gets the musl build the release offers):

- `bin`:
  `github:marcosnils/bin@0.29.4` (pinned;
  keep the `bootstrap` option).
- `mpm`:
  `pipx:meta-package-manager@8.2.0`
  (pinned;
  keep `route: mpm-uvx` and the `bootstrap` option with `distribution: wheel`).
- `bun`:
  `github:oven-sh/bun@bun-v1.3.14` (pinned tag;
  keep `binAliases`).
  The baseline platforms (`linux-x64-baseline` and the musl baseline) are no longer recorded.
- `pnpm`:
  `github:pnpm/pnpm@latest` (keep `binAliases`).
- `ripgrep`:
  `github:BurntSushi/ripgrep@latest`.
- `node`:
  `node@latest`.
- `harper-cli`:
  `github:Automattic/harper@latest` with `"asset": "harper-cli-*"`.
- `dprint`:
  `github:dprint/dprint@latest`.
- `uv`:
  `github:astral-sh/uv@latest`.
- `hyperfine`:
  `github:sharkdp/hyperfine@latest`.
- `caddy`:
  `github:caddyserver/caddy@latest`.
- `watchexec`:
  `github:watchexec/watchexec@latest`.
- `sops`:
  `github:getsops/sops@latest`.
- `age`:
  `github:FiloSottile/age@latest`.
- `cargo-binstall`:
  `github:cargo-bins/cargo-binstall@latest`.
- `cmake`:
  `github:Kitware/CMake@latest`.
- `zig`:
  `zig@0.15.2` (pinned).
- `conda:bubblewrap`:
  `conda:bubblewrap@latest` (keep `os: ["linux"]`;
  the package is a `.conda` file,
  which `install` unpacks with `unzip` and `tar --zstd`).
- `taplo`:
  `github:tamasfe/taplo@latest` (keep `binAliases`).
- `tombi`:
  `github:tombi-toml/tombi@latest` with `"asset": "tombi-cli-*"`.
- `marksman`:
  `github:artempyanykh/marksman@latest`.
- `rumdl`:
  `github:rvben/rumdl@latest`.
- `github:Feel-ix-343/markdown-oxide`:
  `github:Feel-ix-343/markdown-oxide@latest`.
- `zizmor`:
  `github:zizmorcore/zizmor@latest`.
- `superhtml`:
  `github:kristoff-it/superhtml@latest`.
- `github:docker/docker-language-server`:
  `github:docker/docker-language-server@latest`.
- `llama.cpp`:
  `github:ggml-org/llama.cpp@b9829` (pinned).
- `opentofu`:
  `github:opentofu/opentofu@latest`.
- `hcloud`:
  `github:hetznercloud/cli@latest`.
- `betterleaks`:
  `github:betterleaks/betterleaks@latest`.
- `java`:
  `java:temurin@21` (a major line that moves;
  the version is `temurin-<release>.0.LTS`).
- `android-sdk`:
  `android-sdk@latest` (keep `androidPackages`).
- `rust`:
  `rust@nightly` (keep `rustComponents` and `rustTargets`).
- The machine-global plan also holds `go: go@latest`,
  `aws: aws@latest`,
  `rustup: rustup@latest`
  and `go:github.com/charmbracelet/crush@latest` on the `go-install` route.

### A second llama.cpp under another bin name

The machine-global plan holds two llama.cpp builds.
`github:lemonade-sdk/llamacpp-rocm` is the ROCm build for the GPU,
and it declares `llama-cli` with the other `llama-*` names.
`llama.cpp-cpu` is the plain `ubuntu-x64` release asset of `ggml-org/llama.cpp`,
which is the CPU build,
pinned to `b9829` like the `llama.cpp` entry of the repository plan.
A bin name declared twice is refused,
so the CPU build exposes its `llama-cli` as `llama-cpu-cli`.
In `binAliases` the key is the exposed name and the value is the executable inside the archive,
the same direction as the `bunx` and `pnpm` entries.

```jsonc
// ${XDG_CONFIG_HOME:-$HOME/.config}/monochromatic-dev/toolchain/toolchain.plan.jsonc
"llama.cpp-cpu": {
  "request": "github:ggml-org/llama.cpp@b9829",
  "route": "archive",
  "bins": ["llama-cpu-cli"],
  "options": { "binAliases": { "llama-cpu-cli": "llama-cli" } }
},
```

Adding it is the ordinary lifecycle,
never `update --all`:

```sh
# Lock the new entry alone, then converge the machine-global tools; both work from any directory,
# because they name `mpm-shadow`, the program that owns these verbs (the machine-wide `mpm` is the real mpm).
mpm-shadow update -g llama.cpp-cpu
mpm-shadow install -g
```

The lock then records `llama-b9829-bin-ubuntu-x64.tar.gz` for `linux-x64`,
and `command -v llama-cpu-cli` names the exposure directory.
The `llama-cli` of the ROCm build keeps its name.

#### Xiaomi-OCR-0 on the CPU with `llama-cpu-cli`

Issue 2576 asked for a global CPU-only command for `xiaomi-research/Xiaomi-OCR-0` without Python written by this repository.
The research is `doc/research/xiaomi-ocr-without-python.md`,
and the measurements are in `doc/troubleshooting/xiaomi-ocr-0-cpu-inference.md`.
Release `b9829` loaded the community GGUF files of `prithivMLmods/Xiaomi-OCR-0-GGUF`
and printed the same text for the sample image as the Transformers recipe did on 2026-11-01.
Xiaomi says it has not evaluated those community conversions,
and the check covered one image.

Weights are not an mpm tool,
so they are pinned with `hf download`,
which the global plan already provides through `pipx:huggingface_hub`.
They live in `${HOME}/models/Xiaomi-OCR-0-GGUF`,
inside the `models` directory of the home directory,
where `package/hall-monitor/daemon/src/analyze/llama.ts` also keeps its GGUF files.
Both files match the SHA-256 values that the Hub listed for them at the pinned revision:
`1e497ff06b91a3d822c543066df3e9ef69c62c5423be09d739dfa1db55630d3a` for the Q4_K_M text model
and `634d6bc95781ebf4a3f06570a59859fa5082d488db540fd474bc6b4061aec90e` for the BF16 projector.

```sh
# Fetch the text model and the projector at a fixed revision of the community repository.
hf download prithivMLmods/Xiaomi-OCR-0-GGUF Xiaomi-OCR-0.Q4_K_M.gguf Xiaomi-OCR-0.mmproj-bf16.gguf \
  --revision 7bbfa072e0f3d70ac722e26fbccef5b9f64b7121 --local-dir "${HOME}/models/Xiaomi-OCR-0-GGUF"

# Fetch the published sample image at the revision the Transformers recipe uses.
hf download xiaomi-research/Xiaomi-OCR-0 example_pics/qianziwen.jpeg \
  --revision bbd8cf8d21f256b3644b44cb860f3a7cecb93c30 --local-dir ./sample
```

The command line that was run:

```sh
# One-shot OCR of the sample on the CPU; the answer is on stdout between the prompt echo and the statistics line.
llama-cpu-cli \
  --model "${HOME}/models/Xiaomi-OCR-0-GGUF/Xiaomi-OCR-0.Q4_K_M.gguf" \
  --mmproj "${HOME}/models/Xiaomi-OCR-0-GGUF/Xiaomi-OCR-0.mmproj-bf16.gguf" \
  --image ./sample/example_pics/qianziwen.jpeg \
  --prompt "Extract the text in the image." \
  --reasoning off --temp 0 --single-turn --simple-io --no-display-prompt \
  --ctx-size 4096 --n-predict 256 --threads 2
```

- `--single-turn` makes the CLI exit after one answer.
  Do not use `--no-conversation`:
  `llama-cli` at `b9829` answers "please use llama-completion instead",
  and the run that carried it had written 16 GB to a redirected stdout when it was stopped after a few minutes.
- `--ctx-size 4096` bounds the context.
  Without it the CLI takes the model's own context length,
  and the peak resident memory of the sample run was 4.18 GiB instead of 1.21 GiB.
  A full page needs a larger value;
  that case was not measured.
- `--reasoning off` is honoured:
  `--reasoning on` changed the sample's answer,
  while `off` gave the text the Transformers recipe printed.
- `--threads 2` is the two-CPU budget that the measurements used.
- `llama-cli` prints a banner and a statistics line on stdout.
  `llama-mtmd-cli` prints the answer alone,
  after an empty `<think>` block,
  but the ROCm entry owns that name and no CPU alias exists for it.
- No `xiaomi-ocr` verb exists:
  mpm links binaries as they are,
  and this repository ships no wrapper script.

## Commands

- `mpm-shadow install --lockfile-only --offline`:
  rewrites the lock's declaration fields from the plan without the network,
  keeping every resolved version and artifact.
  It refuses a new tool or a resolution-affecting edit
  (`request`,
  `route`,
  `manager`,
  `packageId`,
  `os`,
  `asset`,
  `bootstrap`)
  and names `mpm-shadow update <id>` for just those tools.
- `mpm-shadow install --lockfile-only --frozen-lockfile`:
  the toolchain drift gate for CI,
  beside `file-enforcer check`.
  Writes nothing.
  Exits 1 when the plan and lock disagree or the digest does not match.
  In CI the frozen default applies when a lock exists,
  so `install --lockfile-only` alone is the same check there.
- `mpm-shadow install --lockfile-only`:
  resolves the entries the lock lacks or holds for an older request,
  writes only their rows,
  and installs nothing.
- `mpm-shadow update [<tool>...]`:
  resolves every tool and writes the whole lock,
  or,
  given tool ids,
  only those (see "Updating named tools").
  A brand-new downloaded tool is just a new plan entry:
  every origin has a resolver,
  and none reads the previous lock.
- `mpm-shadow outdated [<tool>...]`:
  reports what `update` would move,
  writes nothing,
  installs nothing,
  and exits 1 when something would move.
- `mpm-shadow install [<tool>...] [--frozen-lockfile | --no-frozen-lockfile]`:
  with a tool,
  first adds the ones the plan does not hold yet and then installs,
  as `pnpm i -g <package>` does
  (see "Adding a tool with install");
  then resolves the plan entries the lock lacks or holds for an older request
  (see "Installing a plan that is ahead of the lock"),
  then installs every locked tool that is not at its locked version,
  makes the postinstall links and removes the ones an earlier run made that the lock no longer declares
  (see "Postinstall links the lock no longer declares"),
  rebuilds the exposure directory (see "What the exposure folder holds"),
  writes the tool environment for mise (see "Tool environment"),
  runs the postinstall commands
  and then retires the old copies of this repository's tools
  (see "Old copies of this repository's tools").
- `mpm-shadow list`:
  compares the installed tools with the lock.
  Exits 1 on drift.
  Also names the old copies of this repository's tools that `install` would retire,
  which are not drift.
  It starts `rustup` twice for each `rustup-toolchain` row whose toolchain is at its version
  (see "The plan" for those two questions),
  and waits for each answer with no bound of its own,
  so a `rustup` that never answers makes it wait for ever.
  `install` waits the same way,
  for the questions it asks about a Rust row and for every other program it
  starts,
  and every program a toolchain command starts through that runner is waited on the same way,
  so a `rustup` or a `cargo` that never answers makes that command wait for ever too.
  Three starts do not go through the runner and print no such line,
  which is what each needs:
  the shadow mode starts the real `mpm` with the standard streams inherited and waits for it
  (`src/shim.rs`),
  a launcher replaces itself with the program its record names
  (`src/toolchain/launcher/starting_it.rs`),
  and the check for the repository link to the 860 drive starts its own program under its own deadline
  (`src/toolchain/repository_link_cause.rs`).
  What a wait does say is one line on standard error once it has passed 2 s,
  naming the program and every argument it was started with:

  ```text
  mpm-shadow: still waiting for `rustup component list --installed --toolchain nightly-2026-11-16` to answer, 2.0 s after this command started it; no deadline ends that wait
  ```

  The 2 s comes from what an ordinary answer costs,
  measured on 2026-11-28 with three arms interleaved in each of 40 rounds:
  this machine's `rustup 1.29.3` answering `--version` took 4.39 ms at the median and 45.61 ms at its slowest,
  and a stand-in answering `which` and `component list` took 12.85 ms and 12.63 ms at their medians
  and 26.58 ms and 26.28 ms at their slowest,
  so 2 s is 75 times the slowest ordinary question and 157 times the median of the two questions,
  and above the spread of the control arm,
  whose slowest sample was 10.4 times its median.
  What that costs when `rustup` answers:
  20 to 116 ms for the whole command with the row's components and targets declared,
  against 6 to 15 ms for a row that declares none and so asks only which toolchain the channel is,
  measured on 2026-11-27 in seven interleaved rounds run three times over,
  with this account's own `rustup 1.29.3`
  and a hard-linked copy of this machine's pinned toolchain,
  on a machine other sessions were loading;
  one question alone cost 4 to 31 ms at the median of those rounds.
  No deadline is shipped for those waits:
  the measurement says what an answer costs,
  not what a late one should be cut off at,
  and a cut-off would tell a person a toolchain's parts could not be read when `rustup` was merely
  slow.
- `mpm-shadow prune --toolchain [--global] [--dry-run] [--lock <file>]...`:
  deletes the cargo tool versions no known lock names
  and the superseded versions of this repository's release archives (see "Cleanup").
  Plain `mpm-shadow prune` is unchanged:
  it cleans cargo's registry.
- `mpm-shadow verify-archive <tool> <file> [--platform <id>]`:
  hashes a downloaded file and compares it with the lock's SHA-256 for that tool and platform.
- `mpm-shadow bin [--json]`:
  prints the directory the executables are linked into;
  with `--json`,
  `exposureDir`,
  `pathDirs`
  and `bins` as JSON for the IDE launcher.
  It reads only the plan and the file system.

`list` refuses when the plan and lock disagree,
and names `mpm-shadow install --lockfile-only --offline` or `mpm-shadow update <ids>`;
`install` does the same when frozen,
and otherwise updates the lock itself:
it resolves the new entries and writes the plan's other declaration changes.

### Tool environment

A tool's `env` block names environment variables the tool needs in every process that uses it:
`RUSTUP_TOOLCHAIN` for the Rust toolchain,
`JAVA_HOME` for the JDK,
`ANDROID_HOME` and `ANDROID_SDK_ROOT` for the Android SDK.
Nothing exports them by itself:
`install` writes them into a generated mise configuration fragment,
and mise,
which runs every task and shell of the repository,
reads that fragment.

- The repository scope writes `.config/mise/conf.d/30-toolchain-env.local.toml` in the worktree.
  The `.local.` part keeps it out of git through the `*.local.*` ignore pattern,
  the way file-enforcer's `10-bin.local.toml` is.
  The machine-global scope (`install --global`) writes no tool environment at all:
  tool variables are exported in this repository's worktrees only,
  never in a shell outside the repository,
  so a global lock's `env` block (its `RUSTUP_TOOLCHAIN`,
  for one) reaches no process through `install --global`.
  The hand-written global `config.toml` is never edited.
- Mise reads every file of a `conf.d` directory in name order,
  so no committed file points at the fragment.
  An `include` of the generated file from a committed one would make mise fail on a fresh clone,
  before the first `install`,
  because mise refuses an include of a file that is missing.
- A value with a `/` is relative to the repository scope's toolchain root
  (`<data home>/monochromatic-dev/toolchain`)
  and is written as an absolute path:
  `android-sdk/23.0` becomes `<toolchain root>/android-sdk/23.0`.
  The lock holds the value with `{version}` already replaced,
  so a version bump moves the path on the next `install`.
  A value that names a directory holding exactly one directory,
  and that inner directory holds the tool's first declared program in its own `bin`,
  is written as that inner directory:
  an Adoptium archive unpacks the whole JDK into `jdk-<release>` below the tool's version directory,
  and Gradle refuses a `JAVA_HOME` one level above it
  (`ERROR: JAVA_HOME is set to an invalid directory`).
  With the layout on the machine,
  `JAVA_HOME` becomes `.../java/temurin-21.0.13+9.0.LTS/jdk-21.0.13+9`
  and `ANDROID_HOME` keeps the SDK root,
  which holds six directories.
  Any other value is a literal (`nightly-2026-11-01`).
  A literal that itself contains a `/` would be read as a path;
  no tool declares one.
- The fragment is rewritten whole from the lock on every full `install`,
  so a tool removed from the lock loses its variables,
  and a lock with no `env` leaves no fragment.
  `update` and other partial syncs leave it alone.
  Tools for another OS family are skipped.
- A name that is not a plain identifier,
  a relative value that is absolute or holds an empty,
  `.` or `..` segment,
  a value with a control character,
  a brace or a dollar sign,
  and one name declared with two values are refused.
  Mise reads every value as a Tera template (`{{ }}`,
  `{% %}`,
  `{# #}`) and expands `$NAME`,
  so these are not escaped but refused.
  The refusal fails the `install` with the tool,
  the variable and the remedy,
  and leaves the fragment of the last good run as it was.
  Quotes and backslashes are escaped,
  since mise reads them back as they were.
- The variables reach mise only through directory activation,
  so a process started from outside the worktree (and outside any activated shell) does not have them.
  `mise exec`,
  `mise run` and an activated shell in the worktree do.

### Updating named tools

`mpm-shadow update <tool>...` mirrors `pnpm update <pkg>...`:
it looks up and moves only the tools it selects,
and every other lock row keeps its version and artifacts
(and takes the plan's current `bins`,
`env`
and `options`,
as offline `install --lockfile-only` would give it).
Adding a tool to the plan and running `mpm-shadow update <new id>` locks that tool alone,
the way `pnpm install` after adding one dependency resolves only that dependency.

- A name is a tool id,
  or a pattern in which `*` stands for any run of characters
  (`mpm-shadow update 'npm:*'`);
  a leading `!` excludes
  (`mpm-shadow update 'npm:*' '!npm:wrangler'`),
  and only exclusions select everything else.
  This is the pattern grammar of `pnpm update "@babel/*" "!@babel/core"`.
- A name that matches no tool of the plan is refused with exit code 2,
  before any lookup,
  naming it and listing every valid id.
  pnpm 12.16.1 accepts such a name silently and reports "Already up to date";
  that hides a typo here,
  where a name selects what to resolve.
- A plan entry the lock cannot hold as it is
  (no lock row,
  or an edit of `request`,
  `route`,
  `manager`,
  `packageId`,
  `os`,
  `options.asset`
  or `options.bootstrap`)
  that the names left out is refused,
  before any lookup,
  with the ids to add to the command.
  Writing a lock with such a row kept would fail `install --lockfile-only --frozen-lockfile`,
  and resolving it unasked would move what the command line did not name.
  With no lock file every entry is such a row,
  so a named update cannot create the lock;
  plain `update` does.
- `outdated` takes the same names:
  it reports only the selected tools and exits 1 when one of them would move.
- Not mirrored:
  `--latest` (a request's range already says what is wanted),
  `<tool>@<range>` (people edit the plan,
  so a range changes there),
  `--interactive`,
  `--depth`,
  and the workspace flags have no counterpart.
  `update` installs only to retry a failed npm or PyPI lookup,
  so it already behaves as `--lockfile-only` does.
- `install --lockfile-only --offline` stays offline.
  `install` resolves a plan that is ahead of the lock,
  as the next section says.

### Adding a tool with install

`mpm-shadow install <tool>...` (alias `i`) declares a tool and installs it in one command,
the way `pnpm i -g <package>` adds a global package
(read with pnpm 12.18.0 on 2026-11-13 in a disposable home:
`pnpm i -g is-odd` printed `+ is-odd 3.0.1` and `pnpm ls -g` then listed it).
There is no separate `add` verb.
With `--global` the command writes the machine's plan and lock;
without it,
this repository's.
With no tool the command keeps its meaning:
it brings everything the plan holds to its lock.

- An argument is a tool id the plan already holds (`mpm i -g tokei`),
  or a request the command adds to the plan first (`mpm i -g cargo:fastmod`).
  An id the plan holds is installed as it is and the plan is not written at all.
- A request is `<prefix>:<name>` with an optional `@<version>`;
  without a version it asks for the newest,
  which the entry records as `@latest` so the plan keeps floating requests.
  Several tools in one command are allowed.
- A bare name that is neither an id of the plan nor a request is refused,
  naming the prefixes it could take;
  the command never guesses a route.

The prefixes a request can add,
and what names each route's binaries:

- `cargo:<crate>` adds the `cargo-binstall` route,
  which runs the `cargo` the machine has and needs no plan entry.
  Its `bins` are crates.io's own `bin_names` for the version the request asks for
  (`GET https://crates.io/api/v1/crates/<crate>`,
  and `.../<crate>/<version>` for a pinned one).
- `npm:<package>` adds the `mpm-pnpm` route,
  whose installer is the plan's `pnpm` entry.
  Its `bins` are the `bin` field's names from the registry's manifest
  (`GET https://registry.npmjs.org/<package>/<version-or-tag>`),
  or the package's own name when `bin` is one text.
- `go:<module>` adds the `go-install` route,
  whose installer is the plan's `go` entry.
  Its `bins` are the module path's last segment,
  which is the name `go install` writes;
  a path that ends in a major-version suffix (`/v2`) names no program and is refused.

A plan that lacks the installer a route runs holds no way to install what the
command adds,
so that request is refused before any lookup,
naming the entry to add first.
Both plans this repository ships hold `pnpm` and `go`.

Every other route needs an entry a request cannot describe,
so the command refuses it and names the plan file and this section:
`github:` picks between `archive` and `bin-sourced` and names no binary (`github:owner/repo`),
`pipx:` installs the entry points PyPI does not publish,
`git:` needs `packageId`,
the conda route carries `options`,
and the named origins (`node`,
`go`,
`zig`,
`rustup`,
`aws`,
`rust`,
`android-sdk`,
`java:temurin`) are products whose entries carry fields a request cannot name.
Those entries are written by hand:
"The plan" (above) lists every field,
and "The `github:` resolver" shows how one release's assets are chosen.

The entry the command writes carries the three fields a request and its route settle:

```jsonc
// toolchain.plan.jsonc, the entry `mpm-shadow install cargo:fastmod` writes
"cargo:fastmod": {
  "request": "cargo:fastmod@latest",
  "route": "cargo-binstall",
  "bins": [
    "fastmod"
  ]
},
```

- It is appended after the last entry of `tools`,
  in the order the command named the tools,
  and every other byte of the file stays as the person wrote it:
  comments,
  trailing commas,
  the indentation of the entry above it,
  and the file's own order.
  A comment beside the last entry stays with that entry:
  the comma goes in front of it and the new entry after it.
- **It lands without the comment that says why the tool is there.**
  Only a person knows that,
  so the command prints
  `toolchain install: no comment says why <id> is there; write one beside its entry in <plan file>`;
  write that comment above the entry.
- The write is one temporary file renamed over the plan,
  under the scope's OS lock,
  and only when the plan still has the content the command read.
  A file another mpm-shadow wrote meanwhile is not overwritten:
  the command refuses and says to run it again.
  The plan also keeps the permissions it had.
- The plan is written first and the lock second.
  A run killed in between leaves the new entry in the plan with no lock row,
  which is exactly the state a plain `mpm-shadow install` finishes
  (see "Installing a plan that is ahead of the lock").
- With `--frozen-lockfile`,
  or in a CI environment,
  the entries the command line added are still resolved and locked,
  as `pnpm add --frozen-lockfile <package>` writes `package.json` and the lockfile.
  Nothing else moves:
  if any other entry needs a lookup,
  or the plan differs from the lock anywhere a run could write without one,
  the frozen refusal stands and no lock is written.
- `install --lockfile-only --frozen-lockfile` is the CI drift check,
  so it adds nothing;
  `install --lockfile-only --offline` resolves nothing,
  so it adds nothing either.
  Both refuse before the plan is touched and name the switch to drop.

### Installing a plan that is ahead of the lock

`mpm-shadow install` mirrors `pnpm install` after a dependency was added to `package.json`:
the entries the lock cannot hold as they are
(the same set the named `update` refuses to leave out:
no lock row,
or an edit of `request`,
`route`,
`manager`,
`packageId`,
`os`,
`options.asset`
or `options.bootstrap`)
are resolved,
only their rows are written to the lock,
and then the install runs.
No other row moves,
exactly as with `mpm-shadow update <those ids>`,
and the report prints `toolchain install: locked <id> <version> ...` for each.

- **Frozen:**
  with `--frozen-lockfile`,
  or when a CI environment is detected,
  `install` resolves nothing into an existing lock:
  it refuses as before,
  names `mpm-shadow update <ids>` for exactly those entries,
  and says why it did not resolve.
  pnpm's `--frozen-lockfile` is "true by default in CI environments"
  and "pnpm detects a CI environment with the same rules as the ci-info package.
  Set `CI=false` to turn detection off"
  (<https://pnpm.io/cli/install>).
  mpm-shadow ports those rules (`src/toolchain/ci_detect.rs`):
  `CI=false` means not CI,
  otherwise `CI`,
  `BUILD_ID`,
  `BUILD_NUMBER`,
  `CI_APP_ID`,
  `CI_BUILD_ID`,
  `CI_BUILD_NUMBER`,
  `CI_NAME`,
  `CONTINUOUS_INTEGRATION`,
  `RUN_ID`,
  or a CI vendor's own variable (for example `GITHUB_ACTIONS`) means CI.
- **`--no-frozen-lockfile`:**
  resolves even in CI,
  like pnpm's `--no-frozen-lockfile`.
  The two switches cannot be combined.
- **A lock that does not exist** is resolved in full and written by `install`,
  frozen by CI or not,
  and then the install runs.
  pnpm's `--frozen-lockfile` defaults to "true,
  if a lockfile is present" in CI,
  so `CI=true pnpm install` with no `pnpm-lock.yaml` resolves and writes one
  (checked with pnpm 12.16.1 in a disposable directory).
  `--frozen-lockfile` given explicitly refuses instead,
  as `pnpm install --frozen-lockfile` does with no lockfile
  (it fails with `ERR_PNPM_NO_LOCKFILE`),
  and names `mpm-shadow update`.
- **A failed lookup** stops `install` before anything is written or installed,
  naming the entry and the URL.
- **Offline-fixable edits:**
  a plan that differs from the lock only in what `install --lockfile-only --offline` fixes
  (`bins`,
  `env`,
  the install options,
  a removed entry,
  the order)
  is written into the lock by a non-frozen `install`,
  without a lookup and without moving any version,
  and the report prints `toolchain install: lock updated from the plan: ...` for each change.
  This is pnpm's non-frozen install,
  which updates the lockfile for any manifest change.
  The same write also carries these edits when a new entry is resolved in the same run.
  Frozen (`--frozen-lockfile`,
  or CI with a lock present)
  refuses instead,
  writes nothing,
  and names `mpm-shadow install --lockfile-only --offline`.
- **A lock whose digest does not match**
  is refused naming the restore,
  under either policy.
  `list` never resolves and never writes.

## How it installs

- `rustup toolchain install --no-self-update <channel>`,
  then `rustup component add` and `rustup target add`,
  straight from the lock.
  A toolchain that is already at the locked version,
  which another program installed,
  is not skipped:
  `install` asks `rustup component list --installed` for that toolchain,
  adds only the declared parts `rustup` does not list,
  and reports the tool as installed with
  one line per part it added.
  A target counts as installed when its `rust-std-<triple>` line is in that listing,
  which is the listing `rustup target list --installed` filters for,
  so the target question costs no second start.
  A listing or an add that fails fails that tool rather than counting it unchanged.
  `install` then asks `rustup which --toolchain <channel> rustc` once for the row,
  after the installers ran,
  to learn the directory the farm links from (see "The plan");
  `list` does not build the farm and never asks that question of an install.
  That question and the listing above are waited on with no bound of their own,
  as every program this command starts is,
  and a wait that passes 2 s prints one line on standard error naming the program and its arguments
  (the measurement behind that bound is under the `list` bullet of "Commands").
  No deadline ends any of those waits.
- `cargo binstall --no-confirm --root <toolchain root>/cargo/<crate>/<locked> --version <locked> <crate>`
  (see "Cargo tool installs").
- `pnpm add --global <package>@<locked>`,
  after which `install` reads the package back with the `pnpm list --global --depth=0 --json` that `list` uses:
  pnpm exits 0 for a package whose links it wrote unreadably,
  so a package that is still absent or listed at another version fails the install,
  naming the package and what was observed.
  `uv tool install --force <package>==<locked>`,
  or a wheel whose SHA-256 the lock records,
  is read back with the `uv tool list` that `list` uses,
  for the same reason:
  a tool that is still absent,
  or listed at another version,
  fails the install
  with the package,
  the locked version,
  what the listing said and the directory `uv tool dir` names.
- `go install <module>@v<locked>` with `GOBIN` set to the `bin` directory of the version's install directory,
  after which `install` reads back the executables that directory holds.
  A Go install writes no record anywhere and no manager lists a Go tool,
  so the executables `go` writes are the whole record:
  a `bin` directory holding none of the declared binaries fails the install,
  naming the module,
  the locked version,
  the binaries it expected and the directory to check.
  The version directory itself proves nothing,
  because `install` creates it before `go` runs,
  and `list` reads the same executables,
  so a Go tool with no binaries is absent there too.
- `archive` tools are downloaded,
  checked against the lock's SHA-256,
  and only then unpacked with `tar` or `unzip`.
  A file that does not match is deleted and nothing is installed from it.
  Every `archive` tool,
  not only the ones this repository publishes,
  is unpacked into a hidden staging directory,
  marked complete,
  and given the version's name in one exchange where the file system can do that,
  or in two renames where it cannot (see "Repository-published tools keep only the newest version");
  only a release archive of this repository is also checked for its first program,
  and only for it does the marker decide whether a version counts as installed.
- `bin-sourced` tools go through the `bin` CLI.
- A name taken from the plan,
  the lock or a release tag is joined onto the toolchain root,
  so each must be one plain directory name:
  a tool id (after `/`,
  `:` and `@` are turned into `-`) and a version must not be empty,
  `.` or `..`,
  hold `/`,
  `\` or NUL,
  or start with a dot,
  a version must not be `current`,
  and a tool id must not map to a directory the toolchain root keeps for something else
  (`cargo`,
  `downloads`,
  `exposure`,
  `launchers`,
  `bin.config.json`;
  the ids `bin` and `node` stay allowed,
  because the shipped lock owns them).
  A bin name must not be empty,
  `.` or `..`,
  or hold `/`,
  `\` or NUL.
  The plan and lock decoders,
  the reader of other worktrees' locks,
  `update` (for a version taken from a release tag)
  and the path builders all refuse such a name,
  and the message names it.
  A bin name is also refused when the file system the exposure folder is on cannot hold the temporary
  name its link is written under:
  every link is written as `.{bin}.tmp.{process id}.{start time}.{namespace}.{boot}` and then renamed over
  the link's name,
  which adds at most 95 bytes to the bin name,
  so the check is made where the plan is read through the reader that carries it
  (`bin --json` opens the plan through a reader without it,
  so `bin` still prints its document for such
  a plan),
  against the longest name the file system itself reports for that folder,
  and the refusal names the row,
  the bin,
  that limit and the longest bin name that would fit.
  On a file system holding 255 bytes that is a bin name of at most 160 bytes;
  a declared bin of 161 bytes or more is refused before anything is downloaded
  (see "What the exposure folder holds" for the temporary name and why it carries those facts).
  A file system that cannot be asked leaves the question open,
  and the link write's own error is what a person meets then.
  Entries in the plan's `postinstall` are not checked this way.
- mpm is never asked to install or look up a version:
  mpm 8.2.0 ignores a pinned version for cargo and pnpm,
  and reports registry failures only in an `errors` array while exiting 0.

## Cargo tool installs

Each `cargo-binstall` tool installs into a cargo root of its own,
`<toolchain root>/cargo/<crate>/<version>`:

```text
# shape of the command install starts for a third-party crate (the paths are examples)
cargo binstall --no-confirm --root <toolchain root>/cargo/fd-find/10.4.2 --version 10.4.2 fd-find

# the same for a crate this repository publishes: the compile and quick-install strategies are switched off
cargo binstall --no-confirm --root <toolchain root>/cargo/monochromatic-mpm-shadow/0.2.5 --version 0.2.5 \
  --disable-strategies compile,quick-install monochromatic-mpm-shadow
```

- Binaries land in `<root>/bin` and cargo's own record in `<root>/.crates.toml`.
  Nothing is written to `$CARGO_HOME/bin` or `$CARGO_HOME/.crates.toml`;
  only cargo's registry cache and `binstall.toml` still live below `$CARGO_HOME`.
- `--root` rather than `--install-path`:
  cargo-binstall 1.26.1 (the locked version) documents `--install-path` as having no effect on a package built from source,
  and its source (`crates/binstalk/src/ops/resolve/resolution.rs`) fails with `CargoInstallDoesNotSupportInstallPath`
  when the build-from-source fallback runs with it.
  `--root` is passed on to the fallback's `cargo install --root`,
  so one flag isolates both ways of installing.
  The fallback exists for third-party crates only (see the next item).
- A crate this repository publishes never compiles:
  `--disable-strategies compile,quick-install` makes a missing or unusable release archive fail the install instead of building the
  crate from source (cargo-binstall 1.26.2 otherwise builds it silently;
  measured 2026-11-04:
  the installed `cli-git-native` of `monochromatic-git-policy-cli` 0.5.34 was 13,710,088 bytes
  against the archive's 13,728,056,
  beside a `cli-git-service` the archive lacks).
  When cargo-binstall stops with `Fallback to cargo-install is disabled`,
  the failure names the tool,
  the crate,
  the version,
  cargo-binstall's own `WARN resolve:` and `ERROR resolve:` lines,
  and the ways forward (publish a release whose archive holds every declared program,
  lock a version whose archive fits,
  or,
  for a timeout or rate limit,
  give the install a GitHub token or a longer
  `BINSTALL_MAXIMUM_RESOLUTION_TIMEOUT`).
  Any other binstall failure keeps the generic message.
  A third-party crate keeps cargo-binstall's strategies,
  so one with no prebuilt archive is still compiled when needed.
  The same flag switches off the quick-install strategy (the third-party cargo-quickinstall host of prebuilt binaries)
  for these crates,
  so their binaries come only from the crate's own release archive.
  cargo-binstall still finds that archive itself from the crate's published binstall metadata;
  mpm-shadow passes no URL.
  Owner decision 2026-11-04 (question tool),
  "A:
  one flag,
  our crates (Recommended)":
  "The point of cargo-binstall is for us to not manually point to gh releases ourselves!"
  Since 2026-11-07 the repository's seven locked tools no longer take this path:
  they use the `archive` route of the
  repository's GitHub releases,
  because their later `-alpha` releases skip crates.io;
  the owner's answer and what is
  still open are in `doc/decision/mpm-toolchain-adoption.md`,
  "Our seven tools install from GitHub release archives".
  A third-party crate keeps both strategies (measured 2026-11-04:
  `fastmod` and `apple-codesign` resolve only through quick-install).
- Tracking stays on (no `--no-track`):
  the root's `.crates.toml` is the record `list` and `install` read,
  and cargo-binstall's own lock then covers two trees installing one version at once.
- A directory without a record for the crate is an unfinished install and counts as absent.
  The legacy `$CARGO_HOME/.crates.toml` and `cargo binstall --list` are never read for this route.
- The `cargo-git` route already installed with `cargo install --git ... --root <tool install dir>` and is unchanged.
- Third-party crates coexist per version:
  each tree's `.cache/toolchain-bin` links the version its own lock names.
- A tool whose crate name starts with `monochromatic-` (the prefix the registry prune already uses for this
  repository's own crates) is repository-published.
  Letter case and `_` versus `-` are ignored,
  as crates.io ignores them,
  so `Monochromatic_X` is the same crate;
  a name that merely contains `monochromatic` elsewhere is not.
  The same rule decides the newest-only layout and the never-compile rule,
  so the two cannot disagree.
  So is an `archive` tool whose request names `github:Aquaticats/Monochromatic`
  (in any letter case,
  because GitHub owner and repository names are case-insensitive)
  and whose `options.tagPrefix` starts with `monochromatic-`:
  a release archive of this repository,
  such as `mpm-shadow` itself,
  whose plan entry is `github:Aquaticats/Monochromatic@latest` with `"tagPrefix": "monochromatic-mpm-shadow-v"`
  so that the lock records the archive's SHA-256 for the bootstrap.
  A tool of any other repository is never judged by this rule,
  whatever its tag prefix.
  The plan and lock carry no second list of them.
  See the next section.

### Repository-published tools keep only the newest version

Older repository-published tools are never kept once a newer one is installed,
whichever route installed them.

- `install` never installs a version older than the newest finished one in either toolchain root
  (see "One set across both toolchain roots"),
  and after installing a version it deletes every older one
  (for a release archive also the archives downloaded for them).
  A cargo version is finished when cargo's record names the crate;
  a release-archive version is finished when its directory carries the completion marker `.mpm-shadow-installed`
  and the tool's first declared bin is found in it as an executable file
  (a tool that declares no bin finishes on the marker alone).
- An archive is unpacked into a hidden staging directory beside the version directory
  (`.<version>.staging.<process id>.<process start time>.<process namespace>.<boot id>`,
  or the shorter `.<version>.staging.<process id>.<process start time>` by a run that cannot read its own boot or
  namespace,
  which no later install judges;
  see "What a kill leaves and what the next install does about it"),
  the first bin of a release archive is looked for in it,
  the marker is written last,
  and only then is the version's name given the finished copy.
  `tar` creates a file with its final mode before it writes the content,
  so without this a truncated archive would leave an executable of the right name and the wrong bytes
  at the version's name,
  count as finished and cause the working older version to be deleted.
  A failed install removes its staging directory and touches no copy that was already there.
  A publish that fails keeps the finished copy under the hidden name that run holds it at,
  which an install either publishes or removes as a second one,
  depending on what the version's name holds by then,
  and its message names that path and the version's name.
  A directory without the marker (one an earlier release unpacked in place,
  or one cut short) is never finished:
  it is never the newest,
  so it never causes a deletion or moves `current`,
  and the next install of that version replaces it.
- Giving the version's name the finished copy is one step where the file system can exchange two names
  (`renameat2` with `RENAME_EXCHANGE`):
  the staged copy and the copy at the version's name change places in that call,
  so no reader ever finds the name absent,
  and the copy that was there is removed under the hidden name afterwards.
  Where the file system cannot exchange them
  (the kernel answers `EINVAL` or `EOPNOTSUPP` for a flag the file system does not support,
  `EPERM` or `EACCES` for a rename of the type it does not allow,
  and `ENOSYS` on a kernel without the call),
  the copy that is there is moved aside first
  (`.<version>.replaced.<process id>.<process start time>.<process namespace>.<boot id>`)
  and the finished one renamed in after it,
  which leaves one instant in which the version's name holds nothing.
  Two of those answers,
  `EPERM` and `EACCES`,
  are denials of permission rather than a file system that cannot exchange two names,
  so a policy that refuses only the exchange would be silent;
  one of them is named on standard error:
  `mpm-shadow: warning: <the error>: changing <the finished copy's path> and <the version's name> in one exchange was refused, so they are changed in two renames instead, and the instant between those renames, in which <the version's name> names nothing, is back; a file system that cannot exchange two names answers the same way, and so does a policy that refuses only that one call`.
  The other three answers fall back in silence.
- A release archive of this repository gets one link per declared program before the marker is written:
  `<version directory>/<program name>`,
  pointing at the program inside the folder the archive unpacked into
  (a relative target,
  so a moved toolchain root still resolves).
  The program is then reachable at `current/<program name>`,
  a path that names no version,
  which is the shape a `cargo-binstall` tool already has (`current/bin/<name>`).
  Without it the program below `current` sits in a folder whose name carries the version,
  and any tree that installs a newer version deletes that folder.
  A version directory an earlier release unpacked in place carries no such link,
  and its program is still found below `current`.
- Downloaded archives are kept in `<toolchain root>/downloads/<tool>/<version>/`
  and removed with their version.
- `<toolchain root>/cargo/<crate>/current` (for a release archive,
  `<toolchain root>/<tool id>/current`) is a link to the newest finished version.
  Every tree's exposed link goes through it,
  and two trees syncing at once converge on the newest.
  Neither the link nor the program named through it dangles when an older version is deleted:
  a crate's program is `current/bin/<name>`,
  and a release archive's is `current/<program name>`,
  because of the link the previous bullet describes.
- A running `mpm-shadow` that installs a newer `mpm-shadow` has just deleted its own file.
  That no longer stops the launchers:
  they are copied from the newest install,
  through its `current` link,
  so nothing reads the running file.
  Only a shadow that belongs to no install (a build folder,
  a copy fetched to a scratch path) is read from its own
  file,
  through `/proc/self/exe` on Linux;
  elsewhere such a deleted running program cannot be read,
  and the sync reports an error that names the vanished path and says to run the command again with the new copy.
- `list` counts an installed version that is newer than the locked one as satisfied,
  not drift.
- A tree whose lock pins an older version than the one that runs is warned,
  in the same words everywhere:

  ```text
  <tool id> is locked at <locked> in this tree's lock, <path to that lock file>, but <used> runs, because older repository tools are never kept once a newer one is installed; upgrade this tree's lock with `mpm update <tool id>` or merge main
  ```

  The lock file is named because a launcher and a shell read whichever lock their own record names,
  so a warning from another checkout's launcher looks stale until the file it read is visible.

  `install` and `list` print it as `toolchain <verb>: warning: ...` in their report on standard output;
  `install --lockfile-only`,
  `update`,
  `outdated` and `bin` print the same line on standard error
  (`bin` writes JSON on standard output);
  the shim path (`mpm`) resolves no tool versions and prints none.

### What a kill leaves and what the next install does about it

A process killed at any instruction is normal on this host,
so each step of an archive install either leaves nothing that a start reads
or leaves what a later install finishes.
The three hidden names carry the installer that made them:
its process id,
the moment that process started,
the process namespace that gave the id out,
and the boot's identifier.
The moment is counted from the start of the boot,
so the same number and the same moment can come back after a reboot;
the boot's identifier separates one boot from the next.
A process id means one thing only inside its own namespace;
the namespace's identifier separates one namespace from the next.
A run that cannot read its own boot's identifier,
or its own process namespace,
writes the shorter name an earlier build wrote,
which carries the process id and the start time alone:
the facts it could not read are left out rather than made up,
and no later install judges such a name,
so a copy a kill leaves under it is never published and never removed.

- During the download:
  the half file `<file>.download` in `<toolchain root>/downloads/<tool>/<version>/`,
  which the next download of that version writes over,
  and no hidden directory.
- During the unpacking,
  before the marker:
  `.staging` holds an unfinished copy.
  The next install of that tool removes it and unpacks the archive again.
- After the marker and before the version's name is given the copy:
  `.staging` holds a finished copy,
  and the next install of that version publishes it,
  downloading nothing.
- Between the two renames a file system without the exchange needs:
  the version's name holds nothing,
  `.replaced` holds the copy that was there,
  and `.staging` holds the finished copy.
  The next install of that version publishes the finished copy,
  downloading nothing;
  when that copy is gone,
  it puts the `.replaced` one back instead.
- During a recovery,
  after a dead install's finished copy was claimed:
  `.recovered` holds it,
  and the next install of that version publishes it,
  downloading nothing.
- After the rename and before the `.replaced` name is removed:
  the version is finished,
  and `.replaced` holds the unfinished copy it replaced,
  which the next install of that tool removes.

A hidden copy that holds a finished install is published by the next install of that version,
downloading nothing,
unless that version's name holds a finished copy by then,
which makes the hidden one a second one that install removes.
That is an install in this process namespace,
and an install after a reboot is the same,
because a name from another boot belongs to an installer that reboot certainly ended.
An install in another process namespace does not judge such a name at all:
it publishes nothing from that copy and removes nothing,
and `prune --toolchain` prints the name on standard error with what deleting it costs.
A newer version of a tool this repository publishes,
installed by then,
leaves the hidden copy where it is,
because an older repository tool is never kept.

The sentence every failed publish ends with is that rule,
written out,
and it names the copy the failure kept:
it is the last thing an ordinary install prints about the failure,
because no install of that tool follows in the same run.
An install reaches six of those failures,
one for each way its publish can go wrong:

- `cannot move the finished install to <version's name>: <error>`:
  the version's name held nothing,
  and the rename that would have given it the finished copy failed.
- `cannot put the finished install in place of the copy at <version's name>: <error>; that copy is as it was`:
  an unfinished copy stood at the version's name,
  and the one exchange that would have replaced it failed with an answer that is not "this file system cannot exchange".
- `cannot move the copy at <version's name> aside to install over it: <error>; that copy is as it was`:
  the file system cannot exchange the two names and the rename that moves the copy aside failed.
- `cannot put the finished install that was at <version's name> back from <aside name>: <error>`:
  a second install published the version in the instant since this one read the name,
  so the copy this one moved aside was a finished one,
  and the rename that would have put it back failed.
- `cannot move the finished install to <version's name>: <error>; the copy that was there is back at <version's name>`:
  the second of the two renames failed after the copy that was there had been moved aside and put back.
- `cannot move the finished install to <version's name>: <error>; the copy that was there could not be put back from <aside name>: <error>`:
  both of those renames failed.

Each of them ends with `the finished copy this run holds is still at <hidden name>`,
following the words above,
and that copy is the one a later install publishes without downloading while that version's name holds no finished copy,
in this process namespace and after a reboot;
an install in another process namespace does not judge that name and leaves the copy where it is,
and a newer version of one of this repository's tools,
installed by then,
leaves the copy hidden,
which is what the sentence itself then says:

```text
toolchain install: mpm-shadow failed: cannot move the finished install to <toolchain root>/mpm-shadow/0.2.7-alpha.13: Input/output error (os error 5); the finished copy this run holds is still at <toolchain root>/mpm-shadow/.0.2.7-alpha.13.staging.4242.99.424242.22222222-2222-2222-2222-222222222222. A later install of that version publishes that copy without downloading when the version's name holds no finished copy by then, and removes it as a second finished copy when that name holds one; it acts on that name in this process namespace and after a reboot, but an install in another process namespace does not judge that name and leaves the copy where it is, and prune --toolchain prints that name and what deleting it costs; a newer version of a tool this repository publishes, installed by then, leaves that copy hidden, because an older repository tool is never kept
```

A recovery's own failure carries the same words and no sentence of its own,
because the install that follows it in the same run changes what is at the version's name;
that report's warning ends with the sentence instead,
and its own words say what this run then did.

The finished copy is what a later install publishes rather than the copy that was there,
because the marker is written after the archive's SHA-256 matched the lock and after the program was found,
while the copy at the version's name was unfinished,
which is why it was being replaced at all.

The completion marker records the tool,
the version,
and the archive's SHA-256 that the download matched before the copy was unpacked.
A later install publishes a hidden copy only when all three match the lock's row for that version,
so the path that skips the download keeps the same check the download makes.
A marker an earlier build wrote records the tool and the version but no archive digest;
it cannot vouch for the bytes,
so its copy is never published.
A name an earlier build wrote carries neither a boot nor a namespace,
so no run can show that its installer ended,
and a copy such a name holds is never removed either:
`prune --toolchain` prints that name on every run,
says what it read there,
and leaves it for a person.
An install of that version that cannot download leaves the version's name empty,
and its failure says that a finished copy of that version sits under that hidden name,
which is what an offline machine has left.
A copy this build wrote the name for is removed as an unverified leftover once its installer has ended,
and an unfinished copy of any version goes the same way;
a finished copy of another version is left for that version's own install to publish.

A recovery whose own publish fails keeps the copy it claimed:
the copy stays under its `.recovered` name with that run's pair,
and the failure names both the version's name and the path the copy is at.
Such a sentence promises nothing about a later run,
because only the run that follows reads what that one would find:
an install publishes the copy when the version may be kept and its own name holds no finished copy by then,
and removes it as a second one when that name holds one.
A recovery that fails does not stop the install that follows it:
the tool is installed from the network when the network is there,
the recovery's message is then a warning in the report,
and when that install fails too the report names both causes in one failure.
That warning ends with what this run then did.
When the install that follows put the version at its name,
the warning says `so the run does not fail for it`
and then the sentence for that state,
which names the removal alone,
because the version's name holds a finished copy by then and no install can publish the kept copy any more:

```text
A later install of that version removes that copy as a second finished copy, because the version's name holds a finished copy by then; it acts on that name in this process namespace and after a reboot, but an install in another process namespace does not judge that name and leaves the copy where it is, and prune --toolchain prints that name and what deleting it costs; a newer version of a tool this repository publishes, installed by then, leaves that copy hidden, because an older repository tool is never kept
```

when that install failed too,
the copy stays where the run's own sentence put it.
A put-back that fails names the copy that was there rather than a finished one,
so its warning stops at what this run then did and says nothing about a later install.
The one failure that names no copy is the tool's directory that could not be listed:
its warning says what this run then did and that the run does not fail for it,
and nothing about a copy it never read.
A recovery that failed is a warning,
and not a failure,
whenever this tree's lock is satisfied after the run:
the install that followed put the version there,
another install had already done so,
or a newer version runs,
and that run ends 0.
For a recovery that failed,
the run ends 1 only when the locked version is not at its name afterwards,
which is when the install that followed failed too.
A run whose recovery put the version at its name also ends 1 when the link that names the newest finished version cannot be placed,
even when a later step of the same run places it:
nothing is removed under a link that does not name the newest finished version yet,
so the version directories older than it stay until `prune --toolchain` removes them.
A publish that finds a copy at the version's name in the instant between its own check and its rename replaces it,
in the same step it uses for a copy that was there all along,
so a second install's put-back costs no finished copy.

The copy a killed install moved aside is put back only when no installer that may still run holds a claim or
a staging name for that version:
such an installer is between its claim and its publish,
and a copy put at the version's name in that instant would make its rename fail.
An installer that may still run is one the system reports as running,
and one whose name belongs to this boot but that this reader cannot see:
a process id in another process namespace,
a record `/proc` hides,
and a name that carries no start time.
Then the copy waits where it is,
and a later install puts it back once nothing may hold that version.
The wait has no end when the name belongs to this boot and its installer died where this reader cannot see it:
an install killed in another process namespace leaves such a name,
no later run can judge it,
the version's name stays empty after every install,
and each install prints which hidden name holds the wait and whether that name holds a finished copy.
A person ends it,
by copying that finished copy's contents into the version's name,
which reaches a name that holds nothing and one that already holds a copy,
or by deleting the unjudged name and letting the next install put the copy that was moved aside back,
which loses the finished copy the unjudged name holds.
A name written by an earlier build carries neither a boot nor a namespace,
so it cannot be placed in any boot,
and no install ever removes it;
waiting for it would keep the copy of a killed install aside for ever,
so it does not hold the copy back.
That is the cost this rule accepts for an earlier build's name:
a live install of an earlier build holding such a name loses the finished copy it holds,
because its own publish does not retry over a name that appeared.
`prune --toolchain` prints such a name,
and its line says what deleting that directory costs.

A dead install's copy is claimed before anything is published from it:
one rename to a `.recovered` name of this process's own,
so two installs recovering the same copy cannot both take it,
and the one that loses finds the name gone and publishes nothing.
Without the claim both would replace the copy at the version's name in turn,
and the second would leave the unfinished copy there and delete the finished one.

A hidden directory is removed only when its installer is known to have ended.
That question is answered by the facts in its name:
a name from another boot names an installer that reboot certainly ended;
a name from this boot and this process namespace is judged by the process id and the start time;
a name from another process namespace is not judged,
because a process id there means another process.
The answer has three values and only one of them allows a removal:
running,
ended,
and the system did not say.
A record that cannot be read is the third,
because an unreadable record is not a dead process.
When `/proc/<pid>/stat` says the number is free,
the kernel's `pidfd_open` is asked whether any process holds it,
so a `hidepid` mount that hides a live process's record does not make it read as ended.
A hidden directory whose name carries no boot or no namespace was written by an earlier release,
so it cannot be judged and is left where it is.
Nothing is read of that other process except its own `/proc/<pid>/stat` record and whether its number is held.
A finished copy under another version's name may be the only copy of that version,
so it is left for that version's own install to publish,
and `prune --toolchain` removes it once no known lock names that version.

`install` prints one sentence for each version a recovery touched,
and the blocks here are that sentence,
clause by clause:
the head says what became of a hidden copy,
and a removal is joined to it with `, and `.
The sentences the steps around a recovery print about what they found are not in this list:
the version the newest-only rule then removed (said below),
the warning a failed recovery becomes rather than a failure (said above),
the copy a failed download names,
and the tool directory no run could list.
The publish it finished,
naming the hidden name the copy was read at:

```text
toolchain install: finished the publish of mpm-shadow 0.2.7-alpha.13 from the finished copy an earlier install left at <toolchain root>/mpm-shadow/.0.2.7-alpha.13.staging.4242.99.424242.22222222-2222-2222-2222-222222222222
```

The copy it dropped because another install published the version meanwhile:

```text
toolchain install: dropped the copy of mpm-shadow 0.2.7-alpha.13 that an earlier install left, because another install published that version meanwhile
```

The finished copy it left hidden because an older repository tool is never kept:

```text
toolchain install: left the finished copy of mpm-shadow 0.2.7-alpha.13 under the hidden name an earlier install left it at, because 0.2.7-alpha.14 is installed and an older repository tool is never kept
```

The copy an earlier install had moved aside and this run put back:

```text
toolchain install: put back the copy of mpm-shadow 0.2.7-alpha.13 that an earlier install had moved aside
```

The copy it left where it is,
because an install that is still running holds a hidden name for that version:

```text
toolchain install: left the copy of mpm-shadow 0.2.7-alpha.13 that an earlier install had moved aside where it is, because an install that is still running holds the hidden name <toolchain root>/mpm-shadow/.0.2.7-alpha.13.staging.4343.101.434343.22222222-2222-2222-2222-222222222222
```

and,
when no run can judge the install that holds that name,
with the finished copy that name holds when it holds one:

```text
toolchain install: left the copy of mpm-shadow 0.2.7-alpha.13 that an earlier install had moved aside where it is, because the hidden name <toolchain root>/mpm-shadow/.0.2.7-alpha.13.recovered.4343.101.434343.22222222-2222-2222-2222-222222222222 may belong to an install that is still running, and nothing here can judge that install; that name holds a finished copy of 0.2.7-alpha.13, which nothing here publishes while that install cannot be judged
```

and the hidden directories it removed,
counted in the singular or the plural,
left behind by installs whose process the system reports ended,
with what went with each one that held a finished copy:

```text
toolchain install: removed 2 hidden directories left behind by installs of mpm-shadow that have ended
```

A removed finished copy is either a second one,
when the run reads the version's own name as finished,
or the one copy the version had here.
The sentence says what that read at the version's name was,
and never what a start of either copy does:
the test is a completion marker and a program found,
so a program cut short under its marker reads as finished too,
and the copy the version's name holds may be the one that cannot run.
A stronger test of that clause was priced and not taken:
keeping a second copy whose program differs means reading both programs on every recovery,
and it still cannot tell a program cut short from one rebuilt at the same size.

```text
toolchain install: removed 1 hidden directory left behind by an install of mpm-shadow that has ended, 1 of which held a second finished copy of 0.2.7-alpha.13, because 0.2.7-alpha.13 holds its completion marker and its program at its name
```

```text
toolchain install: removed 1 hidden directory left behind by an install of mpm-shadow that has ended, 1 of which held a finished copy of 0.2.7-alpha.13 that this run did not publish, because that copy's completion marker does not record the archive this lock names; no finished copy of 0.2.7-alpha.13 is at its name, so that copy is gone
```

A copy of a version that may not be kept beside a newer one says that rule instead:

```text
toolchain install: removed 1 hidden directory left behind by an install of mpm-shadow that has ended, 1 of which held a finished copy of 0.2.7-alpha.13 that this run did not publish, because 0.2.7-alpha.14 is installed and an older repository tool is never kept; no finished copy of 0.2.7-alpha.13 is at its name, so that copy is gone
```

A recovery never publishes an older version beside a newer installed one:
an older repository tool is never kept,
which the run also warns about,
so a version that is older than a finished one installed in either toolchain root stays under its hidden name,
where a later install publishes it if that version may be kept again.
A finished copy of that version whose marker records the lock's row stays with it;
one whose marker the lock does not vouch for is removed instead,
since no install can ever publish it.
After a recovery that does put a version at its name,
`install` points `current` at the newest finished version and then removes every version directory older than it,
the order an install keeps,
so a start between those two steps runs the newest finished version,
and a run killed there leaves a `current` that resolves.
A recovery that puts back the copy a killed install had moved aside counts here too,
even when that copy is unfinished:
`install` points `current` at the newest finished version,
which may be an older version than the one the lock names,
and when the install that follows fails,
`current` names that older version,
so a start runs it while `list` reports the version the lock names as drift.
The report names each version it removed that way and says what that version was in this run:
the one the recovery published,
or one that was installed before the run.

`prune --toolchain` also reports and removes hidden directories whose installer has provably ended,
except a finished copy whose marker records an archive digest,
for a version some known lock names:
such a copy is one that version's own install publishes or removes,
so `prune` leaves that decision to the install.
A hidden name it cannot judge is printed on standard error and left,
and the line says what that run read there and what deleting that directory would cost.
When a known lock names the program that finishes the tool's install,
a copy the run reads as finished is the version's only one here unless the version's own name reads as finished by the
same test:

```text
mpm-shadow: left <toolchain root>/mpm-shadow/.0.2.7-alpha.13.staging.42.100: the installer that made it cannot be judged, so it is not removed; it holds a finished copy of mpm-shadow 0.2.7-alpha.13, and deleting it by hand loses that copy, which nothing publishes while its installer cannot be judged
```

```text
mpm-shadow: left <toolchain root>/mpm-shadow/.0.2.7-alpha.13.staging.42.100: the installer that made it cannot be judged, so it is not removed; it holds a second finished copy of mpm-shadow 0.2.7-alpha.13, because 0.2.7-alpha.13 holds its completion marker and its program at its name, and deleting it by hand loses that second copy
```

That line names the two reads and no conclusion drawn from them:
whether the copy the version's name holds behaves when it is started is something this run did not read.
A directory the run reads as holding no finished copy loses nothing:

```text
mpm-shadow: left <toolchain root>/mpm-shadow/.0.2.7-alpha.13.staging.42.100: the installer that made it cannot be judged, so it is not removed; it holds no finished copy, so deleting it by hand loses no finished copy, once no install uses it
```

A tool no known lock names gives the run no program to look for,
so the completion marker is the whole read there:

```text
mpm-shadow: left <toolchain root>/mpm-shadow/.0.2.7-alpha.13.staging.42.100: the installer that made it cannot be judged, so it is not removed; it holds a completion marker, but no known lock names the program that would finish a copy of mpm-shadow 0.2.7-alpha.13, so deleting it by hand loses whatever that name holds, which nothing publishes while its installer cannot be judged
```

```text
mpm-shadow: left <toolchain root>/mpm-shadow/.0.2.7-alpha.13.staging.42.100: the installer that made it cannot be judged, so it is not removed; it holds no completion marker, so deleting it by hand loses no finished copy, once no install uses it
```

To delete such a directory by hand,
remove the hidden name whose version and purpose it carries,
for example `rm -r <toolchain root>/mpm-shadow/.0.2.7-alpha.13.staging.4242.99.424242.22222222-2222-2222-2222-222222222222`
on the shape this build writes,
or `rm -r <toolchain root>/mpm-shadow/.0.2.7-alpha.13.staging.4242.99` on the shape an earlier build wrote,
once no install is using that tool.

### One set across both toolchain roots

The repository scope installs into `<data home>/monochromatic-dev/toolchain` and the machine-global scope into `<data home>/monochromatic-dev/toolchain-global`.
For this repository's own tools the two roots are one set,
by the owner's decision:
the newest finished copy wins wherever it is.
- The copy that runs is the newest finished version across both roots,
  and the same version in both goes to the scope that asks.
- A scope whose lock pins an older version than the one that runs installs nothing for the tool (`list` counts the newer copy as satisfied),
  keeps no older copy of its own,
  links and launches the newer copy,
  and warns on every run:
  in `install`,
  `list`,
  `update`,
  `outdated` and `bin`,
  and each time the tool runs through its launcher.
- The warning names the upgrade command of that scope as mpm's own pnpm-style verb with the tool id,
  so it moves only that row:
  `mpm update <tool id>` for a tree,
  and `mpm-shadow update -g <tool id>` for the machine-global lock.
  The id is printed as one shell word,
  single-quoted when it holds anything other than letters,
  digits and `@%+=:,./_-`,
  so the command can be pasted as it is.
  It names no mise task,
  because the repository is moving away from mise.
- The machine-global sentence names `mpm-shadow`,
  never `mpm`.
  Only a worktree's `PATH` makes `mpm` this tool:
  the `mpm-shadow` plan entry's postinstall link puts it first under `.cache/mpm-shadow-bin`.
  The machine-wide `mpm` is the real mpm of the global plan's `mpm` entry,
  and it reads `update` as its own `upgrade` verb,
  so a sentence naming `mpm` failed wherever a shell stood outside a worktree
  (`Usage: mpm upgrade [OPTIONS] [PACKAGES_SPECS]...`,
  `Error: No such option '-g'.`,
  measured 2026-11-08).
  A machine whose global plan declares no `mpm-shadow` entry has none of that plan's entries on `PATH` either,
  so the machine-global remedy named a program no shell outside a worktree could find
  (`/bin/sh: line 1: mpm-shadow: command not found`,
  exit status 127,
  measured 2026-11-11 in a home laid out like this machine's).
  The scope's own install puts it there:
  the farm also links the running `mpm-shadow` under its own name,
  which is the program every sentence prints,
  so the command runs as printed in a fresh shell.
  See "The farm also carries the running `mpm-shadow`".
- The exposed links,
  postinstall links and postinstall commands of both scopes use the winning copy,
  so `mpm-shadow install --global` makes `~/.local/bin/git` run the repository root's newest wrapper (through the global launcher) when that is the newest.
- A scope whose lock pins a newer version than any copy installs it into its own root,
  and `install` then retires the older managed copies in the other root.
- The sentence for the machine-global lock,
  in the same words everywhere the tree's sentence appears:

  ```text
  <tool id> is locked at <locked> in this machine's global lock, <path to that lock file>, but <used> runs, because older repository tools are never kept once a newer one is installed; upgrade the global lock with `mpm-shadow update -g <tool id>`
  ```

#### Older managed copies

After its own installs and links,
every `install` removes the older managed copies of this repository's tools:
- Every finished version older than the one that runs,
  in either root,
  is removed (a release archive's downloads with it).
- Equal versions stay,
  and a directory that is not finished is never judged.
- Links that resolve to such a version (in `~/.local/bin`,
  the global farm and every checkout's `.cache/*-bin`) move to the copy that runs first,
  with the same atomic replacement as for old copies in the shared cargo bin directory,
  and the places are searched again before the removal.
- The emptied root keeps its `current` link,
  which now forwards to the other root's `current`:
  a launcher record or a link that names the emptied root's `current` keeps running the newest copy and never dangles.
  The forward is made before the versions go,
  and only if the root still holds no finished version outside the ones being removed when the plan is applied
  (another tree may have installed a newer one meanwhile,
  and it owns its `current`).
- When the emptied root's scope installs a newer version,
  its `current` becomes a real link again and the other root forwards instead.
- `prune --toolchain` leaves a crate directory that holds only such a forward,
  and never removes a cargo version that the other root's `current` forwards to (it reads only this scope's locks).
- Every `install`,
  in either scope,
  that finds the tool already installed points the winning root's `current` at that root's newest finished version first,
  so a link left stale there by a run killed between the completion marker and the link cannot make another scope run an older copy.
- The older copy is never the only working one:
  it is kept when the tool's own install failed in this run,
  or when the newest copy's program for a bin is missing or not executable.
- In the other scope's root it is kept while a place could not be searched (git could not list the worktrees,
  a listed worktree is missing,
  a directory could not be read),
  because that scope's links are not this run's own.
- An `install --global` searches no checkout,
  so it keeps the repository root's older copies and a later `install` inside a checkout removes them.
- The scope's own older copies go whatever the search found.
- A clone that git does not list for the repository is unseen:
  its launchers keep working through the forward,
  and a link that names a version directory itself dangles until that tree's next `install`.

### What the exposure folder holds

The scope's exposure folder is the directory its `PATH` puts first,
and it holds one link per declared bin
(`.cache/toolchain-bin` in a checkout,
`<data home>/monochromatic-dev/toolchain-global/exposure` for the machine-global scope).

- A link is named after the bin its row declares,
  and points at the program that row's route finds.
  A repository-published tool's link points at this tree's launcher instead,
  so a run through it can warn about the lock (see "The launcher").
- Every link an install makes is replaced atomically,
  so a run killed while the folder is rebuilt leaves no half-written name:
  the new link is written under a dotted temporary name beside the real one and renamed over it,
  and a kill between those two steps leaves that temporary name behind.
- Every rebuild also removes what the lock no longer declares:
  a symbolic link whose name no row of the lock declares is removed,
  and the install's report names each one as it removes it and says where it pointed
  (`removed the exposure link rustup (it pointed at ...), which no row of the lock declares`).
  A run that fails at a later removal still names the removals it made before that failure,
  and a link another run removed or renamed away in the instant before this run reached it is no failure here:
  the removal reads the entry's own metadata,
  then the link's target,
  and then takes the link away,
  and a name that any of those three calls no longer finds is gone,
  which is what both runs wanted,
  and the run that took it named it.
  A regular file,
  a directory,
  or anything else at such a name belongs to a person and stays.
  Two kinds of link stay that a name alone would mark:
  - the program's own link,
    which the rebuild made when the running binary belongs to an install,
    and which a rebuild keeps while it resolves into an installed `mpm-shadow` of either toolchain root,
    whatever binary runs that rebuild
    (see "The farm also carries the running `mpm-shadow`");
  - the dotted temporary name of a link an install is in the middle of writing,
    which is that install's while that install may still run,
    and which a rebuild removes and names once it can show that install has ended
    (`removed the temporary link .thirdbin.tmp.... (it pointed at ...), a name of the shape an install writes while it holds a link, and the process that name records is not running`).
    The name carries the facts that answer that question:
    its process id,
    the moment that process started,
    the process namespace that gave the id out,
    and the boot's identifier,
    the facts a hidden name of an install carries too
    (see "What a kill leaves and what the next install does about it").
    The sentence says what the name is and what the run could show of the process it records,
    never that an install wrote it:
    a person's own dotted link can carry the same shape,
    and nothing in the folder tells the two apart.
    A name this build cannot judge against those facts,
    such as one from another process namespace,
    keeps its link,
    and a rebuild names it once and says what deleting it by hand would risk:

    ```text
    toolchain install: left the temporary link <name>: this run cannot judge the install its name records, so it is not removed, and deleting it by hand may make that install fail where it gives the link its name
    ```

    The pass reads the folder's entries in the order the directory lists them,
    and a failing read or removal stops it,
    so a rebuild that stops at a failing read or removal names the names it reached before that
    failure
    and the next install names the ones after it.

    A name an earlier build wrote carries the process id alone,
    and the shorter name this build writes when it cannot read its own boot or process namespace carries a process id and a start time and no namespace:
    no run can show that either name's process is the one that wrote it,
    or that the process was ever in this run's process namespace,
    so both stay,
    named the same way,
    and a person deletes one by hand once no install is running in the folder.
  The `rustup` link this repository's `rust` row left behind when it stopped declaring it
  (issue 6626) is the case the rule comes from:
  it dangled once the proxies went,
  and `list` said the toolchain was in sync (issue 6634).

#### The upgrade, from a release that removes temporary links by number

A release before this one tells a temporary link from a leftover by the process number alone,
and takes any such link whose number no process holds in its own process namespace.
A farm that two sessions rebuild,
one with this release and one with an earlier one,
therefore has two removers with two rules:

- The earlier release takes this release's live temporary link wherever the number is free in its own namespace,
  and the install that holds the link then fails at the rename that gives it its name
  (`cannot place the link <farm>/thirdbin: No such file or directory`).
- This release keeps a live temporary link an earlier release wrote,
  because that name records no process namespace for this run to judge,
  so an install of this release no longer takes an earlier release's live link.

Nothing here can change what an earlier release's remover does.
The window closes when every session that rebuilds the farm runs this release,
and this repository's lock has named `0.2.7-alpha.19` alone since 2026-11-27,
so the scopes that lock covers are past it.

### The farm also carries the running `mpm-shadow`

Every sentence `mpm-shadow` prints tells a person to run `mpm-shadow` or `mpm`,
so every command that syncs a scope (`install`,
and the `update` that syncs the tools it needs)
makes those names reachable from a fresh shell:

- The scope's exposure farm holds a link named after the running `mpm-shadow`,
  pointing at the installed program through the tool's `current` link,
  so a later version move never leaves it dangling.
  It is made when no tool of the scope's lock declares that bin:
  a plan row that declares it owns the farm link and the launcher record behind it,
  even when that row installed nothing.
  It is never made for a program that belongs to no install,
  so a run from a build folder puts no build on the machine's `PATH`
  (`src/toolchain/exposure.rs`,
  `expose_running_shadow`).
  A rebuild keeps it while it resolves into an install of `mpm-shadow` in either toolchain root,
  whatever binary runs that rebuild,
  so a run from a build folder,
  a `cargo install` binary or the bootstrap archive
  no longer takes the name away from a scope that has the program installed.
  An install of `mpm-shadow` is the program's own install directory below either toolchain root:
  `<root>/mpm-shadow` for a release archive,
  or `<root>/cargo/monochromatic-mpm-shadow` for the cargo route.
  A file named `mpm-shadow` inside another tool's install directory is not one,
  and the link to it goes with the other links no row declares.
  A scope that has the program installed nowhere,
  or a link that resolves nowhere,
  removes it as a link no row declares
  (see "What the exposure folder holds").
- The repository plan's `mpm-shadow` row also links `mpm` in `.cache/mpm-shadow-bin`,
  which a worktree's `PATH` holds.
  A machine-global plan need not declare `mpm-shadow` at all,
  and this machine's does not,
  so before the farm link the machine-global sentences
  (`mpm-shadow update -g <tool id>`,
  `mpm-shadow install -g`,
  `mpm-shadow --help`)
  named a program nothing outside a worktree could find
  (`/bin/sh: line 1: mpm-shadow: command not found`,
  exit status 127,
  measured 2026-11-11 in a home laid out like this machine's).
- An install is found by the file it is,
  never by the spelling of its path.
  `/proc/self/exe` reports the resolved spelling (on this machine `/home` is a link to `/var/home`),
  while a scope's root keeps the spelling `HOME` gives,
  and comparing the text found no install at all,
  so every launcher was a copy of whichever program ran `install` instead of the newest installed one.
  A launcher made from a program reached through a per-program link is a hard link of the file itself,
  never of the link,
  because a hard link of a symbolic link resolves its relative target beside the launcher and dangles.
- `tests/printed_remedy.rs` builds a disposable home with that layout,
  runs each printed sentence's own command in a fresh shell,
  and asserts every program it names resolves through the scope's install
  (measured 2026-11-11:
  `mpm-shadow update -g cargo:monochromatic-git-policy-cli` moved the fixture's global lock to the version that runs
  and the warning was gone).

### The launcher

A bare symlink to the binary has nowhere to check the tree's lock,
so the exposed link of a repository-published tool (`.cache/toolchain-bin/<bin>`)
and its postinstall links (`.cache/monochromatic-git-policy-cli-bin/git`) point at a launcher in `.cache/toolchain-launchers/<bin>`.

- The launcher is a hard link of the newest installed `mpm-shadow` (a copy across file systems),
  whichever copy ran `install`:
  the running program's install directory is found from its own path (the nearest ancestor that holds a `current` link),
  and the program under that link in whichever of the two toolchain roots names the newest version is copied
  (`src/toolchain/launcher_source.rs`).
  A program that belongs to no install (a build folder) is copied as it is.
  Before this rule the launcher was a hard link of whichever `mpm-shadow` ran `install`,
  so an installer older than the newest installed one placed a launcher
  that followed no `current` and rewrote no record:
  that is how the machine-global `git` launcher stayed a `0.2.7-alpha.2` copy and started a deleted `0.6.0` folder
  (`doc/troubleshooting/mpm-shadow-launcher-names-a-deleted-release-folder-after-a-newer-install.md`).
  The record beside it names the tool id,
  the tree's lock file,
  the real binary
  and which scope made it (the warning names that scope's upgrade command).
  The real binary is named through the tool's `current` link,
  `current/<program name>` for a release archive and `current/bin/<name>` for a crate,
  so the record names no version.
  The kernel resolves a symlink chain to its final file,
  so a hard link per tree is what lets the running process learn which tree's link started it
  (`/proc/self/exe`).
- Started under that path,
  `mpm-shadow` reads the record and the lock,
  prints the warning above on standard error when the version behind `current` is newer than the lock pins
  (the line ends with `; starting <path>`,
  the program it is about to start,
  never a path a stale record named),
  then replaces itself with the real binary (`exec`):
  argv,
  environment,
  standard streams,
  exit code and signals are the real program's own,
  and standard output is never written.
  An unreadable lock or record never blocks a run.
- The program is the one the record names,
  or,
  when that file no longer exists,
  the program of the same file name under the `current` link above it.
  A record written by a release that carries the newest-only rule names `current/<program name>`,
  which survives every version move;
  one written before that rule names the folder the release archive unpacked into,
  whose name carries the version
  (`monochromatic-cargo-shadow-0.1.4-x86_64-unknown-linux-gnu`),
  so a record written while `current` named 0.1.4 holds a path through that folder,
  and installing 0.1.5 in any tree deletes it.
  Following `current` therefore runs the program the version that runs holds,
  and the rewrite below names `current/<program name>`,
  a path no later version move can stale.
  Before that rule every other tree's launcher exited 127 with `mpm-shadow: could not start <path>: No such file or directory`
  although `current` named a working program
  (measured 2026-11-07:
  122 of the 853 launcher records in this repository's checkouts named a deleted folder).
  A launcher whose record is stale therefore runs the newest copy without a new `install` in its tree,
  and the older-lock warning still names the locked version,
  the version that runs,
  and the path of the lock file it compared,
  which is the lock its own record names.
  The launcher also rewrites its own record to name the program it started (atomically,
  only that field),
  because the shadows behind it (`cargo-shadow`,
  `pnpm-shadow`,
  `search-shadow`) skip a `PATH` candidate only when its record leads back to them,
  and a record that names a deleted folder cannot be resolved:
  without the rewrite `cargo-shadow` took the launcher for the real `cargo`,
  started it again,
  and its recursion guard stopped the run.
  A record that cannot be rewritten is one warning line on standard error and the program still runs.
- A launcher never starts itself.
  Before it replaces itself with the program it chose,
  it follows the records from that program (`src/toolchain/launcher_chain.rs`):
  a program that is this launcher,
  a launcher already followed,
  or a chain of more than eight launchers
  is refused with exit status 126 and a message that says which,
  and nothing is started.
  Before this rule a record that named its own launcher,
  or two records that named each other,
  made each launcher replace itself with the other in the same process without end
  (measured 2026-11-07:
  both bounded tests ran into their five second bound).
  A launcher that names another launcher of a real program is fine.
  A launcher sets no arrival marker and checks none:
  a launcher in front of a shadow is the intended arrangement (`.cache/cargo-shadow-bin/cargo`),
  and the shadow behind it decides (it stops when a shadow that started the launcher's process is the one that reached it).
  It makes no check for a shadow either:
  `install` links `mpm` to a launcher whose record names that tree's `mpm-shadow` binary,
  so a record naming a shadow is the arrangement,
  not a loop,
  and the words for a refused shadow belong to the check the shadow makes on its own start
  (`tests/never_start_another_shadow/launchers_that_must_not_start_themselves.rs` runs the pair through).
  A tree that has not run `install` since these rules shipped still has the launcher binary it had,
  and such a binary exits 127 for a record that names a deleted folder until `mise run toolchain:install`
  or `mpm install` runs in that tree
  (that run places the newest binary and rewrites the record from `current`).
  The rule above fixes the binary a launcher is placed *with*,
  not the ones already on disk:
  a census on 2026-11-08 found 891 of 1132 launcher binaries without the follow-`current` and record-rewriting code.
- When no program of that name exists under `current` either,
  the launcher exits 127 without starting anything:
  it checks that the program is a file before it starts it,
  prints `mpm-shadow: not starting <path>: no file exists there` with no older-lock warning,
  and then
  one sentence naming the tool,
  the version the lock pins,
  the version `current` names
  and the command to run:
  `mpm install` in the tree (`mpm-shadow install -g` for the machine-global launcher,
  because the machine-wide `mpm` is the real mpm and refuses `-g`).
  An unreadable lock or link leaves out the version it would have named.
  A file that vanishes between the check and the start still gets a `could not start` line:
  the message names the file that is missing,
  read from the program's own bytes (`src/named_interpreter.rs`):
  the interpreter its first line names for a script,
  or the dynamic linker a compiled program's own file names,
  and both kinds and neither when the program itself is gone,
  because `execve` answers one error for both;
  the message gives the command that writes the tool's links again,
  and then installing the interpreter or the linker when that is the file that is missing.
  A start the system refused because another process holds a file it needs open for writing gets the same explanation the shadow mode gives:
  some process holds open for writing the program's file,
  or the interpreter the program's own file names,
  which is the program after `#!` on a script's first line or a compiled program's dynamic linker,
  and which of the two whenever the program's own file says so,
  the shadow cannot tell which process holds it or why,
  the same command works once that write has finished and left a whole program,
  and a write cut short leaves a broken program,
  which installing the tool again replaces.
  The launcher does not retry it either,
  for the reason the shadow mode does not.
- The launcher never reads `PATH` or the working directory,
  so which tree's launcher runs depends only on the path it was started by.
  Mise puts the tree's own `.cache/*-bin` directories first on `PATH` for `mise run`,
  `mise exec` and an activated shell in it,
  so a task started that way runs the tree's own launchers,
  and a process started from another checkout's shell without mise (an agent session begun in the main checkout,
  for one)
  finds that checkout's `.cache/*-bin` first.
  Both run the same newest copy;
  they differ only in which lock the older-lock warning compares,
  and the warning names that lock file's path,
  so two runs of the same message are told apart by the line's own text.
- Measured 2026-10-31 with `hyperfine` 2.0.0,
  400 runs per arm,
  `cli-git-native --version`
  (the git wrapper handing `--version` to git),
  order bare,
  launcher,
  copy of bare,
  launcher with warning,
  bare,
  launcher:
  median wall time 1.47 ms bare against 2.04 ms to 2.09 ms through the launcher,
  so the launcher adds about 0.6 ms (about 0.5 million instructions) to a call that takes 1.5 ms.
  Bare against a bare copy and a repeat of bare differed by at most 0.04 ms.

### Cleanup

`mpm-shadow prune --toolchain [--global] [--dry-run] [--lock <file>]...` deletes what no known lock names.
It runs only when asked;
`install` never deletes another tree's version.

- Known locks are this scope's lock,
  the `toolchain.lock.json` of every worktree `git worktree list --porcelain` reports,
  and each file given with `--lock` (a clone that git does not list).
- Hidden installs under `<toolchain root>/<tool id>/` are judged too:
  one whose installer has provably ended is removed,
  except a finished copy whose marker records an archive digest,
  for a version some known lock names.
  Such a copy is one that version's own install publishes or removes,
  so `prune` leaves that decision to the install.
  A name whose installer cannot be judged is printed on standard error and left,
  and its line says what the run read there and what deleting the directory would cost;
  that covers another process namespace,
  a name an earlier release wrote without a boot and a namespace,
  and a copy whose marker an earlier release wrote without an archive digest.
  Delete such a directory by hand once no install is using that tool,
  which costs the copy its line names.
- A third-party version is kept when any known lock names it;
  a repository-published crate keeps only its newest finished version,
  and nothing when no known lock names the crate.
  A release archive of this repository that some known lock names keeps only its newest finished version too,
  and its `current` link follows;
  an archive directory no lock names as a release archive is never judged.
- It refuses,
  deleting nothing,
  when git cannot list the worktrees,
  when a listed worktree's directory is missing and not marked prunable,
  or when a known lock exists but cannot be read,
  or when a known lock names a release archive of this repository under an id,
  version or first bin that is not one
  plain directory name.
  A lock in the TypeScript tool's format is read for `tools.<id>.version`,
  `route` and `packageId` only.
- When two known locks name the same release archive with different first bins,
  the lock read last decides which program shows an install finished
  (this scope's lock first,
  then the worktrees',
  then each `--lock` file).
- A `--lock` is the only way to protect a tree that git does not list.
- What it reads is a snapshot:
  a lock another command replaces between this run's read and its deletions is not read again,
  so the plan can delete a version the newer lock names.
  That takes a second command writing the lock inside that window,
  and the next `install` puts the version back.
  A version that arrives after the installed versions are listed,
  which is any concurrent install,
  is never in the plan at all.
- A concurrent install's `current` link is read just before the prune would remove it,
  and a link naming a version that still exists and is not one of this run's deletions is left alone.
  The instant between that read and the unlink is the remaining window:
  a `current` another command moves inside it can still be removed.
- The link is likewise read just before the prune would move it:
  a version that is not on disk at that moment is never what `current` is pointed at,
  and a link already naming a finished version newer than the one the plan would pick stays.
  The window left is the instant between that read and the link write,
  so a version another command deletes inside it can still leave the link naming a directory that is gone;
  the next `install` repoints it.
  No lock and no wait is added for it.

### Migration from the shared `$CARGO_HOME/bin`

The first `install` with this layout finds no version directory,
installs every `cargo-binstall` tool into its own,
and links the tree's exposure and postinstall links to it.
`list` reports such a tool as absent until that first `install`.
Nothing copies or moves the legacy files,
and a third-party tool's file in `$CARGO_HOME/bin` and its entry in `$CARGO_HOME/.crates.toml` stay exactly as they were.
This repository's own tools are different:
their legacy copies in `$CARGO_HOME/bin` are old copies,
which the same `install` retires (see "Old copies of this repository's tools").

### Postinstall links the lock no longer declares

A postinstall link (such as `cargo` for `cargo-shadow` in `.cache/cargo-shadow-bin`) that is dropped from the plan,
renamed,
or moved to another directory used to stay where it was,
and a directory early on `PATH` kept running a tool the plan had stopped linking.
`install` now keeps a record of the links it makes,
`.postinstall-links.json` in the scope's launcher directory,
and removes a recorded link the lock no longer declares.

- A recorded link is removed only while it is still the symlink the record names.
  A regular file,
  a directory,
  or a symlink to somewhere else at that path belongs to a person:
  it stays untouched,
  `install` prints a warning that names the path and what is there,
  and the path leaves the record so the warning appears once.
- A link made before the record existed is not in it,
  so it is never removed;
  delete such a link by hand once.
- A repository scope retires only links inside its own checkout,
  and none inside a checkout nested in it
  (a directory with a `.git` entry between the checkout root and the link).
  A new worktree starts as a snapshot of the checkout it was made from,
  `.cache` included,
  so its first record names that checkout's links by absolute path;
  `install` in the new worktree never inspects or removes those paths and leaves them out of the record it writes.
  Before this rule,
  that install removed the source checkout's links and left its `.cache/*-bin` folders empty
  (`doc/troubleshooting/mpm-shadow-record-copied-into-a-new-worktree-retires-the-source-links.md`).
- The global scope keeps retiring every path its own record names,
  because nothing copies that record.
- A repository scope no longer retires a link whose plan `linkDir` is an absolute path outside the checkout;
  the plan uses only relative ones,
  and such a link is left where it is.
- A partial `install` (named tools,
  or the `update` that installs a few) keeps the recorded links of the other tools.
- An unreadable record retires nothing and says so.
- Each removal is one line of the install report (`retired the postinstall link ...`).
- Where the link is a hard link instead of a symlink (a platform without symlinks),
  it counts as a person's file
  and is left with a warning.

### Old copies of this repository's tools

The rule,
in the owner's words:
"No older repo-maintained tools should ever exist when newer is present."
An old copy is a regular file in the shared cargo bin directory (`$CARGO_HOME/bin`,
default `~/.cargo/bin`) that is named like a bin the scope's lock declares for a repository-published tool and that mpm-shadow did not install.
Repository-published has the meaning of the section "Cargo tool installs":
a `cargo-binstall` crate named `monochromatic-*`,
or an `archive` tool of `github:Aquaticats/Monochromatic` with a `monochromatic-` tag prefix.
The retired TypeScript sync put such copies there (measured on the owner's machine on 2026-11-01:
`cli-git-native`,
`mpm-shadow`,
`search-shadow`,
`monochromatic-lint` and `monochromatic-nested-wayland-session`),
and links in `~/.local/bin` and in every checkout's `.cache/*-bin` still run them.
Cargo's records cannot say which copies are old:
on that machine `.crates2.json` named `monochromatic-git-policy-cli` 0.2.13 for a 0.4.5 binary and had no entry for `mpm-shadow`,
and a copy placed by hand has no record at all.
So the decision rests on where the file is.
`.crates.toml` (the file `cargo uninstall` reads) is consulted for three things only:
to leave alone a bin it attributes to a crate this repository does not publish,
to keep a copy it says is newer than the managed one,
and to choose between `cargo uninstall` and removing the file.
No runbook step removes an old copy by hand:
`install` does it,
as the headings of this section describe.

#### The order

The owner chose "Re-point,
then remove":
"After its own newest copy is installed:
links in known places (~/.local/bin,
every checkout's .cache/*-bin,
the global farm) that target an old copy are re-pointed to the managed copy;
then the old copy and its cargo record are removed,
each action printed."
The cost the owner accepted:
mpm-shadow edits links it did not create,
but only links that resolve to an old copy of this repository's own tool.
Every `install`,
in the repository scope and with `--global`,
does this once its own installs,
links,
exposure directory and postinstall commands are done:
- Each link in a known place that resolves to the old copy is replaced by a link to the managed copy,
  atomically:
  a temporary link is renamed over the old one,
  so a reader sees one or the other,
  and the file the old link pointed at is never written through.
- Only when no link that could be seen resolves to the copy any more,
  and the managed copy is still an executable file,
  the copy is removed:
  with `cargo uninstall --root <cargo home> --bin <bin> -- <crate>` when `.crates.toml` names the bin (which updates cargo's records and leaves every other bin of that crate alone),
  otherwise the file itself.
- The crate name comes from a file this program did not write,
  so it is read again at removal,
  must still start with `monochromatic-`,
  and follows `--` so cargo never reads it as an option.
- A copy that another run already removed (the file,
  or the file and its record after a `cargo uninstall` that failed on the lock) counts as done,
  not as a failure.
- A copy that `.crates.toml` does not name at all (placed by hand,
  or from a release archive) is removed the same way after its links moved,
  by the owner's answer of 2026-11-01 to "delete it or keep it":
  "Keep it as built:
  any unrecorded copy with a declared bin name is removed after its links are re-pointed."
- Each action is printed after the counts line,
  as `toolchain install: re-pointed <link> -> <managed copy>` and `toolchain install: removed <copy> (...)`.
- A second `install` finds nothing to do and prints nothing.

The mpm-shadow that runs can itself be the old copy it removes.
On Linux it keeps running from its open file,
and the launchers it made were made before the removal.

#### Known places

- `~/.local/bin` (the global scope's link directory) and the global exposure directory (`<data home>/monochromatic-dev/toolchain-global/exposure`).
- The shared cargo bin directory itself,
  because a link beside the copy (`git` to its sibling `cli-git-native`,
  relative) would dangle once the copy is gone;
  rustup's proxies and links to other programs there are never touched.
- Every directory under `.cache` whose name ends in `-bin` (`monochromatic-git-policy-cli-bin`,
  `mpm-shadow-bin`,
  `search-shadow-bin`,
  `toolchain-bin`) in the scope's own worktree and in every worktree that `git worktree list --porcelain` reports.
- Only symbolic links directly in those directories are looked at.
- A link is judged by the file it finally resolves to,
  never by its text:
  on the owner's machine `/home` is a link to `var/home`,
  and links to one copy spell its path both ways.
- A link to a foreign file,
  a link that dangles and a regular file (even one with the old copy's bytes) are never touched.

#### Which managed copy a link gets

A re-pointed link names the `current` path of the copy that runs (`<toolchain root>/cargo/<crate>/current/bin/<bin>`,
or for a release archive the program under `<toolchain root>/<tool id>/current`),
so a later version change never leaves it dangling.
That copy is the newest finished one across both toolchain roots (see "One set across both toolchain roots"),
for a link in `~/.local/bin`,
in the global farm and in a checkout alike.
The next `install` in a checkout replaces its link with that tree's own launcher link as usual.

#### When a copy stays

An old copy is kept,
with `toolchain install: warning: old copy <path> of <id> is kept ...` and the reason,
when:
- the tool's own install failed in this run,
  or no finished managed copy of the bin exists (a version directory that cargo never recorded does not count),
  because the old copy may be the only working one.
- cargo's record says the old copy is newer than the managed one:
  upgrade this tree's lock with `mpm update <tool id>`.
- the entry in the cargo bin directory is a symbolic link and not a copy.
- some place could not be searched:
  git could not list the worktrees,
  a listed worktree's directory is missing and not marked `prunable`,
  a directory could not be read,
  or the run is `install --global`,
  which has no repository and searches no checkout (run `mpm-shadow install` inside a checkout to finish).
  The links that can be seen are still moved.
- a link to the copy appeared after the search,
  or a link could not be re-pointed (that is also a failure,
  so `install` exits 1).

A file that cargo's record attributes to a crate this repository does not publish is another program and is not judged at all.
A `cargo uninstall` that fails (a locked package cache,
an unreadable record) keeps the copy,
whose links already moved,
and is a failure with cargo's message and the remedy:
fix what cargo reports and run `install` again.

#### Flags and other commands

- `install`,
  `install --no-frozen-lockfile` and `install --frozen-lockfile` (the default in CI) all do the cleanup:
  freezing the lock only stops `install` from resolving plan entries into it,
  and the cleanup reads the lock and the file system and uses no network.
- `install --lockfile-only`,
  with or without `--frozen-lockfile` or `--offline`,
  installs nothing,
  so no managed copy is made or checked and nothing is re-pointed or removed (`--offline` exists only together with `--lockfile-only`).
- `update` and `outdated` never do it,
  not even when `update` installs a prerequisite for a lookup.
- The removal of older managed copies in either root (see "Older managed copies") follows the same flags:
  `install` does it with or without `--frozen-lockfile`,
  `--lockfile-only` never does,
  and `list` names what `install` would remove.
- `list` only reads:
  for each old copy it prints a `toolchain list: warning:` line that says what `install` would do (which links it would re-point,
  and that it would remove the copy) or why the copy stays,
  and an old copy is not drift,
  so the exit code does not change.
- `prune --toolchain` does not remove old copies:
  it deletes version directories under the toolchain root that no known lock names,
  an old copy lies outside that root,
  and judging one needs the managed copy that only `install` makes sure of.
- A tree still on the TypeScript toolchain keeps using `$CARGO_HOME/bin`,
  so a copy it installs there is an old copy again,
  and the next `install` removes it.

#### Measured behaviour of cargo

Measured on 2026-11-01 (UTC) with cargo 1.101.0-nightly in disposable homes that held copies of the owner's records:
- `cargo uninstall --root <cargo home> monochromatic-git-policy-cli` removed the file and the entries of both records although `.crates2.json` named another version.
- A crate named only in `.crates2.json` is "did not match any packages" (exit 101),
  and so is a crate named in neither record,
  so `.crates.toml` decides whether cargo is asked.
- `--bin <name>` on a crate with several bins removed that bin only and rewrote the entry with the others.
- `cargo uninstall --root <cargo home> --bin <name> -- <crate>` is accepted:
  `--` before the crate name changes nothing for a real name.
- An unparseable `.crates2.json` made `cargo uninstall` fail with a message naming the file,
  in a run of the built binary against a disposable tree.

## Network

Every npm and PyPI version,
first or later,
is looked up directly at `registry.npmjs.org` and `pypi.org` over HTTPS.
crates.io,
`static.rust-lang.org`,
`nodejs.org`,
`go.dev`,
`proxy.golang.org`,
`ziglang.org`,
`api.adoptium.net`,
`dl.google.com`,
`api.anaconda.org`
and the GitHub API are asked the same way.
The client is `hyper` with `hyper-rustls` (native root certificates),
HTTPS only,
with bounded timeouts and sizes,
and every failure names the URL.
`curl` is never started.

When an npm or PyPI lookup fails for a tool that is in the lock but not installed,
plain `update` installs just the missing tools (and `node`,
`pnpm`
or `uv` when they need them),
then looks each one up again once.
A second failure is final and says the lookup was made twice.
`outdated` never installs.

## Concurrency

`install --lockfile-only` and `update` read the plan and lock,
remember their content hashes,
take an exclusive OS lock for the scope,
read the hashes again,
and publish the new lock (a temporary file renamed over the old one) only if neither file changed.
A changed file refuses the publish and names the file.

## Bootstrap

A fresh machine needs one prebuilt `mpm-shadow` archive whose SHA-256 is in the lock,
or `cargo install monochromatic-mpm-shadow` after a verified `rustup` install.
The archive route installs no `rustup`,
and neither does `mpm-shadow` on any route:
both need a `rustup` the machine already has,
from `rustup-init` or from the distribution's package,
because the `rust` row asks it for the toolchain,
its components and its targets.
The `cargo install` route is the other way a machine gets `mpm-shadow`,
and the `cargo` it runs is the one a verified `rustup` install provides.
`mpm-shadow verify-archive` checks a downloaded archive against the lock before it is run.
Containers and CI can use the same `jq`,
`curl`
and `sha256sum` pattern they use for Node and pnpm.

## mpm shadow mode

mpm 8.2.0 has no cleanup for its `cargo` manager and removes no previous version on install or upgrade
(`doc/decision/mpm-toolchain-adoption.md`,
section on pruning the repository's own crates).
Plain `cargo install` fills `registry/cache/<index>/<crate>-<version>.crate`
and `registry/src/<index>/<crate>-<version>/` with every version it ever fetched.

- Runs the real `mpm` as a child,
  not by replacing itself,
  because the prune has to run after mpm finishes.
- Passes arguments through byte for byte,
  including ones that are not valid UTF-8,
  and adds nothing to standard output.
- Exits with mpm's exit code,
  or `128` plus the signal number when mpm died from a signal.
- Prunes after every run that started,
  whatever the exit code and whatever the verb.
  A run that could not start prunes nothing.
- Touches only crates whose name starts with `monochromatic-`.
  For each,
  it keeps the newest version an install record names as installed,
  or the newest cached version when no record names the crate,
  and deletes every other cached archive and extracted source.
  The records are the version-keyed ones `install` writes under `<toolchain root>/cargo/<crate>/<version>/.crates.toml`,
  read in both the repository root and the machine-global root
  (`<data home>/monochromatic-dev/toolchain` and `toolchain-global`,
  `XDG_DATA_HOME` or `~/.local/share`),
  and cargo's own `<cargo home>/.crates.toml`,
  which trees still on the TypeScript toolchain keep writing;
  the newest across them wins,
  so a stale legacy file cannot keep an old version or delete a newer cached archive.
  A version directory without a record is an unfinished install and names nothing.
- Reports the cleanup as one summary line on standard error,
  `mpm-shadow: post-run cleanup after the mpm command (not part of it): removed <n> superseded cached copies ... (<size>) under <cargo home>/registry`,
  so it cannot be read as part of the mpm command that ran before it.
  Nothing is printed when nothing was removed.
  `MPM_SHADOW_VERBOSE=1` adds one `mpm-shadow: deleted stale <path>` line per removed path after the summary.
  A path that could not be removed is always listed,
  with the system's reason,
  and the summary line counts it.
- Skips every candidate that resolves to its own executable when it looks for the real `mpm`,
  and every launcher whose record names its own executable.
  `install` links `mpm` to a per-tree launcher (a copy or hard link of this binary with a `.launch` record beside it),
  which starts the installed binary under the name `mpm`;
  the launcher is another file,
  so skipping only the binary's own path took the launcher for the real `mpm`,
  started it,
  and the chain repeated until the host refused new processes (issue 6259).
  When no candidate is left,
  the message names every `mpm` the lookup saw,
  in the order it read them,
  with the reason each was passed over
  (this shadow,
  a launcher of it,
  a shadow of another version,
  a directory of that name,
  a file with no execute bit,
  or a symbolic link that leads to no file),
  and the remedy that fits what was found:
  `chmod +x` for a file without its execute bit,
  a replacement or a `PATH` edit for a directory of that name,
  a repaired link,
  and installing the real `mpm` only where that is what is missing.
- Never starts a shadow of any version as the real `mpm`:
  A shadow never starts a shadow of any version as its real tool,
  whatever the path or the version of the candidate
  (owner,
  2026-11-07:
  "We should never invoke ourselves recursively.
  That's bad design.").
  Every shadow program (`cargo-shadow`,
  `mpm-shadow`,
  `pnpm-shadow` and `search-shadow`) carries the same identity text in the ELF section `.monochromatic_shadow_identity`
  (`src/shadow_identity.rs` and its parts under `src/shadow_identity/`,
  byte-identical copies in each crate,
  and `tests/shadow_identity_copies.rs` is one of the copies,
  so a change to the copy in `cargo-shadow`,
  `mpm-shadow` or `pnpm-shadow` fails all four suites,
  and a change to `search-shadow`'s copy alone fails `search-shadow`'s suite and no other).
  The lookup skips a candidate that carries it,
  or a launcher whose record leads to one,
  without starting it,
  and the start checks the open file again.
  A start that check refuses says what was found,
  never the words for a program fault:
  the path the lookup chose,
  the shadow the file is or leads to,
  why starting it would run a shadow instead of `mpm`
  (a shadow looks for the real `mpm` again,
  and two shadows that each stood first for the other started each other without end
  when this was measured on 2026-11-07:
  about 105 starts a second),
  the fact that the same path was not a shadow when the lookup read it a moment earlier,
  so the file there or a launcher record beside it changed in between,
  and what repairs it:
  the message itself lists every `mpm` the lookup read,
  in that order,
  with why it took or passed over each,
  so no sentence sends the reader to a command a shell may reject,
  the shadow's own directory belongs first and the real `mpm`'s after it,
  and `mpm install` in a launcher's tree writes a record that names a shadow again.
  A shadow that finds `MONOCHROMATIC_SHADOW_STARTED_BY` left for its own process or its direct parent
  was started by a shadow:
  it does nothing,
  prints one line and exits with status 2.
  The full description,
  the measurement and the cost are in `package/search-shadow/cli/README.md`,
  section "Never starts a shadow",
  and the incident in `doc/troubleshooting/search-shadow-copies-start-each-other-without-end.md`.
- Counts the shadows that started it in the `MPM_SHADOW_NESTING` environment variable,
  passes the count plus one to the program it starts,
  and refuses to start a program from the fourth level on (exit status 126,
  with a message that says why).
  Since the identity and arrival rules above a shadow that a shadow started stops at once,
  so this count now bounds only what they cannot see:
  indirect recursion through programs the real `mpm` runs.
- Starts the real `mpm` again,
  with pauses that double up to half a second and total at most five seconds,
  when the system refuses the start for lack of processes (`EAGAIN`,
  "Resource temporarily unavailable").
  A refused start started nothing,
  so starting again is safe.
  After a start that needed retries it says so once on standard error.
- Explains,
  and never retries,
  a start the system refused because another process holds open for writing a file it needs
  (`ETXTBSY`,
  "Text file busy"):
  the message says the file is the program or the interpreter the program names,
  which is the program after `#!` on a script's first line or a compiled program's dynamic linker,
  and which of the two whenever the program's own file says so,
  that the shadow cannot tell which process holds it or why,
  that the same command works once that write has finished and left a whole program,
  and that a write cut short leaves a broken program,
  which installing it again replaces.
  It never names a usual cause,
  because no measurement shows one,
  and it never names running the program by hand,
  which shows nothing once the write is over.
  No writer of the two paths a start through this tool resolves opens them for writing.
  Traced 2026-11-27 with a prober starting the published path while the writer ran under `strace`:
  `uv tool upgrade --all` writes the tool environment's console script as `bin/.tmpXXXXXX`
  and renames it,
  then removes the `bin` entry and makes it again:
  the file it publishes is absent for 44.7 ms and open for writing for none of it.
  The real `mpm` on this machine is such a link
  (`~/.local/bin/mpm` to the tool environment's `bin/mpm`).
  `cargo install` stages the program under `bin/cargo-installXXXXXX` and renames it
  (24.9 ms under the temporary name),
  `cargo binstall` renames,
  or copies to a temporary file beside the target and renames that
  (read,
  not run),
  and this tool's own launcher placement,
  download,
  gunzip and `tar` extraction write only the staging name.
  Two writers this tool starts do open a published path for writing:
  `go install` where its work directory is on another file system
  (3.5 ms,
  the one refusal this round measured)
  and `pnpm add --global` (0.09 ms);
  neither writes a path a start through this tool resolves,
  because a Go-route tool is linked straight into the exposure directory.
  A window a retry could carry does exist,
  and it is not a writer:
  a `fork` copies a write descriptor and the child holds that copy from its `fork` to its own `execve`,
  about 1 to 3 ms in this crate's own tests,
  which the record's threaded arm refused in 5 to 12 of 20 starts
  and in none of 20 with the forking thread off.
  A start that meets one of those renames instead of a write finds the program absent,
  which `uv` shows,
  and which the archive install showed before it replaced a version in one exchange
  (see "What a kill leaves and what the next install does about it"),
  and that is reported as a program that is not there:
  one cause and one remedy,
  whichever step found nothing:
  the lookup,
  the check on the open file,
  or the start itself.
  A writer that does hold the published file open is outside this tool
  and keeps it for as long as its own work takes.
  The released behaviour explains the refusal rather than hiding it behind a stall;
  the record ends with the two options,
  repeating it within a bound sized from those windows and explaining it at once,
  for the owner to weigh.
  The arms,
  the probe and the syscall timestamps are in
  `doc/troubleshooting/mpm-shadow-a-start-refused-because-the-program-is-being-written.md`.
- Explains a start that fails for good,
  instead of one bare line:
  the command that was run,
  the program it tried,
  the stage,
  the cause in plain words (a process limit,
  missing permission,
  a missing program or interpreter,
  a program whose file another program is writing,
  a file the kernel will not start as a program,
  a program that is this tool's own shadow or leads to one,
  or the system's own words),
  what to do,
  the exit status (126) and why the command reports it,
  and what had completed (nothing of the command ran,
  and the post-run cleanup does not run).
  Each cause is one arm chosen by what happened,
  so a refused shadow is never read as a program fault,
  and the arm that names the program says what it turned out to be rather than "the first `mpm` on PATH that is not this shadow".
  An error no arm classifies keeps the system's own words and names no step that cannot show the error:
  it does not send the reader to run the program by hand,
  which cannot reproduce a refusal of this start,
  and for `E2BIG`,
  `ENAMETOOLONG` and `ELOOP` it says that running the same command again cannot succeed
  and names what to change instead,
  while the words about a damaged file and the words about a limit stay what to act on.
  The file the kernel will not start has an arm of its own for the same reason.
  Every error that can still reach the last arm,
  with what a shell repeats and which step shows nothing for it,
  is listed in `doc/troubleshooting/mpm-shadow-a-start-that-does-not-happen-says-what-is-known.md`.
- Shows that it is a shadow in the two places a person looks,
  after mpm's own output and only after a successful run:
  a top-level `mpm --version` prints mpm's version text,
  then one line on standard output,
  `mpm-shadow <version>, shadowing mpm at <path of the real mpm>`;
  a top-level `mpm --help` (or `-h`) prints mpm's help,
  then a blank line and a short section naming the shadow,
  its version,
  the real mpm's path,
  and the prune it runs after every run.
  This is how `hub` does it:
  `hub --version` runs `git version` and then prints `hub version <X>`
  (<https://github.com/mislav/hub/blob/master/commands/version.go>),
  and `hub help` appends the commands hub adds to git's help.
- Detects only top-level options,
  the way mpm's parser reads them:
  the words before the subcommand,
  in order,
  skipping the values of the options that take one
  (`--verbosity`,
  `--config`,
  `-t`,
  `-j`
  and the others read from `mpm --help` of mpm 8.2.0;
  a bare `--color` skips the next word only when that word is `auto`,
  `always` or `never`,
  the words mpm accepts as a color value,
  so `mpm --color auto --version` is read as a version request).
  The first eager option wins,
  as in mpm.
  `mpm install --help`,
  `mpm help install`,
  `mpm --help-format json`,
  `--man`,
  `--tree`,
  `--params`,
  and a failing run print exactly what mpm printed,
  with mpm's exit status.
  mpm has no `-V`.
  A new top-level mpm option that takes a value must be added to `VALUE_OPTIONS` in `src/shadow_notice.rs`.

A cached version that a project's `Cargo.lock` still pins is deleted too when it is not the kept one;
cargo downloads it again on the next online build,
but an offline build of that project fails until then.

The artifact is deliberately not named `mpm`,
so a `target/*/mpm` or a tool `bin/mpm` never shadows mpm by accident.
The toolchain plan links it as `mpm` into a gitignored directory that comes first on `PATH`.

## When a stream refuses a write

Every word this program says of its own goes through one writer in `src/notices.rs`:
the report on standard output,
the lines about its own work on standard error,
and the sentence that ends a command that failed.
A write the stream refuses is dropped,
and the command ends with the status its own work gave it,
so a full disk or a pipeline that stopped reading changes what a caller reads and never what happened.

- A report is written between two of a command's own steps in one case:
  `install <tool>` names the entries it added to the plan after that write and before the sync.
  A refused report there leaves the plan changed and the sync still runs,
  so the command ends on what the sync did rather than on the stream.
- The same holds for a notice:
  a warning,
  a line naming what a run left,
  and the line a wait past 2 s prints are all courtesy lines,
  and none of them ends a command.
- A stream another process left non-blocking that answers "not now" is waited out for up to 1 s
  (measured against a reader that drains a full pipe:
  0.082 ms at its slowest in twenty interleaved rounds on 2026-11-28),
  and what does not fit after that is dropped.
  A line and its line end are written in one piece,
  so a line is never left without its end.
- The cost:
  a caller cannot tell that a report was refused.
  The stream is the only place a program says so,
  and it is the stream that refused.
  The statuses below are the work's own in every state.

## Exit codes

- `0`:
  success.
- `1`:
  the plan and lock disagree,
  the lock's digest does not match,
  an install,
  download
  or lookup failed,
  or `list` found drift.
  A recovery whose own publish failed is a warning rather than a failure when the lock is satisfied afterwards
  (see "What a kill leaves and what the next install does about it"),
  so that case ends 0.
- `2`:
  a bad command line,
  or a plan or lock file that cannot be read.
- `126`:
  (shadow mode) the real `mpm` was found but could not be started,
  the shadow refused to start it because the program was or led to a shadow,
  or the shadow refused to start it because too many shadows were nested.
- `127`:
  (shadow mode) no real `mpm` on `PATH`.
- Anything else in shadow mode:
  mpm's own exit code.

## Checks

```bash
mise run //package/mpm-shadow/cli:lint
mise run //package/mpm-shadow/cli:lint:types
mise run //package/mpm-shadow/cli:lint:clippy
mise run //package/mpm-shadow/cli:test
mise run //package/mpm-shadow/cli:test:debug
```
