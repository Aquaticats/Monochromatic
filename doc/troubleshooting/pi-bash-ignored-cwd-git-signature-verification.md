# Pi 0.80.6 Bash ignores per-call cwd, contaminating Git identity and making SSH-signed commits unverified

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

## Symptom

Commits created in `Aquaticat/Monochromatic` after 2026-07-09 17:48:21 EDT appear unverified on GitHub.
The first affected commit is `c16e52451097299b02ca1e7b62d4a2ce5f2a90da`.
A protected-branch push reported:

```text
Commits must have verified signatures.
Found 1 violation:
8fd72562c7f03f6cb1c73e9771a0f2bfa7b939e0
```

This is not a signing failure.
The initial diagnostic scan found 69 unique commits reachable through the pre-diagnosis repository refs or reflogs
whose committer timestamps fell between local midnight and the 20:34:34 EDT snapshot.
Every one contains an SSH `gpgsig` header and verifies cryptographically against the configured public key.
GitHub still cannot associate the affected commits' committer email with a user.

The GitHub API shows the verification boundary:

```json
{
  "sha": "4d1d66a0e2440f4402ea1eb454db6e49aacfbc48",
  "committer_email": "an@aquati.cat",
  "verified": true,
  "reason": "valid"
}
{
  "sha": "c16e52451097299b02ca1e7b62d4a2ce5f2a90da",
  "committer_email": "fixture@example.invalid",
  "verified": false,
  "reason": "no_user"
}
```

The repository currently overrides the global identity with these local values:

```text
local  file:.git/config  user.email fixture@example.invalid
local  file:.git/config  user.name Final newline fixture
```

A separate local symptom can be misleading.
`git log` signature placeholders and `git verify-commit` emit this error when SSH trust policy is not configured:

```text
error: gpg.ssh.allowedSignersFile needs to be configured and exist for ssh signature verification
```

That error affects local verification only.
It neither disables commit signing nor explains GitHub's `no_user` result.

## Root cause

### A fixture command ran in the main repository

Pi session `019f4890-fbdd-7998-bf0d-e090acb9ad30` recorded the triggering Bash call at
2026-07-09 21:48:15.338Z,
 or 17:48:15 EDT.
The call intended to use a disposable fixture directory:

```json
{
  "command": "git init\ngit config user.email fixture@example.invalid\n...",
  "cwd": "/tmp/agent/final-newline-hook-fixture-oCurQRYW",
  "timeout": 180
}
```

The command did not contain `cd` or a command-native `-C` option.
The `cwd` field was unsupported and had no effect,
 so the shell stayed in `/var/home/user/Monochromatic`.
The command's first output line disclosed the wrong target immediately:

```text
Reinitialized existing Git repository in /var/home/user/Monochromatic/.git/
```

The unscoped `git config` commands therefore wrote repository-local values.
Git's current `git-config` documentation states that writes target the repository-local configuration by default.
The `.git/config` modification timestamp is `2026-07-09 17:48:20.651256678 -0400`,
 one second before the first
fixture-identity commit.

The same accidental command created commit `c16e5245` in the main repository.
The next commit,
 `8fd72562`,
 reverted the fixture files,
 but Git configuration is not part of a commit tree.
Reverting tracked files could not revert `.git/config`,
 so the fixture identity remained active for later commits.

### Pi validates and then drops the unknown field

The source trace uses `earendil-works/pi` tag `v0.80.6`,
 commit
`2b3fda9921b5590f285165287bd442a25817f17b`.
The clone was created under a private `/tmp/agent/` directory,
 and its origin,
 tag,
 and commit were verified before
inspection.

The built-in Bash schema has only `command` and `timeout`.
`package/coding-agent/src/core/tools/bash.ts:40-43` contains:

```ts
const bashSchema = Type.Object({
	command: Type.String({ description: "Bash command to execute" }),
	timeout: Type.Optional(Type.Number({ description: "Timeout in seconds (optional, no default timeout)" })),
});
```

This object schema does not set `additionalProperties: false`.
Pi's general validator clones and converts arguments,
 then returns them whenever the schema validator accepts them.
`package/ai/src/utils/validation.ts:278-299` contains:

