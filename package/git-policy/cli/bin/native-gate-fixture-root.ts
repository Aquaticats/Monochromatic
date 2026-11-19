/**
 The fixture root the gate's test container makes for itself, and the arguments that put it there.

 Why a tmpfs and not the container's own tree: the tests' fixtures inherit the container root's file system, which is
 an overlay of the image and, on a rootless podman whose storage sits on a ZFS dataset, a user-space
 `fuse.fuse-overlayfs` mount over that dataset. The mount records no birth time, is on neither of the product's
 local-file-system lists, and makes Git refuse `core.fsmonitor`, and the ZFS dataset under it refuses a name that is
 not UTF-8 under `utf8only=on`; the causes measured behind those four facts failed 185 tests there
 (`doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md`). A tmpfs answers all four the way the kernel
 overlay of the earlier btrfs host did, and the tests already read the variable that moves their root.

 Why the root stays below the image's `/work` tree instead of a fresh `/gate-fixtures`: a fixture crate that runs
 Cargo takes the configuration of the directories above it, and the image's vendored-source mapping lives in
 `/work/.cargo/config.toml`. The root this replaces (`target/tmp` and `target/test-fixtures`) was below `/work`, so
 its fixture crates resolved their dependencies from that mapping offline; a root outside `/work` would leave the
 two `package_repair_prewarm_job` controls without a source for their `serde` dependency
 (`doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md`).

 The size is a ceiling under the container's memory bound: tmpfs pages count against the container's memory, so a
 fixture tree that outgrows the ceiling fails with `ENOSPC` naming the fixture root. That keeps much more of the
 container's memory for its processes than the fixtures take, which is why the size is far below the memory bound
 (`doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md`, the section on the measured whole-suite run).
 */
import { imagePackageRoot, } from './native-layered-build.ts';
import { subjectCrate, } from './native-wrapper-snapshot.ts';

/**
 The directory inside the container the tests create their fixtures under.

 It is a child of the image's `/work` tree, so a fixture crate below it reads the same `/work/.cargo/config.toml` its
 ancestors gave the root this replaces; the image keeps the package under `imagePackageRoot`, and the build's
 `target` directory holds the mount.
 */
export const gateFixtureRoot: string = `${imagePackageRoot}/${subjectCrate}/target/gate-fixtures`;

/**
 Size of the tmpfs mounted at {@link gateFixtureRoot}.

 Measured 2026-11-19: the whole gate suite peaked at 222 MiB of fixture bytes on this tmpfs (the run recorded in
 `doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md`), so this ceiling leaves 1278 MiB of headroom,
 and the suite's own container peaked at 1192 MiB of its 2 GiB bound.
 */
export const gateFixtureRootSize: string = '1500m';

/**
 Permission bits of {@link gateFixtureRoot}: every test process of the container writes below one root, whatever user
 the image gives it.
 */
export const gateFixtureRootMode: string = '1777';

/**
 Options of the tmpfs mount, in the order `podman run --tmpfs` takes them.
 */
export const gateFixtureRootMountOptions: string = `rw,size=${gateFixtureRootSize},mode=${gateFixtureRootMode}`;

/**
 Environment variable that moves the fixture root of the Rust tests.

 The name is the one both faces of the suite read: `FIXTURE_ROOT_VARIABLE` in
 `src/native/testing/test_support.rs` for the unit tests and in `src/native/binary/binary_support_fixture_directories.rs`
 for the binary controls.
 */
export const gateFixtureRootVariable: string = 'CLI_GIT_TEST_ROOT';

/**
 Arguments of `podman run` that mount the fixture tmpfs and point the tests at it.

 @returns the mount and environment arguments, to spread into the suite's `podman run`

 @example
 ```ts
 podman run ...limits, ...fixtureRootArguments(), image
 ```
 */
export function fixtureRootArguments(): readonly string[] {
  return [
    '--tmpfs',
    `${gateFixtureRoot}:${gateFixtureRootMountOptions}`,
    '--env',
    `${gateFixtureRootVariable}=${gateFixtureRoot}`,
  ];
}
