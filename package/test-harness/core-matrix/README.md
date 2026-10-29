# @monochromatic-dev/test-harness-core-matrix

Test matrix runner.
Runs test files across a cartesian product of OS,
 user context,
 and JS runtime
using podman containers or directly on the host.

## Usage

```ts
import { matrix, } from '@monochromatic-dev/test-harness-core-matrix';

await matrix({
  os: ['container:cachyos', 'container:ubuntu',],
  user: ['root', 'user',],
},);
```

This replaces manual container orchestration loops with a single function call.
The package handles container lifecycle,
 prerequisite installation,
runtime setup,
 user creation,
 and result collection via `describe`/`it`
from `@monochromatic-dev/test-harness-core`.

## API

### `matrix(options)`

Generates a cartesian product of `files x os x user x runtime`,
filters out excluded combinations,
 and executes each via the appropriate backend.

All combinations run concurrently by default.
Each combination is fully isolated (separate container or separate process),
so concurrency is safe without additional coordination.

**Parameters:
**

- **`os`** (required):
   array of OS specifications with protocol prefix.
  - `container:`:
     podman container (e.g. `'container:ubuntu'`,
     `'container:cachyos'`)
  - `host:`:
     run directly on the host,
     no container (e.g. `'host:'`)
  - `vm:`:
     reserved for mvm,
     not yet implemented

- **`files`**:
   array of file paths to execute inside each environment.
  Defaults to discovering `*.unit.matrix.test.ts` in the current directory.

- **`user`**:
   array of user contexts.
   `'root'` runs as root,
  `'user'` creates a non-root user (uid 1000) with passwordless sudo.
  Defaults to `['root']`.

- **`runtime`**:
   array of JS runtimes to install.
  Supported:
   `'bun'`,
   `'deno'`,
   `'node'`.
   Defaults to `['bun']`.
  `'node'` is installed through mise from the repository config mounted at `/workspace`,
  so the version comes from `mise.toml`/`mise.lock` at run time.

- **`exclude`**:
   array of partial match objects to exclude from the cartesian product.
  All specified fields must match for a combination to be excluded.
  Each field accepts an exact value or a predicate function.

- **`concurrency`**:
   maximum number of combinations to run concurrently.
  Defaults to `4`.
   Caps how many containers or host processes execute at once
  to avoid saturating CPU and memory on large matrices.

  Set to `1` for sequential execution (debugging or resource-constrained hosts).

## Concurrency

All combinations launch concurrently (one podman container or host process each),
capped at `concurrency` (default 4) simultaneous executions via `p-limit`.
This is safe because each combination runs in its own isolated environment.

For large matrices,
 increase or decrease `concurrency` to match available resources:

```ts
await matrix({
  os: ['container:cachyos', 'container:ubuntu', 'container:alpine',],
  user: ['root', 'user',],
  runtime: ['bun', 'deno', 'node',],
  concurrency: 8,
},);
```

## Protocols

### `container:`: podman containers

Runs test files inside ephemeral podman containers.
Handles prerequisite installation,
 runtime installation,
 and user creation.

```ts
await matrix({
  os: ['container:cachyos', 'container:ubuntu',],
  user: ['root', 'user',],
  runtime: ['bun',],
},);
```

To run a Node package's `*.unit.matrix.test.ts` files,
select the `node` runtime:

```ts
await matrix({
  os: ['container:cachyos', 'container:ubuntu',],
  user: ['root', 'user',],
  runtime: ['node',],
},);
```

The node runtime bootstraps mise with `curl -fsSL https://mise.run | sh`,
adds `$HOME/.local/bin` to `PATH`,
changes to `/workspace`,
and runs
`MISE_QUIET=1 MISE_TERMINAL_PROGRESS=false MISE_SOPS_STRICT=false MISE_TRUSTED_CONFIG_PATHS=/workspace mise install node`.
Node's version is read from the mounted repository's `mise.toml` and `mise.lock`
at run time;
this package hardcodes no Node version.

Test files execute as
`MISE_QUIET=1 MISE_TERMINAL_PROGRESS=false MISE_SOPS_STRICT=false MISE_TRUSTED_CONFIG_PATHS=/workspace $HOME/.local/bin/mise exec node -- node <file>`.
The `MISE_SOPS_STRICT=false` flag means a checkout that carries the gitignored,
age-encrypted `.env.local.json`
does not fail in a container that has no age key;
the quiet flags keep progress output out of the captured UTF-8 stream.

Node's official Linux builds require glibc and `libatomic`.
Before installing Node the runner installs the distro's atomic library
(`libatomic1` on apt,
`libatomic` on dnf,
`gcc-libs` on pacman,
which covers both Arch and CachyOS).
Alpine is rejected for the node runtime
because its musl libc cannot load the glibc Node builds.

The runner installs only Node;
package-specific tests install tools such as `sops` and `age` themselves,
using the same mise bootstrap.

