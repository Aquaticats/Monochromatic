# Pi 0.82.0: openai-codex sessions eagerly dump `PI_*` environment variables before task work

Pi 0.82.0 added a system-prompt guideline telling the agent to inspect the new `PI_*` session
environment variables.
Codex-family models act on it eagerly at session start,
running an `env`-dump command before touching the actual task,
which then trips the local approval guard because the command reads the process environment.

## Symptom

At the start of a session with an openai-codex model
(observed with `gpt-5.3-codex-spark`,
 `gpt-5.6-sol`,
 and `gpt-5.6-terra`),
the agent's first tool call is an environment dump unrelated to the user's request:

```bash
pwd && env | rg '^PI_' && git status --short
```

Variants seen in local session logs:

```bash
printenv | rg '^PI_' . || true
env | grep '^PI_' || true
env | rg '^PI_' . /dev/null || true
```

Because the command pipes `env` output,
 the approval guard interrupts with a dialog:

```text
Command needs approval. Agent's explanation:
> The command reads environment variables prefixed `PI_`, which may contain credentials or
other sensitive values, in addition to routine repository status checks.
```

The behavior started immediately after the 0.82.0 update (released 2026-07-24).
It is not deterministic:
 some openai-codex sessions skip the dump,
 and no non-codex session in the local logs ran one.

## Root cause

Step 1:
 0.82.0 added "Session-aware,
 streaming bash integrations".
The built-in bash tool now injects `PI_SESSION_ID`,
 `PI_SESSION_FILE`,
 `PI_PROVIDER`,
 `PI_MODEL`,
 and `PI_REASONING_LEVEL`
into every command it runs (`CHANGELOG.md` of `@earendil-works/pi-coding-agent`):

```text
Exposed `PI_SESSION_ID`, `PI_SESSION_FILE`, `PI_PROVIDER`, `PI_MODEL`, and `PI_REASONING_LEVEL`
to commands run by built-in and factory-created bash tools.
```

Step 2:
 to advertise the feature to the model,
 `createBashToolDefinition` attaches a prompt guideline,
 gated on the same `exposeSessionEnvironment` flag.
Installed 0.82.0,
 `dist/core/tools/bash.js:232`:

```js
promptGuidelines: exposeSessionEnvironment
    ? ["Inspect PI_* environment variables for current model and session details."]
    : undefined,
```

The same line is still present on upstream `main` at
`packages/coding-agent/src/core/tools/bash.ts:329-331`
(checked via `gh api` on 2026-07-27).

Step 3:
 that guideline lands in the system prompt's Guidelines list for every session,
 regardless of provider.

Step 4:
 Codex-family models read the imperative "Inspect ..." as an instruction to execute now,
 so the session opens with an environment dump.
