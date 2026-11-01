# `patch-invalid` hides the finding that caused it

## Symptom

A `git commit` through `git-policy-cli` fails with only this on stderr,
and exit code 2:

```json
{"schemaVersion":1,"sequence":0,"type":"engine-failure","code":"patch-invalid",
 "message":"Patch must contain exactly declared ordinary text path doc/handover/translation-repair.md",
 "trigger":"pre-forward","path":"doc/handover/translation-repair.md"}
```

The message points at the patch,
so the natural next step is to inspect the patch,
and the patch is fine.
Checking it against everything `commitTransactionPatch` actually rejects
(`package/git-policy/cli/src/policy-engine/commit-transaction-patch.ts`)
finds exactly one `diff --git` line, one `---`, one `+++`, one `index`,
and none of the `FORBIDDEN_PATCH_PREFIXES`.
Retrying reproduces it exactly.

## Cause

The patch under validation is not the one the working tree holds.
It is a patch the policy engine generated to APPLY A FIX for a separate finding,
and that finding is what needs addressing.
The `patch-invalid` failure is downstream of it.

In the observed case the real finding was:

```json
{"type":"finding","policyId":"final-newline","severity":"warn",
 "code":"final-newline/noncanonical-final-newline",
 "message":"Non-empty text file must end with exactly one LF byte."}
```

The file ended with two LF bytes rather than one.

## Why the real finding is invisible at first

It is only printed once the pathspec is staged.
`git commit -- <pathspec>` on an unstaged file reports the engine failure alone;
`git add -- <pathspec>` followed by the same commit prints the finding first
and then the same engine failure.

## Fix

Stage the pathspec explicitly, read the finding that now appears, and fix that.
For the final-newline case, trim the file to exactly one trailing LF.

The usual trigger is appending to a document with a template literal
that ends in a blank line, so the write leaves `\n\n` at the end of the file:

```ts
// Leaves two trailing LF bytes, and the commit then fails as patch-invalid.
await appendFile(path, `\n## New section\n\nBody text.\n\n`,);

// Writes exactly one.
await writeFile(path, `${text.replace(/\n+$/u, '',)}\n${section.replace(/\n+$/u, '',)}\n`,);
```

## What not to do

Do not reach for `--no-enforce-*`.
`CLG` reserves those for a change no scoped pathspec fits,
and this is not that: the guard is correct and the file is malformed.
