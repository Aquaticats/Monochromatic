#!/usr/bin/env node
/**
 Mutation-test the native wrapper inside disposable, bounded containers.

 Phase 1 plants known guard removals and requires the named control to fail,
 proving the harness can observe a missing guard before any mutation result is trusted.
 Phase 2 runs cargo-mutants over the already tested source snapshot.
 Source mutation and compilation stay inside containers; only reports leave them.
 */
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

import { exclusionArguments } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts';

import {
  buildLimitArguments,
  parseContainerLimitOptions,
  runLimitArguments,
} from './container-limits.ts';
import { runPlantedControls } from './native-planted-controls.ts';
import {
  baselineSkipArguments,
  parseScopes,
  parseTimeout,
} from './native-mutation-options.ts';
import {
  createContainer,
  NativeVerificationError,
  runCommand,
} from './native-verification-process.ts';

// Scratch lives inside the package's gitignored node_modules/.monochromatic, never the system temporary directory,
// whose backing device is unknowable.
/**
 Parent of every scratch directory this campaign creates.
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
 Concurrent worktrees set GIT_POLICY_NATIVE_IMAGE_TAG so one snapshot never mutates another's image.
 */
const imageTag = process.env
  .GIT_POLICY_NATIVE_IMAGE_TAG
  ?? 'development';
/**
 Already tested source snapshot this campaign mutates; built by bin/test-native-container.ts.
 */
const testImage = `localhost/git-policy-native-test:${imageTag}`;
/**
 Campaign image layered over the tested snapshot.
 */
const mutationImage = `localhost/git-policy-native-mutation:${imageTag}`;
/**
 Report directory inside the container; the tester's home is the only writable place outside the package.
 */
const containerReport = '/home/tester/mutation-report';
/**
 Length of a full content-addressed Podman image or container ID: a SHA-256 digest in hexadecimal.
 */
const contentAddressedIdLength = 64;
/**
 Digits a full content-addressed Podman ID may contain.
 */
const lowercaseHexDigits = '0123456789abcdef';
/**
 Check for a full content-addressed Podman ID with one linear scan.

 @param text - command output that should be exactly one ID
 
 @returns true only for 64 lowercase hexadecimal digits, so a short or empty ID never passes
 */
function isContentAddressedId(text: string): boolean {
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

 @param scopes - `src/native` file globs

 @param examine - mutant-name patterns to restrict to

 @param timeoutSeconds - per-mutant test limit

 @returns complete argument array, also recorded in the manifest
 */
function mutationCommand({
  scopes,
  examine,
  timeoutSeconds,
}: {
  readonly scopes: readonly string[];
  readonly examine: readonly string[];
  readonly timeoutSeconds: number;
}): string[] {
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
    ...exclusionArguments([]),
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
/**
 Container limits parsed from the command line (`--memory`, `--cpus`, `--pids-limit`), and the other options.
 */
const {
  limits: containerLimits,
  rest: campaignOptions,
} = parseContainerLimitOptions({
  options: process.argv
    .slice(2),
  acceptPidsLimit: true,
  leadingOnly: false,
});
/**
 Same bounds as the test gate: no network, and the memory, CPUs and PIDs of the container limits.
 */
const limits = [
  '--init',
  '--network=none',
  ...runLimitArguments({
    limits: containerLimits,
    withPidsLimit: true,
  }),
];
/**
 Campaign options parsed from the command line: the per-mutant timeout, the mutant-name patterns and the rest.
 */
const {
  timeoutSeconds,
  examine,
  scopeOptions
} = parseTimeout(campaignOptions);
/**
 Validated `--file` scopes, or none for a whole-package campaign.
 */
const scopes = parseScopes(scopeOptions);
/**
 Disposable build context that holds the Containerfile and the copied tool.
 */
await using context = await mkdtempDisposable(join(
  scratchBase,
  'cli-git-native-mutation-',
));
/**
 Directory under the package's `target/` that keeps the evidence of every campaign.
 */
const evidenceRoot = join(
  process.cwd(),
  'target',
  'verification',
);
await mkdir(
  evidenceRoot,
  { recursive: true },
);
/**
 Evidence directory of this campaign.
 */
const evidence = await mkdtemp(join(
  evidenceRoot,
  'native-mutation-',
));
/**
 Result of inspecting the tested image for its ID.
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
 Content-addressed ID of the tested image, which binds the campaign to one source snapshot.
 */
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
/**
 Cargo home that holds the installed cargo-mutants.
 */
const cargoHome = process.env
  .CARGO_HOME
  ?? join(
    homedir(),
    '.cargo',
  );
/**
 Host path of the installed cargo-mutants executable.
 */
const tool = join(
  cargoHome,
  'bin',
  'cargo-mutants',
);
/**
 Bytes of the executable, hashed into the manifest.
 */
const toolBytes = await readFile(tool);
/**
 Digest that identifies the exact tool build in the manifest.
 */
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
/**
 The cargo-mutants command line, recorded in the image and the manifest.
 */
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
    ...buildLimitArguments(containerLimits),
    '--tag',
    mutationImage,
    context.path,
  ],
});
/**
 The campaign container, removed when the script ends whether it passed or failed.
 */
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
        memory: containerLimits.memory,
        cpus: containerLimits.cpus,
        pids: containerLimits.pidsLimit,
        buildTimeoutSeconds: 300,
        testTimeoutSeconds: 90,
      },
    },
    null,
    2,
  )}\n`,
);
/**
 Outcome of the campaign run, whose nonzero exit can mean missed mutants rather than a failure.
 */
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
/**
 Outcome of copying the report out of the container.
 */
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
      status: result.status ?? null,
      signal: result.signal ?? null,
      reportCopied: copy.status === 0,
      reportCopyError: copy.status === 0 ? undefined : copy.stderr,
    },
    null,
    2,
  )}\n`,
);
if (result.status !== 0)
  throw new NativeVerificationError(
    `Mutation campaign exited ${String(result.status ?? 'without a status')}; report ${copy.status === 0 ? 'copied' : 'not generated or not copied'}; inspect ${evidence}.`,
  );
if (copy.status !== 0)
  throw new NativeVerificationError(`Mutation campaign passed but its report was not copied: ${copy.stderr}`);
