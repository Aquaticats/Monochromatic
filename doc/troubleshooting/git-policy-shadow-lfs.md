# Scoped RPC commit fails at an unrelated LFS filter

## Original consumer evidence

On 2026-10-18,
the configured MiMo RPC worker `pi-mimo-rpc-session-9Tl0ts` attempts a protected,
explicit-path commit of `package/pi-plugin/auto-mode/native/build-reuse.ts`
and `package/pi-plugin/auto-mode/native/build-reuse.platform.test.ts`.
The bounded Fedora 44 consumer has a disposable home,
2 GiB memory and two CPUs.
Its qualified wrapper is active.
The commit exits 128:

```text
Error cleaning Git LFS object: open <worktree>/.git/cli-git/shadow/<transaction>/lfs/tmp/2882421905: no such file or directory
error: packet write failed: Broken pipe
error: external filter 'git-lfs filter-process' failed
fatal: package/ide/app/design/screenshots/2026-10-05-annotations-as-built/caret-card-dark.png: clean filter 'lfs' failed
```

The named LFS asset is outside the requested commit paths.
The owned source changes remain staged.
The worker stops Git retries and preserves its complete original tool event and stderr.
Supervisor delivery of those exact paths through the qualified host wrapper succeeds as `9363d8998b`.
This establishes an environment-sensitive boundary failure,
without identifying why the LFS temporary path is unavailable.

## Owned boundary and report

`package/git-policy/cli/src/native/shadow_create.rs` creates the transaction's repository.
`shadow_config.rs` includes the real common configuration.
`commit_preparation.rs` supplies the shadow Git directory,
original worktree and private index to native commit.
The complete child environment,
filter process ownership and shadow lifetime still require controlled reproduction.

[Issue 1770](https://github.com/Aquaticats/Monochromatic/issues/1770) records the failed real consumer,
successful host delivery and required disposable native-Git/wrapper/filter controls.
Searches across all issue states for `LFS shadow`,
`filter-process` and `unable to write new index file` find no matching incident before filing.
The retained worker event is `call_f94fb711ff224917b3b6ec0b` in
`${HOME}/temp/agent/pi-mimo-rpc-session-9Tl0ts/host-events.jsonl`.

No wrapper repair is claimed.
No Git,
Git LFS,
Pi,
Node or Linux defect is attributed from this observation.
The separate source worker's `unable to write new index file` failure
and the historical `bad line length character: Not ` failure remain separate incidents until their boundaries match.
This also differs from the missing-wrapper PATH incident in
[issue 1768](https://github.com/Aquaticats/Monochromatic/issues/1768)
and [divergent auto-push reconciliation](git-policy-auto-push-reconciliation.md).
