# pnpm 11 and 12 documentation says `--no-bail` lets all tasks finish, but dependents of a failed task are skipped

The command reference for `pnpm recursive` says that with `--no-bail` "all tasks will finish".
On pnpm 11.28.9,
12.14.0 and 12.16.1 a task that depends on a failed task does not run.
pnpm 10.34.9 does run it,
so the sentence is correct there.

## Symptom

The sentence in question,
at the v12 source ([old sentence][old-sentence]):

```md
<!-- pnpm/docs/cli/recursive.md:69-71 at pnpm/pnpm commit cf2556e97c -->
This config does not affect the exit code.
Even if `--no-bail` is used, all tasks will finish but if any of the tasks fail,
the command will exit with a non-zero code.
```

The orchestration page in the same documentation tree says the opposite
([orchestration section][orchestration-section]):
after a task fails,
tasks that depend on it are skipped,
and with `--no-bail` independent ready tasks continue.

## Root cause

The two pages disagree,
and the command reference is the one that is wrong for pnpm 11 and 12.
The scheduler source was not read for this document.
The behavior is established by the runs under Verification,
and by the orchestration page.

Where the text must be edited matters.
The pnpm.io repository keeps generated publication copies of the v12 and v11 pages,
and its README says not to edit them
([maintenance rule][maintenance-rule]).
The v12 source is `pnpm/docs/cli/recursive.md` and the v11 source is `pnpm11/docs/cli/recursive.md`,
both in `pnpm/pnpm`.
Only the v10 page is edited in pnpm.io,
and it is correct.

## Verification

Reproduction:
<https://github.com/Aquaticat/pnpm-no-bail-dependents-repro>,
workflow `.github/workflows/no-bail.yml`.
`lib` fails,
`app` depends on `lib` through `workspace:*`,
`free` is independent.
Each `test` script writes a marker file,
and the job runs `pnpm -r --no-bail run test` on a clean `ubuntu-24.04` runner.
Each pnpm release is downloaded from its GitHub release and checked against a pinned SHA-256.

Run 40912660582 on the commit that added the workflow:

- 12.16.1,
  12.14.0 and 11.28.9:
  `lib: ran`,
  `app: did not run`,
  `free: ran`,
  and the command exits with code 1.
- 10.34.9:
  `lib: ran`,
  `app: ran`,
  `free: ran`,
  and the command still exits with code 1.

## Verified workarounds

None needed.
This is a documentation error,
not a runtime failure.

## What does not work

Editing the 10.x page,
which the issue offered as an option,
would make a correct page false.
Editing `docs/` or `versioned_docs/version-11.x/` in pnpm.io changes generated copies that the next sync overwrites.

Pushing the fork branch from a shallow,
blobless clone stalled for more than ten minutes with no output.
The commit was created on the fork through GitHub's `createCommitOnBranch` mutation instead.

## Upstream filing decision

The pnpm.io issue already exists,
so no new issue was drafted:
<https://github.com/pnpm/pnpm.io/issues/987>.
No duplicate was found among open and closed pull requests that mention `no-bail`.
pnpm's contribution guide allows agent-assisted pull requests that disclose the agent and model.
The vale prose check reports no errors on the edited files,
and the three warnings it prints are on lines the change does not touch.
The website build and the new `#--no-bail` anchor were not checked locally.

Filed with the poster's approval:

- Pull request <https://github.com/pnpm/pnpm/pull/17484> changes the v12 and v11 sources,
  3 lines each.
- A comment on the issue corrects its scope (10.x is unaffected) and points to the pull request:
  <https://github.com/pnpm/pnpm.io/issues/987#issuecomment-6502752064>.

Outcome of the pull request is not yet known.

[old-sentence]: https://github.com/pnpm/pnpm/blob/cf2556e97c3b344e63e9504cf2da3d96426f1748/pnpm/docs/cli/recursive.md?plain=1#L69-L71
[orchestration-section]: https://github.com/pnpm/pnpm/blob/cf2556e97c3b344e63e9504cf2da3d96426f1748/pnpm/docs/workspace-task-orchestration.md?plain=1#L228-L230
[maintenance-rule]: https://github.com/pnpm/pnpm.io/blob/7aded9fe30ceb31d4302b4189fcac84c958582b0/README.md?plain=1#L6-L10
