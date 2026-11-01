# macOS 26 `diskutil info -plist` rejects ordinary file subpaths instead of resolving their volume

## Symptom

A macOS GitHub runner executing:

```text
diskutil info -plist /Users/runner/work/Monochromatic/Monochromatic
```

exited `1` and returned an error plist:

```xml
<key>ErrorMessage</key>
<string>Could not find disk: /Users/runner/work/Monochromatic/Monochromatic</string>
```

The failure is recorded in filesystem-identity workflow run
[29064478741][failed-run].
Linux and Windows passed the same package probe;
only the macOS path-to-volume lookup failed.

## Root cause

The earlier implementation handed an arbitrary existing repository subdirectory directly to `diskutil info`.
The macOS manual defines the command as:

```text
diskutil info [-plist] device | -all
```

Its device forms include disk identifiers,
device nodes,
UUIDs,
volume names,
and volume mount points.
An ordinary descendant path is not promised as a device.
The runner path was a descendant of its mounted volume,
not the volume mount point itself.

Apple's `diskutil` implementation source is not published with a public contribution repository,
so there is no source call chain to clone and quote.
The primary inspectable contract is the `diskutil(8)` manual's `info` grammar and `DEVICES` section
([archived manual][diskutil-man]).

The package now performs an explicit path-to-device step in
`package/fs-id/core/src/platform-resolvers.ts:274-301`:

```ts
const mountOutput = await adapters.run({
  command: 'df',
  args: [
    '-P',
    path,
  ],
},);
const device = parseDfDevice(mountOutput,);
const output = await adapters.run({
  command: 'diskutil',
  args: [
    'info',
    '-plist',
    device,
  ],
},);
```

`df(1)` explicitly accepts a file and reports the filesystem of which that file is a part
([archived manual][df-man]).
`package/fs-id/core/src/parsers.ts:270-309` reads only the first field of a portable `df -P` data row,
requires a safe `/dev/` path,
and ignores localized header text.
`package/fs-id/core/src/parsers.ts:311-365` then reads invariant `VolumeUUID` plist structure instead of human
presentation labels.

## Verification

Affected environments:

- GitHub `macos-latest`,
  workflow run [29064478741][failed-run];
- physical macOS 26.5.2 host available as `ssh m1`.

### Failing form

The failed workflow log captured:

```text
Error: Command failed: diskutil info -plist /Users/runner/work/Monochromatic/Monochromatic
```

### Working forms

On `ssh m1`,
`df -P /Users/user` returned:

```text
/dev/disk3s5 ... /System/Volumes/Data
```

`diskutil info -plist /dev/disk3s5` then returned a structured plist containing:

```xml
<key>VolumeUUID</key>
<string>4FF632A6-EE54-4AEF-8781-C7257CD88603</string>
```

The exact package host-evidence script was copied to a disposable directory,
run with Node 26.4.0,
and removed afterward.
It reported:

```json
{"platform":"darwin","preferredSource":"volume-uuid","preferredStable":true,"degradedSource":"device-number","degradedStable":false,"colonFree":true}
```

Workflow run [29064685354][passing-run] passed Linux,
macOS,
and Windows with the same preferred-plus-degraded host probe.

Patterns that work cleanly:

- `diskutil info -plist /dev/disk3s5`;
- `df -P <ordinary-path>` followed by `diskutil info -plist <first-data-field>`;
- plist `VolumeUUID` parsing independent of display language.

Patterns that fail or are rejected:

- an ordinary descendant path passed directly to `diskutil info`;
- `df` output without a safe `/dev/` data row;
- plist output without a `VolumeUUID` string;
- plist UUID text containing trust-key delimiters.

## Verified workaround

Resolve the mounted device with `df -P` first,
validate its `/dev/` token,
then request structured plist output from `diskutil`.

Tradeoffs:

- this adds one unprivileged subprocess on macOS;
- the parser intentionally supports device-backed filesystems and degrades when `df` reports a non-`/dev/` source;
- a mount change between `df` and `diskutil` makes the preferred lookup fail and enter the explicit degraded path rather
  than silently trusting an unrelated UUID.

## What does not work

### Pass every existing path directly to `diskutil`

Rejected by the real runner with `Could not find disk`.
Existence as a filesystem path does not make a descendant path a diskutil device.

### Parse human-readable `diskutil info`

Rejected because field labels are presentation output and can vary by locale.
`-plist` provides invariant keys.

### Treat BSD `stat -f %d` as the preferred stable identity

Rejected because it is a runtime device number,
not a reboot-stable Volume UUID.
It remains the explicit warned fallback only.

## Upstream filing decision

No matching exemption was found under `.out-of-scope/`.
There is no public `diskutil` source repository or GitHub tracker to search for duplicate implementation issues.

The six filing constraints resolve as follows:

1. Upstream fault:
    no.
   The manual asks for a device;
   passing an ordinary subpath was consumer misuse.
2. Upstream can fix it:
    technically possible,
   but not required by the documented grammar.
3. Supported use case:
    no evidence that arbitrary descendant paths are supported device arguments.
4. Contribution welcome:
    no public source or contribution process exists for `diskutil`.
5. Likely fix:
    no;
   this is documented input scope rather than a demonstrated defect.
6. Compatible prototype:
    the verified consumer-side path-to-device adapter is implemented and tested,
   but there is no upstream codebase in which to prototype a change.

Nothing should be filed upstream.
No draft issue is retained because the behavior is not an upstream defect.

[diskutil-man]: https://www.manpagez.com/man/8/diskutil/osx-10.12.6.php
[df-man]: https://www.unix.com/man_page/osx/1/df/
[failed-run]: https://github.com/Aquaticat/Monochromatic/actions/runs/29064478741
[passing-run]: https://github.com/Aquaticat/Monochromatic/actions/runs/29064685354
