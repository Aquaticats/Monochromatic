# Agent Self-Review Is Not Independent Review

Do not ask the same model,
 session,
 or local skill to evaluate its own reasoning and present that as an independent check.

A same-session "advisor pass" is self-review.
 It is not a second opinion,
 not clean-room review,
 and not evidence that a claim is sound.
 Do not create or use skills that tell an agent to critique itself when no external reviewer exists.

When a correction exposes a blind spot,
 do one of these instead:

- Re-read the relevant source files,
   documentation,
   logs,
   or command output.
- Run a concrete verification command and report the result.
- Launch an actually separate reviewer in a visible terminal or separate tool when the user asks for independent review.
- State plainly that no independent review was run.

Never satisfy a hook
or final response by writing a magic phrase such as
`Advisor pass: no blockers found`
unless a real independent reviewer produced that result.
