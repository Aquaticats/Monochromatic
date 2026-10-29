#!/usr/bin/env node
/**
 Run one native-interoperability driver in a bounded container.

 The native gate image has Git 2.56.0 and no Node; the host has Node and an older Git.
 A driver that needs both runs here:
 the gate's Git 2.56.0 base,
 the Node executable of the same Debian release,
 and the native executable built in release mode from this snapshot.
 The retired TypeScript wrapper is not part of the image; the drivers that ran against it, and the step that rebuilt
 it from its pinned revision, are recorded in `doc/handover/cli-git-native-interop.md`.

 Usage: `node bin/interop-native-container.ts <driver> [arguments...]`,
 where `<driver>` names a file under `native-interop/` without its `.ts` suffix.
 */
import {
  cp,
  mkdir,
  mkdtemp,
  mkdtempDisposable,
  readFile,
  writeFile,
} from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import {
  scannerSnapshotEntries,
  type SnapshotEntry,
  vendorLockedDependencies,
} from './native-scanner-snapshot.ts';
import {
  NativeVerificationError,
  runCommand,
} from './native-verification-process.ts';

// Scratch lives inside the package's gitignored node_modules/.monochromatic, never the system temporary directory,
// whose backing device is unknowable.
/**
 Parent of every scratch directory this runner creates.
 */
const scratchBase = join(
  import.meta.dirname,
  '..',
  'node_modules',
  '.monochromatic',
  'git-policy'
);
await mkdir(
  scratchBase,
  { recursive: true }
);

/**
 Concurrent worktrees set GIT_POLICY_NATIVE_IMAGE_TAG so one snapshot never runs another's image.
 */
const imageTag = process.env
  .GIT_POLICY_NATIVE_IMAGE_TAG
  ?? 'development';
/**
 Image this runner builds and runs.
 */
const interopImage = `localhost/git-policy-native-interop:${imageTag}`;
/**
 The audited Git 2.56.0 image the native gate also builds on.
 */
const gitBase = '6ec87f6d290a2f59bda5b3ffd4197058fe0749d02b4978c877e8edf6dc38802a';
/**
 Node of the gate base's Debian release (trixie), copied as one executable.
 */
const nodeImage = 'docker.io/library/node:24-trixie-slim';

/**
 Copy the native crate, its path dependencies and the interoperability drivers into the build context.
 Copies run concurrently, and every copy settles before this returns or throws.

 @param context - build context directory receiving the repository-shaped snapshot
 */
async function copySnapshot({ context, }: {
  readonly context: string;
},): Promise<void> {
  /**
   Where the wrapper crate lands in the repository-shaped snapshot.
   */
  const subject = join(
    context,
    'package/git-policy/cli',
  );
  /**
   Where the JSONC crate the wrapper links lands in the snapshot.
   */
  const dependency = join(
    context,
    'package/jsonc-edit/core-rust',
  );
  /**
   Everything the image builds from: the wrapper, its path dependencies, the scanner crates and the drivers.
   */
  const entries: readonly SnapshotEntry[] = [
    ...[
      'Cargo.toml',
      'Cargo.lock',
      'README.md',
      'src/native',
    ].map(function crateEntry(name) {
      return {
        from: resolve(name),
        to: join(
          subject,
          name,
        ),
      };
    }),
    ...[
      'Cargo.toml',
      'Cargo.lock',
      'src',
      'fixtures',
    ].map(function dependencyEntry(name) {
      return {
        from: resolve(
          '../../jsonc-edit/core-rust',
          name,
        ),
        to: join(
          dependency,
          name,
        ),
      };
    }),
    ...scannerSnapshotEntries({ context }),
    {
      from: resolve('../../../clippy.toml'),
      to: join(
        context,
        'clippy.toml',
      ),
    },
    {
      from: resolve('native-interop'),
      to: join(
        context,
        'native-interop',
      ),
    },
  ];
  /**
   Errors of the copies that failed; a failed copy never stops the others, so none is still in flight afterwards.
   */
  const failures: unknown[] = [];
  await Promise.all(entries.map(async function copyEntry(entry): Promise<void> {
    try {
      await cp(
        entry.from,
        entry.to,
        { recursive: true },
      );
    }
    catch (error) {
      failures.push(error);
    }
  }));
  if (failures.length > 0)
    throw new NativeVerificationError(
      `Copying ${String(failures.length)} snapshot entries into ${context} failed.`,
      { cause: failures },
    );
}

/**
 Characters a driver name may hold: lowercase words and dashes.
 */
const driverNameCharacters = 'abcdefghijklmnopqrstuvwxyz0123456789-';

/**
 Prefix of the crate version line in `Cargo.toml`.
 */
const versionLinePrefix = 'version = "';

/**
 Whether text names a driver under `native-interop/`.

 @param text - requested driver name
 
 @returns whether it is non-empty and holds only lowercase letters, digits and dashes
 */
function isDriverName(text: string): boolean {
  /**
   The name split into single characters, each checked against the allowed set.
   */
  const characters = text
    .split('');
  return (characters.length > 0)
    && characters.every(function isAllowed(character) {
      return driverNameCharacters.includes(character);
    });
}

/**
 The crate version a `Cargo.toml` declares.

 @param manifest - text of `Cargo.toml`
 
 @returns the first `version = "..."` line's value under `version`, no `version` when no line has that shape
 */
