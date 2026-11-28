# `prepare` in a guest that is already prepared

**Non-final,
unapproved.**
This records what an agent ran and what a guest answered;
it is not the project's acceptance of the change,
and the owner alone decides that.

It answers issue 6638:
the guest recipe of this package
(`e2e/run-fixture.ts`)
could not run its `prepare` step in a guest the same recipe had prepared,
because that step removed the two kernel modules with
`modprobe --remove ... || true`
and then inserted its own with `insmod`,
so a module a mounted dataset held met an
`insmod: ERROR: could not insert module ...: File exists`.

The change is the step in `e2e/load-the-built-module-step.ts`,
whose four callers are
`prepare`,
`STEP10_LOAD_THE_MODULE` (both step 10 steps)
and the two module-changing gate arms
(`zfs-gate-version-arm` and `zfs-gate-change-arm`).
Its host-side check is `e2e/load-the-built-module-step.check.ts`,
run by
`mise run //package/zfs/request-to-openzfs-kernel-device:test:tooling`,
and both of this package's READMEs state what it does.

## Where and how

Every run was in a throwaway `mvm` guest on the `libvirt` backend,
created and destroyed through the recipe's own steps,
one guest at a time,
and named `issue-6638-before`,
`issue-6638-after` and `issue-6638-driver`.
The host was read only:
no host command touched ZFS or opened `/dev/zfs`.

The guest is `mvm`'s Ubuntu 24.04 image on the plan's kernel,
`6.8.0-147-generic`,
prepared by the recipe itself.
Each host step ran as
`systemd-run --user --scope --slice=agent-work.slice
mise run //package/zfs/request-to-openzfs-kernel-device:e2e -- <step>`.

## What a guest answers about its own modules

Measured in the prepared `issue-6638-before` guest,
which is what the step reads:

- the two modules this recipe builds publish their own versions:
  `/sys/module/zfs/version` and `/sys/module/spl/version` both read `2.4.4-1`;
- `modinfo --field version /lib/modules/6.8.0-147-generic/extra/zfs.ko`,
  and the same for `spl.ko`,
  read `2.4.4-1`,
  so the version of the file this recipe installed is readable without loading it;
- `modinfo --field version zfs` reads `2.2.2-0ubuntu9.5`,
  the distribution's module,
  because `modules.dep` holds
  `kernel/zfs/zfs.ko.zst: kernel/zfs/spl.ko.zst` for those names,
  and the recipe's files under `extra/` are not what `modprobe` resolves;
- the image ships no `zfs-*` unit files,
  so a fresh guest boots with no module loaded;
- with a 1 GiB file-backed pool `issue6638pool` imported and nothing mounted,
  `/sys/module/zfs/refcnt` read `0`,
  and `modprobe --remove zfs` exited 0 and took the module out;
- with one dataset of that pool mounted,
  the mount table held
  `51 29 0:43 / /root/issue-6638-pool/mnt rw,relatime shared:20 - zfs issue6638pool/data rw,xattr,noacl,casesensitive`,
  `/sys/module/zfs/refcnt` read `1`,
  and `modprobe --remove zfs` refused with `modprobe: FATAL: Module zfs is in use.`

So a mounted dataset is what holds the module,
and an imported pool alone does not.
That is why the refusal reads `/proc/self/mountinfo` as well as
`/proc/spl/kstat/zfs`,
and says which of the two it found.

## The fault, before the change

Four `prepare` runs in the one guest `issue-6638-before`,
each through the recipe's step:

- in the fresh guest,
  the first `prepare` exited 0;
- a second `prepare`,
  with no pool imported,
  exited 0 as well,
  so a second `prepare` on its own does not fail:
  the fault needs something holding the module;
- after a 1 GiB file-backed pool was imported and one of its datasets mounted,
  `prepare` exited 1,
  and the step's own log ended:

  ```text
  modprobe: FATAL: Module zfs is in use.
  modprobe: FATAL: Module spl is in use.
  insmod: ERROR: could not insert module /lib/modules/6.8.0-147-generic/extra/spl.ko: File exists
  ```

  which is the issue's own end,
  and the recipe's process reported
  `Error: mvm exec exited 1; see guest-build.log`;
- the run after it made the same end again,
  exit 1 and those three lines,
  so the end is not a one-off.

### The three other places that removed and inserted the modules

Each was run verbatim,
as the step it belongs to runs it,
in the same guest while the module was held:

- `gateChangeArm`'s load
  (a removal,
  `test ! -d /sys/module/zfs`,
  then an insert):
  exit 1,
  and its log held only the two `modprobe: FATAL: Module ... is in use.` lines,
  because the `test` that stopped it says nothing;
- `gateVersionArm`'s load,
  with the installed file patched to `2.4.4-2` as that arm patches it,
  while `2.4.4-1` was loaded and held:
  exit 1,
  and its log held the two FATAL lines and
  `insmod: ERROR: could not insert module .../spl.ko: File exists`,
  because the `test ... = 2.4.4-2` that stopped it says nothing either;
