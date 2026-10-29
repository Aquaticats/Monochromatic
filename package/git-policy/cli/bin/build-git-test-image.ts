#!/usr/bin/env node
/**
 Build the selected latest-stable Git into the already used Rust verification image.
 */
import { createHash } from 'node:crypto';
import {
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile
} from 'node:fs/promises';
import {
  join,
  resolve
} from 'node:path';
import {
  NativeVerificationError,
  createContainer,
  runCommand
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
 Spaces of indentation in the evidence files.
 */
const jsonIndent = 2;

/**
 Length of a full container ID in hexadecimal digits.
 */
const containerIdLength = 64;

/**
 Digits a container ID may hold.
 */
const hexDigits = '0123456789abcdef';

/**
 Whether text is a full container ID.

 @param text - text returned by `podman create`
 
 @returns whether it holds exactly the expected number of lowercase hexadecimal digits
 */
function isContainerId(text: string): boolean {
  const characters = text
    .split('');
  return (characters.length === containerIdLength)
    && characters.every(function isHexDigit(character) {
      return hexDigits.includes(character);
    });
}

const [sourceArgument, ...extra] = process.argv
  .slice(2);
if ((!sourceArgument) || (extra.length > 0))
  throw new NativeVerificationError('Provide the audited git/git v2.56.0 source checkout path.');
const source = resolve(sourceArgument);
// This is the latest stable release selected for the rewrite on 2026-10-04, not an older-version matrix.
const revision = 'a018953688f1b10bddf91bff8747068f5f4746a4';
const checkedOut = await runCommand({
  command: 'git',
  args: [
    '-C',
    source,
    'rev-parse',
    'HEAD'
  ],
  capture: true
});
const checkedOutRevision = checkedOut.stdout
  .trim();
if (checkedOutRevision !== revision)
  throw new NativeVerificationError('The checkout is not the inspected Git 2.56.0 revision.');
const contextPath = await mkdtemp(join(
  scratchBase,
  'cli-git-native-image-'
));
/**
 Removes the build context when the script ends, whether the build passed or failed.
 */
await using context = {
  path: contextPath,
  async [Symbol.asyncDispose]() {
    await rm(
      contextPath,
      {
        recursive: true,
        force: true
      }
    );
  }
};
const evidenceRoot = resolve('target/verification');
await mkdir(
  evidenceRoot,
  { recursive: true }
);
const evidence = await mkdtemp(join(
  evidenceRoot,
  'git-image-'
));
const base = '84f24e75017a7d8afa1c69e51c52f37e7d8d644cd2597a01f4732ad0b386dc20';
const outputTag = 'localhost/cli-git-test-rust:git-2.56.0';
const archive = join(
  context.path,
  'git-source.tar'
);
await runCommand({
  command: 'git',
  args: [
    '-C',
    source,
    'archive',
    '--format=tar',
    `--output=${archive}`,
    revision
  ]
});
const sourceSha256 = createHash('sha256')
  .update(await readFile(archive))
  .digest('hex');
await writeFile(
  join(
    context.path,
    'Containerfile'
  ),
  [
    '# The compiler image was previously verified; Git source is an exact upstream release archive.',
    `FROM ${base}`,
    'ADD git-source.tar /work/git-source/',
    'WORKDIR /work/git-source',
    'ENV CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=2',
    '',
  ].join('\n')
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
    'localhost/cli-git-native-source:git-2.56.0',
    context.path,
  ]
});
// CLI tests use English diagnostics and no Tk UI. Perl/Python, curl, fsmonitor and Git's default Rust support remain enabled.
const make = [
  'make',
  '--jobs=2',
  'prefix=/usr',
  'NO_GETTEXT=YesPlease',
  'NO_TCLTK=YesPlease',
  'PYTHON_PATH=/usr/bin/python3',
  'CARGO_ARGS=--offline --release',
  'install'
];
const limits = [
  '--init',
  '--network=none',
  '--memory=2g',
  '--cpus=2',
  '--pids-limit=128',
  '--ulimit',
  'nofile=4096:4096'
];
/**
 Removed with force when the script ends, so even a container with a malformed ID is cleaned up.
 */
await using container = await createContainer({
  args: [
    'create',
    ...limits,
    'localhost/cli-git-native-source:git-2.56.0',
    ...make
  ]
});
if (!isContainerId(container.id))
  throw new NativeVerificationError('Container identity was not returned.');
await writeFile(
  join(
    evidence,
    'manifest.json'
  ),
  `${JSON.stringify(
    {
      revision,
      sourceSha256,
      base,
      make,
      limits,
      container: container.id,
      outputTag
    },
    null,
    jsonIndent
  )}\n`
);
await runCommand({
  command: 'podman',
  args: [
    'start',
    '--attach',
    container.id
  ]
});
await runCommand({
  command: 'podman',
  args: [
    'commit',
    '--change',
    'WORKDIR /work',
    container.id,
    outputTag
  ]
});
const inspected = await runCommand({
  command: 'podman',
  args: [
    'image',
    'inspect',
    outputTag,
    '--format',
    '{{.Id}}'
  ],
  capture: true
});
const image = inspected.stdout
  .trim();
const reported = await runCommand({
  command: 'podman',
  args: [
    'run',
    '--rm',
    ...limits,
    image,
    '/usr/bin/git',
    '--version'
  ],
  capture: true
});
const version = reported.stdout
  .trim();
if (version !== 'git version 2.56.0')
  throw new NativeVerificationError(`Built image returned an unexpected Git version: ${version}`);
await writeFile(
  join(
    evidence,
    'result.json'
  ),
  `${JSON.stringify(
    {
      image,
      version
    },
    null,
    jsonIndent
  )}\n`
);
console.log(`Latest-stable Git test image: ${image}; evidence ${evidence}`);
