# Codex synchronous user input works in Default mode

## Observed behavior

On 2026-11-02,
`functions.request_user_input` successfully asked a question in a Codex session running in Default mode.
The assistant received the user's answer as the tool result before continuing.
The user confirmed:
"Looks like it's working.
Great.
Where should we document this?"

This records a working capability at the conversation boundary.
It establishes neither a preferred question tool nor a rule about when to ask questions.

## Tool descriptions available during the test

The session exposed both tools:

- `functions.request_user_input`:
  its description said it asks questions and waits for the response,
  and listed availability in Default or Plan mode.
- `functions.request_user_input_async`:
  its description said it returns immediately and receives any reply asynchronously as a new user message.
  This tool was not invoked in the test.

These descriptions came from the live session's tool catalog.
They are distinct from the observed result of the synchronous invocation.

## Verification

The user requested:
"Invoke the request_user_input tool.
We'll check if it really is sync."

The assistant invoked `functions.request_user_input` with this payload:

```jsonc
// doc/troubleshooting/codex-user-input.md: recorded tool-call arguments
{
  "questions": [
    {
      "header": "Sync check",
      "id": "sync_check",
      "question": "This is the blocking question tool test. Which response would you like to send?",
      "options": [
        {
          "label": "Complete test (Recommended)",
          "description": "Submit this response so I can report what the tool returned."
        },
        {
          "label": "Alternate response",
          "description": "Submit a different response to verify that your selection is returned."
        }
      ]
    }
  ]
}
```

The tool returned:

```jsonc
// doc/troubleshooting/codex-user-input.md: recorded tool result
{
  "answers": {
    "sync_check": {
      "answers": [
        "None of the above",
        "user_note: Looks like it's working. Great. Where should we document this?"
      ]
    }
  }
}
```

The next assistant message acknowledged the returned answer and continued the conversation.
The returned free-text note also demonstrates that the user could supply an answer beyond the offered choices.

### What worked

- The synchronous tool accepted the invocation in Default mode.
- The result carried the question identifier and the user's free-text answer.
- The user confirmed the interaction worked.

### Unverified surfaces

The test did not exercise Plan mode,
the async tool,
cancellation,
an unanswered question,
or concurrent background work.
It did not measure wait duration or inspect the host's internal scheduling.
The Codex host version and build identifier were not captured.
The result therefore describes this session,
not availability across every Codex host or release.

## Interpretation

The successful invocation disproves the premise that this session could ask questions only asynchronously.
No tool failure or need for a workaround was observed.
No implementation root cause is asserted,
and no source investigation was performed.

## Upstream filing decision

Nothing to add upstream:
this was a successful capability check,
with no defect established and no issue or comment drafted or sent.

- Upstream fault:
  none established.
- Fixability:
  not applicable because no defect was identified.
- Supported use case:
  the live tool description listed Default mode,
  and the invocation worked there.
- Contribution policy:
  not investigated because no filing was proposed.
- Maintainer willingness:
  not investigated because no change was proposed.
- Minimal fix:
  none needed for the tested interaction.

The [Codex harness maintenance policy](../../.out-of-scope/codex-harness.md) was checked.
It does not prohibit investigation or issue tracking.
No upstream tracker search was performed because this check established no reportable failure.