- `STEP10_LOAD_THE_MODULE`'s guard,
  in a guest running the patched `2.4.4-2` module held by the same mount:
  exit 1,
  with the two FATAL lines and the `insmod ... File exists` line.

All three had the same fault,
and two of them ended on a bare non-zero status with no sentence at all.
A fourth place,
`prepareFromTemplate` (the custom-template `prepare`),
loads a module with `modprobe --quiet zfs 2>/dev/null || true`
and removes nothing;
it is unchanged here,
and this change's refusal does not reach it.

## The change

The step reads whether each module is loaded from its `/sys/module` directory,
and the version a loaded module publishes from that directory's `version` file,
then the version each installed file states,
with `modinfo --field version`.
zfs's version is the identity,
because it is the release every step of this recipe pins;
spl's is compared where both sides record one.
Four states follow:

- the built module is loaded:
  it says so and changes nothing,
  so a repeat `prepare` removes and inserts nothing and the guest's pools and mounts
  keep running;
- nothing is loaded:
  it inserts the built modules;
- another module is loaded and can be removed:
  it removes it,
  `zfs` first and then `spl`,
  prints each removal,
  and inserts the built ones;
- another module is loaded and will not go:
  it refuses,
  naming the loaded version,
  the built version and the file it came from,
  `modprobe`'s own words,
  or that it printed nothing,
  the kernel's reference count,
  the datasets the kernel lists as mounted and the pools it lists as imported,
  and both ways out:
  unmount and export or destroy,
  then run the step again,
  or reboot the guest and run it again,
  which reads again what is loaded.

Every removal's status is kept,
and the module's directory is read again after it,
so a removal that fails ends the step in those words and no `insmod` is reached with
a module still there.
Every insert is checked the same way,
including that the loaded module reports the version its file states.

## The guest runs after the change

### The issue's own three runs, in `issue-6638-after`

- `prepare` in the fresh guest:
  exit 0,
  and its step log read

  ```text
  prepare: no zfs or spl module is loaded; inserting the modules this recipe built (zfs 2.4.4-1, spl 2.4.4-1)
  prepare: the guest now runs the module this recipe built (zfs 2.4.4-1, spl 2.4.4-1, from /lib/modules/6.8.0-147-generic/extra/zfs.ko and /lib/modules/6.8.0-147-generic/extra/spl.ko)
  ```

- `prepare` again,
  with no pool imported:
  exit 0,
  and its step log read

  ```text
  prepare: the guest already runs the module this recipe built: zfs 2.4.4-1 against /lib/modules/6.8.0-147-generic/extra/zfs.ko stating 2.4.4-1, spl 2.4.4-1 against /lib/modules/6.8.0-147-generic/extra/spl.ko stating 2.4.4-1; nothing was removed and nothing was inserted
  ```

- `prepare` with a 1 GiB file-backed pool `issue6638pool` imported and its dataset
  `data` mounted,
  so `/sys/module/zfs/refcnt` read `1`:
  exit 0 with the same sentence as the run before,
  and afterwards `zpool list -H -o name` still printed `issue6638pool`,
  `/sys/module/zfs/version` read `2.4.4-1`,
  and `/sys/module/zfs/refcnt` read `1`.

  That is the run the issue is about,
  and it now ends 0,
  says so,
  and changes nothing.

### The other module it can remove

In the same guest,
the recipe's module was taken out and `modprobe zfs` was asked for a module,
which `modules.dep` answers with the distribution's
`2.2.2-0ubuntu9.5`.
The step,
run as `prepare` runs it,
exited 0:

```text
prepare: the guest runs another module (zfs 2.2.2-0ubuntu9.5, spl 2.2.2-0ubuntu9.5); removing it before inserting the modules this recipe built (zfs 2.4.4-1, spl 2.4.4-1)
prepare: removed zfs 2.2.2-0ubuntu9.5
prepare: removed spl 2.2.2-0ubuntu9.5
prepare: the guest now runs the module this recipe built (zfs 2.4.4-1, spl 2.4.4-1, from /lib/modules/6.8.0-147-generic/extra/zfs.ko and /lib/modules/6.8.0-147-generic/extra/spl.ko)
```

The whole `prepare` was not run for this state,
because its build step installs the recipe's own files before the step reads them;
what is shown is the step's own shell,
run exactly as `prepare` runs it.

### The other module it cannot remove

The installed file was patched to `2.4.4-2` and loaded,
a pool and a mounted dataset were made on it,
and the approved `2.4.4-1` file was put back beside the loaded module,
so the installed file stated `2.4.4-1` and the loaded module reported `2.4.4-2`.
The step exited 1:

