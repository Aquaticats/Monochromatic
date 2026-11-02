import { timeoutExitStatus } from './constants.ts';
import { CargoMutantsToolingError } from './error.ts';
import { exclusionArguments } from './excluded-mutants.ts';

/**
 Seconds a campaign may spend building each mutant before cargo-mutants gives up on it.
 */
export const defaultBuildTimeoutSeconds = 300;

/**
 Largest build limit, in seconds, a `--build-timeout` option may request when the product names no maximum of its own.
 */
export const maximumBuildTimeoutSeconds = 3_600;

/**
 Length of a full content-addressed Podman image or container ID: a SHA-256 digest in hexadecimal.
 */
const contentAddressedIdLength = 64;

/**
 Digits a full content-addressed Podman ID may contain.
 */
const lowercaseHexDigits = '0123456789abcdef';

/**
 Comment that opens every campaign Containerfile.
 */
const containerfileComment = '# The tested image ID binds this campaign to an exact source snapshot.';

/**
 Check for a full content-addressed Podman ID with one linear scan.

 @param text - command output that should be exactly one ID
 
 @returns true only for 64 lowercase hexadecimal digits, so a short or empty ID never passes
 
 @example
 ```ts
 isContentAddressedId('a'.repeat(64));
 ```
 */
export function isContentAddressedId(text: string): boolean {
  if (text.length !== contentAddressedIdLength)
    return false;
  for (const character of text) {
    if (!lowercaseHexDigits.includes(character))
      return false;
  }
  return true;
}

/**
 Build the cargo-mutants command a campaign image runs.

 The fixed options come first: in-place mutation, a baseline run of the unmutated source, `--iterate` so the mutants
 named in the report directory's `mutants.out/previously_caught.txt` are left out (that file exists only when the
 campaign is resumed, see `campaign-ledger.ts`), nextest as the test runner so the baseline and every mutant run as the
 gate does, offline and locked Cargo, and the repository-wide exclusions.
 The product's own options, such as its scopes and filters, follow as `tail`.

 @param outputDirectory - report directory inside the container
 
 @param timeoutSeconds - per-mutant test limit
 
 @param buildTimeoutSeconds - per-mutant build limit
 
 @param additionalExclusions - patterns the product excludes on top of the repository-wide kinds
 
 @param tail - the product's options, placed after the exclusions
 
 @param useProjectConfig - true when the crate's own `.cargo/mutants.toml` applies, so `--no-config` is left out; false for
 the other products, which pass every option on the command line
 
 @returns the complete argument array, also recorded in the manifest
 
 @example
 ```ts
 campaignCommand({ outputDirectory: '/work/report', timeoutSeconds: 300, buildTimeoutSeconds: 300, additionalExclusions: [], tail: [] });
 ```
 */
export function campaignCommand({
  outputDirectory,
  timeoutSeconds,
  buildTimeoutSeconds,
  additionalExclusions,
  tail,
  useProjectConfig = false,
}: {
  readonly outputDirectory: string;
  readonly timeoutSeconds: number;
  readonly buildTimeoutSeconds: number;
  readonly additionalExclusions: readonly string[];
  readonly tail: readonly string[];
  readonly useProjectConfig?: boolean;
}): string[] {
  return [
    'cargo',
    'mutants',
    '--in-place',
    '--baseline',
    'run',
    '--iterate',
    '--build-timeout',
    String(buildTimeoutSeconds),
    '--timeout',
    String(timeoutSeconds),
    ...(useProjectConfig ? [] : ['--no-config']),
    '--no-shuffle',
    '--test-tool',
    'nextest',
    '--output',
    outputDirectory,
    '--cargo-arg=--offline',
    '--cargo-arg=--locked',
    ...exclusionArguments(additionalExclusions),
    ...tail,
  ];
}

/**
 Replace the baseline run of a campaign command by cargo-mutants' baseline skip.

 `--baseline skip` makes cargo-mutants assume the unmutated tree passes its tests, so the caller must hold proof that it
 does. The campaign key hashes the command with the baseline run, so a campaign that gains the proof later still finds
 its earlier outcomes.

 @param command - a campaign command that runs the baseline, as {@link campaignCommand} builds it

 @returns the command with `--baseline skip` in place of `--baseline run`; the input is not changed

 @throws CargoMutantsToolingError when the command has no `--baseline run`

 @example
 ```ts
 skipBaselineArguments(['cargo', 'mutants', '--baseline', 'run']);
 ```
 */