### `host:`: direct execution

Runs test files directly on the host machine.
No container,
 no prerequisite installation,
 no user creation.
The specified runtime must already be installed and available on PATH.

The `user` axis still generates combinations (for labeling and matrix completeness)
but does not affect execution;
 the process runs as the current user.
Use `exclude` to filter out inapplicable user combinations.

```ts
await matrix({
  os: ['host:',],
  user: ['root', 'user',],
  runtime: ['bun',],
  exclude: [
    {
      os: 'host:',
      user: function notUser(user,) {
        return user !== 'user';
      },
    },
  ],
},);
```

### Mixing protocols

Container and host entries can coexist in the same matrix:

```ts
await matrix({
  os: ['host:', 'container:cachyos', 'container:ubuntu',],
  user: ['root', 'user',],
  runtime: ['bun',],
  exclude: [
    {
      os: 'host:',
      user: function notUser(user,) {
        return user !== 'user';
      },
    },
  ],
},);
```

This runs 5 combinations:
 1 host (user/bun) + 4 container (2 OS x 2 users).

## Supported distros

The package manager is detected from the distro name:

- **apt**:
   ubuntu,
   debian
- **dnf**:
   fedora,
   centos,
   rhel,
   rocky,
   alma
- **apk**:
   alpine
- **pacman**:
   arch,
   cachyos

### Container images

A distro name becomes the image reference `<distro>:latest`
(`ubuntu` runs `ubuntu:latest`,
 `fedora:39` runs `fedora:39`).
`cachyos` is the exception:
it runs the CachyOS image for the host CPU level,
chosen once when `matrix()` starts (see "CachyOS base image selection").
`cachyos:<tag>` is an explicit choice:
it runs `docker.io/cachyos/cachyos:<tag>`,
the plain image,
and skips the CPU-level choice.

`cachyos` is the repository's primary matrix target:
list `'container:cachyos'` first in the `os` array,
and keep the other distros to test portability.

```ts
await matrix({
  os: ['container:cachyos', 'container:ubuntu',],
  user: ['root', 'user',],
},);
```

Fedora was dropped from the repository's own matrices on 2026-10-29 (its package mirrors were too slow to test against).
The `fedora` and other `dnf` distro mappings stay,
so an explicit `'container:fedora'` still resolves to `fedora:latest` and `dnf`.

The package picks no default `os`:
the option is required,
so each consumer chooses its own list.

A `container:cachyos` cell is a pacman test,
so on a host that is not x86-64 `matrix()` throws `NoAlpineVariantError` before running anything.
List `container:alpine` explicitly in the portability rows instead.

## CachyOS base image selection

Per the owner's 2026-10-29 decision (`doc/decision/container-base-distro.md`),
containers this repository builds start from the most optimized CachyOS image the host CPU runs,
unpinned.
`resolveBaseImage` picks it:

- `docker.io/cachyos/cachyos-v4:latest` when the glibc loader reports `x86-64-v4 (supported, searched)`
- `docker.io/cachyos/cachyos-v3:latest` when it reports `x86-64-v3 (supported, searched)` and not v4
- `docker.io/cachyos/cachyos:latest` otherwise,
  including when the loader is missing or prints no levels
  (the reason is logged at warn level)
- `docker.io/library/alpine:latest` when `process.arch` is not `x64`
  and the caller says its image has an Alpine variant;
  without one,
  `NoAlpineVariantError` names the image and says no Alpine variant exists

Docker Hub has no v2 image.
The package writes no Alpine Containerfiles:
a caller that passes `alpineVariant: true` owns its Alpine build.

```ts
import {
  cachyosBuildArguments,
  resolveBaseImage,
} from '@monochromatic-dev/test-harness-core-matrix/ts';

// \{ image: 'docker.io/cachyos/cachyos-v4:latest', kind: 'cachyos-v4', reason: '...' \}
const resolution = await resolveBaseImage({ imageName: 'terminal', alpineVariant: false, },);

// ['--build-arg', 'CACHYOS_IMAGE=docker.io/cachyos/cachyos-v4:latest']
const args = await cachyosBuildArguments({ imageName: 'terminal', alpineVariant: false, },);
```

A Containerfile declares `ARG CACHYOS_IMAGE` before its `FROM ${CACHYOS_IMAGE}`.
`resolveBaseImage`, `detectCachyosLevel` and `readLoaderOutput` take injected architecture, loader output and logger,
so every branch is testable without depending on the host.

Shell callers use the `cachyos-image` bin:

```sh
podman build --build-arg "$(cachyos-image --image-name terminal --build-argument)" --tag terminal .
cachyos-image --image-name terminal --alpine-variant
```

It prints the image (or `CACHYOS_IMAGE=<image>` with `--build-argument`) on stdout,
sends the logged reasoning to stderr,
and exits 1 when no base image exists for the host,
2 for a usage error.