Other models (observed:
 `openrouter/moonshotai/kimi-k3`,
 plus Anthropic models per the user's report) treat it as on-demand guidance and only read the
variables when asked which model or provider is running,
 which is the use case `docs/environment-variables.md` actually documents:

```text
When asked which model or provider is running, inspect these variables instead of inferring
the answer from the system prompt
```

Step 5:
 the approval prompt is a local guard reacting to the `env` read,
 not part of Pi's feature.
The `PI_*` values themselves are non-secret session metadata
(session ID,
 session file path,
 provider,
 model,
 reasoning level),
 but a guard cannot tell `env | rg '^PI_'` from a credential exfiltration attempt by shape alone.

## Verification

Version under test:
 `@earendil-works/pi-coding-agent` 0.82.0,
 installed at
`package/pi-plugin/advisor/node_modules/@earendil-works/pi-coding-agent`
(and a second pnpm copy under the repo's root `node_modules/.pnpm`).
Pi 0.82.1 does not change the guideline;
 its changelog has no entry touching the bash tool prompt,
 and upstream `main` still carries the line (see "Root cause").

Harness:

```bash
grep -n "Inspect PI" \
  package/pi-plugin/advisor/node_modules/@earendil-works/pi-coding-agent/dist/core/tools/bash.js
```

prints line 232 with the guideline.

Catalog from `~/.pi/agent/sessions`,
 sessions dated 2026-07-24 through 2026-07-27
(the 0.82.0 era),
 scanning for the `PI_'` grep pattern in recorded bash commands:

- Sessions that ran an eager `PI_*` dump:
  `gpt-5.6-terra` (1),
  `gpt-5.6-sol` (1),
  `gpt-5.3-codex-spark` (1),
  all provider `openai-codex`.
- Sessions that did not:
  `gpt-5.6-sol` (4),
  `openrouter/moonshotai/kimi-k3` (2).

So the trigger correlates with openai-codex models but fires in only some of their sessions,
matching the upstream report's "not deterministically reproducible".

## Verified workarounds

The behavior is harmless in content:
 the dumped values are session metadata,
 not credentials.

- Approve or deny the guard prompt per session.
  Tradeoff:
  recurs at every eager session start;
  denying just means the model proceeds without the metadata,
  which it never needed for the task.
- Add a trust rule auto-approving the exact command shape.
  Tradeoff:
  any rule broad enough to catch the variants (`env`,
  `printenv`,
  `rg`,
  `grep`) also auto-approves genuine environment-read commands,
  weakening the credential guard;
  a rule pinned to one literal command only fixes that one spelling.
- Add a counter-instruction to the project or global `AGENTS.md`,
  for example "do not inspect `PI_*` variables unless asked which model or provider is running".
  Tradeoff:
  prompt-level shaping,
  not a guarantee;
  it argues with the baked-in guideline instead of removing it,
  and effectiveness against the baked-in guideline is unverified until the next eager session runs.
- Rewrite the guideline with an extension.
  Pi's `before_agent_start` event fires after prompt submission and before the agent loop on
  every turn,
  and lets an extension replace the chained system prompt (`docs/extensions.md:520`,
  the `prompt-customizer.ts` example).
  A single file under `~/.pi/agent/extensions/` removes the trigger text before the model sees it:

  ```ts
  // ~/.pi/agent/extensions/pi-env-guideline-override.ts
  import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";

  const GUIDELINE = "Inspect PI_* environment variables for current model and session details.";
  const REPLACEMENT =
  	"When asked which model, provider, or reasoning level is running, read the PI_* environment " +
  	"variables of the bash tool; do not inspect them otherwise.";

  export default function (pi: ExtensionAPI) {
  	pi.on("before_agent_start", async (event) => {
  		if (!event.systemPrompt.includes(GUIDELINE)) return;
  		return { systemPrompt: event.systemPrompt.replace(GUIDELINE, REPLACEMENT) };
  	});
  }
  ```

  Rewording instead of deleting keeps the documented on-demand use
  (reading `PI_*` when asked which model or provider is running).
  Applying the same rewrite on every turn keeps the prompt prefix stable,
  so provider prompt caching is unaffected.
  Tradeoffs:
  one small file to maintain;
  if upstream changes the guideline wording the `includes` check stops matching,
  which fails safe (the original behavior returns,
  nothing breaks).
  Status:
  drafted from the documented `before_agent_start` contract but not yet installed or exercised
  in this workspace,
  so unlike the entries above it is unverified.

There is no user-facing setting to disable the injection:
 `exposeSessionEnvironment: false` exists only on the extension-facing `createBashTool()` API
(`docs/extensions.md:2099`),
 not for the built-in bash tool.

## What does not work

- Updating to 0.82.1:
  the changelog contains no bash-prompt change and the line is still on upstream `main`.
- Locating the guideline with `rg` against the installed package:
  `rg` respects ignore files and skips `dist/`,
  so it reports no match even though the string is present.
  Use `grep -rn` or `rg --no-ignore` when searching installed build output.

## Upstream filing decision

`.out-of-scope/` check:
 no entry covers Pi prompt content or this bug class;
 the only Pi entry is `pi-gpt55-long-context.md`,
 which scopes out context-window work.
No exemption applies.

Duplicate search:
 `gh search issues --repo earendil-works/pi "PI_ environment variables"` found
[earendil-works/pi#7128][issue-7128],
 "New default PI_* guideline in system prompt over-encourages unnecessary bash calls",
 filed 2026-07-26 against 0.82.1 by a new contributor and auto-closed by the repo's
new-contributor bot pending maintainer review.
The thread contains only the auto-close notice.

Six-constraint walk:

1.  Really upstream's fault:
    partially.
    The guideline wording ("Inspect ...") invites eager execution,
    but the eager first-call behavior is model-side,
    and the approval prompt is the local guard,
    not Pi.
    The wording concern is upstream's.
2.  Can upstream fix it:
    yes;
    rewording or demoting the guideline is a one-line change
    (`src/core/tools/bash.ts:329-331`).
3.  Supporting this use case:
    yes;
    the session environment is a documented 0.82.0 feature.
4.  Would the repo welcome the contribution:
    unknown-to-cautious.
    New-contributor issues are auto-closed and only reopened after maintainer review
    (per the bot notice on #7128 and `CONTRIBUTING.md`).
5.  Will they likely fix it:
    no signal yet;
    #7128 awaits the daily maintainer review of auto-closed issues.
6.  Minimal fix prototyped:
    not applicable;
    the candidate change is a prompt-wording decision,
    not a code defect with a verifiable patch.

Decision:
 do not file a new issue;
 #7128 is the same behavior.
The additive material this investigation has beyond the thread
(exact source location on `main`,
 affected-version data back to 0.82.0,
 the observed command shapes,
 the guard-interaction,
 and the local session-log correlation with openai-codex models)
is kept below as a comment draft.
It has not been posted;
 posting waits on explicit user authorization,
 and the issue's auto-closed state means the comment may not be visible to maintainers until
reopened.

~~~md
Additional data points on the same behavior, from local installs and session logs:

- The guideline comes from `createBashToolDefinition` in
  `packages/coding-agent/src/core/tools/bash.ts` (still on `main`):

  ```ts
  promptGuidelines: exposeSessionEnvironment
      ? ["Inspect PI_* environment variables for current model and session details."]
      : undefined,
  ```

- Affected since 0.82.0, where the session environment feature landed; reproduced on 0.82.0.
- Observed eager first-commands in openai-codex sessions (`gpt-5.3-codex-spark`, `gpt-5.6-sol`,
  `gpt-5.6-terra`), each before any task work:

  ```bash
  pwd && env | rg '^PI_' && git status --short
  printenv | rg '^PI_' . || true
  env | grep '^PI_' || true
  ```

- In local session logs since the 0.82.0 update, 3 of 7 openai-codex sessions opened with such a
  dump; 0 of 2 openrouter sessions did. Consistent with "not deterministically reproducible".
- Side effect: command-approval guardrails flag the `env` pipe as a possible credential read, so
  the eager call also costs the user an approval prompt at session start.

A wording along the lines of the docs ("when asked which model or provider is running, inspect
these variables") would keep the guidance without inviting an eager dump.
~~~

[issue-7128]: https://github.com/earendil-works/pi/issues/7128
