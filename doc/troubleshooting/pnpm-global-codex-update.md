# Codex 0.169.0 advertises 0.170.0 before pnpm 12.21.0 selects it

Investigation by Codex on `2026-11-18T17:04:32-05:00`, pending owner review.

## Symptom

Codex says `Run pnpm add -g @openai/codex to update.`
During the incident, `pnpm i -g @openai/codex` printed `+ @openai/codex 0.169.0` and exited successfully.
The global manifest recorded `^0.169.0`.
`pnpm update -g` updated `@deepseek-ai/dsh` while Codex stayed at `0.169.0`.

## Root cause

Codex takes the GitHub release version and checks that the npm `latest` tag matches and the package has tarball
and integrity metadata. The check has no pnpm release-age input.
[`codex-rs/tui/src/updates.rs:97`][codex-update], [`codex-rs/tui/src/npm_registry.rs:25`][codex-registry]

```rust
// codex-rs/tui/src/updates.rs
let latest_version = fetch_latest_github_release_version(&client_pool).await?;
// After fetching the npm package metadata:
npm_registry::ensure_version_ready(&package_info, &latest_version)?;
```

pnpm defaults `minimumReleaseAge` to 1440 minutes and returns a mature match before trying a newer fallback.
npm published `0.170.0` at `2026-11-18T03:45:45.798Z` and tagged it `latest`.
The unqualified install ran around `2026-11-18T21:59Z`, so the resolver selected the mature `0.169.0`.
[`pnpm/crates/config/src/settings.rs:1521`][pnpm-default],
[`pnpm/crates/resolving-npm-resolver/src/pick_package/version_pick.rs:118`][pnpm-pick]

```rust
// pnpm/crates/config/src/settings.rs
#[default(_code = "Some(24 * 60)")]
pub minimum_release_age: Option<u64>,
```

```rust
// pnpm/crates/resolving-npm-resolver/src/pick_package/version_pick.rs
if mature.is_some() {
    return Ok(mature);
}
pick_release_age_fallback(picker_opts, target_spec, meta)
```

Plain global `update` reuses the manifest selector.
`^0.169.0` allows `0.169.x` and excludes `0.170.0`.
`--latest` replaces the selector with `@latest`, which still goes through the age filter.
[`pnpm/crates/cli/src/cli_args/global/selectors.rs:104`][pnpm-global]

```rust
// pnpm/crates/cli/src/cli_args/global/selectors.rs
} else if latest && is_plain_version_spec(spec) {
    format!("{alias}@latest")
} else {
    format!("{alias}@{spec}")
}
```

The `Skipped updating @pnpm/exe.linux-x64` warning came from a separate global package whose `file:` source
was missing. The warning renderer emits it for that group and the update continues with the other groups.
[`pnpm/crates/cli/src/cli_args/global/selectors.rs:126`][pnpm-global]

```rust
// pnpm/crates/cli/src/cli_args/global/selectors.rs
let source = local_file_path(spec, &pkg.install_dir)?;
matches!(source.try_exists(), Ok(false)).then_some((alias, source))
```

## Verification

### Find the saved range

`pnpm root --global` prints the global package root.
Each install group has its own `package.json` under that root.
In `pnpm list --global --json`, the dependency's `path` identifies its group as the directory before `node_modules`.
Open that group's `package.json` and read `dependencies["@openai/codex"]`.
The sibling `pnpm-lock.yaml` records the saved `specifier` and resolved `version` separately.

For an unqualified add, pnpm saves a caret range around the version it resolves by default.
Here it saved `^0.169.0`, which admits stable versions from `0.169.0` up to `0.170.0` exclusive.
[`pnpm/crates/registry/src/range_spec_style.rs:32`][pnpm-save-range]

```rust
// pnpm/crates/registry/src/range_spec_style.rs
RangeSpecStyle::Major | RangeSpecStyle::None => "^",
```

### Reproduce the selection

Use `pnpm 12.21.0` and a disposable global directory. These results were observed before `0.170.0`
reached the default cutoff. The registry confirms publication timestamps for the wrapper and each platform build.

