# Kimi limitations

Documenting failures during a `/grilling` session run by Kimi on 2026-07-10,
so future sessions do not repeat the same pattern.

## Pi goal extension grilling session (2026-07-10)

Goal:
Run a `/grilling` session on
[issue 360](https://github.com/Aquaticat/Monochromatic/issues/360),
which proposes replacing the installed `npm:@narumitw/pi-goal` extension with a
repository-owned Pi extension package at `package/pi-plugin/goal`.
The replacement should preserve useful `/goal` persistence and explicit
`goal_complete` behavior,
 but remove the upstream global stale-tool-call latch
that can deadlock an agent after an interruption.

Files and resources involved:

- `https://github.com/Aquaticat/Monochromatic/issues/360`
- `/var/home/user/.pi/agent/npm/node_modules/@narumitw/pi-goal/src/goal.ts`
- `/var/home/user/.pi/agent/npm/node_modules/@narumitw/pi-goal/README.md`
- `/home/user/.pi/agent/skills/grilling/SKILL.md`
- `doc/limitation/kimi.md` (this file)

### What was inspected before the session

Kimi did look up relevant facts before responding:

- Read the `grilling` skill,
   which instructs the agent to ask one question at a
time,
   wait for feedback,
   offer a recommended answer,
   and not enact the plan until
shared understanding is reached.
- Fetched issue 360 with `gh issue view 360`.
- Listed sibling Pi plugins under `package/pi-plugin/` to understand package
conventions.
- Read the Pi extension lifecycle docs at
`/var/home/user/Monochromatic/node_modules/.pnpm/@earendil-works+pi-coding-agent@0.80.6/node_modules/@earendil-works/pi-coding-agent/docs/extensions.md`.
- Read the upstream `@narumitw/pi-goal` source file `src/goal.ts` and its
`README.md`.

Despite that preparation,
 the first three questions all failed.

### User instructions for the session

After the first question,
 the user set these constraints for the rest of the
grilling session:

> Do not ask internal implementation questions.
>  Only ask design questions.
>  No
> question should be "What do you think".
>  Always present me with options to pick
> from.

### Question 1: per-session state isolation

What was asked:

> Do you want all mutable runtime goal state isolated in a per-session object
> created at `session_start` and destroyed at `session_shutdown`,
>  instead of
> module-level variables?

The question was accompanied by a recommendation,
 but it ended with
"What do you think?"
 and offered no selectable options.

Why it failed:

This is an internal implementation question,
 not a design question.
Whether state lives in a class instance,
 a closure,
 or module-level variables is
a code-structure decision,
 not a user-facing design choice.
The user had explicitly asked for design questions with options.

Correct behavior:
Open with a design-level question about user-facing behavior,
 not about
internal state layout.

### Question 2: command surface compatibility

What was asked:

A three-option question about whether the `/goal` command surface and
`goal_complete` tool should be:

- **A.**
   byte-for-byte compatible with upstream `0.12.0`;
- **B.**
   kept but with minor cleanups;
- **C.**
   redesigned entirely into a richer command set such as `/goal set`,
  `/goal list`,
   and separate pause/resume/clear tools.

Why it failed:

Option C was nonsensical for `/goal`.
`/goal` is a single active goal with start,
 edit,
 pause,
 resume,
 clear,
 and a
`goal_complete` tool.
It is intentionally not a multi-goal backlog or list manager.
The user responded:

> Your C's inner proposal doesn't make sense.
>  Do you understand how /goal really
> is used?
>  If not,
>  research it.

Kimi then re-read the upstream `README.md` and confirmed the single-active-goal
usage model,
 but the option should never have been offered.

Correct behavior:
Research the upstream README and source before offering design alternatives.
Ground every option in the actual usage model.
If no meaningful redesign axis exists,
 do not invent one.

### Question 3: pause semantics

What was asked:

A three-option question about what `/goal pause` should do to the current turn:

- **A.**
   hard pause:
   abort the current agent turn immediately;
- **B.**
   soft pause:
   let the current turn finish normally;
- **C.**
   conditional pause:
   abort only if Pi is still in the LLM generation
  phase,
   otherwise let tools finish.

Why it failed:

The question assumed that a Pi extension command could abort an arbitrary
in-flight agent turn.
The user corrected this:

> A /goal pause ,
>  even if implemented by our own extension,
>  it can't implement
> any behavior other than soft pause.
>  And /goal pause shouldn't suppress further
> executions anyway - it merely means the goal continuation policy pauses being
> enforced.

A Pi extension controls its own continuation policy,
 not the Pi runtime's turn
lifecycle.
The only meaningful design choice for pause is whether and how the extension
stops enqueuing its own follow-up messages.

Correct behavior:
Confirm what an extension can and cannot do before posing options.
When the question is constrained by the Pi extension API,
 state that constraint
up front and ask the design question within it.

### Termination of the session

After the third failure,
 the user ended the grilling session:

> So far,
>  three questions,
>  three fails.
>  Forget about grilling,
>  write a
> @doc/limitation/kimi.md detailing this session's failures instead.

This file is the result.

### Lessons

Do not use Kimi to run grilling sessions on Pi extension designs without first
verifying that the model understands:

- the actual command and tool semantics of the extension being replaced;
- the boundary between extension policy and Pi runtime behavior;
- the difference between design questions and implementation questions;
- the user's stated format constraints (options,
   no "What do you think").

If any of those are unclear,
 research the upstream source and the Pi extension
lifecycle docs before asking the first question.
