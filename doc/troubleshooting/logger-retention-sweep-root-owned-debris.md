# `@monochromatic-dev/logger-core` file-sink sweeps report `log retention could not stamp ... EACCES` over root-owned `node_modules/.monochromatic` debris

## Symptom

Any non-root task whose logger resolves the file sink inside a workspace prints this line
on stderr while the task runs:

```text
logger internal error: log retention could not stamp "/var/home/user/Monochromatic/package/wg/cli-quicker/node_modules/.monochromatic": EACCES: permission denied, open '/var/home/user/Monochromatic/package/wg/cli-quicker/node_modules/.monochromatic/.logger-retention-sweep'
```

Observed on 2026-10-31 in `mise run //package/wg/cli-quicker:lint:oxlint` output.
The sweep is housekeeping:
it logs the fault and skips itself,
so the message does not change the task's exit status.
The same task was red at that moment for an unrelated reason
(six lint warnings in `src/scratch-parent.ts`,
cleared by `aaa3c76bf0`),
which is what an earlier reading blamed the `EACCES` line for.

Quote disproving that reading:
`package/logger/core/src/sink/file-retention-sweep.ts:151-155` reports and returns,
never rethrowing:

```ts
    } catch (error) {
      reportError({
        context: `log retention could not stamp ${JSON.stringify(directory,)}`,
        error,
      },);
      return;
    }
```

The real damage is silent:
until the stamp is writable again,
every later sweep skips pruning,
so the foreign-owned records stay forever.

## Root cause

### Step 1: privileged runs write into the shared user-workspace log directory

The file sink resolves its destination to the nearest `node_modules/.monochromatic`
(package sources do the same for scratch,
see `package/wg/cli-quicker/src/scratch-parent.ts`),
so a `sudo`-relaunched `wg-quicker` or any sudo-spawning test writes log records,
probe files,
and the sweep stamp as uid 0 into the user's checkout.

Measured ownership in `package/wg/cli-quicker/node_modules/.monochromatic`
before repair:

```text
drwx------. 1 root root  4118 Oct 29 10:10 2026-10-29
-rw-------. 1 root root    40 Oct 29 09:42 2026-10-29T13-42-51.642Z.b3aebdd4-711f-4dda-a6f6-f7fcec0f42f1.probe.tmp
-rw-------. 1 root root    40 Oct 29 10:08 2026-10-29T14-08-59.592Z.b12dd861-90ba-453b-8c97-851ccb812d41.probe.tmp
-rw-------. 1 root root    40 Oct 29 10:10 2026-10-29T14-10-30.491Z.0f015fe9-a1d5-48d7-b6e7-5e6fcc41f48d.probe.tmp
-rw-------. 1 root root    40 Oct 29 10:10 2026-10-29T14-10-30.676Z.56e4c848-00b1-4dba-af80-9d0e8914c9ec.probe.tmp
-rw-------. 1 root root    40 Oct 29 10:10 2026-10-29T14-10-31.148Z.e04e5767-46c1-4f37-9ca1-2bc194259c84.probe.tmp
-rw-------. 1 root root    25 Oct 29 09:42 .logger-retention-sweep
```

Which privileged command created them is not recorded;
the timestamps are 2026-10-29 09:42 to 10:10 local,
inside the window of the package-directory move commits.

### Step 2: the sweep stamp becomes unrewritable and the sweep self-disables

`sweepLogDirectory`
(`package/logger/core/src/sink/file-retention-sweep.ts`)
rewrites `.logger-retention-sweep` before pruning.
A root-owned stamp makes that write fail for the user,
the catch reports `log retention could not stamp ... EACCES`,
and the function returns before any deletion.

### Step 3: debris accumulates

Later user-side sweeps cannot delete the root-owned day directory
(`2026-10-29`) or probe files either,
so the directory keeps both the records and the error line indefinitely.

## Verification

Ownership probe,
before and after repair:

```console
$ ls -la package/wg/cli-quicker/node_modules/.monochromatic | rg root
drwx------. 1 root root  4118 Oct 29 10:10 2026-10-29
...
$ sudo chown -R user:user package/wg/cli-quicker/node_modules/.monochromatic
$ ls -la package/wg/cli-quicker/node_modules/.monochromatic | rg root
$ find package -mindepth 5 -maxdepth 5 -path '*/node_modules/.monochromatic/*' -user root -print | wc -l
0
```

Task-output harness,
same task before and after repair:

```console
mise run //package/wg/cli-quicker:lint:oxlint
```

Before repair the output contains `logger internal error: log retention could not stamp ...`;
after repair it does not.
Scope:
the `find` probe above reports zero root-owned entries in any other package's
`node_modules/.monochromatic`,
so this debris set was confined to `package/wg/cli-quicker`.

### Patterns that fail

- Any non-root node process in a package whose `node_modules/.monochromatic` holds
  a root-owned `.logger-retention-sweep`,
  once its logger verifies the file sink.
- Retention pruning of that directory:
  it stays skipped while the stamp is foreign-owned.

### Patterns that work cleanly

- The same tasks after ownership repair:
  the sweep stamps,
  prunes,
  and stays quiet.
- Packages whose `node_modules/.monochromatic` is user-owned throughout,
  which covers every other package in this checkout.

## Verified workarounds

1.  Repair ownership of the whole log directory:

    ```console
    $ sudo chown -R user:user package/wg/cli-quicker/node_modules/.monochromatic
    ```

    Tradeoff:
    this is per-directory housekeeping;
    the next privileged run of workspace tooling in that package can recreate root-owned
    records and a root-owned stamp.
    It preserves every log record,
    unlike deleting the debris.

## What does not work

- Re-running the task unchanged.
  The stamp stays foreign-owned,
  so every sweep reports the same `EACCES` and skips pruning again.

## Durable fix not landed

Two first-party changes would remove the recurrence:

1.  Privileged runs should write logs to a root-owned destination
    (the sink already has a per-user fallback path)
    instead of the user's `node_modules/.monochromatic`.
2.  The sweep should treat a foreign-owned stamp or entry as absent
    and keep pruning what it can delete,
    instead of disabling itself for the whole directory.

Neither change has landed.

## Upstream filing decision

Nothing to file.
Both the sweep and the sink are first-party
(`package/logger/core`),
so there is no external tracker.

Walking the six constraints anyway:

1.  **Is it really upstream's fault?**
    No;
    there is no upstream and the debris is produced by our sudo relaunch path.
2.  **Can upstream fix it?**
    Not applicable;
    the fix would land in `package/logger/core`.
3.  **Are they supporting this use case?**
    Not applicable.
4.  **Would the repo welcome our contribution?**
    Not applicable.
5.  **Will they likely fix it?**
    Not applicable.
6.  **Have we prototyped a minimal fix compatible with their architecture?**
    Not attempted;
    the two candidate changes in
    [Durable fix not landed](#durable-fix-not-landed)
    are recorded for the owning package.
