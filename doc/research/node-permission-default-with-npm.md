# Default Node.js permissions with the normal npm command

Research draft with public sources and disposable local probes from 2026-11-18.
The question is whether the system Node.js permission model can be enabled by default
while the installed npm still works through the usual command.

## Public precedent

- A user asked to configure Node and npm with filesystem and network limits and keep using the normal invocation,
  writing “Use `node` as usual.” A Node maintainer closed the request and cited several similar feature requests.
  [Request](https://github.com/nodejs/node/issues/56841#issuecomment-2628920640) ·
  [Maintainer response](https://github.com/nodejs/node/issues/56841#issuecomment-2629680975)
- The Node.js Security Working Group opened a package-manager adoption discussion and invited npm and Yarn maintainers.
  A Yarn maintainer pointed out that legitimate install hooks can launch native build tools beyond the scope of
  Node's permission model. The issue closed after inactivity.
  [Working group issue](https://github.com/nodejs/security-wg/issues/1300) ·
  [Yarn maintainer response](https://github.com/nodejs/security-wg/issues/1300#issuecomment-2088667648)
- A user reported that a restricted Node process in a container launched `npm install`, which failed while creating a
  relative symlink on Node 22.19.0. That version rejected relative symlink targets even with wildcard filesystem grants.
  The later full-filesystem-grant check permits these links with both wildcard grants.
  [Install report](https://github.com/nodejs/node/issues/59752) ·
  [Earlier symlink check](https://github.com/nodejs/node/commit/041d435be97866f40b8b87a22a220f5e93f58fcc) ·
  [Full-grant check](https://github.com/nodejs/node/commit/3296b2e734cc1b329f3c32aee2c7bdeb4ade7cf4)
- The documented npm integration lets `npx` or `npm exec` supply Node flags to the program they launch.
  npm's `node-options` setting is applied to lifecycle processes after npm itself has started.
  [Merged Node documentation](https://github.com/nodejs/node/pull/56539#issuecomment-2581314791) ·
  [npm configuration](https://docs.npmjs.com/cli/v12/using-npm/config/#node-options)

Public searches combined `NODE_OPTIONS`, `--permission`, `npm install`, and default-on or shell configuration terms.
They established demand for the setup and related integration work. They yielded no documented deployment of
path-bounded enforcement that preserved ordinary npm installs across the workloads probed here.

## What worked on the system binaries

The system packages provided `/usr/bin/node` v26.13.1 and `/usr/bin/npm` v12.2.3.
Each install used a disposable project, home, and npm cache. Bash invoked literal `npm install`.
The tests used the installed npm launcher, the public registry, and a controlled project `postinstall` script.

- With inherited `NODE_OPTIONS` granting project and cache writes, reads of the project, test home, and
  `/usr/lib/node_modules`, plus network and subprocess access, npm installed `is-number@7.0.0` and ran the hook.
  An ordinary Node process under the same settings could write in the granted workspace.
  A direct `node:fs` write to a sibling scratch directory returned `ERR_ACCESS_DENIED`.
- With the same path-limited profile, `semver@7.7.2` failed with
  `ERR_ACCESS_DENIED` and `fs.symlink API requires full fs.read and fs.write permissions.`
  Setting `ignore-scripts=true` also produced the error.
  npm was creating the executable link `node_modules/.bin/semver`, which is part of package installation.
  Node checks the entire filesystem permission scope when creating a symlink.
  [Node v26.13.1 source](https://github.com/nodejs/node/blob/e9c277643b0249c50f6e6de5413be8a88ea11b6a/lib/internal/fs/promises.js#L1833-L1838)
- Giving the same processes `--allow-fs-read=*` and `--allow-fs-write=*` in addition to `--allow-net`
  and `--allow-child-process` let plain `npm install` create the relative link and run the hook.
  The baseline without Node options also completed both actions.
  Those broad options apply to every Node process inheriting them, including the Node process in the hook.
  Node emitted the `PERM0002` warning for the subprocess grant.
- With `NODE_OPTIONS=--permission-audit`, the same install created the executable link and the hook wrote a
  fixture marker. Audit mode can publish permission-check violations through `diagnostics_channel`
  while execution continues.
  [Audit mode documentation](https://nodejs.org/docs/latest-v26.x/api/permissions.html#audit-mode)

## What keeping both commands means

Node v26.13.1 takes permission flags from `NODE_OPTIONS` for each inheriting process.
The permission namespace in `node.config.json` applies when Node is launched with `--config-file`.
The documented options have no entry-point selector for applying different rules to npm.
[Environment options](https://nodejs.org/docs/latest-v26.x/api/cli.html#node_optionsoptions) ·
[Config file flag](https://nodejs.org/docs/latest-v26.x/api/cli.html#--config-filepath---config-file)

An early preload can select different permissions in custom code.
In a scratch probe, the preload let the installed npm CLI retain broad grants and dropped filesystem writes,
network access, and subprocess creation from other Node entry points via `process.permission.drop()`.
Plain `npm install` linked the `semver` executable and ran a read-only hook.
A Node hook that needed to write a fixture file failed under the same preload.
A further policy would need to decide what npm's descendants may do.
[Runtime permission API](https://nodejs.org/docs/latest-v26.x/api/permissions.html#permissiondropscope-reference)

If the goal is controlling dependency install scripts, npm 12 supports that through `allowScripts`
with the usual invocation. In a disposable loopback registry probe, plain `npm install` installed a fixture
package while blocking its dependency hook. Adding the fixture's exact version to the project's `allowScripts`
let that hook create a marker on the next independent install.
This controls whether dependency hooks run. It does not set runtime permissions for Node programs.
[npm RFC](https://github.com/npm/rfcs/pull/868) ·
[npm configuration](https://docs.npmjs.com/cli/v12/using-npm/config/#allow-scripts)

## Persistent audit default on CachyOS

The machine default was set to `NODE_OPTIONS=--permission-audit` on `2026-11-18T19:35:41-05:00`.

- `/etc/environment` supplies the login value through `pam_env.so` in `/etc/pam.d/system-login`.
  The installed systemd user environment generator also reads this file.
- `/etc/xdg/plasma-workspace/env/60-node-permission-audit.sh` exports the same value before Plasma
  publishes its login environment. See [the Plasma startup ordering](../troubleshooting/plasma-overwrites-environment-d-path.md).
- `/etc/systemd/system.conf.d/60-node-permission-audit.conf` sets `DefaultEnvironment` for system services.
  The live system manager, user manager, and session D-Bus activation environment were updated in place.

Node v26.13.1 inherited the setting in a PAM-spawned Fish login shell, from the installed Plasma environment
loader, and in fresh system and user services. Each probe read `/dev/null` and observed an audit event.
npm v12.2.3 printed its version from a fresh user service with a disposable cache.
The full Plasma login was left running. Programs launched by an existing process continue to inherit that
process's environment until the process is relaunched.

## What the audit default refuses

Audit mode logs path checks instead of refusing them, but it still refuses every operation Node cannot tie to a path.
The Node 26 permissions page ([Permissions][node-26-permissions], read 2026-11-18) says:
"Some node:fs operations act on an already-open file descriptor rather than a path, so they cannot be tied to a
--allow-fs-read or --allow-fs-write grant. When the permission model is enabled these operations are disabled and
throw ERR_ACCESS_DENIED, regardless of how the descriptor was obtained. This applies both to the top-level node:fs
functions and to the equivalent FileHandle methods, and currently includes fsync/fdatasync, fchmod, fchown, and futimes
(and their synchronous variants)."

### Measured on this machine

On 2026-11-18 at about 22:20-05:00, Node v26.13.1 ran in a fresh user unit, which inherited
`NODE_OPTIONS="--permission-audit"`. Every form of syncing a file was refused with `ERR_ACCESS_DENIED`:

- `fs.fsyncSync`;
- `fs.fdatasyncSync`;
- the callback forms `fs.fsync` and `fs.fdatasync`;
- `FileHandle.sync` and `FileHandle.datasync`;
- `fs.writeFileSync` with `flush: true`;
- `fs/promises` `writeFile` with `flush: true`.

The same write without `flush` ran.
The script and its output are under
`~/temp/agent/node-permission-audit-refusals-of-sync-calls-2026-11-18/keep/each-way-node-can-sync-a-file-in-a-fresh-user-unit.txt`.
The first program seen to meet the refusal was the agent-cap disk probe at 2026-11-18T21:18:17-05:00, which called
`fsyncSync` and exited at its first reading.

### Callers found on 2026-11-18

A search of `package/` and `.config/` for every form in the measured list, plus `fchmod`, `fchown` and `futimes`, found
five calls, all `FileHandle.sync()`, each of which throws under the default:

- `package/git-policy/cli/bin/native-planted-controls-record.ts:168`;
- `package/mutation-test/cli-cargo-mutants/src/claim-files.ts:83` and `:124`;
- `package/wg/cli-quicker/src/opensnitch-config-file.ts:164`;
- `package/wg/cli-quicker/src/opensnitch-toolchain-rules.ts:130`.

None of the 25 Pi run feeds written between 19:35 and 22:20 named the refusal, and in the user journal over the same
time only the disk probe did.
Third-party packages in pnpm's store that call `fsync` include `atomically`, `stubborn-fs`, `postject`, `prettier`'s
experimental command line, `socket`, `typescript-language-server`, `unenv`, and npm's own `write-file-atomic`.
Which of them a command here reaches is known only through those feeds and the journal.

### Keeping durability under the default

The owner's standing words: "I turned on the permission model by default. And we should figure out how to work with
it, not disable it."
Decided by the owner on 2026-11-18 (question tool, "Call coreutils sync (Recommended)"): after the write, the file is
closed and synced by running GNU coreutils `sync --data <file>` as a child process. `sync` with file operands syncs
only those files, and `--data` syncs only their data (`sync --help`, coreutils on this machine).
Measured the same evening with hyperfine, twenty runs each: median 0.78 ms per `sync --data` call on the home dataset,
against 0.37 ms for `/usr/bin/true`.
Limits: the call needs `--allow-child-process` once the model refuses instead of audits, and file operands are GNU
coreutils behaviour (Linux, which all five callers target).
The options not chosen were a repository-built sync helper binary and moving each durable write into Rust.

### Directory sync for published claims

The five TypeScript call sites found under "Callers found on 2026-11-18" now use `sync --data -- <path>`.
The claim writer also passes its directory to this command after linking or renaming the claim.
GNU coreutils 9.12's [`sync_arg`][coreutils-sync] passes `--data` to `fdatasync`.
OpenZFS 2.4.4 [uses `zpl_fsync` for directories][zfs-linux-sync],
and [`zfs_fsync`][zfs-sync] commits that object's ZIL regardless of the data-only flag when sync is enabled.

A disposable probe on `zpcachyos/ROOT/cos/home` (`sync=standard`) inherited `NODE_OPTIONS=--permission-audit`.
Node's `fsyncSync` failed with `ERR_ACCESS_DENIED: fsync API is disabled when Permission Model is enabled.`
The measured signal was `zil_commit_count` in `/proc/spl/kstat/zfs/zil`.
Each round wrote a fresh file and used fresh destination directories for separate hard links:

- Running `true` after a write or after a link: deltas `0, 0, 0` for each control.
- Running `sync --data -- <file>` after a write: deltas `1, 1, 1`.
- Running `sync --data -- <directory>` after a link: deltas `1, 1, 1`.
- Running `sync -- <directory>` after a link: deltas `1, 1, 1`.

The controls show that the directory command reaches the same log commit mechanism as full sync on this dataset.
This was not a power-loss test and does not establish data-only directory sync semantics on other filesystems.

[coreutils-sync]: https://github.com/coreutils/coreutils/blob/v9.12/src/sync.c
[zfs-linux-sync]: https://github.com/openzfs/zfs/blob/zfs-2.4.4/module/os/linux/zfs/zpl_file.c
[zfs-sync]: https://github.com/openzfs/zfs/blob/zfs-2.4.4/module/zfs/zfs_vnops.c
[node-26-permissions]: https://nodejs.org/docs/latest-v26.x/api/permissions.html#:~:text=Some%20node%3Afs%20operations%20act%20on%20an%20already-open%20file%20descriptor
