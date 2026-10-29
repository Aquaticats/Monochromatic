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
  linterCopyLines,
  linterSnapshotEntries,
  linterStageLines,
  vendorLinterDependencies,
} from './native-linter-snapshot.ts';
import {
  copyNextest,
  nextestCopyLines,
} from './native-nextest-snapshot.ts';
import {
  scannerSnapshotEntries,
  vendorLockedDependencies,
} from './native-scanner-snapshot.ts';
import {
  NativeVerificationError,
  runCommand,
} from './native-verification-process.ts';
import { mkdirSync as makeScratchBase } from 'node:fs';

// Scratch lives inside the package's gitignored node_modules/.monochromatic, never the system temporary directory,
// whose backing device is unknowable.
const scratchBase = join(
  import.meta.dirname,
  '..',
  'node_modules',
  '.monochromatic',
  'git-policy'
);
makeScratchBase(
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

 @param request - build context directory receiving the repository-shaped snapshot
 */
async function copySourceSnapshot({ context }: { context: string }) {
  const subject = join(
    context,
    'package/git-policy/cli',
  );
  const dependency = join(
    context,
    'package/jsonc-edit/core-rust',
  );
  const entries = [
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
  const copies = await Promise.allSettled(entries.map(function copyEntry(entry) {
    return cp(
      entry.from,
      entry.to,
      { recursive: true },
    );
  }));
  const failures = copies.filter(function isRejected(copy) {
    return copy.status === 'rejected';
  });
  if (failures.length > 0)
    throw new NativeVerificationError(
      `Copying ${String(failures.length)} source snapshot entries into ${context} failed.`,
      { cause: failures },
    );
}

// Build one mount-free source image, then run tests and Clippy against that exact image identity.
await using context = await mkdtempDisposable(join(
  scratchBase,
  'cli-git-native-test-',
));
const evidenceRoot = resolve('target/verification');
await mkdir(
  evidenceRoot,
  { recursive: true },
);
const evidence = await mkdtemp(join(
  evidenceRoot,
  'native-',
));
// The audited Git 2.56.0 image is the rewrite's selected native consumer baseline.
const base = '6ec87f6d290a2f59bda5b3ffd4197058fe0749d02b4978c877e8edf6dc38802a';
await copySourceSnapshot({ context: context.path });
await vendorLockedDependencies({ context: context.path });
await vendorLinterDependencies({ context: context.path });
await copyNextest({ context: context.path });
await writeFile(
  join(
    context.path,
    'Containerfile',
  ),
  [
    // The Markdown policy's linter is built from this snapshot in its own stage; only its executable is kept.
    ...linterStageLines({ base }),
    `FROM ${base}`,
    ...nextestCopyLines(),
    ...linterCopyLines(),
    'COPY package /work/package',
    // Some crate archives carry files only their owner may read, so the tester must own the vendored copy.
    'COPY --chown=1000:1000 vendor /work/vendor',
    'COPY --chown=1000:1000 cargo-config /work/.cargo',
    'COPY clippy.toml /work/clippy.toml',
    'COPY cli-git.config.jsonc /work/cli-git.config.jsonc',
    'RUN ["mkdir", "--parents", "/home/tester/.cargo"]',
    'RUN ["chown", "--recursive", "1000:1000", "/work/package", "/home/tester"]',
    'ENV HOME=/home/tester CARGO_HOME=/home/tester/.cargo CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=2 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0',
    // A test that waits on a lock nothing releases (a mutant that never releases one) fails after 20 seconds
    // instead of waiting out the 30 minute product limit, which cargo-mutants would record as a timeout.
    'ENV CLI_GIT_LOCK_WAIT_SECONDS=20 CLI_GIT_PUSH_LOCK_WAIT_SECONDS=20',
    'USER 1000:1000',
    'WORKDIR /work/package/git-policy/cli',
    // Compile once while the image is built. The test run, each planted control and each mutant then rebuild
    // only what changed, instead of the linked scanner and its dependencies every time.
    'RUN ["cargo", "nextest", "run", "--offline", "--locked", "--all-targets", "--no-run"]',
    // nextest's default concurrency: one test per CPU the container may use.
    'CMD ["cargo", "nextest", "run", "--offline", "--locked", "--all-targets"]',
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
    testImage,
    context.path,
  ],
});
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
const image = inspected.stdout
  .trim();
const limits = [
  '--rm',
  '--init',
  '--network=none',
  '--memory=2g',
  '--cpus=2',
  '--pids-limit=128',
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
const sysroot = await runCommand({
  command: 'rustc',
  args: [
    '--print',
    'sysroot',
  ],
  capture: true,
});
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
