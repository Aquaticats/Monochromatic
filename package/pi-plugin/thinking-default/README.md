# pi-plugin-thinking-default

Model-aware thinking default extension for pi.

GPT-shaped models get `xhigh` thinking.
Non-GPT models get `xhigh` thinking when the model supports it,
 and `high` thinking otherwise.

## Installation

Add this package to global pi settings at `~/.pi/agent/settings.json`:

```json
{
  "packages": [
    "/var/home/user/Monochromatic/package/pi-plugin/thinking-default"
  ],
  "defaultThinkingLevel": "high"
}
```

Keep project `.pi/settings.json` package-free unless this behavior becomes shared project policy.
The project settings file currently documents that user-specific workflow packages belong in global settings.

## Behavior

The extension applies the policy when pi starts or reloads a session,
and whenever the active model changes through `/model`,
 Ctrl+P cycling,
 or session restore.

Manual thinking changes remain in effect until the next session start or model selection.
The extension skips `pi.setThinkingLevel()` when the current level already matches the policy target.

Pi persists `pi.setThinkingLevel()` calls to global `defaultThinkingLevel`.
After each model-aware application,
 this extension rewrites the persisted scalar default back to `high`,
 the safe fallback for models that do not expose `xhigh`,
 while still raising GPT and xhigh-capable sessions to `xhigh`.

Pi clamps requested thinking levels to the selected model capabilities.
A non-reasoning model can therefore display `off` even though this extension requested `high`.

## Level selection

A model is GPT-shaped when the final slash-delimited segment of its model id starts with `gpt-`,
after lowercasing.
GPT-shaped ids always resolve to `xhigh`.

Non-GPT ids resolve to `xhigh` when the model supports it.
Pi treats `xhigh` as opt-in:
a model supports `xhigh` only when it declares `reasoning`
and maps `xhigh` to a non-null value in its `thinkingLevelMap`,
so a missing `xhigh` entry means unsupported
(unlike the other levels,
where a missing entry falls back to provider defaults).
Models without that mapping resolve to `high`.

Examples that resolve to `xhigh`:

- `gpt-5.5` (GPT-shaped)
- `openai/gpt-5.4` (GPT-shaped)
- `GPT-5.5` (GPT-shaped)
- `synthetic/hf:zai-org/GLM-5.2` (non-GPT,
   declares `xhigh: "max"`)

Examples that resolve to `high`:

- `synthetic/hf:moonshotai/Kimi-K2.6` (no `xhigh` mapping)
- `synthetic/hf:zai-org/GLM-5.1` (no `xhigh` mapping)
- `claude-sonnet-4-5` (no `xhigh` mapping)

## Source structure

```text
src/
  index.ts                         # Extension entry point and pi event registration
  model-policy.ts                  # Model id leaf extraction, xhigh availability, and thinking target selection
  apply-thinking-default.ts        # Side-effect boundary around getThinkingLevel and setThinkingLevel
  global-settings.ts               # Restores persisted defaultThinkingLevel to high
  model-policy.unit.test.ts        # Pure policy tests
  apply-thinking-default.unit.test.ts
  index.unit.test.ts               # Extension registration and handler tests
  mise.verify-extension.ts         # Built extension smoke verification
```

## Validation

Run package validation from the repository root:

```sh
mise run //package/pi-plugin/thinking-default:build
mise run //package/pi-plugin/thinking-default:test:unit
mise run //package/pi-plugin/thinking-default:lint
mise run //package/pi-plugin/thinking-default:verify:extension
```
