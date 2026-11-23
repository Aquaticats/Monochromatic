# Merges and publishes do not wait for reviews, mutation runs or a gate

## Status

The owner's standing rule,
said on 2026-11-04 for reviews and mutation runs,
and on 2026-11-23 for gates and passes before a merge;
recorded 2026-11-23T19:05:00-05:00.

## What the owner said

On 2026-11-23,
after the coordinating session reported that nothing had merged into `main` since about 05:00
because about eight branches waited on a container gate or a Sonnet pass:
"Merging to main shouldn't need a gate or a pass."

The earlier words are in the section "The owner's words of 2026-11-04 and before",
copied verbatim from lines 49,550 to 49,570 of the frozen old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md`),
where the sorter of that range found them held nowhere else.
The handover rewrite of 2026-11-23 left this rule out,
and the coordinating session then held merges on gates and passes it should not have.

## What follows

- A branch merges into `main` when its own round has ended and its report says what was done.
  No container gate,
  Sonnet pass,
  review or mutation run is a condition of the merge.
- A gate or a pass that is already running goes on as a check after the merge;
  what it finds becomes a fix round against `main`,
  not a verdict on the merge.
- Unchanged:
  the coordinating session still reads each round's report before merging it,
  and a merge whose branch is shown to answer worse than `main` waits for its fix round,
  because the owner's merge bar is that a branch may not make anything `main` does well worse.

## The owner's words of 2026-11-04 and before

- Standing authorization (2026-10-13):
  "bump and publish any updates of the native wrapper crate and its `forbidden-strings`
  dependency during this session,
  after the gate,
  a dry-run publish and a real-credential commit have passed."
  It was given for that session;
  releases since have gone through `crate-release` under it.
  This session (2026-11-04) the owner said:
  "I'm authorizing you to bump and publish any updates this session."
  It covers every crate,
  a crate's first publication included;
  the owner,
  2026-11-04,
  on an earlier coordinator line that required approval for a first publication:
  "Didn't I always give you approval to publish?"
  and "By "first" I meant "first in this session of any crate"",
  and "didn't I also say reviews and mutation testings don't block publishing?".
  Earlier (session transcript):
  "Mutation runs don't block publish and release."
  So reviews and mutation campaigns never block a merge or a publish;
  their findings become follow-up fixes and later releases.
