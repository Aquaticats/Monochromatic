# Make cli-git trust failures actionable in noninteractive sessions

Note 2026-11-04T22:23:58-05:00:
the program `cli-git-native` is renamed `monochromatic-git-policy-cli`,
the command word `git cli-git` becomes `git monochromatic-git-policy-cli`,
the settings file `cli-git.config.jsonc` becomes `monochromatic-git-policy-cli.config.jsonc`,
and the folders,
logs and service units that carry `cli-git` carry `monochromatic-git-policy-cli`
(the list is in `doc/planning/git-policy-cli-program-rename.md`).
The new names arrive with the first release after this date;
this repository switches its installed program and its settings file when it installs that release.
This record keeps the names of its time.

## Status

Implemented on `main` on 2026-08-13
and user-boundary verified on 2026-08-14.

Implementation commits:

- `6835b8c4e` classifies unavailable trust consent;
- `6694f90b9` makes untrusted-config remediation actionable;
- `34e8aa2a9` adds successful namespace and trust help;
- `f35927c7a` preserves real Git global short-circuit flags before namespace dispatch;
- `5f1894f59` classifies terminal EOF as unavailable consent
  and preserves previous records after unavailable re-trust;
- `54ad78083` distinguishes global option tokens from flag-shaped option values.

Verification passed for the package build,
full unit suite,
type lint,
packed trust consumer,
Markdown lint,
and direct pseudo-terminal probes of root and recursive EOF.
Package Oxlint still reports its existing `test-import(require-eventual-artifact)` baseline:
96 diagnostics across 42 test and performance files,
with no production-source diagnostic.

## Incident

At repository commit `044e70ea639e45626aadddd7700bb827558e3c64`,
a Git operation in an untrusted disposable worktree emitted:

```json
{
  "schemaVersion": 1,
  "sequence": 0,
  "type": "engine-failure",
  "code": "config-untrusted",
  "message": "cli-git configuration is not trusted; run `git cli-git trust` after reviewing it."
}
```

The recommended bare command ran without terminal input.
Cli-git printed its trust disclosure and then emitted:

```json
{
  "schemaVersion": 1,
  "sequence": 0,
  "type": "engine-failure",
  "code": "trust-failed",
  "message": "Trust declined; no persistent record was installed."
}
```

The caller then tried pipes and pseudo-terminals before discovering that the supported command was:

```sh
git cli-git trust --yes
```

A disposable registry reproduction produced the same disclosure,
message,
and exit `2`.
The focused existing test also passed while asserting the misleading behavior:

```sh
mise run //package/git-policy/cli:test:unit -- \
  package/git-policy/cli/src/trust/management-runtime.unit.test.ts
```

`package/git-policy/cli/src/trust/management-runtime.unit.test.ts:258-269` names noninteractive input a decline
and expects the quoted `trust-failed` event.

## Diagnosis

This is a product-interface defect plus a caller discipline defect.
The product interface should remain usable even when the caller discipline fails.

### Cli-git conflates unavailable consent with rejection

`promptForTrust` returns `false` when either standard input or standard error is not a terminal
(`package/git-policy/cli/src/trust/management-runtime.ts:107-111`).

Both trust implementations treat that boolean exactly like a rejected prompt:

- MJS behavior is at `package/git-policy/cli/src/trust/explicit-trust.ts:220-223`.
- TypeScript behavior is at `package/git-policy/cli/src/trust/explicit-typescript-trust.ts:283-286`.

No user declined in this path.
The runtime had no interactive consent channel.
A boolean cannot preserve that distinction.

### The recovery diagnostic omits its automation path

The `config-untrusted` event recommends only bare `git cli-git trust`
(`package/git-policy/cli/src/trust/trust-service.ts:258-260`).
That recommendation predictably fails in a noninteractive process,
even though `--yes` is already accepted by the parser
(`package/git-policy/cli/src/management-parser.ts:78-80`).

### Help is discoverable but behaves like an error

`git cli-git trust --help` currently reveals `trust [--yes]` through the generic usage text,
but exits `2`.
The parser has no help action,
so `--help` follows the same refusal path as an unknown option
(`package/git-policy/cli/src/management-parser.ts:116-189`).

On 2026-08-13,
local probes returned:

```text
mise trust --help exit=0
gh repo delete --help exit=0
tofu apply --help exit=0
git cli-git trust --help exit=2
```

The installed `mise 2026.7.0` help describes `--yes` as answering confirmation prompts.
The installed GitHub CLI help for `gh repo delete` documents `--yes`
and adds a safety condition for noninteractive deletion.
The installed OpenTofu help documents `-auto-approve` as skipping interactive approval.
These precedents keep bypass consent explicit and make it visible in successful help output.
Online primary pages were located but could not be fetched during the investigation,
so this precedent evidence is intentionally limited to the installed command help.

