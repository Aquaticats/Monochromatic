# Pi auto-mode 0.0.1 `/home` alias sends private scratch inspection to an unavailable reviewer

> Scratch-path note:
> `/tmp/agent` remains a compatibility path.
> Use `~/temp/agent` for current work.

## Symptom

Pi 0.82.1 with `@monochromatic-dev/pi-plugin-auto-mode` 0.0.1 can treat a read under
`~/temp/agent` as outside its private scratch allowlist when account home has two filesystem spellings.
The observed host returned `/home/user` as home,
while `realpath /home/user` returned `/var/home/user`.

A structured `read` of `/home/user/temp/agent/...` then reaches normal judge evaluation instead of
being accepted as a private scratch read.
Read-only Bash inspection can reach the same path when its complete shell shape is not positively proven.
Affected command families include:

- ripgrep over repositories under `~/temp/agent`;
- non-mutating `find ... -print` pipelines;
- output-only `sort`,
   `paste`,
   and `printf` stages;
- literal `for` loops running `git -C "$repo" tag --points-at HEAD`.

When every configured reviewer candidate returns neither a structured tool call nor finalized text,
the first exact diagnostic is:

```text
Structured reviewer direct JSON returned no text
```

After fallback candidates are exhausted,
the outer error starts with:

```text
Structured review unavailable after attempts by
```

These reviewer diagnostics are secondary.
The legitimate scratch read should not have reached the reviewer.

## Root cause

### Lexical home and canonical directory identity were compared at different boundaries

Before commit `b05363ad9`,
`package/pi-plugin/auto-mode/src/temp-allowlist.ts:128-134` derived the candidate directly from lexical home:

```ts
const candidateDirs = [
  join(
    home,
    'temp',
    'agent',
  ),
  historicalAgentTempDir,
];
```

The same pre-fix file at lines 75 to 79 rejected any candidate whose canonical path differed from its
lexical absolute path:

```ts
const stats = await lstat(dir,);
if (!stats.isDirectory())
  return false;
if ((await realpath(dir,)) !== resolve(dir,))
  return false;
```

For `/home/user/temp/agent`,
`realpath(dir)` was `/var/home/user/temp/agent`,
while `resolve(dir)` stayed `/home/user/temp/agent`.
The private directory therefore disappeared from the allowlist even though it had mode `0700`,
was owned by current UID,
and was not itself a symlink escape.

The earlier reading that reviewer transport caused the approval prompt was wrong.
Reviewer failure explains only the final error.
The mismatched home identity explains why the call entered reviewer evaluation.

### Canonicalizing account home fixes identity without trusting descendant symlinks

Current `package/pi-plugin/auto-mode/src/temp-allowlist.ts:114-130` canonicalizes account home,
with a fail-closed lexical fallback when metadata lookup fails:

```ts
async function canonicalHomePath(
  home: string,
): Promise<string> {
  try {
    return await realpath(home,);
  }
  catch (error) {
    const innerL = tagged({
      tag: canonicalHomePath.name,
      l: moduleLogger,
    },);
    innerL.debug(`home realpath failed for ${home}: ${String(error,)}`,);
    return resolve(home,);
  }
}
```

Current lines 158 to 173 derive scratch from canonical home:

```ts
const canonicalHome = await canonicalHomePath(home,);
const candidateDirs = [
  join(
    canonicalHome,
    'temp',
    'agent',
  ),
  historicalAgentTempDir,
];
```

`isTrustedAgentTempDir()` still applies the strict check at current lines 74 to 84.
That preserves directory type,
canonical identity,
current UID ownership,
and absence of group or other permission bits.
A symlink at `temp`,
`agent`,
or a command target below scratch still fails canonical containment.

### Whole-shell proof must establish command shape and runtime word provenance

A command-name allowlist is insufficient because supported tools also have mutating or executable forms.
Current `package/pi-plugin/auto-mode/src/read-only-bash-proof.ts:57-91` rejects incomplete parsing,
background execution,
command or process substitution,
function definitions,
unknown command shapes,
and any word whose path scope is not proven:

