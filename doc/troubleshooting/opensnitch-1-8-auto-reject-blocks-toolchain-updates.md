# OpenSnitch 1.8.0 auto-reject rules block toolchain updates until a precedence allow covers the install roots

## Symptom

With `DefaultAction = allow` in `/etc/opensnitchd/default-config.json`,
the OpenSnitch UI still decides each unknown program.
When nobody answers the prompt,
the UI's timeout action is the configured default,
and the answer is saved as an exact-path rule named `reject-always-simple-<path>`.
A toolchain update then fails at the network level the first time a program at a new path connects.
The failing connect reports `os error 103` (`ECONNABORTED`).
New versions install into new directories,
so every update creates new paths.
Measured in `/etc/opensnitchd/rules` on 2026-10-31:
reject rules for old mise-era installs (`mise-installs-node-26-10-0-bin-node`,
`zig-0-15-2-zig`,
`git-lfs`,
`rclone`,
`/opt/mise`),
and for two agent worktree builds
(`.claude/worktrees/agent-.../package/file-enforcer/cli/target/debug/file-enforcer` and
`.../browserslist/core-rust/target/debug/monochromatic-browserslist`),
which made an agent's network test fail.

## Root cause

### A non-precedence allow never beats a matching reject

OpenSnitch 1.8.0 selects the rule in `daemon/rule/loader.go`,
tag `v1.8.0`:

- `sortRules` (lines 369 to 377) collects the enabled rules and sorts their names with `sort.Strings`,
  a bytewise sort.
- `FindFirstMatch` (lines 497 to 515) walks that order.
  It keeps the last matching rule,
  and returns at once when a matching rule is a `reject`,
  a `deny`,
  or has `precedence: true`.

Consequences:

- An allow rule without `precedence` loses to any matching reject or deny,
  whatever its name.
  The coordinator's earlier repair of the rustup path,
  `allow-always-simple-var-home-user-cargo-bin-rustup`,
  is a non-precedence allow;
  it takes effect only while no reject rule exists for that path,
  and the rules directory lists none today.
- A precedence allow named after the reject loses to it,
  because the reject is reached first and ends the walk.
- A precedence allow named before the reject wins.
  The daemon saves prompt answers as `reject-always-simple-*`,
  `deny-always-simple-*` and `allow-always-simple-*`,
  all starting with a letter,
  so a `000-` prefix sorts first.
  The host already follows this convention with `000-allow-cargo.json` and `000-allow-localhost.json`.

The GUI sends the saved rule without `precedence` (`ui/opensnitch/dialogs/prompt/__init__.py`,
`_send_rule`),
so an auto-saved reject is never a precedence rule.

### Live reload handles only Write and Remove

`liveReloadWorker` (lines 457 to 490) acts on `fsnotify.Write` for `*.json` by loading the rule,
and on `fsnotify.Remove` by unloading it.
A file renamed into place raises `Create` only and is ignored until the daemon restarts.
Moving a file out of the directory raises a rename event,
not `Remove`,
so the rule stays loaded.
The rule key is the JSON `name`,
while `deleteRule` derives the name from the file name,
so a file must be named `<name>.json`.

### Case-insensitive patterns are lowercased before compiling

`Operator.Compile` in `daemon/rule/operator.go` lowercases `Data` when `sensitive` is false,
then compiles it.
`[A-Z]` becomes `[a-z]` and `\S` becomes `\s`,
and `/var/home/user/Monochromatic` would only match lowercase input.
A rule that fails to compile is logged and skipped.

## Verified workarounds

`wg-quicker opensnitch-rules apply` writes three precedence allow rules named
`000-allow-monochromatic-toolchain-install-roots`,
`000-allow-monochromatic-toolchain-exposure-links` and `000-allow-monochromatic-toolchain-rust-tools`,
each a case-sensitive RE2 expression on `process.path`.
See [wg-quicker README](../../package/wg/cli-quicker/README.md#opensnitch-toolchain-allowance).

## Verification

The daemon could not be built or run in the session that wrote this record,
so selection was verified two ways:

- By reading `loader.go` and `operator.go` at the `v1.8.0` tag
  (cloned with `gh repo clone evilsocket/opensnitch ... -- --depth 1 --branch v1.8.0`).
- By a model of `FindFirstMatch` in `package/wg/cli-quicker/src/fixture.toolchain-rules.ts`,
  exercised in `opensnitch-toolchain-rules.unit.test.ts`.
  It shows that an auto-saved reject loses to the generated allowance,
  that a non-precedence allow named `000-` loses to the reject,
  and that a precedence allow named `zzz-` loses to it too.
  A model written from the same reading proves the reading consistent,
  not the daemon correct.

After applying the rules to the live host,
confirm with the daemon log:
`Ruleset changed due to 000-allow-monochromatic-toolchain-...json, reloading` for each file,
and no `compil` warning.

## What does not work

- Renaming a temporary file over a rule file:
  the daemon sees `Create` and keeps the old rule.
- A non-precedence allow with a `000-` name:
  the auto-saved reject still ends the walk first.
- A precedence allow with a name that sorts after `reject-`:
  the reject is reached first.
- A case-insensitive pattern containing uppercase letters or class escapes.

## Sources

- OpenSnitch `daemon/rule/loader.go` and `daemon/rule/operator.go`,
  tag `v1.8.0`.
- OpenSnitch wiki,
  [Rules](https://github.com/evilsocket/opensnitch/wiki/Rules#rule-precedence),
  section Rule precedence.
