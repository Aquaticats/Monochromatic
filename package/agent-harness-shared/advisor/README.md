# @monochromatic-dev/agent-harness-shared-advisor

Host-neutral Advisor core for agent harness adapters:
the scoped secondary-review
pipeline behind narrow host ports.

The package holds configuration merging,
effective-scope resolution,
model-budgeted
serialized context,
bounded completion recovery,
operation accounting,
and plain-text
summaries.
Harness adapters keep registration,
slash commands,
rich TUI rendering,
and
session plumbing.
The core reads host facts only through the three ports in `src/ports.ts`:
model resolution,
transcript,
and theme.

## Provenance

The core moved out of `package/pi-plugin/advisor/src` on 2026-10-20 so a DSH variant can
share it without consuming pi build output.
The pi adapter stays at
`package/pi-plugin/advisor` and imports this package through the `/ts` subpath
(source-level workspace imports,
`doc/decision/workspace-ts-source-imports.md`).
A future `package/dsh-plugin/advisor` adapter plugs into the same ports.

## The three ports

Model resolution (`AdvisorModelResolution`) supplies registry reads and request auth.
A pi `ModelRegistry` satisfies it structurally,
so adapters pass their registry unchanged.

```typescript
export type AdvisorModelResolution = {
  getAvailable: () => readonly Model<Api>[];
  getAll: () => readonly Model<Api>[];
  getProvider: (provider: string) => Provider | undefined;
  getApiKeyAndHeaders: (model: Model<Api>) => Promise<AdvisorRequestAuth>;
};
```

Transcript (`AdvisorTranscriptSerialization`,
plus `buildContextEntries` on
`AdvisorTranscript`) supplies LLM conversion,
serialization,
token estimation,
and the
frozen session snapshot.
Host conversion functions satisfy it directly because the
parameters mirror the host's mutable message arrays.

```typescript
export type AdvisorTranscriptSerialization = {
  convertToLlm: (messages: AdvisorAgentMessage[]) => Message[];
  serializeConversation: (messages: Message[]) => string;
  estimateInputTokens: (options: {
    readonly systemPrompt: string;
    readonly contextText: string;
  }) => number;
};

export type AdvisorTranscript = AdvisorTranscriptSerialization & {
  buildContextEntries: () => readonly ForeignBorrowed<AdvisorSessionEntry>[];
};
```

Theme (`AdvisorTheme`) supplies semantic foreground tokens for plain-text renderers.
`AdvisorThemeColor` is a closed union (a subset of pi `ThemeColor`),
so host themes
satisfy the port under strict variance.
A headless adapter may implement `fg` and `bold`
as identity functions.

```typescript
export type AdvisorThemeColor =
  | 'accent'
  | 'dim'
  | 'error'
  | 'success'
  | 'toolOutput'
  | 'toolTitle'
  | 'warning';

export type AdvisorTheme = {
  fg: (color: AdvisorThemeColor, text: string) => string;
  bold: (text: string) => string;
};
```

## Adapter guide

1. Expose host facts as an `AdvisorRunContext`:
   `modelRegistry`,
   `transcript`,
   `cwd`,
   plus optional `model`,
   `getScopedModels`,
   and `scopedModels`.
2. Call `runAdvisor({ ctx, config, ... })` from `src/run-advisor.ts` with that context.
3. Render results with `renderAdvisorSummary` and `renderAdvisorOperationSummary`,
   or
   build host components on top of their plain-text output.
4. Keep tool schemas,
   registration,
   and slash commands in the adapter;
   translate onto
   `AdvisorToolParams`,
   `AdvisorDetails`,
   and `AdvisorRunResult`.

The pi adapter shows the full wiring in `package/pi-plugin/advisor/src/run-context.ts`
(`toAdvisorRunContext`) and `package/pi-plugin/advisor/src/tool.ts` (`runAdvisor`
wrapper over `PiAdvisorRunOptions`).

## Moved modules

Core sources moved from `package/pi-plugin/advisor/src` (35 files):
`active-tool`,
`advisor-client`,
`advisor-completion`,
`advisor-completion-client`,
`advisor-completion-error`,
`advisor-deadline`,
`advisor-request`,
`advisor-requested-model`,
`advisor-selection`,
`config`,
`config-schemas`,
`constants`,
`context`,
`context-user`,
`dispatch-scope`,
`operation`,
`operation-attempt`,
`operation-candidates`,
`operation-clock`,
`operation-error`,
`operation-ledger`,
`operation-progress`,
`operation-types`,
`operation-usage`,
`operation-test-fixture`,
`output-eligibility`,
`project-context`,
`provider-credit`,
`rendering-operation`,
`rendering-summary`,
`run-advisor`,
`run-preparation`,
`tool-context-selection`,
`tool-params`,
`types` (each `*.ts` in `src/`).

Unit tests moved alongside them (18 files):
`advisor-client-failure`,
`advisor-client`,
`advisor-request`,
`config-hedging`,
`config`,
`context`,
`core`,
`operation-clock`,
`operation-collection`,
`operation-ledger`,
`operation-recovery`,
`project-context`,
`provider-credit`,
`rendering-operation`,
`rendering-summary`,
`run-advisor-context`,
`run-advisor-operation`,
`selection.characterization` (each `*.unit.test.ts` in `src/`).
`rendering-operation.unit.test.ts` split:
shared keeps the
`renderAdvisorOperationSummary` cases,
the pi package keeps the `renderAdvisorResult`
delegation case.

