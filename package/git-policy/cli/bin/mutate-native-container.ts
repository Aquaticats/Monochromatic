#!/usr/bin/env node
/**
 Mutation-test the native wrapper inside disposable, bounded containers.

 Phase 1 plants known guard removals and requires the named control to fail,
 proving the harness can observe a missing guard before any mutation result is trusted.
 Phase 2 runs cargo-mutants over the already tested source snapshot.
 Source mutation and compilation stay inside containers; only reports leave them.
 */
// The bin scripts sit outside the package tsconfig's include list, so type-aware lint checks them in an inferred
// program that loads no ambient Node types; this reference loads them, so `process` and `node:` imports stay typed
// in this file on its own instead of depending on a sibling module that carries the same reference.
/// <reference types="node" />
import { createHash } from 'node:crypto';
import {
  copyFile,
  mkdir,
  mkdtemp,
  mkdtempDisposable,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';

import { runPlantedControls } from './native-planted-controls.mjs';
import {
  baselineSkipArguments,
  parseScopes,
  parseTimeout,
} from './native-mutation-options.mjs';
import {
  createContainer,
  NativeVerificationError,
  runCommand,
} from './native-verification-process.mjs';
import { mkdirSync as makeScratchBase } from 'node:fs';

// Scratch lives inside the package's gitignored node_modules/.monochromatic, never the system temporary directory,
// whose backing device is unknowable.
const scratchBase = join(import.meta.dirname, '..', 'node_modules', '.monochromatic', 'git-policy');
makeScratchBase(scratchBase, { recursive: true });


/** Concurrent worktrees set GIT_POLICY_NATIVE_IMAGE_TAG so one snapshot never mutates another's image. */
const imageTag = process.env
  .GIT_POLICY_NATIVE_IMAGE_TAG
  ?? 'development';
/** Already tested source snapshot this campaign mutates; built by bin/test-native-container.mjs. */
const testImage = `localhost/git-policy-native-test:${imageTag}`;
/** Campaign image layered over the tested snapshot. */
const mutationImage = `localhost/git-policy-native-mutation:${imageTag}`;
/** Same bounds as the test gate: no network, 2 GiB, 2 CPUs, 128 PIDs. */
const limits = [
  '--init',
  '--network=none',
  '--memory=2g',
  '--cpus=2',
  '--pids-limit=128',
];
/** Report directory inside the container; the tester's home is the only writable place outside the package. */
const containerReport = '/home/tester/mutation-report';
/** Length of a full content-addressed Podman image or container ID: a SHA-256 digest in hexadecimal. */
const contentAddressedIdLength = 64;
/** Digits a full content-addressed Podman ID may contain. */
const lowercaseHexDigits = '0123456789abcdef';
/**
 Mutant kinds no campaign tries, as regular expressions matched against the names `cargo mutants --list` prints.

 Turning `+= 1` into `*= 1`, or `-= 1` into `/= 1`, leaves a counter unchanged, so a loop stepping one never ends
 and the mutant only reaches the test timeout.
 cargo-mutants exits with its timeout status whenever any mutant timed out and has no option to change that,
 so one such mutant fails a whole campaign.
 The human accepted on 2026-10-05 that these two kinds are never tried.
 `replace += with -=` stays active as the check on every counter.
 */
const excludedMutantKinds = [
  String.raw`replace \+= with \*=`,
  'replace -= with /=',
];

/**
 Check for a full content-addressed Podman ID with one linear scan.

 @param {string} text - command output that should be exactly one ID
 @returns {boolean} true only for 64 lowercase hexadecimal digits, so a short or empty ID never passes
 */
function isContentAddressedId(text) {
  if (text.length !== contentAddressedIdLength)
    return false;
  for (const character of text) {
    if (!lowercaseHexDigits.includes(character))
      return false;
  }
  return true;
}

/**
 cargo-mutants command the campaign image runs.

 @param {{ scopes: readonly string[], examine: readonly string[], timeoutSeconds: number }} request - `src/native` file
   globs, mutant-name patterns to restrict to, and per-mutant test limit
 @returns {string[]} complete argument array, also recorded in the manifest
 */
function mutationCommand({
  scopes,
  examine,
  timeoutSeconds
}) {
  return [
    'cargo',
    'mutants',
    '--in-place',
    '--baseline',
    'run',
    // Binary-level controls bound each wrapped command to 5 seconds, so a looping mutant fails well inside this limit.
    '--build-timeout',
    '300',
    '--timeout',
    String(timeoutSeconds),
    '--no-config',
    '--no-shuffle',
    // The same runner as the gate: the baseline and every mutant run through nextest.
    '--test-tool',
    'nextest',
    '--output',
    containerReport,
    '--cargo-arg=--offline',
    '--cargo-arg=--locked',
    ...excludedMutantKinds.flatMap(function exclusionArguments(kind) {
      return [
        '--exclude-re',
        kind,
      ];
    }),
    ...examine.flatMap(function examineArguments(pattern) {
      return [
        '--re',
        pattern,
      ];
    }),
    // Scoped runs are evidence for the named files only. The unmutated baseline still runs over everything.
    ...scopes.flatMap(function scopeArguments(scope) {
      return [
        '--file',
        scope,
      ];
    }),
    ...baselineSkipArguments(),
  ];
}

// Build and run the tool over an immutable input image, then retain its complete report.
const {
  timeoutSeconds,
  examine,
  scopeOptions
} = parseTimeout(process.argv
  .slice(2));
const scopes = parseScopes(scopeOptions);
await using context = await mkdtempDisposable(join(
  scratchBase,
  'cli-git-native-mutation-',
));
const evidenceRoot = join(
  process.cwd(),
  'target',
  'verification',
);
await mkdir(
  evidenceRoot,
  { recursive: true },
);
const evidence = await mkdtemp(join(
  evidenceRoot,
  'native-mutation-',
));
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
const base = inspected.stdout
  .trim();
if (!isContentAddressedId(base))
  throw new NativeVerificationError('Test image did not resolve to a full content-addressed image ID; run native:test:container first.');
console.log(`Mutation evidence: ${evidence}`);
// Parallel campaigns over one tested snapshot prove the harness once; `NATIVE_MUTATION_SKIP_PLANTED=1` skips the repeat.
if (process.env
  .NATIVE_MUTATION_SKIP_PLANTED
  === '1')
  console.log('Planted controls skipped: another campaign over this snapshot ran them.');
else
  await runPlantedControls({
    base,
    limits,
    context: context.path,
    evidence,
  });
const cargoHome = process.env
  .CARGO_HOME
  ?? join(
    homedir(),
    '.cargo',
  );
const tool = join(
  cargoHome,
  'bin',
  'cargo-mutants',
);
const toolBytes = await readFile(tool);
const toolSha256 = createHash('sha256')
  .update(toolBytes)
  .digest('hex');
await copyFile(
  tool,
  join(
    context.path,
    'cargo-mutants',
  ),
);
const command = mutationCommand({
  scopes,
  examine,
  timeoutSeconds
});
await writeFile(
  join(
    context.path,
    'Containerfile',
  ),
  [
    '# The tested image ID binds this campaign to an exact source snapshot.',
    `FROM ${base}`,
    'COPY cargo-mutants /usr/local/cargo/bin/cargo-mutants',
    'RUN ["cargo", "mutants", "--version"]',
    'ENV CARGO_NET_OFFLINE=true',
    `CMD ${JSON.stringify(command)}`,
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
    mutationImage,
    context.path,
  ],
});
await using container = await createContainer({
  args: [
    'create',
    ...limits,
    mutationImage,
  ],
});
if (!isContentAddressedId(container.id))
  throw new NativeVerificationError('Container creation did not return a complete container ID.');
