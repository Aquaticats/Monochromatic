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
 fixture tree that outgrows the ceiling fails with `ENOSPC` naming the fixture root instead of ending the run with an
 out-of-memory kill. The integration pass of 2026-11-19 ran a whole suite on this root: its kernel `memory.peak` was
 1123.8 MiB of the gate's 2048 MiB bound, while the highest charge any sample read, with 3 s between iterations, was
 953.1 MiB, 170.7 MiB below that peak, so its figures are the samples' and not the run's. What the bound leaves
 beside the kernel's peak is 924.2 MiB, and this ceiling sits 156.2 MiB below that, at 38% of the bound, so a run
 whose other pages hold to that peak meets the ceiling before the memory bound (the record is
 `perf/gate-resource-peaks-2026-11-19-integration-pass.jsonl`, read in
 `doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md`).
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

 Measured by the integration pass of 2026-11-19, 211 samples taken with a 3-second delay between iterations across
 780 s and kept in `perf/gate-resource-peaks-2026-11-19-integration-pass.jsonl`: the whole gate suite's kernel
 `memory.peak` was 1123.8 MiB of the gate's 2048 MiB bound, and the highest charge any sample read was 953.1 MiB,
 170.7 MiB below that peak, so the samples' figures are the samples' and not the run's. The bound beside the kernel's
 peak leaves 924.2 MiB for fixtures, and this ceiling sits 156.2 MiB below it, at 3.94 times the 195.1 MiB sampled
 fixture peak, so a run whose other pages hold to that peak meets the ceiling before the memory bound. A gate review's
 run on the same code apart from the repaired location controls read 916.4 MiB of `memory.peak` and a 221.4 MiB
 sampled fixture peak, of which this ceiling is 3.47 times.
 */
export const gateFixtureRootSize: string = '768m';

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