## Config policy

`defaultReviewerModel` stays user config only,
settable in the global config file at
`~/.pi/agent/extensions/pi-advisor.json` and overridable per project in
`.pi/extensions/pi-advisor.json`.
It never becomes a schema default:
shipped defaults
must not pick a reviewer model on the user's behalf,
and the field is optional with no
default in `AdvisorConfigFileSchema` (`src/config-schemas.ts`) and `AdvisorConfig`
(`src/types.ts`).
A non-string value fails schema validation;
an empty or blank value
clears the key even when another scope sets one.

## Model selection order

`selectAdvisorModel` (`src/advisor-selection.ts`) and `selectAdvisorRunContext`
(`src/tool-context-selection.ts`) pick the initial reviewer in this order:

1. Explicit `model` tool argument or `/advisor <slug>`:
   exact request,
   no cross-model
   fallback,
   no overlap.
2. `defaultReviewerModel`,
   when it resolves inside the output-eligible scope under the
   accepted explicit forms (canonical `provider/modelId`,
   unique bare `model.id`,
   unique
   `model.name`).
   It selects the initial reviewer of a default call and leads the
   fallback candidate order;
   the configured pick is exact only in position,
   so serial
   fallback and overlap recovery still cover it.
   It wins even when it is the current
   main model that default ranking would avoid.
   An absent,
   blank,
   unresolvable,
   or
   capacity-ineligible key falls through to the default ranking without error,
   and the
   selection reason records the fall-through.
3. Variant fallback ranking:
   expected call cost (pi behavior,
   unchanged and
   bit-identical when the key is unset),
   or the external
   ranking policy a harness adapter supplies.

`resolveConfiguredDefaultReviewer` reports the lookup as a
`ConfiguredReviewerResolution` (`keyText` with optional `entry`),
so callers can print
resolution state without treating a miss as an error.

## Ranking seam

Default (non-explicit) ranking goes through
`rankDefaultAdvisorCandidates` (`src/default-ranking.ts`).
Callers may pass a `RankAdvisorDefaultCandidates`
policy (`src/types.ts`) via `AdvisorRunOptions.rankDefaultCandidates`;
it receives the eligible
candidates with their input-token estimates and the requested output budget,
and returns pre-ranked
candidates plus a human-readable ranking reason.
The policy is a comparator or
pre-ordering computed inside the harness adapter,
so no harness types enter the core.
The core verifies the policy returned a
permutation of its inputs and throws otherwise.

Without a policy,
the built-in expected-cost ranking runs unchanged,
keeping full cost
scores in `AdvisorDefaultSelection.ranking` and the details `costRanking`.
With a
policy,
ranking entries carry canonical slugs and the policy's reason replaces the
cost reason, so serial fallback and `defaultReviewerModel` composition keep
working on either ranking.

## Behavior picks (Q5 and Q6)

Extraction follow-ups resolve copy disagreements by the grilling picks recorded in
`doc/handover/pi-advisor-dsh-variant-grilling.md`:

- Q5,
  default to richer:
  when merged copies disagree,
  adopt the richer behavior without
  an approval round and report before/after per pick in the end-of-series report.
  Applied:
  the local `formatDuration` copy in `src/rendering-summary.ts` now follows
  `@monochromatic-dev/numeric-format-core` precision rules (`DECIMAL_BELOW_MS`):
  one decimal below 10ms (`0.3ms`),
  whole milliseconds to 999ms (`51ms`),
  one decimal
  in seconds (`1.2s`).
  Before,
  it printed the raw number plus unit (`0.34ms`,
  `51.23ms`).
- Q6,
  incumbent wins on ambiguity:
  when the richer variant is ambiguous,
  keep the
  incumbent (the repo's existing shared implementation when one exists,
  otherwise the
  information-preserving variant) and report the discrepancy.
  Reviewed and not applied:
  the local `truncateContext` (src/context.ts),
  `buildAdvisorUserMessageText` (src/advisor-request.ts),
  and config `readJsonFile`
  (src/config.ts) copies keep their incumbent behavior.

## Lint suppressions record

`src/ports.ts` carries four `no-restricted-syntax/no-nullish-union` suppressions,
each a documented declaration suppressed with the `LN6` block form.
Config cannot
express them:
the rule forbids the nullish union shape outright,
and every suppressed
member must mirror a pi-coding-agent host type exactly so adapters assign host values
without casts.
The mirrors and their sources:

- `getProvider: (provider: string) => Provider | undefined`,
  mirroring
  `ModelRegistry.getProvider` (`dist/core/model-registry.d.ts`).
- `readonly fromId: string | null` on `AdvisorBranchSummarySessionEntry`,
  mirroring
  `BranchSummaryMessage.fromId` (`dist/core/messages.d.ts`).
- `readonly fromId: string | null` on `AdvisorBranchSummaryAgentMessage`,
  same mirror.
- `exitCode: number | undefined` on `AdvisorOpaqueAgentMessage`,
  mirroring
  `BashExecutionMessage.exitCode` (`dist/core/messages.d.ts`).
