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

## Chromium build pre-allowance at 2026-11-03T16:13:29+00:00

The user requested pre-allowing the Chromium contribution's network processes.
The live daemon was OpenSnitch `1.8.0` under `opensnitch.service`.
The new rule is `000-allow-chromium-contribution-network`,
stored in `/etc/opensnitchd/rules` with root ownership and mode `0600`.
It uses `action: allow`,
`duration: always`,
`precedence: true`,
and a case-sensitive anchored `process.path` expression.
The selected paths cover the container network helpers and the build's Git,
Python,
curl,
package-manager,
Node,
and CIPD executables.

The final pathname was written directly.
The daemon logged
`Ruleset changed due to 000-allow-chromium-contribution-network.json, reloading ...`
at `2026-11-03T15:35:59Z`.
The installed rule's SHA-256 is
`16ed85d6d526bd613ca3f9e44c596248073f827302124e02d70700b0f969be89`.

The native Git,
curl,
and CIPD clients completed live read requests in the disposable build environment.
Node and hermetic Python reached the Google source endpoint but received HTTP `503` responses.
Their subsequent requests to the pinned GitHub mirror file returned HTTP `200`;
Git's mirror query also passed.
The failed Google requests remain recorded separately from the successful mirror checks.

This is not evidence that OpenSnitch caused the interrupted bulk fetch.
The existing file named `reject-always-simple-usr-bin-pasta-avx2.json`
contained `action: allow` when inspected.
Its filename was not the active decision.
The prior silent Git HTTPS fetch's cause remains unknown.
The [Chromium contribution handover](../handover/chromium-live-histogram-contribution.md)
retains the build state and remaining native validation work.

## Homebrew and Codex allowances at 2026-11-14T09:00:44-05:00

Status:
agent-verified host repair;
owner approval remains open.
Codex installed the rules and ran the checks described here.

### Observed failures and correction

`uupd-manual.service` failed because its Homebrew module reported:

```text
Error: Failed to download https://formulae.brew.sh/api/internal/packages.x86_64_linux.jws.json!
```

The same invocation's Flatpak module completed successfully.
Its interleaved output did not identify a separate Flatpak failure.
The Homebrew curl executable resolved to
`/var/home/linuxbrew/.linuxbrew/Cellar/curl/8.23.0/bin/curl`.
Its enabled exact-path rule had `action: reject` and `duration: always`.
Before the repair,
this curl failed to connect to `formulae.brew.sh:443`,
returned exit status `7`,
and reported HTTP `000`.

The daemon log also recorded saved rejects for Codex app-server releases `0.168.0` and `0.169.0`.
When their contents were read during this repair,
both files already contained `action: allow` despite their `reject-` filenames.
The initial filename-based description of those files as current rejects was wrong.
Their exact release paths still did not cover subsequent updates.

### Permanent rules and scope

The host runs OpenSnitch `1.8.0` under `opensnitch.service`.
The installed files are root-owned,
mode `0600`,
in `/etc/opensnitchd/rules`:

- `000-allow-homebrew-curl.json` covers Homebrew curl in any version of its existing Cellar root,
  plus the `opt/curl` spelling.
  The saved copy is [the Homebrew curl rule](opensnitch-homebrew-curl-rule.json).
- `000-allow-codex-updates.json` covers the app-server's release directories,
  the ChatGPT executable,
  pnpm and legacy mise Codex packages,
  and JetBrains Codex ACP installations.
  The saved copy is [the Codex update rule](opensnitch-codex-updates-rule.json).

Both use `action: allow`,
`duration: always`,
`precedence: true`,
and case-sensitive anchored expressions on `process.path`.
They cover both `/home` and `/var/home` spellings.
Version directory names are variable;
the executable names and install roots stay constrained.
They allow outbound destinations for these executables,
including future endpoints those executables contact.
They do not verify the executable's publisher or checksum.

`rpm --query --configfiles opensnitch` does not list either custom rule as a package-owned config file.
They are persistent host configuration,
independent of the Homebrew and Codex version directories.
A relocation to a different install layout or deliberate removal of these files would require a new rule.
No future software update or reboot was performed as part of this verification.

OpenSnitch logged live reloads for both final filenames at `2026-11-14T14:00:44Z`.
The existing precedence-selection explanation in this document applies to these `000-` rules.
The selecting predicate was re-read in
[`daemon/rule/loader.go:497 to 515`](https://github.com/evilsocket/opensnitch/blob/b404c4c6316760fa7bc415509d3f8d747f7dc9cc/daemon/rule/loader.go#L497-L515):

```go
// daemon/rule/loader.go
if rule.Action == Reject || rule.Action == Deny || rule.Precedence == true {
    return rule
}
```

`agent-browser` confirmed that the pinned selection contains this predicate.

### Cleanup and verification

The user then requested deletion of rules that should no longer be present.
The cleanup removed `18` rules covered by the new allowances:
the Homebrew curl reject,
the Codex version-specific entries,
and the older pnpm-only Codex expression.
Each file was read before deletion,
and its SHA-256 was checked again immediately before unlinking it.
The replacement rules were required to be enabled,
permanent precedence allows before any deletion.

The original download probe was:

```sh
# /home/linuxbrew/.linuxbrew/opt/curl/bin/curl
/home/linuxbrew/.linuxbrew/opt/curl/bin/curl \
  --fail --silent --show-error \
  --connect-timeout 10 --max-time 25 \
  --output /dev/null --write-out 'HTTP %{http_code}\n' \
  https://formulae.brew.sh/api/internal/packages.x86_64_linux.jws.json
```

After installing the rules,
the same command returned HTTP `200` and exit status `0`.
Current and hypothetical future release paths matched the expressions;
unrelated users,
unrelated executables,
and suffix lookalikes did not match the fixture cases.

The real Codex `0.169.0` executable was also exercised through its stdio app-server protocol,
using a private disposable Codex home copied from the configured home.
`account/read` with `refreshToken: false` and `account/rateLimits/read` both returned successful responses.
Only response field names were logged;
account identities and credentials were not printed.
This verified the reported account-read boundary and an authenticated network read.
It did not run a model request or restart the user's active Codex session.

After deleting the superseded rules,
the download probe again returned HTTP `200`,
and both Codex account methods again succeeded.
The daemon log recorded deletion of every superseded file at `2026-11-14T14:03:46Z`.
The updater's stale `start-limit-hit` state was cleared with
`systemctl reset-failed uupd-manual.service`.
The full system updater was not rerun.

The saved JSON copies matched the installed files by SHA-256.
The Markdown record passed the scoped `lint:monochromatic-lint` task.
Its CommonMark rendering through Pandoc preserved the incident headings,
rule links,
account method names,
and download URL.

### Upstream filing decision

No upstream issue or comment is proposed.
The evidence identifies local application policy rather than an upstream defect:
the matching Homebrew reject was loaded,
and adding the intended allowance changed the same download probe from HTTP `000` to HTTP `200`.
The upstream-fault constraint fails.
Upstream can implement rule changes,
and application rules are an existing supported use case,
but contribution acceptance and maintainer willingness were not investigated because no upstream change is requested.
No upstream source patch was prototyped.
`gh search issues 'precedence reject rules' --repo evilsocket/opensnitch --json number,title,state,url --limit 100`
returned no matches.
No matching OpenSnitch exemption was found in `.out-of-scope/`.
