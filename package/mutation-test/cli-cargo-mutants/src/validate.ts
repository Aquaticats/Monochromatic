import {
  alphanumericCharacters,
  isPositiveWholeNumber,
  startsThenUses,
  usesOnly,
  wordStartCharacters,
} from '@monochromatic-dev/shell-argument-core/ts/character-set.ts';
import {
  environmentNameCharacters,
  maximumWholeNumberDigits,
  plainNameCharacters,
  systemdValueCharacters,
} from './constants.ts';
import { CargoMutantsToolingError } from './error.ts';

/**
 Accept a name made of letters, digits, underscores and hyphens that starts with a letter or digit.

 @param option - option name for the message
 
 @param value - text under test
 
 @returns the value unchanged
 
 @throws CargoMutantsToolingError when the value has other characters
 
 @example
 ```ts
 plainName({ option: 'label', value: 'w5' });
 ```
 */
export function plainName({
  option,
  value,
}: {
  readonly option: string;
  readonly value: string;
}): string {
  if (!startsThenUses({
    text: value,
    first: alphanumericCharacters,
    rest: plainNameCharacters,
  }))
    throw new CargoMutantsToolingError(
      `${option} must start with a letter or digit and use only letters, digits, underscores and hyphens, got ${JSON.stringify(value)}.`,
    );
  return value;
}

/**
 Accept a systemd resource value such as `12G`, `500%` or `infinity`.

 @param option - option name for the message
 
 @param value - text under test
 
 @returns the value unchanged
 
 @throws CargoMutantsToolingError when the value has characters outside letters, digits, dots and percent signs
 
 @example
 ```ts
 systemdValue({ option: '--unit-memory-max', value: '12G' });
 ```
 */
export function systemdValue({
  option,
  value,
}: {
  readonly option: string;
  readonly value: string;
}): string {
  if (!usesOnly({
    text: value,
    allowed: systemdValueCharacters
  }))
    throw new CargoMutantsToolingError(
      `${option} must be a systemd value such as 12G or 500%, got ${JSON.stringify(value)}.`,
    );
  return value;
}

/**
 Accept a positive whole number written without a sign, leading zero or fraction.

 @param option - option name for the message
 
 @param value - text under test
 
 @returns the number
 
 @throws CargoMutantsToolingError when the text is not a positive whole number
 
 @example
 ```ts
 positiveWholeNumber({ option: '--timeout', value: '900' });
 ```
 */
export function positiveWholeNumber({
  option,
  value,
}: {
  readonly option: string;
  readonly value: string;
}): number {
  if (!isPositiveWholeNumber({
    text: value,
    maximumDigits: maximumWholeNumberDigits
  }))
    throw new CargoMutantsToolingError(`${option} must be a positive whole number, got ${JSON.stringify(value)}.`);
  return Number(value);
}

/**
 Accept a text that is non-empty and has no NUL character.

 @param option - option name for the message
 
 @param value - text under test
 
 @returns the value unchanged
 
 @throws CargoMutantsToolingError when the text is empty or holds NUL
 
 @example
 ```ts
 nonEmptyText({ option: '--task', value: '//p:t' });
 ```
 */
export function nonEmptyText({
  option,
  value,
}: {
  readonly option: string;
  readonly value: string;
}): string {
  if ((value === '') || value.includes('\0'))
    throw new CargoMutantsToolingError(`${option} must be non-empty text without NUL, got ${JSON.stringify(value)}.`);
  return value;
}

/**
 Accept an environment entry of the form `NAME=value`.

 @param entry - text under test
 
 @returns the entry unchanged
 
 @throws CargoMutantsToolingError when the name is not a shell-style identifier or the entry holds NUL
 
 @example
 ```ts
 environmentEntry('NATIVE_MUTATION_SKIP_PLANTED=1');
 ```
 */
export function environmentEntry(entry: string): string {
  /**
   Position of the first equals sign, which ends the name.
   */
  const separator = entry.indexOf('=');
  if ((separator === (-1)) || entry.includes('\0')
    || (!startsThenUses({
      text: entry.slice(
        0,
        separator
      ),
      first: wordStartCharacters,
      rest: environmentNameCharacters,
    })))
    throw new CargoMutantsToolingError(`--setenv must look like NAME=value, got ${JSON.stringify(entry)}.`);
  return entry;
}