```ts
export function validateToolArguments(tool: Tool, toolCall: ToolCall): any {
	const args = structuredClone(toolCall.arguments);
	Value.Convert(tool.parameters, args);

	const validator = getValidator(tool.parameters);
	// ... non-TypeBox coercion path omitted

	if (validator.Check(args)) {
		return args;
	}
```

A direct v0.80.6 probe confirmed that validation preserves the unsupported field:

```json
{
  "schemaKeys": ["command", "timeout"],
  "schemaAdditionalProperties": "unspecified",
  "validatedKeys": ["command", "cwd"],
  "validatedCwd": "/intended-target"
}
```

The Bash executor then destructures only the supported fields and supplies the factory-time session directory to the
spawn context.
`package/coding-agent/src/core/tools/bash.ts:304-312` contains:

```ts
async execute(
	_toolCallId,
	{ command, timeout }: { command: string; timeout?: number },
	signal?: AbortSignal,
	onUpdate?,
	_ctx?,
) {
	const resolvedCommand = commandPrefix ? `${commandPrefix}\n${command}` : command;
	const spawnContext = resolveSpawnContext(resolvedCommand, cwd, spawnHook);
```

The unsupported `cwd` survives validation but disappears during parameter destructuring.
The `cwd` passed to `resolveSpawnContext` is the session directory captured when the Bash tool was created.

A disposable two-directory harness measured the result:

```json
{
  "intendedCwd": "/tmp/agent/pi-bash-intended-root-iYEs84St",
  "actualCwd": "/tmp/agent/pi-bash-session-root-1jDVuW9A",
  "unknownCwdSurvivedValidation": true
}
```

### The identity change breaks GitHub attribution, not cryptographic signing

The global signing configuration remained active:

```text
global  file:/home/user/.gitconfig  user.signingkey /home/user/.ssh/github_sign.pub
global  file:/home/user/.gitconfig  gpg.format ssh
global  file:/home/user/.gitconfig  commit.gpgsign true
```

The global configuration file was last modified on 2026-07-05.
The public signing key file was last modified on 2026-05-20.
Neither changed at the 2026-07-09 failure boundary.

GitHub's REST documentation defines verification reason `no_user` as:

```text
No user was associated with the `committer` email address in the commit.
```

That exactly matches `fixture@example.invalid`.
Git still used the configured SSH key and added a cryptographically valid `gpgsig` object header.
GitHub could not associate the commit's committer email with any GitHub user.
The immediately preceding `an@aquati.cat` commit verifies on GitHub as `valid`.

## Verification

### Versions and artifacts

- Pi coding agent:
   `0.80.6`,
   tag `v0.80.6`,
   commit `2b3fda9921b5590f285165287bd442a25817f17b`.
- Git:
   `2.54.0`.
- OpenSSH:
   `10.2p1`.
- Repository-local configuration modification:
   2026-07-09 17:48:20 EDT.
- First affected commit:
   `c16e52451097299b02ca1e7b62d4a2ce5f2a90da` at 17:48:21 EDT.
- Affected commits currently reachable through refs or reflogs:
   36.
- Initial pre-diagnosis snapshot at main `797061b59`:
   69 cryptographically verified,
   0 verification failures.

### Signature-presence harness

This checks the commit object directly and does not require an allowed-signers file:

```bash
present=0
absent=0
while read -r hash; do
  if git cat-file commit "$hash" | grep --quiet '^gpgsig '; then
    present=$((present + 1))
  else
    absent=$((absent + 1))
  fi
done < <(git log --all --reflog --since=midnight --format='%H' | sort --unique)
printf 'signature_present=%s\nsignature_absent=%s\n' "$present" "$absent"
```

Observed output before this documentation branch added further signed commits:

```text
signature_present=69
signature_absent=0
```

The absolute present count increases as new signed commits are created;
the invariant under test is zero absent signatures.

### Cryptographic-verification harness

A disposable allowed-signers file associated both observed commit principals with the configured public key.
The harness verified the same frozen pre-diagnosis set:

