# Depot Sandbox SDK 0.1.0-beta.7 custom snapshots need explicit executables and operation diagnostics

## Symptom

A custom Depot thin snapshot was accepted and reached `running`,
but a command using plain `node` failed:

```text
# Sandbox 6m8d0p0pvnrqlb8
[internal] execute command: execute: stream error: unknown: execute 6m8d0p0pvnrqlb8 command:
start exec: exec: "node": executable file not found in $PATH
```

A separate sandbox reached the same custom runtime,
then its filesystem consumer failed with `FileSystemError`,
message `the operation timed out`,
and code `OTHER`.
The fixture attempted executable `stat` and a 20,315,448-byte executable read before execution,
but retained neither an operation boundary nor the error's `syscall` and `path` fields.
The receipt cannot distinguish those operations.

These incidents do not establish unsupported custom runtimes,
missing Node files on disk,
a filesystem size limit,
or a shared cause.

## Root cause and source trace

The command API executes an executable directly.
The [deployed SDK reference][reference] requires either an executable on the sandbox's `PATH`
or an absolute executable path.
An image's installed tools do not establish the command environment used by another product.
The first fixture assumed plain Node discovery without measuring it.

The timeout's underlying cause remains unknown.
The diagnostic loss is in the research fixture:
it retained only `name`,
`message`,
and `code`.
SDK source commit `0e740b788d2613128b2ba6ed9b4129333244ee56`
preserves the missing fields.
`src/filesystem.ts:201` and `src/filesystem.ts:208` identify read failures as `open`:

```typescript
// depot/sandbox-sdk src/filesystem.ts:201
async readFile(path: string, opts?: ReadFileOpts): Promise<Buffer | string> {
  const chunks: Uint8Array[] = []
  try {
    for await (const chunk of this.client.rpc.readFile({...this.ref(), path})) {
      if (chunk.data.length > 0) chunks.push(chunk.data)
    }
  } catch (err) {
    throw toFileSystemError(err, 'open', path)
  }
  const buf = Buffer.concat(chunks)
  return opts?.encoding ? buf.toString(opts.encoding) : buf
}
```

The independent `statImpl` catch at `src/filesystem.ts:335` supplies a different operation:

```typescript
// depot/sandbox-sdk src/filesystem.ts:335
throw toFileSystemError(err, followSymlinks ? 'stat' : 'lstat', path)
```

`src/errors.ts:138` retains that distinction for an unclassified Connect failure:

```typescript
// depot/sandbox-sdk src/errors.ts:138
return new FileSystemError({code: 'OTHER', syscall, path, message: err.rawMessage})
```

The SDK therefore did not erase those operation fields.
A local consumer against the installed package injected the same timeout into `stat` and `readFile`.
The old captured triples were identical;
capturing `syscall` and `path` distinguished `stat` from `open`.
That positive control does not reconstruct the lost actual operation.

## Verification

The [retained receipt][evidence] contains source,
registry admission,
both failed attempts,
the successful consumer,
the local diagnostic positive control,
and fresh terminal `Get` results.
The installed package is `@depot/sandbox` version `0.1.0-beta.7`.

The admitted image is the immutable manifest
`sha256:fb197837831d438592d9455b3a31d808a20ad8bdc2ae76b05d8b36f6656419cd`
behind the recorded `tools-r2-git256` reference.
Its format is a Depot thin block snapshot,
not an ordinary OCI root filesystem image.
The source owner allowed read-only use;
the probe did not retag or delete it.

The successful harness is repository commit `3edf13d76b0f53b1612bbcb31bc3ff4fd0abf8b3`,
`.depot/probe/platform-sdk/mise.toml`,
also embedded in the receipt's `sources`.
It uses `mise run probe`,
a one-create guard,
2 CPUs,
4 GiB requested memory,
a two-minute server lifetime,
client RPC deadlines,
and terminal cleanup with a fresh client.
The completed fixture deliberately refuses reuse of its receipt path.

### Working catalog

- Sandbox `dmdwcf07jzx2rxj` executed the admitted absolute Git path,
  returned `git version 2.56.0`,
  and exited zero.
- Its filesystem consumer read the 15,557-byte known image manifest,
  verified the named synthetic exclusions,
  and wrote/read/hashed/deleted a 124-byte nonce artifact.
- The nonce's prior successful read and subsequent `exists: false` provide an unlink positive control.
- Pre-create and post-consumer registry reads matched the admitted digest.
- All three known custom-runtime allocations reached `finished`,
  independently confirmed by `Get`.

### Failing or unresolved catalog

- Plain `node` discovery failed in `6m8d0p0pvnrqlb8`.
- The pre-execution filesystem sequence timed out in `b2f4z72r3951f5k`.
  The exact operation and cause cannot be recovered from that collector.
- The optional detached log stream hit its five-second abort.
  A later query of only that command and its lifecycle window returned no rows.
  Asynchronous capture and indexing mean neither result establishes absent retained output.
- Full executable transfer,
  its runtime SHA256,
  arbitrary OCI roots,
  named runtimes,
  and another architecture's custom snapshot were not qualified by this consumer.

## Verified workaround

Use the measured absolute executable path before filesystem transfer.
Keep small-file and command checks independent,
retain `syscall`,
`path`,
and `errno` with every filesystem failure,
and record each operation's start and finish.
The executed source implements these changes with ten-second filesystem RPC deadlines.

The tradeoff is that the executable path belongs to this admitted snapshot.
A different image needs its own executable and environment contract.
This workaround proves neither Node discovery nor a large-transfer remedy.
Stopping a sandbox also does not prove physical erasure of provider storage.

## What does not work

- Treating image reference admission as proof that the intended tool executed.
- Letting an auxiliary executable download prevent the primary execution control from running.
- Inferring a file-size limit from a timeout whose operation was not retained.
- Treating small-file success as proof that the previous timeout was fixed.
- Treating an empty retained-log query as proof of complete capture.

## Upstream filing decision

The repository's `.out-of-scope/` entries contain no matching Depot Sandbox exemption.
Public issue and pull request searches for `filesystem timeout`,
plus the broader `sandbox image` issue search,
returned no matching records on 2026-10-18.
This does not establish absence of an upstream incident.

1.  Upstream fault is not established:
    executable discovery followed the documented contract,
    the timeout's operation was lost by our collector,
    and the corrected small consumer passed.
2.  A fix location cannot be assigned until an actual failing operation and cause are retained.
3.  Image references,
    direct execution,
    and filesystem access are documented SDK use cases.
4.  The public repository has issues enabled and an MIT license.
    The clone contains no contribution or issue-template files;
    no inspected policy prohibits external reports.
5.  No inspected source or tracker result establishes a maintainer refusal.
    Silence is not a rejection.
6.  No upstream patch is prototyped because the first constraint fails.
    The consumer-side collector correction has a passing local positive control.

There is no additive upstream defect report to file from these receipts.
Do not report custom snapshots or filesystem access as unsupported from the failed fixtures.
The separate deployed documentation contradiction is retained in the
[platform audit](../planning/depot-platform-surface-audit.md).
No external issue or comment was posted.

[reference]: https://depot.dev/docs/api/sandbox-sdk-reference
[evidence]: ../audit/depot-sdk-custom-runtime-2026-10-18.evidence.json
