# Mpm shadow install takes a tool owner answers 2026 11 13

## Status

Moved on 2026-11-23 from the old handover,
verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 23597 to 23652 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

#### Issue 6300: `mpm-shadow install` takes a tool, with the owner's three answers; two reviews with defects go back to their authors (2026-11-13T18:39:31-05:00)

Not final and not approved:
written by the coordinating session from the issue,
the owner's answers and the reviews' own text.

##### Issue 6300

The owner sent the issue's address at about 18:35 and nothing else.
The issue is mpm-shadow's,
which the owner made this session's on 2026-11-13 at 15:26,
so it was taken as work to do.
It records the owner's direction:
no separate `add` verb;
the existing `install` (alias `i`) accepts a tool,
as `pnpm i -g <package>` does.

Three points the issue leaves open were asked through the question tool,
each with its options ranked:

- What the argument names
  (a request or an existing id,
  a request alone,
  an existing id alone):
  "Request or existing id (Recommended)".
  `mpm i -g cargo:fastmod` adds and installs a new tool from a request with a route prefix;
  `mpm i -g tokei` installs the entry the plan already holds under that id;
  a bare name that is neither is refused and never guessed;
  several tools in one command are allowed.
- Whether it works without `--global`
  (both scopes,
  or global alone):
  "Both scopes (Recommended)".
  Without `--global` it writes this repository's plan and lock,
  keeping every comment and the file's layout.
- Where the how-to lives
  (the READMEs,
  or a runbook):
  "READMEs (Recommended)".
  The command and the entry shapes in `package/mpm-shadow/cli/README.md`;
  the four `toolchain:*:global` tasks beside the local ones in the root README's section "Tool inventory and drift".

`install` with no argument keeps its meaning.
Left to the run to settle and report:
which routes a prefix can select,
how a new tool's binaries are found,
how the JSONC plan is written without losing a comment,
and what a kill between the plan's write and the lock's leaves.

The run is queued at the head:
`issue-6300-mpm-shadow-install-takes-a-tool-and-adds-it-to-the-plan`,
branch `feat/mpm-shadow-cli-install-takes-a-tool-and-adds-it-to-the-plan`,
brief `~/temp/agent/pi-runs/issue-6300-mpm-shadow-install-takes-a-tool-and-adds-it-to-the-plan-prompt.md`.
The issue carries a comment with the three answers and the label `ready-for-agent`.
It meets the cleanup run of the same package in what an install does.
