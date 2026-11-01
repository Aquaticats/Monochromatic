# correction-reminder

Claude Code `UserPromptSubmit` hook that detects correction phrases in user input
and reminds Claude to re-check evidence before its next substantive response.

## What it does

When the user submits a prompt containing a correction phrase,
 this hook injects the following block into Claude's conversation context:

```text
<correction-detected>
The user is correcting a substantive claim from your previous response.
Per AGENTS.md rule CKB: a user correction is an
approach-change moment, not a small patch. The blind spot that produced
the original claim is still active for the revision.

Before your next substantive response, re-check the claim against primary
sources, local files, logs, or command output. Do not run a same-session
self-review or write an `Advisor pass: ...` line; self-review is not
independent evidence. See `doc/agent/self-review.md`.
</correction-detected>
```

When the prompt is neutral,
 the hook returns an empty `additionalContext` so the rest of the pipeline runs unchanged.

## Triggering phrases

The hook fires on any of the following patterns in user input (case-insensitive,
 word-boundary matched):

- "demonstrably false" / "demonstrably wrong"
- "you missed"
- "didn't you"
- "you're wrong" / "you are wrong"
- "shouldn't have" / "should not have"
- "why would you"
- "that's wrong" / "this is wrong"
- "you got that/this/it wrong"
- "please be more careful"

## Why this hook exists

Session evidence (`1cbe8d82`):
 four consecutive user corrections each produced a fresh error in the revision.
 The blind spot that produced the original claim was active for each revision
 because each was framed as a small patch
 rather than as a change-of-approach moment that warrants concrete re-verification.

This hook fires before the response is written,
 which is the highest-leverage point at which to redirect.

## Hook configuration

```jsonc
// In .claude/settings.json
{
  "hooks": {
    "UserPromptSubmit": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "cccr",
          },
        ],
      },
    ],
  },
}
```