```ts
if (!analysis.parsed)
  return false;
if (analysis.hasBackground)
  return false;
if (analysis.hasCommandSubstitution)
  return false;
if (analysis.hasProcessSubstitution)
  return false;
// ...
for (const command of executedCommands) {
  if (!commandIsReadOnly(command,))
    return false;
  commandScopePromises[commandScopePromises.length] = commandWordsStayInReadScope({
    command,
    ctx,
    trustedAgentTempDirs,
  },);
}
```

The shared analyzer now preserves each parsed argument value with its original shell spelling at
`package/agent-harness-shared/shell-command-analyzer/src/types.ts:130-164`.
It also preserves literal loop source spellings at lines 10 to 28.
That distinction matters because quoted `'*/*.ts'` is a literal predicate pattern,
while unquoted `/repo/*` expands to a runtime path set.

Current `package/pi-plugin/auto-mode/src/read-only-bash-word-proof.ts:245-271` accepts only literal words
or exact quoted references to proven lexical loop values:

```ts
if (references.length === 0) {
  if (value.includes('$',) || value.includes('`',))
    return UNPROVEN_WORD;
  if (value.startsWith('~',))
    return UNPROVEN_WORD;
  if (sourceHasUnsafeExpansion(sourceText,))
    return UNPROVEN_WORD;
  return [value,];
}
// ...
if ((sourceText !== `"$${name}"`)
  && (sourceText !== `"\${${name}}"`)) {
  return UNPROVEN_WORD;
}
```

Unquoted pathname,
brace,
tilde,
and extended-glob syntax therefore retains normal judge handling.
Every path candidate still passes secret-path checks and canonical cwd or trusted-scratch containment.

### Empty reviewer output produces the secondary diagnostics

`package/pi-shared/model-review/src/stream-collection.ts:55-71` defines the direct empty-output error:

```ts
class EmptyStructuredReviewTextError extends Error {
  constructor() {
    super('Structured reviewer direct JSON returned no text',);
    this.name = 'EmptyStructuredReviewTextError';
  }
}
```

After candidate exhaustion,
`package/pi-shared/model-review/src/fallback.ts:84-103` combines attempted identities and diagnostics:

```ts
super(
  `Structured review unavailable after attempts by ${joinText({
    values: attemptedCandidateIdentities,
    separator: ', ',
  },)}: ${joinText({
    values: diagnostics,
    separator: '; ',
  },)}`,
  ...(cause === undefined ? [] : [{ cause, },]),
);
```

Fixing scratch classification removes this reviewer dependency for proven reads.
It does not weaken fallback behavior for calls that still require review.

## Verification

### Versions and source identities

- Pi host:
   `@earendil-works/pi-coding-agent` 0.82.1.
- Auto-mode package:
   `@monochromatic-dev/pi-plugin-auto-mode` 0.0.1.
- Shell parser:
   `unbash` 4.0.3.
- Canonical-home fix:
   commit `b05363ad9`.
- Initial whole-shell proof:
   commit `0a7f913d1`.
- Source-spelling provenance:
   commit `2cead0e31`.
- Expansion hardening:
   commit `333865aad`.

### Filesystem probe

Run with the same account that starts Pi:

```bash
printf 'home=%s\n' "$HOME"
realpath -- "$HOME"
realpath -- "$HOME/temp/agent"
stat --format='mode=%a uid=%u type=%F' -- "$HOME/temp/agent"
```

The diagnosed host produced two spellings for account home,
then one canonical scratch path.
Scratch mode was `700`,
UID was `1000`,
and type was `directory`.

### Build-backed regression harness

Run from repository root:

```bash
mise run buildAndTest -- \
  package/agent-harness-shared/shell-command-analyzer/src/index.unit.test.ts
mise run buildAndTest -- \
  package/pi-plugin/auto-mode/src/index.unit.test.ts
mise run //package/agent-harness-shared/shell-command-analyzer:lint:types
mise run //package/pi-plugin/auto-mode:lint:types
```

The analyzer test passed after building its shipped artifact.
The auto-mode entry-point test passed after building the extension artifact.
Both type-lint tasks passed.

### Forms accepted without reviewer evaluation

The entry-point harness verifies these classes:

```bash
rg --line-number pattern ~/temp/agent/repo/src
find ~/temp/agent/repo -type f -print | sort
find ~/temp/agent/repo -name '*.test.ts' -print | sort
for repo in ~/temp/agent/one ~/temp/agent/two; do
  printf '%s\t' "$repo"
  git -C "$repo" tag --points-at HEAD | paste --serial --delimiters=, -