```bash
allowed="$(mktemp /tmp/agent/git-signing-allowed-signers-XXXXXXXX)"
awk '{
  print "an@aquati.cat " $1 " " $2
  print "fixture@example.invalid " $1 " " $2
}' /home/user/.ssh/github_sign.pub > "$allowed"

while read -r hash; do
  git -c gpg.ssh.allowedSignersFile="$allowed" verify-commit "$hash"
done < <(
  git log --all --reflog \
    --since='2026-07-09T00:00:00-04:00' \
    --until='2026-07-09T20:34:34-04:00' \
    --format='%H' |
    sort --unique
)
rm --force -- "$allowed"
```

Observed aggregate result:

```text
checked=69
cryptographic_verification_passed=69
cryptographic_verification_failed=0
allowed_signers_removed=true
```

The configured public key cryptographically verifies every signature in the frozen sample.
GitHub's `no_user` outcome is therefore an identity-association failure rather than an invalid-signature result.

### Automatic-signing harness

A disposable repository inherited the current global Git configuration.
Its first commit used the global identity,
 and its second commit used the contaminated local identity.
Both contained SSH signatures:

```text
author=Aquaticat <an@aquati.cat>
raw_signature=present

author=Final newline fixture <fixture@example.invalid>
raw_signature=present
```

This falsifies both an automatic-signing disablement and an SSH-agent signing failure.
With `commit.gpgSign=true`,
 the installed Git and SSH backend still sign successfully.

### Local-verification harness

The global-identity commit failed local verification without trust configuration:

```text
error: gpg.ssh.allowedSignersFile needs to be configured and exist for ssh signature verification
exit_status=1
```

The same commit passed when a disposable allowed-signers file associated the configured public key with
`an@aquati.cat`:

```text
Good "git" signature for an@aquati.cat with ED25519 key
```

### Patterns that work cleanly

- A Bash command that begins with `cd -- <absolute-path> &&` runs in that directory.
- A command using a native directory option,
   such as `git -C <path>`,
   targets that path.
- A commit with the global `Aquaticat <an@aquati.cat>` identity contains a signature and verifies on GitHub.
- `git verify-commit` validates an SSH signature when `gpg.ssh.allowedSignersFile` points to a matching trust file.
- Unsetting the repository-local fixture identity restores the global identity in a disposable repository and leaves
  automatic signing active.

### Patterns that fail

- Supplying `cwd` in Pi 0.80.6's built-in Bash tool arguments does not change the command directory.
- Running unscoped `git config user.name` or `git config user.email` from the wrong repository writes its local config.
- A commit whose committer email is `fixture@example.invalid` receives GitHub verification reason `no_user` even though
  the commit contains an SSH signature.
- Local SSH verification without an allowed-signers file fails before it can establish signer trust.

## Verified workarounds

### Restore the global identity for future commits

Record all local values first,
 then remove only the accidental repository-local overrides:

```bash
git config get --local --all user.name
git config get --local --all user.email
git config unset --local --all user.name
git config unset --local --all user.email
git config get --show-origin --show-scope --all --regexp '^user\.(name|email)$'
```

A disposable repository with duplicate local values verified that `unset --local --all` removes every local identity
entry,
 restores `Aquaticat <an@aquati.cat>` from global configuration,
 and leaves automatic signing active.

Tradeoff:
this repairs future commits only.
Existing commit objects retain their recorded names,
 emails,
 hashes,
 and GitHub verification records.

### Pin every mutating Bash command inside its command text

Use an explicit directory transition plus a target assertion:

```bash
target=/absolute/fixture/path
cd -- "$target" &&
if test "$(pwd -P)" != "$target" || test "$(git rev-parse --show-toplevel)" != "$target"; then
  printf '%s\n' 'wrong target' >&2
  exit 1
fi
# Mutating commands follow only after both checks pass.
```

For Git-only operations,
 prefer the command-native form:

```bash
git -C /absolute/fixture/path status --short
```

Tradeoff:
callers must include the guard in every mutating shell body.
Pi's built-in Bash call still has one session-level directory,
 so this is command composition rather than a per-call
API field.

This repository added the same rule to `AGENTS.md` immediately after the incident in commit `b3ffcfb26`.

### Configure local SSH verification separately

