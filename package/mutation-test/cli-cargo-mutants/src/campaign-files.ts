import { exclusionArguments } from './excluded-mutants.ts';

/**
 Seconds a campaign may spend building each mutant before cargo-mutants gives up on it.
 */
export const defaultBuildTimeoutSeconds = 300;

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

 The fixed options come first: in-place mutation, a baseline run of the unmutated source, nextest as the test runner so
 the baseline and every mutant run as the gate does, offline and locked Cargo, and the repository-wide exclusions.
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
 
 @param reportCopyError - the copy's error text, kept only when the copy failed
 
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
}: {
  readonly status: unknown;
  readonly signal: unknown;
  readonly reportCopied: boolean;
  readonly reportCopyError?: string;
}): Record<string, unknown> {
  return {
    status: status ?? null,
    signal: signal ?? null,
    reportCopied,
    ...(reportCopyError === undefined ? {} : { reportCopyError }),
  };
}
