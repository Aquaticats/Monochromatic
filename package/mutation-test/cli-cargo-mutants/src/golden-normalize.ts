import { realpath } from 'node:fs/promises';
import { resolve } from 'node:path';

import {
  alphanumericCharacters,
  usesOnly,
} from './character-set.ts';
import type { FakePodman } from './fake-podman-fixture.ts';
import type { ScriptRun } from './run-fixture-program.ts';

/**
 Repository package directory, three levels above this package's `src`.
 */
export const packageRoot: string = resolve(
  import.meta.dirname,
  '..',
  '..',
  '..',
);

/**
 What a scenario produced, with the paths and random suffixes that differ between runs replaced.
 */
export type GoldenOutcome = {
  readonly status: number;
  readonly errorMessage: string;
  readonly stdout: string;
  readonly calls: readonly (readonly string[])[];
  readonly containerfiles: readonly string[];
  readonly evidenceFiles: readonly (readonly [
    string,
    string
  ])[];
};

/**
 Directory names that end in six random characters, up to and including the last hyphen.
 */
const randomSuffixMarkers: readonly string[] = [
  'verification/native-mutation-',
  'verification/mutation-',
  'verification/coverage-',
  'verification/processors-mutation-',
  'cli-git-native-mutation-',
  'forbidden-regex-mutation-',
  'forbidden-regex-test-',
  'monochromatic-lint-mutation-',
  'scanner-snapshot-',
];

/**
 Length of the random suffix `mkdtemp` appends.
 */
const randomSuffixLength = 6;

/**
 Marker of the throwaway build-context directory of the inline processor task.
 */
const contextMarker = 'processor-mutants-';

/**
 Placeholder that stands for a random suffix.
 */
const idPlaceholder = '<ID>';

/**
 Replace the random suffix after one marker wherever the marker occurs.

 @param text - text to rewrite
 
 @param marker - text that precedes a random suffix
 
 @returns the text with each suffix replaced by a placeholder
 
 @example
 ```ts
 maskSuffix({ text: 'a/x-AbC123/y', marker: 'x-' });
 ```
 */
function maskSuffix({
  text,
  marker,
}: {
  readonly text: string;
  readonly marker: string;
}): string {
  /**
   Position of the first marker.
   */
  const at = text.indexOf(marker);
  if (at === (-1))
    return text;
  /**
   Position where the suffix would end.
   */
  const end = at + marker.length
    + randomSuffixLength;
  return usesOnly({
    text: text.slice(
      at + marker.length,
      end,
    ),
    allowed: alphanumericCharacters,
  })
    ? `${text.slice(
      0,
      at + marker.length,
    )}${idPlaceholder}${maskSuffix({
      text: text.slice(end),
      marker,
    })}`
    : `${text.slice(
      0,
      at + marker.length,
    )}${maskSuffix({
      text: text.slice(at + marker.length),
      marker,
    })}`;
}

/**
 Paths a run reports in two spellings, to be replaced by placeholders.
 */
export type PathReplacements = readonly (readonly [
  string,
  string
])[];

/**
 Work out the paths to mask for a fixture, including their symlink-resolved spellings.

 @param fake - fixture of the run
 
 @returns pairs of path and placeholder, longest paths first
 
 @example
 ```ts
 await pathReplacements(fake);
 ```
 */
export async function pathReplacements(fake: FakePodman): Promise<PathReplacements> {
  return [
    [
      await realpath(fake.workingDirectory),
      '<WORK>'
    ],
    [
      fake.workingDirectory,
      '<WORK>'
    ],
    [
      await realpath(fake.directory),
      '<FAKE>'
    ],
    [
      fake.directory,
      '<FAKE>'
    ],
    [
      packageRoot,
      '<PACKAGE>'
    ],
  ];
}

/**
 Replace what differs between runs in one string.

 @param text - text from a run
 
 @param replacements - paths to mask
 
 @returns the text with stable placeholders
 
 @example
 ```ts
 normalizeText({ text, replacements });
 ```
 */
function normalizeText({
  text,
  replacements,
}: {
  readonly text: string;
  readonly replacements: PathReplacements;
}): string {
  if ((!text.includes(' ')) && text.includes(contextMarker))
    return '<CONTEXT>';
  return randomSuffixMarkers.reduce(
    function mask(
      current,
      marker
    ) {
      return maskSuffix({
        text: current,
        marker,
      });
    },
    replacements.reduce(
      function replacePath(
        current,
        [path, placeholder]
      ) {
        return current.replaceAll(
          path,
          placeholder,
        );
      },
      text,
    ),
  );
}

/**
 Find the message of the first line that reports an error, without the error class name.

 @param stderr - standard error text
 
 @returns the message, or the empty text when no line reports an error
 
 @example
 ```ts
 errorMessageOf('NativeVerificationError: bad');
 ```
 */
function errorMessageOf(stderr: string): string {
  /**
   Separator between an error class name and its message.
   */
  const separator = 'Error: ';
  for (const line of stderr.split('\n')) {
    /**
     Position of the separator in this line.
     */
    const at = line.indexOf(separator);
    if ((at === 0) || ((at > 0) && usesOnly({
      text: line.slice(
        0,
        at,
      ),
      allowed: alphanumericCharacters,
    })))
      return line.slice(at + separator.length);
  }
  return '';
}

/**
 Reduce a run to the outcome that is pinned.

 @param run - what the program did
 
 @param replacements - paths to mask
 
 @returns the normalized outcome; a failure keeps only its message, without the error class name or stack
 
 @example
 ```ts
 normalizeRun({ run, replacements });
 ```
 */
export function normalizeRun({
  run,
  replacements,
}: {
  readonly run: ScriptRun;
  readonly replacements: PathReplacements;
}): GoldenOutcome {
  /**
   Normalize one string.

   @param text - text from the run

   @returns the text with stable placeholders
   */
  function normalize(text: string): string {
    return normalizeText({
      text,
      replacements,
    });
  }
  return {
    status: run.status,
    errorMessage: normalize(errorMessageOf(run.stderr)),
    stdout: normalize(run.stdout),
    calls: run.calls
      .map(function normalizeCall(call) {
        return call.map(normalize);
      }),
    containerfiles: run.containerfiles
      .map(normalize),
    evidenceFiles: run.evidenceFiles
      .map(function normalizeFile([path, text]) {
        return [
          normalize(path),
          normalize(text),
        ] as const;
      }),
  };
}
