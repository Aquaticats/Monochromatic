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
- `mpm-shadow install [--frozen-lockfile | --no-frozen-lockfile]`:
  first resolves the plan entries the lock lacks or holds for an older request
  (see "Installing a plan that is ahead of the lock"),
  then installs every locked tool that is not at its locked version,
  makes the postinstall links and removes the ones an earlier run made that the lock no longer declares
  (see "Postinstall links the lock no longer declares"),
  rebuilds the exposure directory,
  writes the tool environment for mise (see "Tool environment"),
  runs the postinstall commands
  and then retires the old copies of this repository's tools
  (see "Old copies of this repository's tools").
- `mpm-shadow list`:
  compares the installed tools with the lock.
  Exits 1 on drift.
  Also names the old copies of this repository's tools that `install` would retire,
  which are not drift.
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
  `install` asks `rustup component list --installed` and `rustup target list --installed` for that
  toolchain,
  adds only the declared parts `rustup` does not list,
  and reports the tool as installed with
  one line per part it added.
  A listing or an add that fails fails that tool rather than counting it unchanged.
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
  and renamed to the version's name (see "Repository-published tools keep only the newest version");
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
  (`.<version>.staging.<process id>`),
  the first bin of a release archive is looked for in it,
  the marker is written last,
  and only then is the staging directory renamed to the version's name.
  `tar` creates a file with its final mode before it writes the content,
  so without this a truncated archive would leave an executable of the right name and the wrong bytes
  at the version's name,
  count as finished and cause the working older version to be deleted.
  A failed install removes its staging directory and touches no copy that was already there.
  A directory without the marker (one an earlier release unpacked in place,
  or one cut short) is never finished:
  it is never the newest,
  so it never causes a deletion or moves `current`,
  and the next install of that version replaces it.
  A run killed without cleanup leaves its staging directory behind;
  nothing reads it and it can be deleted by hand.
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

### The farm also carries the running `mpm-shadow`

Every sentence `mpm-shadow` prints tells a person to run `mpm-shadow` or `mpm`,
so each scope's install makes those names reachable from a fresh shell:

- The scope's exposure farm holds a link named after the running `mpm-shadow`,
  pointing at the installed program through the tool's `current` link,
  so a later version move never leaves it dangling.
  It is made when no tool of the scope's lock declares that bin:
  a plan row that declares it owns the farm link and the launcher record behind it.
  It is never made for a program that belongs to no install,
  so a run from a build folder puts no build on the machine's `PATH`
  (`src/toolchain/exposure.rs`,
  `expose_running_shadow`).
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
  A file that vanishes between the check and the start still gets the `could not start` line and the same sentence.
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
  (`src/shadow_identity.rs`,
  a byte-identical copy in each crate;
  `package/search-shadow/cli/tests/shadow_identity_copies.rs` fails when the copies differ).
  The lookup skips a candidate that carries it,
  or a launcher whose record leads to one,
  without starting it,
  and the start checks the open file again.
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
- Explains a start that fails for good,
  instead of one bare line:
  the command that was run,
  the program it tried,
  the stage,
  the cause in plain words (a process limit,
  missing permission,
  a missing program or interpreter,
  or the system's own words),
  what to do,
  the exit status (126) and why the command reports it,
  and what had completed (nothing of the command ran,
  and the post-run cleanup does not run).
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
- `2`:
  a bad command line,
  or a plan or lock file that cannot be read.
- `126`:
  (shadow mode) the real `mpm` was found but could not be started,
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
