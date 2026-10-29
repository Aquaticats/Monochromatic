# sudo 1.9.17p2 secure_path hides workspace `wg-quicker` after non-root config access fails

## Symptom

Running workspace CLI as desktop user fails before any WireGuard operation because `/etc/wireguard` config is root-only:

```console
$ wg-quicker up mx-que-mx1
[error] [config-load] failed to read /etc/wireguard/mx-que-mx1.conf: Error: EACCES: permission denied
ConfigError: Config file does not exist or is unreadable: /etc/wireguard/mx-que-mx1.conf
```

Retrying bare command through sudo fails at command lookup:

```console
$ sudo wg-quicker up mx-que-mx1
sudo: wg-quicker: command not found
```

On affected workstation,
`wg-quicker` resolves to workspace-local path:

```text
/var/home/user/Monochromatic/node_modules/.bin/wg-quicker
```

That directory is present in interactive user's `PATH` but absent from sudo command search path.

## Root cause

### CLI crossed privilege boundary too late

Before `e43e24afa`,
`package/wg/cli-quicker/src/index.ts:93-101` at `3b92eb5c8` loaded config directly in original process:

```ts
/**
 * Parsed config for the requested interface.
 */
const config = await loadConfig({
  arg: target,
  expandAllowedIps: subcommand === 'up',
},);

await (subcommand === 'up' ? up({ config, },) : down({ config, },));
```

Because `loadConfig()` ran before any privilege transition,
normal root-only mode on `/etc/wireguard/<interface>.conf` produced `EACCES`.

Current `package/wg/cli-quicker/src/index.ts:94-115` restores caller context,
delegates,
and reads config only in root child:

```ts
const processArguments = await restorePrivilegeContext();
// ...validate public arguments...
const delegated = await relaunchWithRootIfNeeded();
if (delegated)
  return;
const config = await loadConfig({
```

`package/wg/cli-quicker/src/privilege-context.ts:154-186` opens internal context with `O_NOFOLLOW`,
then checks regular-file type,
owner against `SUDO_UID`,
link count,
mode,
and bounded size before parsing and applying allowlisted values.
Context carries caller `HOME`,
cache locations,
`IPINFO_TOKEN`,
runtime directory,
exemption UID,
companion command,
and original path under private name.
It never assigns original path to privileged `PATH`.

### sudo intentionally replaces command search path

Installed sudo is `1.9.17p2`.
Source tag `v1.9.17p2` resolves to commit `d1b48c651cec19fe37d1f0d3299d2283fb0f88e4`.

`plugins/sudoers/sudoers.c:1110-1113` replaces user path with configured `secure_path` before resolution:

```c
if (def_secure_path && !user_is_exempt(ctx))
    path = def_secure_path;

ret = resolve_cmnd(ctx, cmnd_in, &cmnd_out, path, runchroot);
```

`plugins/sudoers/find_path.c:107-114` treats arguments containing slash as direct paths:

```c
if (strchr(infile, '/') != NULL) {
    if (strlcpy(command, infile, sizeof(command)) >= sizeof(command)) {
        errno = ENAMETOOLONG;
        debug_return_int(NOT_FOUND_ERROR);
    }
    found = cmnd_allowed(command, sizeof(command), runchroot, sbp,
        allowlist);
    goto done;
}
```

Bare names instead iterate selected path in `plugins/sudoers/find_path.c:121-139`:

```c
for (cp = sudo_strsplit(path, pathend, ":", &ep); cp != NULL;
    cp = sudo_strsplit(NULL, pathend, ":", &ep)) {

    /*
     * Search current dir last if it is in PATH.
     * This will miss sneaky things like using './' or './/' (XXX)
     */
    if (cp == ep || (*cp == '.' && cp + 1 == ep)) {
        checkdot = 1;
        continue;
    }
```

When no candidate exists,
`plugins/sudoers/sudoers.c:572-582` emits exact observed diagnostic:

```c
} else if (cmnd_status == NOT_FOUND) {
    if (ISSET(ctx->mode, MODE_CHECK)) {
        audit_failure(ctx, ctx->runas.argv, N_("%s: command not found"),
            ctx->runas.argv[1]);
        sudo_warnx(U_("%s: command not found"), ctx->runas.argv[1]);
    } else {
        audit_failure(ctx, ctx->runas.argv, N_("%s: command not found"),
            ctx->user.cmnd);
        sudo_warnx(U_("%s: command not found"), ctx->user.cmnd);
```

`plugins/sudoers/env.c:1043-1045` also installs secure path in privileged child environment:

```c
if (def_secure_path && !user_is_exempt(ctx)) {
    CHECK_SETENV2("PATH", def_secure_path, true, true);
    SET(didvar, DID_PATH);
}
```

Live environment probe confirmed same policy:

```console
$ WG_QUICKER_ENV_RESET_PROBE=present sudo -- <exact-node> --eval '<print probe and PATH>'
absent
/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/var/lib/snapd/snap/bin:/home/linuxbrew/.linuxbrew/bin
```

This is documented sudo policy behavior,
not sudo defect.
Local bugs were relying on bare workspace executable lookup,
assuming custom environment survived elevation,
and expanding `~` only after sudo changed caller context.

## Verification

### Version and source

```console
$ sudo --version
Sudo version 1.9.17p2
```

Source inspected from:

```text
https://github.com/sudo-project/sudo.git
v1.9.17p2
d1b48c651cec19fe37d1f0d3299d2283fb0f88e4
```

### Reproduction harness

Disposable mode-zero config reproduces non-root config seam without parsing a real tunnel or mutating networking:

```console
$ fixture_directory=$(mktemp --directory /tmp/wg-quicker-privilege.XXXXXXXX)
$ fixture_path="$fixture_directory/restricted.conf"
$ printf '[Interface]\nExemptMark = invalid\n' > "$fixture_path"
$ chmod 000 "$fixture_path"
$ node --input-type=module-typescript --eval \
  "const { loadConfig } = await import('./package/wg/cli-quicker/src/config.ts'); await loadConfig({ arg: process.argv[1], expandAllowedIps: true });" \
  "$fixture_path"
ConfigError: Config file does not exist or is unreadable: .../restricted.conf
```

Before fix,
the CLI called this seam directly.
Current CLI delegates before reaching it.
No command against `mx-que-mx1` was used for verification.

### Failing catalog before fix

- Non-root CLI plus root-only config:
   `EACCES` from `config-load`.
- `sudo` plus bare workspace bin name:
   `sudo: wg-quicker: command not found`.
- Preserving only user `PATH`:
   weakens sudo path isolation and still relies on ambient lookup order.

### Working catalog after fix

- Built CLI plus fake sudo in first `PATH` entry records exact Node executable,
  exact bundle path,
  original arguments,
  and complete allowlisted private context before config read.
- Real sudo with mode-zero config reaches parser rather than `EACCES`.
- Real sudo expands `AllowedIPsFromFiles = ~/...` against captured caller home,
  then fails on intentionally invalid fixture input before network mutation.
- Missing configured companion fails before interface creation.
- Built and root direct resolver calls select paired release binary outside sudo `secure_path`.
- Root CLI execution in disposable netns skips nested elevation and completes integration lifecycle.
- Exact paths containing slash bypass sudo path iteration by inspected and tested source path.

The regression test is
`package/wg/cli-quicker/src/privilege-launch.unit.test.ts`.
It executes built artifact and confirms fake sudo receives:

```text
-- <exact process.execPath> <exact dist/final/node/index.mjs> \
  --wg-quicker-privilege-context <private-file> up <config>
```

Fake sudo reads private file while parent retains it and confirms every allowlisted value.
`package/wg/cli-quicker/src/privilege-context-data.unit.test.ts` checks valid schema plus malformed JSON,
arrays,
versions,
UIDs,
unknown keys,
non-string values,
and invalid home.
`package/wg/cli-quicker/src/application-exemption-command.unit.test.ts` checks explicit path,
missing path,
captured caller path,
and paired repository release.

Verification commands:

```console
mise run //package/wg/cli-quicker:buildAndTest
mise run //package/wg/cli-quicker:lint:types
mise run //package/wg/cli-quicker:lint:oxlint
mise run //package/wg/cli-quicker:test:integration:bypass
```

All passed after fix.

## Verified workarounds

### Use package self-elevation

Current CLI detects non-root effective UID,
then creates private context and launches:

```text
sudo -- <exact Node executable> <exact CLI bundle> \
  --wg-quicker-privilege-context <private-file> <original arguments>
```

Sudo inherits terminal streams,
so password prompt works.
Root child receives `SUDO_UID`,
validates private file against it,
and restores only allowlisted values.
Application watcher uses sudo identity for desktop cgroup selection.
Source-file `~` expansion and ASN cache selection retain caller context.
Rust companion resolves to exact configured,
paired workspace,
privileged-path,
or captured-caller-path executable before network mutation.

Tradeoff:
this intentionally executes current user-owned workspace runtime,
bundle,
and selected paired or caller-path companion as root after sudo authentication.
That matches local-development workflow but is inappropriate when workspace integrity is not trusted.

### Install root-owned artifact and invoke exact path

An administrator can install runtime and CLI under root-owned directories,
then execute exact absolute path through sudo.

Tradeoff:
installation must be refreshed after builds,
and package currently targets repository-local development rather than published system package.

## What does not work

- `wg-quicker up <interface>` before fix:
  process lacks permission to read root-only config.
- `sudo wg-quicker up <interface>`:
  sudo resolves bare name through `secure_path`,
  not interactive user's workspace path.
- Copying interactive `PATH` wholesale into root process:
  removes sudo's intended command lookup isolation and permits earlier user-writable entries to shadow system tools.
- Relying on custom variables to survive default `env_reset`:
  loses home,
  cache,
  token,
  UID override,
  runtime override,
  and companion selection.
- Resolving `wg-quicker-exempt` only after interface setup:
  discovers missing companion after network mutation.
- Re-executing by bare `node` and bare `wg-quicker`:
  both names can disappear or resolve differently under `secure_path`.

## Upstream filing decision

No `.out-of-scope/` entry matches sudo command lookup.

Broad open and closed searches for `secure_path` found sudo tracker discussions,
including issue `#348` about whether path should reset by default
and pull request `#409` about configure-time secure path control.
Exact search for `secure_path` plus `command not found` found no matching issue or pull request.
No duplicate needs a comment because observed behavior matches documented policy.

Constraint check:

1. **Really upstream's fault:**
    No.
   Sudo deliberately applies `secure_path` and documents this behavior.
2. **Can upstream fix it:**
    Not applicable as defect.
   Changing default would weaken or alter established policy semantics.
3. **Supported use case:**
    Sudo supports both bare command search and explicit paths;
   workspace-specific path inheritance is not promised.
4. **Contribution welcome:**
    Not evaluated beyond public tracker because first constraint fails.
5. **Likely upstream fix:**
    No defect exists to fix.
6. **Compatible prototype:**
    Consumer-side exact-path relaunch is implemented and tested in this repository;
   no upstream patch is appropriate.

Upstream artifact:

```md
Nothing to file or comment upstream.
Sudo 1.9.17p2 behaves as documented by applying `secure_path` to bare command lookup.
The actionable defect was `wg-quicker` reading privileged config before elevation and relying on bare workspace command name.
```
