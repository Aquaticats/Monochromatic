# Cli-git require-root refuses a tool's git question from its own package folder

## Symptom

A program that runs from a folder inside this repository asks Git for the repository root and gets a refusal
instead of the root:

```text
CommandAnswerMissingError: git rev-parse --show-toplevel answered 1:
{"schemaVersion":1,"sequence":0,"type":"finding","trigger":"pre-forward","policyId":"require-root","severity":"error","code":"require-root/not-at-root","message":"monochromatic-git-policy-cli: not at the root of the git repository. Repo root is /var/mnt/encrypted/repository/Monochromatic but effective cwd is /var/mnt/encrypted/repository/Monochromatic/package/pi-shared/subagent-driver. ...","fix":"none"}
```

On 2026-11-11 every one of ten starts of a Pi batch ended with exit status 1 and no worktree or run folder.
The batch printed only the last line of the failing start's output,
which is Node's `Node.js v26.13.0`,
so the refusal above stayed hidden until it was replayed by hand.

## Root cause

The `require-root` policy of `monochromatic-git-policy-cli` refuses every Git command whose working folder is not the top of the
repository,
and it warns about that in its own finding (`code: require-root/not-at-root`, "Tip: cd to ... or pass `-C ...` before the
subcommand").

`mise` runs a task in the folder its own `mise.toml` lives in,
so a package task's working folder is `package/<group>/<name>`,
and every child it starts inherits that folder.
A tool that asks `git rev-parse --show-toplevel` there cannot learn the root,
because the command that would tell it the root is the one the policy refuses.

This is a closed loop, not a misconfiguration: the policy exists so that people and agents notice where they are,
and a tool cannot satisfy it before it knows the answer the policy wants.

## Fix

Find the root without asking Git,
by walking up from the working folder to the nearest folder holding `.git`,
which is a folder in the main checkout and a file in a linked worktree.
`package/pi-shared/subagent-driver/src/checkout-of-a-command.ts` does this,
and `--checkout-folder` still names a checkout outright.

After the root is known,
every Git command that matters is run with that root as its working folder,
which the policy allows.
The commands that do this are `git worktree add`, `git worktree remove` and `git status`,
all through `package/pi-shared/subagent-driver/src/git-command.ts`.

A tool that must ask Git from a folder it cannot choose can pass `-C <root>`,
which the policy's own tip names,
but that only works once the root is known.

## What hid it

A batch printed the last line of what each start printed,
and Node ends an uncaught error with its version,
so ten different failures read as ten identical version strings.
The package now prints the error's own message and appends the whole of what the start printed to
`<runs-folder>/<name>-start-failure.log`,
which the printed line names.