Create an allowed-signers file containing a trusted principal and public key,
 then point
`gpg.ssh.allowedSignersFile` at it.
A disposable file verified the current key with:

```bash
git -c gpg.ssh.allowedSignersFile=/path/to/allowed_signers verify-commit HEAD
```

Tradeoff:
this establishes local trust and fixes local status output only.
It does not change GitHub's identity association or repair the 36 existing `no_user` commits.
The trust file must be updated when principals or signing keys rotate.

### Treat historical repair as a coordinated history rewrite

Changing the identity in an existing commit changes the commit object and every descendant hash.
Repairing the 36 affected commits therefore requires rewriting,
 re-signing,
 and force-updating already-pushed history.

Tradeoff:
this disrupts branches,
 tags,
 open pull requests,
 open worktrees,
 forks,
 links,
 and downstream clones based on the existing hashes.
A safe rewrite needs backup refs,
 branch-protection and force-push authorization,
 collaborator coordination,
 and verification that every rewritten commit was re-signed.
Do not perform it as an incidental signing-config fix.
It requires explicit authorization and coordination.

## What does not work

- **Rotating or replacing the SSH signing key.
  **
  The configured key cryptographically verifies all 69 commits in the frozen sample.
  Key rotation does not repair the fixture committer email.
- **Setting `commit.gpgSign` again.
  **
  The effective value is already `true`,
   and all 69 sampled commits are signed.
- **Adding only `gpg.ssh.allowedSignersFile`.
  **
  This fixes local trust evaluation,
   not GitHub's `no_user` attribution.
- **Reverting the fixture commit.
  **
  Git commits track trees and metadata,
   not `.git/config`.
  Commit `8fd72562` reverted files but left the local identity active.
- **Pushing the same commits again.
  **
  Commit objects are immutable,
   and GitHub's persistent verification record is tied to each commit object.
- **Relying on the unsupported `cwd` argument.
  **
  Pi 0.80.6 retains it during validation and drops it during Bash execution.

## Upstream filing decision

No matching exemption exists under this repository's `.out-of-scope/` directory.
Searches covered open and closed issues and pull requests for `bash cwd`,
 `additionalProperties`,
 and unknown tool
arguments.
The exact duplicate is
[earendil-works/pi#5904](https://github.com/earendil-works/pi/issues/5904),
`bash tool: cwd parameter is silently dropped`.
The thread and all comments were read.

The six filing constraints resolve as follows:

1.  **Is it really upstream's fault?
    ** No as a feature request,
     and only partly as a diagnostic-quality issue.
    The agent supplied a field absent from the advertised Bash schema.
    Pi's permissive schema and destructuring make that mistake silent,
     but a maintainer states in #5904 that `cwd` is
    intentionally not a valid Bash parameter.
2.  **Can upstream fix it?
    ** Yes technically.
    Pi could reject unknown top-level fields,
     strip them with an explicit warning,
     or add a per-call directory field.
    Technical possibility does not override the maintainers' intentional API boundary.
3.  **Are they supporting this use case?
    ** No.
    The maintainer explicitly rejected per-call `cwd` as a valid parameter.
    The original reporter withdrew the request.
4.  **Would the repo welcome our contribution?
    ** No for this generated artifact.
    `CONTRIBUTING.md` requires issue text in the contributor's own voice and says not to use an LLM to generate it.
    Recent maintainer responses also reject AI-generated tracker comments.
5.  **Will they likely fix it?
    ** No based on direct signal.
    Issue #5904 is closed as `not planned`,
     received the `no-action` label,
     and records the API choice as intentional.
6.  **Have we prototyped a minimal fix compatible with their architecture?
    ** No.
    The automatic prototype gate does not trigger because constraints 1,
     3,
     4,
     and 5 fail.
    A local runtime probe is sufficient to verify the existing behavior;
     changing a third-party clone would not produce a
    welcome contribution.

Nothing should be posted upstream.
The duplicate already documents the same schema and execution mismatch.
This incident adds a downstream consequence,
 Git identity contamination,
 but it does not change the maintainers'
intentional API decision,
 and an AI-generated additive comment would violate their contribution policy.