function crateVersionOf(manifest: string): { readonly version?: string; } {
  /**
   The first line that declares the crate version.
   */
  const line = manifest
    .split('\n')
    .find(function isVersionLine(candidate) {
      return candidate.startsWith(versionLinePrefix) && candidate.endsWith('"');
    });
  if (line === undefined)
    return {};
  return {
    version: line.slice(
      versionLinePrefix.length,
      -1,
    ),
  };
}

// Build the image once per call, then run the named driver in it under the gate's bounds without a PID cap.
/**
 Driver name and the arguments passed on to it.
 */
const [driver, ...driverArguments] = process.argv
  .slice(2);
if ((driver === undefined) || (!isDriverName(driver)))
  throw new NativeVerificationError('Name one driver under native-interop/ in lowercase words and dashes.');
/**
 Disposable build context that holds the snapshot, the vendored sources and the Containerfile.
 */
await using context = await mkdtempDisposable(join(
  scratchBase,
  'cli-git-native-interop-',
));
/**
 Directory under the package's `target/` that keeps the evidence of every interoperability run.
 */
const evidenceRoot = resolve('target/verification');
await mkdir(
  evidenceRoot,
  { recursive: true },
);
/**
 Evidence directory of this run.
 */
const evidence = await mkdtemp(join(
  evidenceRoot,
  'interop-',
));
await copySnapshot({ context: context.path });
await vendorLockedDependencies({ context: context.path });
/**
 Crate version recorded in the manifest, absent when `Cargo.toml` has no version line.
 */
const { version: nativeCrateVersion, } = crateVersionOf(await readFile(
  resolve('Cargo.toml'),
  'utf8',
));
await writeFile(
  join(
    context.path,
    'Containerfile',
  ),
  [
    `FROM ${gitBase}`,
    'COPY package /work/package',
    'COPY --chown=1000:1000 vendor /work/vendor',
    'COPY --chown=1000:1000 cargo-config /work/.cargo',
    'COPY clippy.toml /work/clippy.toml',
    'RUN ["mkdir", "--parents", "/home/tester/.cargo", "/work/scratch"]',
    'RUN ["chown", "--recursive", "1000:1000", "/work/package", "/work/scratch", "/home/tester"]',
    'ENV HOME=/home/tester CARGO_HOME=/home/tester/.cargo CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0',
    `COPY --from=${nodeImage} /usr/local/bin/node /usr/local/bin/node`,
    'USER 1000:1000',
    'WORKDIR /work/package/git-policy/cli',
    'RUN ["cargo", "build", "--offline", "--locked", "--release", "--bin", "cli-git-native"]',
    'USER 0:0',
    // Copied after the compilation steps so editing a driver never rebuilds the native executable.
    'COPY native-interop /fixture/native-interop',
    'RUN ["mkdir", "--parents", "/opt/native/bin"]',
    // The native executable gets its `git` name only inside this disposable image, never on the host.
    'RUN ["ln", "--symbolic", "/work/package/git-policy/cli/target/release/cli-git-native", "/opt/native/bin/git"]',
    'USER 1000:1000',
    'WORKDIR /work/scratch',
    '',
  ].join('\n'),
);
await runCommand({
  command: 'podman',
  args: [
    'build',
    '--network=none',
    '--http-proxy=false',
    '--pull=never',
    '--memory=2g',
    '--cpu-period=100000',
    '--cpu-quota=200000',
    '--tag',
    interopImage,
    context.path,
  ],
});
/**
 Result of inspecting the built image for its ID.
 */
const inspected = await runCommand({
  command: 'podman',
  args: [
    'image',
    'inspect',
    interopImage,
    '--format',
    '{{.Id}}',
  ],
  capture: true,
});
/**
 Content-addressed ID of the built image, so the run uses exactly the image just built.
 */
const image = inspected.stdout
  .trim();
/**
 Bounds of the driver container: removed on exit, no network, 2 GiB and 2 CPUs, with no process cap.
 */
const limits = [
  '--rm',
  '--init',
  '--network=none',
  '--memory=2g',
  '--cpus=2',
];
await writeFile(
  join(
    evidence,
    'manifest.json',
  ),
  `${JSON.stringify(
    {
      gitBase,
      nodeImage,
      nativeCrateVersion,
      image,
      limits,
      driver,
      driverArguments,
      user: '1000:1000',
    },
    null,
    2,
  )}\n`,
);
console.log(`Native interoperability evidence: ${evidence}`);
/**
 Outcome of the driver, whose output is kept as evidence and echoed.
 */
const run = await runCommand({
  command: 'podman',
  args: [
    'run',
    ...limits,
    image,
    'node',
    `/fixture/native-interop/${driver}.ts`,
    ...driverArguments,
  ],
  capture: true,
  allowFailure: true,
});
await writeFile(
  join(
    evidence,
    'stdout.txt',
  ),
  run.stdout,
);
await writeFile(
  join(
    evidence,
    'stderr.txt',
  ),
  run.stderr,
);
process.stdout
  .write(run.stdout);
process.stderr
  .write(run.stderr);
if (run.status !== 0)
  throw new NativeVerificationError(
    `Driver ${driver} exited with ${String(run.status ?? 'no status')} (signal ${run.signal ?? 'none'}).`,
  );