export function skipBaselineArguments(command: readonly string[]): string[] {
  /**
   Position of the baseline option.
   */
  const flagAt = command.indexOf('--baseline');
  if ((flagAt < 0) || (command[flagAt + 1] !== 'run'))
    throw new CargoMutantsToolingError('A baseline skip replaces a baseline run, but this campaign command has no --baseline run.');
  return command.map(function skipOrKeep(
    argument,
    index
  ) {
    return index === (flagAt + 1) ? 'skip' : argument;
  });
}

/**
 Build the Containerfile that layers the cargo-mutants executable over a tested image.

 @param baseImage - full content-addressed ID of the tested image, which binds the campaign to one source snapshot
 
 @param command - the campaign command, recorded as the image's default command
 
 @returns the Containerfile text, ending in a newline
 
 @example
 ```ts
 campaignContainerfile({ baseImage: 'a'.repeat(64), command: ['cargo', 'mutants'] });
 ```
 */
export function campaignContainerfile({
  baseImage,
  command,
}: {
  readonly baseImage: string;
  readonly command: readonly string[];
}): string {
  return [
    containerfileComment,
    `FROM ${baseImage}`,
    'COPY cargo-mutants /usr/local/cargo/bin/cargo-mutants',
    'RUN ["cargo", "mutants", "--version"]',
    'ENV CARGO_NET_OFFLINE=true',
    `CMD ${JSON.stringify(command)}`,
    '',
  ].join('\n');
}

/**
 Render a record the way the evidence files are written: two-space indentation and a final newline.

 @param record - JSON-serializable value
 
 @returns the file text
 
 @example
 ```ts
 evidenceJson({ status: 0 });
 ```
 */
export function evidenceJson(record: unknown): string {
  return `${JSON.stringify(
    record,
    null,
    2,
  )}\n`;
}

/**
 Build the record of how a campaign container ended, written as `exit.json`.

 @param status - exit status of the container as the process API reports it: a number, or no number when a signal ended it
 
 @param signal - signal that ended it as the process API reports it
 
 @param reportCopied - whether the report was copied out of the container
 
 @param reportCopyError - the report's error text, kept only when it was not composed

 @param attempt - number of the attempt that ran, counted from 1 across the campaign's interruptions

 @param skippedMutants - mutants earlier attempts had scored, which this attempt left out

 @returns the record; a missing error text is left out of the file
 
 @example
 ```ts
 exitRecord({ status: 0, signal: undefined, reportCopied: true });
 ```
 */
export function exitRecord({
  status,
  signal,
  reportCopied,
  reportCopyError,
  attempt,
  skippedMutants,
}: {
  readonly status: unknown;
  readonly signal: unknown;
  readonly reportCopied: boolean;
  readonly reportCopyError?: string;
  readonly attempt?: number;
  readonly skippedMutants?: number;
}): Record<string, unknown> {
  return {
    status: status ?? null,
    signal: signal ?? null,
    reportCopied,
    ...(reportCopyError === undefined ? {} : { reportCopyError }),
    ...(attempt === undefined ? {} : { attempt }),
    ...(skippedMutants === undefined ? {} : { skippedMutants }),
  };
}

/**
 Build the failure message of a campaign container that exited with a nonzero status.

 A timeout status names both limits that can cause it, because cargo-mutants reports a build timeout and a test timeout
 with the same status: the `TIMEOUT` lines of the output say which phase ran out, and each limit has its own option.

 @param status - exit status of the container
 
 @param reportCopied - whether the report was copied out of the container
 
 @param evidence - directory that holds the evidence files
 
 @returns the message, ending with where to look
 
 @example
 ```ts
 campaignFailureMessage({ status: 2, reportCopied: true, evidence: '/e' });
 ```
 */
export function campaignFailureMessage({
  status,
  reportCopied,
  evidence,
}: {
  readonly status: number;
  readonly reportCopied: boolean;
  readonly evidence: string;
}): string {
  return `Mutation campaign exited ${status}${status === timeoutExitStatus
    ? ' (a mutant timed out in its build or in its tests; the TIMEOUT lines name the phase, and --build-timeout or --timeout raises the limit)'
    : ''}; report ${reportCopied ? 'copied' : 'not generated or not copied'}; inspect ${evidence}.`;
}
