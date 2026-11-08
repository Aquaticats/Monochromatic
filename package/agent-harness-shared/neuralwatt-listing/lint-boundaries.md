# Intentional nullish-union suppressions

## Rule and configuration

`package/config/oxlint/src/rule/restriction.ts` enables
`no-restricted-syntax/no-nullish-union` at error severity.
It rejects `T | null` and `T | undefined` unions
because this repo models absence with optional properties
under `exactOptionalPropertyTypes`
(`no-restricted-syntax/no-optional-escape` bans the `Partial` and
sentinel workarounds).

## Why the config cannot express this package

This package mirrors two upstream wire shapes exactly
(`doc/decision/pi-model-retirement.md` records the same
mirror-first posture):

- `NeuralWattEffortsDict` and `NeuralWattThinkingLevelMap` mirror
  `dsh-llm-pi-ai`'s `PiAiReasoningEfforts = Partial<Record<ModelThinkingLevel,
  string | null>>` (`lib/types/catalog.d.ts`) and pi-ai's `thinkingLevelMap`
  (`providers/model-schema.ts`).
  In those shapes `null` is not absence:
  it is the instruction "level unsupported",
  distinct from an absent key ("supported, dispatch nothing").
  The expansion must keep both spellings reachable,
  or the pi projection would silently change dispatch behavior.
- `NeuralWattReasoningEfforts = false | NeuralWattEffortsDict` mirrors
  `PiAiModelProfile['reasoningEfforts'] = false | PiAiReasoningEfforts`,
  where `false` is the declaration "non-reasoning model",
  distinct from the field staying absent ("the listing disclosed no efforts").
  Its scoped disable sits on the alias declaration.

The statement-scoped suppression in `expandThinkingLevelMap`
covers only the mutable scratch record that builds the exported map;
the exported declaration carries its own documented suppression.
The offered-dict construction in `listing.ts` needs none,
since its scratch holds spellings only.
