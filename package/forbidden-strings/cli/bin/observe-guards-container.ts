#!/usr/bin/env node
/**
 Independently validate exact guard failure causes on the already tested immutable guard images.
 */
import { createHash } from 'node:crypto';
import {
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

import { mapInSequence } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts';

import {
  run,
  ScannerVerificationError,
} from './container-snapshot.ts';
import {
  type Control,
  selectControl,
} from './guard-control-selection.ts';
import { guardVariants } from './guard-injection.ts';

/**
 Scratch lives inside the package's gitignored node_modules/.monochromatic, never the system temporary directory,
 whose backing device is unknowable.
 */
const scratchBase = join(
  import.meta.dirname,
  '..',
  'node_modules',
  '.monochromatic',
  'forbidden-strings',
);

/**
 Cargo arguments of the single observer test.
 */
const observerCommand: readonly string[] = [
  'cargo',
  'test',
  '--offline',
  '--locked',
  '--test',
  'panic_contract',
  'boundary_observer::observations',
  '--',
  '--exact',
  '--nocapture',
  '--test-threads=1',
];

/**
 Exit status of a Rust binary that panicked and was not caught, which an unguarded boundary must produce.
 */
const rustPanicStatus = 101;

/**
 Path of the consumer test inside the scanner crate, as a manifest records it.
 */
const consumerPath = 'package/forbidden-strings/cli/tests/panic_contract.rs';

/**
 Digest of a text in hexadecimal.

 @param text - the text to hash

 @returns the SHA-256 digest

 @example
 ```ts
 sha256Of('a');
 ```
 */
function sha256Of(text: string): string {
  return createHash('sha256')
    .update(text)
    .digest('hex');
}

/**
 The observations the exact test must print for a variant: which boundary unwinds and which hook-owned output appears.

 @param variant - the control variant

 @returns each observation key with its expected value

 @example
 ```ts
 observationsOf('protected');
 ```
 */
function observationsOf(variant: string): Readonly<Record<string, boolean | number | string>> {
  return {
    content: variant === 'without-content-catch' ? 'unwound' : 'engine-error',
    name: variant === 'without-name-catch' ? 'unwound' : 'engine-error',
    load: variant === 'without-load-catch' ? 'unwound' : 'error',
    'hook-calls': 7,
    'startup-exit': variant === 'without-process-catch' ? rustPanicStatus : 2,
    'worker-exit': variant === 'without-process-catch' ? rustPanicStatus : 2,
    'startup-payload': variant === 'without-output-hook',
    'worker-payload': variant === 'without-output-hook',
    'runtime-content-exit': variant === 'without-content-catch' ? 2 : 1,
    'runtime-name-exit': variant === 'without-name-catch' ? 2 : 1,
    'runtime-content-payload': variant === 'without-output-hook',
    'runtime-name-payload': variant === 'without-output-hook',
  };
}

/**
 Build the observer image on a tested guard image.

 @param context - the build context to fill

 @param selected - the tested control the observer builds on

 @param consumer - the consumer test followed by the observer test

 @returns the content-addressed ID of the observer image

 @throws ScannerVerificationError when the image build fails

 @example
 ```ts
 const image = await buildObserverImage({ context, selected, consumer });
 ```
 */
async function buildObserverImage({
  context,
  selected,
  consumer,
}: {
  readonly context: string;
  readonly selected: Control;
  readonly consumer: string;
}): Promise<string> {
  await writeFile(
    join(
      context,
      'panic_contract.rs',
    ),
    consumer,
  );
  await writeFile(
    join(
      context,
      'Containerfile',
    ),
    [
      `FROM ${selected.manifest
        .image}`,
      'COPY panic_contract.rs /work/package/forbidden-strings/cli/tests/panic_contract.rs',
      `CMD ${JSON.stringify(observerCommand)}`,
      '',
    ].join('\n'),
  );
  /**
   File that receives the ID of the built image.
   */
  const imageFile = join(
    context,
    'image.id',
  );
  await run({
    command: 'podman',
    args: [
      'build',
      '--network=none',
      '--http-proxy=false',
      '--pull=never',
      '--memory=2g',
      '--cpu-period=100000',
      '--cpu-quota=200000',
      '--iidfile',
      imageFile,
      context,
    ],
  });
  return (await readFile(
    imageFile,
    'utf8',
  )).trim();
}

/**
 Observe one variant on its tested guard image.

 @param variant - the control variant

 @param root - directory holding the evidence directories

 @param original - the consumer test the control ran

 @param observer - the observer test appended to it

 @throws ScannerVerificationError when the control ran another consumer test, or the observations differ

 @example
 ```ts
 await observeVariant({ variant: 'protected', root, original, observer });
 ```
 */
async function observeVariant({
  variant,
  root,
  original,
  observer,
}: {
  readonly variant: string;
  readonly root: string;
  readonly original: string;
  readonly observer: string;
}): Promise<void> {
  /**
   The newest completed control of the variant.
   */
  const selected = await selectControl({
    root,
    variant,
  });
  /**
   Digest of the consumer test the control ran.
   */
  const originalSha256 = sha256Of(original);
  /**
   The recorded consumer test of the control.
   */
  const source = selected.manifest
    .sources
    .find(function isConsumer(item) {
      return item.path === consumerPath;
    });
  if (source?.sha256 !== originalSha256)
    throw new ScannerVerificationError(`Consumer assertions differ in ${selected.directory}.`);
  /**
   The container limits the control ran under.
   */
  const { limits } = selected.manifest;
  /**
   The disposable build context.
   */
  await using context = await mkdtempDisposable(join(
    scratchBase,
    'scanner-guard-observer-',
  ));
  /**
   The evidence directory of this observation.
   */
  const evidence = await mkdtemp(join(
    root,
    `guard-observer-${variant}-`,
  ));
  /**
   The observer image.
   */
  const image = await buildObserverImage({
    context: context.path,
    selected,
    consumer: `${original}\n${observer}`,
  });
  await writeFile(
    join(
      evidence,
      'manifest.json',
    ),
    `${JSON.stringify(
      {
        variant,
        baseControl: selected.directory,
        baseImage: selected.manifest
          .image,
        image,
        originalSha256,
        observerSha256: sha256Of(observer),
        command: observerCommand,
        limits,
      },
      null,
      2,
    )}\n`,
  );
  /**
   What the observer container left behind.
   */
  const result = await run({
    command: 'podman',
    args: [
      'run',
      '--rm',
      ...limits,
      '--env',
      'SCANNER_GUARD_HOME=/home/tester',
      '--env',
      'FORBIDDEN_STRINGS_CACHE_DIR=/home/tester/cache',
      image,
    ],
    allowFailure: true,
    transcript: evidence,
  });
  /**
   Standard output of the observer container.
   */
  const stdout = await readFile(
    join(
      evidence,
      'stdout.log',
    ),
    'utf8',
  );
  /**
   The observations this variant must print.
   */
  const observations = observationsOf(variant);
  /**
   Whether every expected observation line was printed.
   */
  const matches = Object.keys(observations)
    .every(function isPrinted(key) {
      return stdout.includes(`observation-${key}=${String(observations[key])}\n`);
    });
  /**
   Whether the exact test passed with every expected observation.
   */
  const expected = (result.status === 0) && stdout.includes('1 passed')
    && matches;
  await writeFile(
    join(
      evidence,
      'control.json',
    ),
    `${JSON.stringify(
      {
        variant,
        status: result.status ?? null,
        expected,
        observations,
      },
      null,
      2,
    )}\n`,
  );
  if (!expected) throw new ScannerVerificationError(`Exact guard observation failed: ${variant}; inspect ${evidence}.`);
}

/**
 Host hooks are counted per operation; both executable cases and real parallel CLI fault paths run independently.

 @throws ScannerVerificationError when arguments are given or an observation fails

 @example
 ```ts
 await main();
 ```
 */
async function main(): Promise<void> {
  if (process.argv
    .length
    !== 2) throw new ScannerVerificationError('This task accepts no arguments.');
  await mkdir(
    scratchBase,
    { recursive: true },
  );
  /**
   Directory holding the retained control evidence.
   */
  const root = resolve(
    import.meta.dirname,
    '../target/verification',
  );
  /**
   The consumer test the controls ran.
   */
  const original = await readFile(
    join(
      import.meta.dirname,
      'guard-fixture.rs.txt',
    ),
    'utf8',
  );
  /**
   The observer test appended to it.
   */
  const observer = await readFile(
    join(
      import.meta.dirname,
      'guard-observer.rs.txt',
    ),
    'utf8',
  );
  await mapInSequence({
    items: guardVariants,
    run: function observeOf(variant) {
      return observeVariant({
        variant,
        root,
        original,
        observer,
      });
    },
  });
}

await main();