```text
prepare: the guest runs another module (zfs 2.4.4-2, spl 2.4.4-1); removing it before inserting the modules this recipe built (zfs 2.4.4-1, spl 2.4.4-1)
prepare: refusing to insert the module this recipe built: this guest runs a module this recipe did not build, and module zfs would not go.
  loaded: zfs 2.4.4-2, spl 2.4.4-1
  built: zfs 2.4.4-1 (/lib/modules/6.8.0-147-generic/extra/zfs.ko), spl 2.4.4-1 (/lib/modules/6.8.0-147-generic/extra/spl.ko)
  modprobe --remove zfs said: modprobe: FATAL: Module zfs is in use.
  the kernel counts 1 references to module zfs
  the kernel lists these ZFS datasets mounted: issue6638pool/data
  the kernel lists these pools imported: issue6638pool
  a mounted dataset is what holds module zfs: this recipe measured the count at 1 with one dataset mounted, and at 0 with a pool imported and nothing mounted, where modprobe took the module out
  to run the module this recipe built: unmount each mounted dataset (zfs unmount <dataset>, or umount its mountpoint), export each pool that may go (zpool export <pool>) or destroy it when what it holds may be destroyed (zpool destroy <pool>), stop anything else holding /dev/zfs (zfs-zed opens it), and run this step again
  the guest is left running: zfs 2.4.4-2, spl 2.4.4-1; to start from nothing instead, reboot the guest and run this step again, which reads again what is loaded
```

The same shell,
run with each of the other three places' own labels,
answered with the same lines and exit 1 each:

- `step 10`,
- `gate version arm`,
- `gate change arm`.

### The way out it names, taken

The pool was destroyed,
which unmounted the dataset and removed the holder.
The step then exited 0:

```text
prepare: the guest runs another module (zfs 2.4.4-2, spl 2.4.4-1); removing it before inserting the modules this recipe built (zfs 2.4.4-1, spl 2.4.4-1)
prepare: removed zfs 2.4.4-2
prepare: removed spl 2.4.4-1
prepare: the guest now runs the module this recipe built (zfs 2.4.4-1, spl 2.4.4-1, from /lib/modules/6.8.0-147-generic/extra/zfs.ko and /lib/modules/6.8.0-147-generic/extra/spl.ko)
```

### A driver's own steps in a fresh guest

`mise run //package/git-policy/cli:e2e:zfs-slot-state -- all --name issue-6638-driver`
created a guest,
prepared it,
ran plan step 12's slot-state arm in it,
and destroyed the guest,
all in one command,
unchanged from how a round calls it.
The whole run exited 0 in 809 seconds.

Its `prepare` step is the recipe's,
and its log read the same two lines a fresh `prepare` reads in this record:

```text
prepare: no zfs or spl module is loaded; inserting the modules this recipe built (zfs 2.4.4-1, spl 2.4.4-1)
prepare: the guest now runs the module this recipe built (zfs 2.4.4-1, spl 2.4.4-1, from /lib/modules/6.8.0-147-generic/extra/zfs.ko and /lib/modules/6.8.0-147-generic/extra/spl.ko)
```

Its arm then built the wrapper crate in the guest in 6 minutes 50 seconds and passed:

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1369 filtered out; finished in 2.52s
```

with every marker the driver requires,
ending `SLOT-STATE done`.
So a first `prepare` still leaves what a driver's arm needs:
Rust,
`cargo-nextest`,
the OpenZFS userspace and the recipe's module,
with the kernel's type information for it.

## The check on the host

`mise run //package/zfs/request-to-openzfs-kernel-device:test:tooling`
runs `e2e/load-the-built-module-step.check.ts`,
which runs the step's own shell in `sh` with `set -eu`,
against stand-in `/sys`,
`/proc` and module trees and stand-in
`modinfo`,
`modprobe`,
`insmod` and `uname`,
once per state:

- a fresh guest;
- a guest already running the built modules,
  with a pool listed;
- a guest running the distribution's module;
- a held module with a mounted dataset named,
  and one whose pool has nothing mounted;
- a removal that exits 0 and leaves the module loaded,
  and one that fails silently;
- a build that records no spl version,
  both with a loaded spl that publishes none and
  with one that publishes another version,
  which is replaced;
- a guest holding spl alone,
  and one holding zfs alone;
- a module file that is not there,
  one that states no version,
  and one `modinfo`
  cannot read;
- an insert that fails,
  and one whose module comes up as another version;
- zfs removed and spl held,
  which must say the guest is left with spl alone.

It also reads `run-fixture.ts` and holds that the recipe imports the step,
asks no `modprobe --remove` and no `insmod` itself,
and calls the step at all four places.
The stand-ins are stand-ins:
what they prove is the step's decision,
not the kernel's answer,
and the guest runs above are what show the kernel answering.

## Limits

- The identity is the version string alone,
  so two builds of one release read alike.
  That is what a repeat `prepare` wants;
  a step that needs a rebuilt module loaded starts with nothing loaded,
  which is how both gate arms work,
  by destroying every pool and rebooting.
- The refusal names the mounts and pools the kernel lists.
  It cannot name a program holding `/dev/zfs`,
  and its words about what else could hold the module say so.
- The holder measurement is one guest,
  one kernel release and one module release.
  An imported pool alone left the count at 0 there;
  another release could disagree.
