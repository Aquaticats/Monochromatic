#!/usr/bin/env node
/**
 Verify native wrapper modules against the selected real Git in an isolated source snapshot.
 */
import {
  cp,
  mkdir,
  mkdtemp,
  mkdtempDisposable,
  writeFile,
} from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import {
  buildLimitArguments,
  parallelJobs,
  parseContainerLimitOptions,
  runLimitArguments,
} from './container-limits.ts';
import {
  linterSnapshotEntries,
  vendorLinterDependencies,
} from './native-linter-snapshot.ts';
import { copyNextest, } from './native-nextest-snapshot.ts';
import {
  scannerSnapshotEntries,
  type SnapshotEntry,
  vendorLockedDependencies,
} from './native-scanner-snapshot.ts';
import { copyStraceSource, } from './native-strace-snapshot.ts';
import { testContainerfile, } from './native-test-containerfile.ts';
import {
  NativeVerificationError,
  runCommand,
} from './native-verification-process.ts';

// Resource limits default to the repository's test-container bounds; `--memory`, `--cpus` and `--pids-limit` override them.
/**
 Container limits of this gate, and the arguments left over, which are a usage error.
 */
const {
  limits: containerLimits,
  rest: unknownOptions,
} = parseContainerLimitOptions({
  options: process.argv
    .slice(2),
  acceptPidsLimit: true,
  leadingOnly: false,
});
if (unknownOptions.length > 0)
  throw new NativeVerificationError(`Only --memory <size>, --cpus <count> and --pids-limit <count> are accepted, got ${unknownOptions.join(' ')}.`);

// Scratch lives inside the package's gitignored node_modules/.monochromatic, never the system temporary directory,
// whose backing device is unknowable.
/**
 Parent of every scratch directory this gate creates.
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
 Source snapshot image this gate builds, tests, and lints.
 */
const testImage = `localhost/git-policy-native-test:${imageTag}`;

/**
 Copy the native crate and its path dependency into the build context.
 Copies run concurrently, and every copy settles before this returns or throws,
 so removing the context afterwards never races an in-flight copy.

 @param context - build context directory receiving the repository-shaped snapshot
 */
async function copySourceSnapshot({ context, }: {
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
   Everything the gate image builds from: the wrapper, its path dependencies and the repository files its tests read.
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
    ...linterSnapshotEntries({ context }),
    {
      from: resolve('../../../clippy.toml'),
      to: join(
        context,
        'clippy.toml',
      ),
    },
    // The configuration control in `src/native/config_repository_tests.rs` reads the repository's own file.
    {
      from: resolve('../../../cli-git.config.jsonc'),
      to: join(
        context,
        'cli-git.config.jsonc',
      ),
    },
  ];
  /**
   Result of every copy; a failed copy never stops the others, so none is still in flight afterwards.
   */
  const copies = await Promise.allSettled(entries.map(function copyEntry(entry) {
    return cp(
      entry.from,
      entry.to,
      { recursive: true },
    );
  }));
  /**
   Reasons of the copies that failed.
   */
  const failures: unknown[] = [];
  for (const copy of copies) {
    if (copy.status === 'rejected')
      failures.push(copy.reason);
  }
  if (failures.length > 0)
    throw new NativeVerificationError(
      `Copying ${String(failures.length)} source snapshot entries into ${context} failed.`,
      { cause: failures },
    );
}

// Build one mount-free source image, then run tests and Clippy against that exact image identity.
/**
 Disposable build context that holds the snapshot, the vendored sources and the Containerfile.
 */
await using context = await mkdtempDisposable(join(
  scratchBase,
  'cli-git-native-test-',
));
/**
 Directory under the package's `target/` that keeps the evidence of every gate run.
 */
const evidenceRoot = resolve('target/verification');
await mkdir(
  evidenceRoot,
  { recursive: true },
);
/**
 Evidence directory of this gate run.
 */
const evidence = await mkdtemp(join(
  evidenceRoot,
  'native-',
));
// The audited Git 2.56.0 image is the rewrite's selected native consumer baseline.
/**
 Content-addressed ID of the audited Git 2.56.0 base image.
 */
const base = '6ec87f6d290a2f59bda5b3ffd4197058fe0749d02b4978c877e8edf6dc38802a';
await copySourceSnapshot({ context: context.path });
await vendorLockedDependencies({ context: context.path });
await vendorLinterDependencies({ context: context.path });
await copyNextest({ context: context.path });
await copyStraceSource({ context: context.path });
await writeFile(
  join(
    context.path,
    'Containerfile',
  ),
  testContainerfile({
    base,
    jobs: parallelJobs(containerLimits),
  }),
);
await runCommand({
  command: 'podman',
  args: [
    'build',
    '--network=none',
    '--http-proxy=false',
    '--pull=never',
    ...buildLimitArguments(containerLimits),
    '--tag',
    testImage,
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
    testImage,
    '--format',
    '{{.Id}}',
  ],
  capture: true,
});
/**
 Content-addressed ID of the built image, so the run and Clippy steps use exactly the tested snapshot.
 */
const image = inspected.stdout
  .trim();
/**
 Bounds of every gate container: removed on exit, no network, and the memory, CPUs and processes of the container limits.
 */
const limits = [
  '--rm',
  '--init',
  '--network=none',
  ...runLimitArguments({
    limits: containerLimits,
    withPidsLimit: true,
  }),
];
await writeFile(
  join(
    evidence,
    'manifest.json',
  ),
  `${JSON.stringify(
    {
      base,
      image,
      limits,
      user: '1000:1000',
    },
    null,
    2,
  )}\n`,
);
console.log(`Native wrapper verification evidence: ${evidence}`);
await runCommand({
  command: 'podman',
  args: [
    'run',
    ...limits,
    image,
  ],
});
/**
 Result of asking the host's `rustc` for its sysroot.
 */
const sysroot = await runCommand({
  command: 'rustc',
  args: [
    '--print',
    'sysroot',
  ],
  capture: true,
});
/**
 Host Rust toolchain directory that the Clippy step mounts read-only.
 */
const compiler = sysroot.stdout
  .trim();
await runCommand({
  command: 'podman',
  args: [
    'run',
    ...limits,
    '--security-opt',
    'label=disable',
    '--volume',
    `${compiler}:/toolchain:ro`,
    '--env',
    'PATH=/toolchain/bin:/usr/local/cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin',
    '--env',
    'RUSTC=/toolchain/bin/rustc',
    '--env',
    'RUSTDOC=/toolchain/bin/rustdoc',
    image,
    '/toolchain/bin/cargo',
    'clippy',
    '--offline',
    '--locked',
    '--all-targets',
    '--',
    '-D',
    'warnings',
  ],
});
await writeFile(
  join(
    evidence,
    'passed.json',
  ),
  `${JSON.stringify({
    tests: true,
    clippy: true,
  })}\n`,
);