### How the CPU level is detected

The loader report comes from `/lib64/ld-linux-x86-64.so.2 --help`
(then `/lib/ld-linux-x86-64.so.2`),
the method of the [CachyOS repository installer][cachyos-repo-script]
(`check_supported_isa_level` greps `x86-64-v4 (supported, searched)` in the loader's `--help`),
which the [CachyOS wiki's optimized repositories page][cachyos-wiki-repos] documents as
`/lib/ld-linux-x86-64.so.2 --help | grep supported`.
The installer chooses its v4 repository on that line alone and its v3 repository otherwise.

The wiki page also warns that Intel 12th generation and newer CPUs may report v4 support
yet "cannot run AVX-512 instructions",
and says to treat them as v3.
The installer script does not act on that warning,
and neither does this package:
it follows the loader's report and adds no CPU model check,
so a host whose loader wrongly reports v4 would build and run the v4 image and fail at run time.

[cachyos-repo-script]: https://mirror.cachyos.org/cachyos-repo.tar.xz
[cachyos-wiki-repos]: https://wiki.cachyos.org/features/optimized_repos/

## Supported runtimes

- **bun**:
   in containers,
   installed via `curl -fsSL https://bun.sh/install | bash`;
   on host,
   must be on PATH
- **deno**:
   in containers,
   installed via `curl -fsSL https://deno.land/install.sh | sh`;
   on host,
   must be on PATH
- **node**:
   in glibc containers,
   bootstrapped through mise (`curl -fsSL https://mise.run | sh`),
   with the version resolved from the mounted repository's
   `mise.toml`/`mise.lock` by `mise install node`;
   on host,
   must be on PATH.
   Alpine is rejected because Node's official Linux builds are glibc-only.

## What the package handles per combination

### Container combinations

1. Detect package manager from distro name (cachyos becomes pacman)
2. `podman run --rm -v ${monorepoRoot}:/workspace:Z ubuntu:latest sh -c "..."`
3. Inside the container:
   - Install prerequisites (curl,
      unzip,
      optionally sudo and bash)
   - Install runtime system libraries (for node:
     the distro's atomic library)
   - If `user` context:
      create non-root user with passwordless sudo
   - Install the selected runtime
   - Execute each file with the runtime
4. Report pass/fail through `describe`/`it` from `module-test`

### Host combinations

1. Spawn the runtime binary with the test file path
2. Report pass/fail through `describe`/`it` from `module-test`

## Errors

- `discoverTestFiles` throws `"No matrix test files (*.unit.matrix.test.ts) found in ${cwd}"`
  when no files match the default discovery glob and no explicit `files` option was provided.
- `matrix()` throws `"vm: protocol not yet implemented"` when any `os` entry uses the `vm:` protocol.
- `parseOs` throws when the OS specification is missing a protocol prefix or uses an unknown protocol.
- `detectPackageManager` throws when the distro name does not match any known distro.
- `runtimeDependencyCommand` throws for the `node` runtime on an `apk` (Alpine) image,
  naming the glibc requirement and the remedy (exclude node combinations or use a glibc distro).

## File naming convention

Inner test files are named `*.unit.matrix.test.ts`.
The standard `test:unit` mise task discovers `*.unit.test.*` and skips these.
Test orchestrators discover them via the `files` option or default glob.

## Consumer example

Before (218-line orchestrator):

```ts
// mise.container-test.ts -- manual matrix, podman args, result collection
const MATRIX = [
  { image: 'ubuntu:latest', asRoot: true, preInstall: '...', },
  // ... 4 entries with duplicated install commands
];
for (const entry of MATRIX) {
  // build command, spawn podman, collect results...
}
```

After:

```ts
import { matrix, } from '@monochromatic-dev/test-harness-core-matrix';

await matrix({
  os: ['container:cachyos', 'container:ubuntu',],
  user: ['root', 'user',],
},);
```

## Excluding combinations

### Exact match

```ts
await matrix({
  os: ['container:cachyos', 'container:ubuntu', 'container:alpine',],
  user: ['root', 'user',],
  exclude: [
    { os: 'container:alpine', user: 'user', },
  ],
},);
```

This runs 5 combinations instead of 6,
excluding Alpine with non-root user.

### Predicate functions

Each field in an exclude entry accepts a function predicate
that returns `true` when the combination should be excluded:

```ts
await matrix({
  os: ['host:', 'container:ubuntu',],
  user: ['root', 'user',],
  exclude: [
    {
      os: 'host:',
      user: function notUser(user,) {
        return user !== 'user';
      },
    },
  ],
},);
```

## Dependencies

- `@monochromatic-dev/test-harness-core`:
   `describe`/`it` for execution and reporting
- `@monochromatic-dev/logger-core`:
   tagged logger
- `nano-spawn`:
   process execution (podman and host runtimes)
- `@monochromatic-dev/fs-path-core`:
   monorepo root detection