### The caller skipped native-option discovery

Before attempting pipes or pseudo-terminals,
the caller should have run `git cli-git trust --help` or inspected current cli-git docs/source.
The generic usage would have exposed `[--yes]` even with its incorrect exit status.
`AGENTS.md` now records that agent-side requirement as `CLH`.

## Recommendation

Implement the complete interface correction while preserving the explicit trust boundary.

### Represent consent as an outcome

Replace `TrustConsentAdapters.prompt(): Promise<boolean>`
with an outcome that distinguishes at least:

```ts
type TrustConsentOutcome = 'approved' | 'declined' | 'unavailable';
```

Return `unavailable` when either terminal stream required by the prompt is absent
or input ends before a response.
Continue to return `declined` for a completed interactive response other than exact `yes`.

When consent is unavailable,
retain exit `2` and emit a stable machine-readable code such as `trust-consent-unavailable`:

```json
{
  "schemaVersion": 1,
  "sequence": 0,
  "type": "engine-failure",
  "code": "trust-consent-unavailable",
  "message": "Interactive consent unavailable. After review, rerun `git cli-git trust --yes`; no new record installed."
}
```

Keep `trust-failed` for an actual interactive rejection unless a broader failure-code redesign is separately approved.
The new code must be added to the engine failure union and compatibility tests.

### Make the first diagnostic actionable

Change `config-untrusted` to name the affected config and both supported paths:

```text
cli-git configuration is not trusted.
Review <config-path>, then run `git cli-git trust` in an interactive terminal.
For explicit noninteractive consent, run `git cli-git trust --yes`.
```

This does not weaken consent.
It makes the existing explicit path discoverable at the point where recovery begins.

### Implement real help

Make `git cli-git --help` and `git cli-git trust --help` successful help actions with exit `0`.
Trust help should state that:

- bare trust requires terminal input and output;
- `--yes` is explicit noninteractive consent;
- disclosures are still printed;
- `--yes` accepts every applicable consent stage,
  including recursive descendant authority;
- trusted code receives full account permissions.

Help should short-circuit before config discovery,
candidate building,
trust-registry access,
or other repository mutation.
Unknown options should continue to exit `2`.

### Keep bare trust fail-closed

Do not interpret a noninteractive invocation of bare `git cli-git trust` as approval.
The second consent stage can grant authority over current and future descendant repositories.
TTY detection,
CI detection,
or command invocation alone must not silently grant that authority.

## Options

### Complete outcome, diagnostic, and help correction

Pros:

- reports what happened truthfully;
- gives automation a stable branch condition;
- makes the supported command discoverable before terminal emulation;
- preserves explicit `--yes` consent and all disclosures.

Cons:

- adds a failure code to a stable JSON union;
- changes the consent adapter and both trust implementations;
- requires source,
built-artifact,
specification,
and decision-document updates.

### Message-only correction

Pros:

- can be localized to current diagnostics;
- preserves the existing failure-code union.

Cons:

- machine consumers still cannot distinguish rejection from unavailable input;
- `--help` still exits as an error;
- the boolean adapter continues to erase a domain-relevant state.

### Treat bare noninteractive trust as approval

Pros:

- removes automation friction;
- requires no `--yes` discovery.

Cons:

- weakens the deliberate consent boundary;
- can grant recursive descendant authority without a separate affirmative signal;
- makes missing terminal wiring indistinguishable from intended approval.

Ranking:
complete correction > message-only correction > implicit approval.
The complete correction outranks the message-only patch because stable machine classification
and successful help prevent recurrence,
not just this message.
The message-only patch outranks implicit approval because preserving explicit authority
is more important than removing one flag.

## Verification requirements

Exercise the built CLI at the user boundary,
not only direct trust functions.
Cover:

- terminal input plus terminal error output with exact `yes`;
- terminal input plus terminal error output with a rejecting or invalid response;
- nonterminal input with terminal error output;
- terminal input with nonterminal error output;
- both streams nonterminal;
- EOF or aborted prompt input;
- bare trust and `trust --yes`;
- MJS and TypeScript configs;
- recursive and non-recursive configs;
- disclosure output for every successful `--yes` stage;
- unchanged trust registry after every unavailable path,
  including re-trust with a previous record;
- namespace and trust help exit `0` without reading or executing repository config;
- unknown options still exit `2`;
- packed CLI behavior through the existing built trust fixture.

Update `package/git-policy/cli/README.md`,
`package/git-policy/cli/SPEC.md`,
and `doc/decision/cli-git-policies-platform.md` together with implementation and tests.