```bash
# Disposable shell session outside the repository
mkdir --parents "$HOME/temp/agent" && chmod 700 "$HOME/temp/agent"
fixture=$(mktemp --directory "$HOME/temp/agent/pnpm-codex-repro.XXXXXXXX")
mkdir "$fixture/bin"
export PNPM_HOME="$fixture" PATH="$fixture/bin:$PATH"
export PNPM_CONFIG_GLOBAL_DIR="$fixture/global" PNPM_CONFIG_GLOBAL_BIN_DIR="$fixture/bin"
export PNPM_CONFIG_CACHE_DIR="$fixture/cache" PNPM_CONFIG_STORE_DIR="$fixture/store"
pnpm root --global
pnpm install --global @openai/codex
codex --version
pnpm update --global --latest @openai/codex
codex --version
PNPM_CONFIG_MINIMUM_RELEASE_AGE=0 pnpm update --global @openai/codex
codex --version
pnpm add --global @openai/codex@0.170.0
codex --version
```

- Default unqualified install and default `update --latest` each resolved `0.169.0`.
- With the cutoff disabled, plain update of `^0.169.0` resolved `0.169.1`, while an unqualified install selected `0.170.0`.
- Explicit `@0.170.0` with defaults installed the Linux executable, and `codex --version` returned `codex-cli 0.170.0`.
- Explicit `@0.170.0` with `PNPM_CONFIG_MINIMUM_RELEASE_AGE=1440` and noninteractive input raised
  `ERR_PNPM_NO_MATURE_MATCHING_VERSION` for the wrapper and all platform builds.

## Verified workaround

`pnpm add --global @openai/codex@0.170.0` made the exact release the sole candidate for the requested version.
pnpm's built-in age policy defaults to non-strict, so the resolver's fallback could install it.
This accepts a release before the default waiting period and records an exact manifest version.
Use `--latest` for a subsequent update across the saved version.
An explicitly configured strict policy can require interactive approval or waiting for the cutoff.
[`pnpm/crates/config/src/registry_options.rs:196`][pnpm-strict]

```rust
// pnpm/crates/config/src/registry_options.rs
self.minimum_release_age_strict.unwrap_or_else(|| {
    self.explicit_settings.contains_key("minimumReleaseAge")
})
```

## What does not work

During the release-age window, repeating the banner's unqualified command preserves the mature version.
Waiting lets the unqualified command select the new release, while a plain `update` continues to honor `^0.169.0`.

## Upstream filing decision

- The mismatch is in Codex's update guidance. pnpm applied its documented version-selection behavior.
- Codex can change the guidance for pnpm installs.
- The existing pnpm-specific update action establishes that Codex supports this path.
- Upstream already hosts the same report. Contribution acceptance for new material was outside this investigation.
- The report is open with no maintainer comments as of the check. Acceptance of a fix remains unknown.
- This investigation verified the install command. It produced no patch to Codex.

No new filing. The open report already records the release-age mismatch and the misleading successful update.
[OpenAI Codex issue `#39533`][codex-issue]

[codex-update]: https://github.com/openai/codex/blob/d32063f96c2dff1dc96af839b3952abb6acfeda4/codex-rs/tui/src/updates.rs#L97-L111
[codex-registry]: https://github.com/openai/codex/blob/d32063f96c2dff1dc96af839b3952abb6acfeda4/codex-rs/tui/src/npm_registry.rs#L25-L40
[pnpm-default]: https://github.com/pnpm/pnpm/blob/b8d388835f9a40ee1ac53fbc2d4b58c3ebfa5600/pnpm/crates/config/src/settings.rs#L1521-L1531
[pnpm-pick]: https://github.com/pnpm/pnpm/blob/b8d388835f9a40ee1ac53fbc2d4b58c3ebfa5600/pnpm/crates/resolving-npm-resolver/src/pick_package/version_pick.rs#L118-L129
[pnpm-global]: https://github.com/pnpm/pnpm/blob/b8d388835f9a40ee1ac53fbc2d4b58c3ebfa5600/pnpm/crates/cli/src/cli_args/global/selectors.rs#L104-L141
[pnpm-strict]: https://github.com/pnpm/pnpm/blob/b8d388835f9a40ee1ac53fbc2d4b58c3ebfa5600/pnpm/crates/config/src/registry_options.rs#L196-L213
[pnpm-save-range]: https://github.com/pnpm/pnpm/blob/b8d388835f9a40ee1ac53fbc2d4b58c3ebfa5600/pnpm/crates/registry/src/range_spec_style.rs#L32-L51
[codex-issue]: https://github.com/openai/codex/issues/39533#:~:text=Because%20of%20the%20new%20pnpm