await writeFile(
  join(
    evidence,
    'manifest.json',
  ),
  `${JSON.stringify(
    {
      baseImage: base,
      toolSha256,
      container: container.id,
      command,
      scopes,
      limits: {
        memory: '2g',
        cpus: 2,
        pids: 128,
        buildTimeoutSeconds: 300,
        testTimeoutSeconds: 90,
      },
    },
    null,
    2,
  )}\n`,
);
const result = await runCommand({
  command: 'podman',
  args: [
    'start',
    '--attach',
    container.id,
  ],
  allowFailure: true,
});
// Preserve evidence even when missed mutants correctly make the campaign nonzero.
const copy = await runCommand({
  command: 'podman',
  args: [
    'cp',
    `${container.id}:${containerReport}/.`,
    evidence,
  ],
  capture: true,
  allowFailure: true,
});
await writeFile(
  join(
    evidence,
    'exit.json',
  ),
  `${JSON.stringify(
    {
      status: result.status,
      signal: result.signal,
      reportCopied: copy.status === 0,
      reportCopyError: copy.status === 0 ? undefined : copy.stderr,
    },
    null,
    2,
  )}\n`,
);
if (result.status !== 0)
  throw new NativeVerificationError(
    `Mutation campaign exited ${String(result.status)}; report ${copy.status === 0 ? 'copied' : 'not generated or not copied'}; inspect ${evidence}.`,
  );
if (copy.status !== 0)
  throw new NativeVerificationError(`Mutation campaign passed but its report was not copied: ${copy.stderr}`);
