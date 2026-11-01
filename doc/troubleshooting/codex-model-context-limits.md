# Codex 0.163.0 needs a custom catalog for different model context limits

## Symptom

The requested local settings are `gpt-6.1-sol` at `1000000` tokens and `gpt-6-astra` at `750000` tokens.
The documented `model_context_window` setting describes the active model,
so setting it globally does not express different limits for those models.

## Root cause

The [official configuration reference][reference] documents `model_catalog_json`
as a JSON model catalog loaded at startup.
The [official schema][schema] defines `model_context_window` as an integer,
not a map indexed by model name.

Source inspected at upstream commit `d70c2c2cdf0226b4b5d393d30c6785850d0ce85b`,
separately from the installed `codex-cli 0.163.0` binary:

- `codex-rs/core/src/config/mod.rs:2199` parses the file as a full catalog:

  ```rust
  // codex-rs/core/src/config/mod.rs
  let catalog = serde_json::from_str::<ModelsResponse>(&file_contents).map_err(|err| {
  ```

- `codex-rs/models-manager/src/manager.rs:375` stores the catalog entries directly:

  ```rust
  // codex-rs/models-manager/src/manager.rs
  pub fn new(auth_manager: Option<Arc<AuthManager>>, model_catalog: ModelsResponse) -> Self {
      Self {
          remote_models: model_catalog.models,
          auth_manager,
      }
  }
  ```

- `codex-rs/models-manager/src/manager.rs:827` returns the stored entries:

  ```rust
  // codex-rs/models-manager/src/manager.rs
  Box::pin(async { self.remote_models.clone() })
  ```

The catalog therefore carries model metadata as a snapshot.
It needs refreshing when upstream metadata changes.

## Verification

Verified on 2026-10-19 with `codex-cli 0.163.0`.
The disposable catalog was tested before changing the user configuration.
The saved user configuration was then checked with the real command:

```sh
# Run from the repository root.
codex debug models
```

The JSON result contained:

- `gpt-6.1-sol`:
  `context_window = 1000000`,
  `max_context_window = 1000000`.
- `gpt-6-astra`:
  `context_window = 750000`,
  `max_context_window = 872000`.

The original cached catalog reported `500000` and `272000` context tokens respectively.
That baseline distinguishes the new values from an unchanged catalog.
All other cached model entries were preserved.

This checks client catalog loading.
It does not test a request containing those token counts or a running Desktop process.
Server capacity is not increased by editing client metadata.

## Verified configuration

The user configuration references the catalog:

```toml
# ~/.codex/config.toml
model_catalog_json = "/home/user/.codex/model-context-catalog.json"
```

The catalog was copied from the current `models_cache.json` model list,
wrapped in a `models` object,
and changed only at the requested models' context fields.
The backup of the original config is in the private agent scratch directory for this change.

The tradeoff is a static copy of model metadata,
including model instructions and the available model list.
Refresh from a current catalog before adopting later upstream metadata changes.
Startup-only loading means an already running app must restart before using the catalog.

## What does not work

A single global `model_context_window` value cannot encode both requested limits.
No legacy profile tables were added:
the current [advanced configuration documentation][advanced] uses separate profile files,
and choosing a profile would require an additional selection beyond switching models.

## Upstream filing decision

The policy in `../../.out-of-scope/codex-harness.md` was checked;
it permits requested configuration work and does not prohibit issue tracking.
A tracker search for `model_catalog_json` found existing reports,
including startup-only catalog reload request `openai/codex#49930`.
No upstream report was filed or drafted because this task established a documented configuration path.

- Upstream fault:
  no defect established by this configuration task.
- Fixability:
  no upstream patch is required for the verified setup.
- Supported use:
  the official reference documents a custom model catalog.
- Contribution acceptance:
  not investigated because no contribution is proposed.
- Maintainer willingness:
  not assessed because no upstream change is needed.
- Prototype:
  not applicable;
  the consumer configuration was tested directly.

There is no new defect evidence to contribute from this task.

[reference]: https://developers.openai.com/codex/config-reference/
[schema]: https://developers.openai.com/codex/config-schema.json
[advanced]: https://developers.openai.com/codex/config-advanced/
