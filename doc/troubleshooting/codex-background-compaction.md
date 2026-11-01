# Codex CLI 0.162.1 cannot enable background compaction until a supporting release is installed

## Symptom

On 2026-10-15,
enabling background compaction through the feature name `background_compaction` failed.
The correct name is `async_compaction`,
which the installed stable release also rejects as a command-line feature flag.
The user chose to retain stable releases and preconfigure enablement for a future stable update.

## Root cause and evidence limits

The installed release is `0.162.1`.
The [latest stable release][stable] predates [PR #53800][pr],
which merged on 2026-10-13.
The release catalog still listed `0.162.1` as latest stable when checked on 2026-10-15.

The [official configuration reference][config] documents automatic compaction through
`model_auto_compact_token_limit`.
It lists `features.context_management.experimental_mode` as unavailable.
The inspected reference did not provide the new background-compaction switch.
The existing user configuration already contains `context_management = true` under `[features]`.
That flag alone does not establish background-compaction behavior.

The first lookup was incomplete:
absence from stable documentation and rejection of the guessed name did not establish feature absence upstream.
The user supplied PR #53800,
which identified `AsyncCompaction` as the gate.

The published `rust-v0.163.0-alpha.11` source registers the correct key in
`codex-rs/features/src/lib.rs:2031` ([source][alpha-features]):

```rust
//! Feature registry excerpt from codex-rs/features/src/lib.rs:2031.
        id: Feature::AsyncCompaction,
        key: "async_compaction",
        stage: Stage::UnderDevelopment,
        default_enabled: false,
```

The stable implementation ignores unknown feature keys while recording a tracing warning in
`codex-rs/features/src/lib.rs:688` ([source][stable-features]):

```rust
//! Unknown-key handling excerpt from codex-rs/features/src/lib.rs:688.
                None => {
                    tracing::warn!("unknown feature key in config: {k}");
                }
```

This permits storing the future boolean setting without enabling an unsupported feature in `0.162.1`.
It does not prove future releases will retain the same key.
The alpha schema confirms that `features.async_compaction` accepts a boolean
in `codex-rs/core/config.schema.json:7118` ([schema][alpha-schema]).

The attempted private scratch-directory setup was rejected by automatic approval review because
the execution wrapper writes to `cgroup.procs`.
The subsequent read-only `gh api` request could not connect to `api.github.com`.
The source evidence was then fetched through the web tool.
No upstream source was cloned or modified.
The user approved session cgroup placement,
and the later configuration write succeeded with escalation.

## Verification

These read-only invocations used the installed `codex-cli 0.162.1` binary:

```text
# Installed CLI output, 2026-10-15
$ codex --version
codex-cli 0.162.1
$ codex features list
context_management                       under development  true
$ codex --enable background_compaction features list
Error: Unknown feature flag: background_compaction
$ codex --enable async_compaction features list
Error: Unknown feature flag: async_compaction
```

The feature-list excerpt contains only the relevant row.
Version inspection and feature enumeration succeeded.
Both attempted command-line feature overrides failed with exit status `1`.
The command-line override does not persist configuration.
No live compaction was triggered or measured.

Configuration-path probes succeeded with exit status `0`:

```text
# Installed CLI probes, 2026-10-15
codex --config features.async_compaction=true features list
codex --config features.goals=false features list
```

The future setting produced exactly the same complete output as the baseline feature list.
The positive control changed `goals` from `true` to `false`,
demonstrating that configuration overrides reached feature resolution.
After saving the future setting,
`codex features list` again matched the baseline and `codex --version` remained `0.162.1`.
The shell sandbox's PATH-alias warning appeared in baseline and probe output alike.

## Verified workarounds

The user configuration now contains this entry in its existing features table:

```toml
# ~/.codex/config.toml
[features]
async_compaction = true
```

This preconfigures enablement when a subsequently installed stable release recognizes the key.
It does not install updates,
select an alpha release,
or activate background compaction in the current process.
The current release records an unknown-key tracing warning while ignoring the setting.
The setting's future effect is inferred from the alpha registry and schema;
the alpha binary and live compaction were not exercised.

## What does not work

- `codex --enable background_compaction features list` rejects the feature name.
- `codex --enable async_compaction features list` also fails on the installed stable release.
- Treating the existing `context_management = true` flag as proof of background compaction
  is unsupported by the inspected evidence.
- `codex --strict-config --config features.async_compaction=true features list`
  cannot test strict parsing because `--strict-config` is unsupported for `codex features`.
- Installing an alpha was rejected by the user's stable-channel preference.

## Upstream filing decision

No matching exemption directory was present beside the troubleshooting skill.
Nothing was filed or drafted upstream.

1.  No upstream defect was established:
    the installed stable release predates the implementation.
2.  Upstream has implemented the feature in PR #53800.
3.  The published alpha source and schema cover the requested feature.
4.  Contribution policy was not inspected because no upstream filing is proposed.
5.  PR #53800 already supplies the relevant upstream implementation;
    no separate defect report is warranted.
6.  No prototype was needed because the requested consumer-side preconfiguration succeeded.

The troubleshooting document passed the repository's scoped Markdown formatter and was rendered through
the existing `package/codex-plugin/bash-limit` `docs:render` task.

[config]: https://learn.chatgpt.com/docs/config-file/config-reference
[stable]: https://github.com/openai/codex/releases/tag/rust-v0.162.1
[pr]: https://github.com/openai/codex/pull/53800
[alpha-features]: https://raw.githubusercontent.com/openai/codex/rust-v0.163.0-alpha.11/codex-rs/features/src/lib.rs
[stable-features]: https://raw.githubusercontent.com/openai/codex/rust-v0.162.1/codex-rs/features/src/lib.rs
[alpha-schema]: https://raw.githubusercontent.com/openai/codex/rust-v0.163.0-alpha.11/codex-rs/core/config.schema.json
