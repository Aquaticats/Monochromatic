# `@monochromatic-dev/fs-id-core`

Resolve a colon-free identity for the filesystem or volume containing a path.
The module prefers reboot-stable operating-system volume identifiers and degrades to an explicit runtime-only identity
when stable metadata is unavailable.

## Install

The package is workspace-private while cli-git integration is under development.
Workspace consumers declare:

```json
{
  "dependencies": {
    "@monochromatic-dev/fs-id-core": "workspace:*"
  }
}
```

## Use

```ts
import { resolveFsId, } from '@monochromatic-dev/fs-id-core';

const resolution = await resolveFsId({ path: process.cwd(), });
console.log(resolution.value,);
console.log(resolution.stable,);
console.log(resolution.source,);
```

A stable result uses one of:

- Linux filesystem UUID from unprivileged `findmnt`;
- macOS Volume UUID from structured `diskutil info -plist`;
- Windows volume serial from locale-invariant `Win32_LogicalDisk.VolumeSerialNumber` CIM output.

A preferred-command or preferred-output failure falls back to:

- Linux `f_fsid` from GNU `stat`;
- macOS device number from BSD `stat`;
- Windows nonzero runtime device number from Node `stat`.

Degraded results set `stable: false`,
include `reason`,
and emit a tagged warning.
A degraded value is used only when its platform mechanism returns a nonempty safe identity;
Windows zero device numbers are rejected because they cannot distinguish volumes.
The identity may change after reboot.
Unsupported platforms and total preferred-plus-fallback failure throw.

## Identity grammar

Generated values use a source-qualified grammar such as:

```text
fs-uuid_1bb3d23e-...
volume-uuid_1234abcd-...
volume-serial_1a2b-3c4d
f-fsid_a281dfd5d0534daf
device-number_2049
```

Values contain lowercase ASCII letters,
digits,
hyphen,
period,
and one source separator underscore.
They never contain colon,
slash,
backslash,
whitespace,
or traversal segments.
`isFsId` validates this generated shape.
Consumers still use reversible encoding when mapping a complete identity into filesystem path components.

## Fresh observation

`resolveFsId` canonicalizes and resolves identity on every call.
It intentionally does not memoize across calls:
a volume replaced at the same canonical path must not receive a stale identity from process memory.

Adapter factories,
platform parsers,
and command resolvers are available only from the explicit
`@monochromatic-dev/fs-id-core/testing` subpath for built fixture tests.
Ordinary consumers use the smaller package-root interface.

## Security scope

Filesystem identity prevents a trust record for one volume from matching another volume mounted at the same path.
It is not an authorization mechanism on its own.
Cli-git combines it with canonical config path,
exact byte snapshots,
registry permissions,
and explicit consent.

The module does not accept environment-selected commands,
platforms,
or identity values.
Windows command text contains only a validated drive letter.

## Verification

Package completion requires:

```text
mise run //package/fs-id/core:build
mise run //package/fs-id/core:lint
mise run //package/fs-id/core:lint:types
mise run //package/fs-id/core:test:unit
```

Cross-platform CI exercises preferred resolution on real Linux,
macOS,
and Windows runners and records the source and stability fields without treating the concrete machine identifier as a
fixture constant.