done
```

The home-alias fixture also verifies that a structured read through lexical account home resolves to
canonical private scratch and does not enter approval reuse or judge evaluation.

### Forms retained on normal judge path

The entry-point harness verifies these classes remain flagged:

```bash
touch ~/temp/agent/target.txt
find ~/temp/agent -type f -delete
find ~/temp/agent -type f -exec touch {} \;
find -L ~/temp/agent -type f -print
find ~/temp/agent -follow -type f -print
rg --pre cat pattern ~/temp/agent
rg --ignore-file /outside/ignore pattern ~/temp/agent
rg pattern ~/temp/agent/*
git -C ~/temp/agent tag release-candidate
rg pattern ~/temp/agent > ~/temp/agent/target.txt
sort --output=~/temp/agent/target.txt
sort --temporary-directory=~/temp/agent
printf -v result '%s' fixture
MODE=fixture rg pattern ~/temp/agent
```

The same catalog covers an unquoted loop reference,
a brace-expanded loop list,
an outside path,
a secret-looking path,
and a scratch symlink resolving outside its trusted root.

## Verified workarounds

### Rebuild and restart with canonical-home and read-proof fixes

Build dependency order explicitly:

```bash
mise run //package/agent-harness-shared/shell-command-analyzer:build
mise run //package/pi-plugin/auto-mode:build
```

Start a new Pi process so it loads the rebuilt extension artifact.
The tradeoff is that an existing Pi process retains its already-loaded module instance.
The build-backed entry test verifies the new artifact before restart.

### Use normal approval for forms outside the positive proof

Commands with unproven expansion or mutation-capable options intentionally remain on normal judge or
user-approval handling.
The tradeoff is an approval round trip.
This is preferable
to broadening static trust because unknown runtime path sets cannot be canonically checked in advance.

No safe configuration-only workaround was found for the pre-fix home-identity bug.
Changing package-wide agent guidance does not alter extension path classification.

## What does not work

- Removing `realpath(dir) === resolve(dir)` from scratch trust allows a symlink below account home to
  masquerade as private scratch.
- Adding package policy to repository-wide `AGENTS.md` exposes implementation details to every agent
  and does not change extension execution.
- Relying on reviewer fallback treats the secondary empty-output error as the cause and leaves valid reads
  dependent on reviewer availability.
- Allowing tools by command name alone accepts mutating `find`,
  executable ripgrep preprocessing,
  Git tag creation,
  writable redirects,
  and named-output options.
- Preserving only parsed word values loses whether wildcard syntax was quoted.
  It cannot distinguish a literal `find` predicate from runtime pathname expansion.
- Trusting literal loop variable names without their finite source values allows unbound or expansion-derived
  paths to escape proof.

## Upstream filing artifact

### Duplicate search

Repository issue searches for `auto-mode temp agent home alias`,
`Structured reviewer direct JSON returned no text`,
`auto-mode`,
and `temp agent` found no matching report.
Issue `#279` concerns maintainer-local skill allowlist test paths and is not this failure.

### Upstream filing decision

1. **Is it really upstream's fault?
   ** No.
   The false classification was in this repository's auto-mode extension.
   Pi surfaced the extension result as designed.
2. **Can upstream fix it?
   ** Pi upstream cannot correct repository-local scratch policy.
   This repository can and did fix it.
3. **Are they supporting this use case?
   ** Pi supports extension `tool_call` handlers,
   but canonical scratch policy belongs to this extension.
4. **Would the repo welcome our contribution?
   ** The relevant repository is already the repository being changed.
   No external contribution policy is involved.
5. **Will they likely fix it?
   ** Not applicable to Pi upstream because no upstream change is requested.
6. **Have we prototyped a minimal fix compatible with their architecture?
   ** Yes for the owning repository.
   Commits `b05363ad9`,
   `2cead0e31`,
   and `333865aad` implement and test the fix at existing policy and analyzer seams.

Nothing should be filed against Pi upstream.
There is no external issue draft or additive comment to send.
