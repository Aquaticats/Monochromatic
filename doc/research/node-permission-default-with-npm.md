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
