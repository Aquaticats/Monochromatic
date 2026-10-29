# @monochromatic-dev/git-executable-core

Resolve a native Git executable without re-entering the workspace Git policy wrapper.

The resolver promotes common platform Git locations only when `PATH` exposes them,
then scans remaining candidates in shell order.
It recognizes Windows `PATHEXT`,
rejects non-regular candidates without blocking on their contents,
inspects script candidates up to 64 KiB for `@monochromatic-dev/git-policy-cli` delegation,
rejects larger non-native candidates,
rejects a candidate that resolves to the native wrapper executable `cli-git-native`
(the installed `git` is a link to it),
and accepts any other ELF,
PE,
Mach-O,
and universal Mach-O executables from their headers.

Successful resolutions are cached by their effective absolute candidate sequence.
The cache retains 16 least-recently-used successful sequences.
Concurrent calls for the same sequence share one lookup.
Failed lookups are not cached,
so installing Git or changing candidate files allows a later call to retry.

Workspace consumers import TypeScript source:

```ts
import { resolveRealGit, } from '@monochromatic-dev/git-executable-core/ts';

const gitPath = await resolveRealGit();
```

Tests can inject `PATH`,
platform,
`PATHEXT`,
working directory,
and common paths through `ResolveRealGitOptions`.
Empty and relative `PATH` entries resolve against the supplied working directory.

## Development

Run package tasks through mise:

```console
mise run //package/git-executable/core:buildAndTest
mise run //package/git-executable/core:lint
mise run //package/git-executable/core:lint:types
```
