import {
  alphanumericCharacters,
  isPositiveWholeNumber,
  startsThenUses,
  usesOnly,
  wordStartCharacters,
} from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts/character-set.ts';

import {
  commitScopeCharacters,
  crateNameCharacters,
  hexadecimalCharacters,
  maximumShaLength,
  maximumWholeNumberDigits,
  minimumShaLength,
  repositoryPartCharacters,
  systemdValueCharacters,
  taskNameCharacters,
  unitNameCharacters,
  workflowNameCharacters,
} from './constants.ts';
import { CrateReleaseError } from './error.ts';

/**
 Code of the first printable ASCII character, the space.
 */
const firstPrintableCode = 0x20;

/**
 Code of the last printable ASCII character, the tilde.
 */
const lastPrintableCode = 0x7E;

/**
 Code of the delete character, the one control character above the printable range.
 */
const deleteCode = 0x7F;

/**
 Input shared by every text parser.
 */
type TextInput = {
  /**
   Option or argument name for the message.
   */
  readonly option: string;
  /**
   Text under test.
   */
  readonly value: string;
};

/**
 Check whether a text holds a control character: a line break, tab, NUL, escape or delete.

 @param text - value under test

 @returns true when any character is a control character

 @example
 ```ts
 hasControlCharacter('a\nb');
 ```
 */
export function hasControlCharacter(text: string): boolean {
  for (const character of text) {
    /**
     Code point of the character.
     */
    const code = character.codePointAt(0) ?? 0;
    if ((code < firstPrintableCode) || (code === deleteCode))
      return true;
  }
  return false;
}

/**
 Build the refusal of a value.

 @param option - option or argument name

 @param expectation - what a valid value looks like

 @param value - the refused value

 @returns the error to throw

 @example
 ```ts
 refusal({ option: '--scope', expectation: 'a scope', value: 'a b' });
 ```
 */
function refusal({
  option,
  expectation,
  value,
}: {
  readonly option: string;
  readonly expectation: string;
  readonly value: string;
}): CrateReleaseError {
  return new CrateReleaseError(`${option} must be ${expectation}, got ${JSON.stringify(value)}.`);
}

/**
 Accept a crate or binary name: a letter or underscore, then letters, digits, underscores and hyphens.

 @param option - option or argument name for the message

 @param value - text under test

 @returns the value unchanged

 @throws CrateReleaseError when the value has other characters

 @example
 ```ts
 parseCrateName({ option: '--crate-name', value: 'monochromatic-git-policy-cli' });
 ```
 */
export function parseCrateName({
  option,
  value,
}: TextInput): string {
  if (!startsThenUses({
    text: value,
    first: wordStartCharacters,
    rest: crateNameCharacters
  }))
    throw refusal({
      option,
      expectation: 'a name that starts with a letter and uses only letters, digits, underscores and hyphens',
      value
    });
  return value;
}

/**
 Accept a path inside the repository: relative, with plain segments, no `.` or `..` and no backslash.

 @param option - option or argument name for the message

 @param value - text under test

 @returns the value unchanged

 @throws CrateReleaseError when the path is empty, absolute, leaves the repository or has an empty or dotted segment

 @example
 ```ts
 parseRepositoryPath({ option: '--crate-dir', value: 'package/git-policy/cli' });
 ```
 */
export function parseRepositoryPath({
  option,
  value,
}: TextInput): string {
  if ((value === '')
    || value.includes('\0')
    || value.includes('\\')
    || value.startsWith('/')
    || value.split('/')
      .some(function isEscapingSegment(segment) {
        return (segment === '') || (segment === '.')
          || (segment === '..');
      }))
    throw refusal({
      option,
      expectation: 'a relative path inside the repository, such as package/some/crate, without ., .. or empty segments',
      value
    });
  return value;
}

/**
 Accept a commit scope: letters, digits and the `_ . - /` of a package path, not starting with a slash or holding `..`.

 @param option - option or argument name for the message

 @param value - text under test

 @returns the value unchanged

 @throws CrateReleaseError when the scope has other characters, which would break the commit subject

 @example
 ```ts
 parseCommitScope({ option: '--scope', value: 'git-policy/cli' });
 ```
 */
export function parseCommitScope({
  option,
  value,
}: TextInput): string {
  if ((!usesOnly({
    text: value,
    allowed: commitScopeCharacters
  })) || value.startsWith('/')
    || value.endsWith('/')
    || value.includes('..'))
    throw refusal({
      option,
      expectation: 'a scope of letters, digits and _ . - / that does not start or end with a slash or hold ..',
      value
    });
  return value;
}

/**
 Accept a systemd resource value such as `12G`, `500%` or `infinity`.

 @param option - option or argument name for the message

 @param value - text under test

 @returns the value unchanged

 @throws CrateReleaseError when the value has characters outside letters, digits, dots and percent signs

 @example
 ```ts
 parseSystemdValue({ option: '--memory-max', value: '12G' });
 ```
 */
export function parseSystemdValue({
  option,
  value,
}: TextInput): string {
  if (!usesOnly({
    text: value,
    allowed: systemdValueCharacters
  }))
    throw refusal({
      option,
      expectation: 'a systemd value such as 12G or 500%',
      value
    });
  return value;
}

/**
 Accept a systemd unit name part: a letter or digit, then letters, digits, underscores and hyphens.

 @param option - option or argument name for the message

 @param value - text under test

 @returns the value unchanged

 @throws CrateReleaseError when the value has other characters

 @example
 ```ts
 parseUnitName({ option: '--unit-name', value: 'cli-git-publish-dry' });
 ```
 */
export function parseUnitName({
  option,
  value,
}: TextInput): string {
  if (!startsThenUses({
    text: value,
    first: alphanumericCharacters,
    rest: unitNameCharacters
  }))
    throw refusal({
      option,
      expectation: 'a unit name that starts with a letter or digit and uses only letters, digits, underscores and hyphens',
      value
    });
  return value;
}

/**
 Accept a mise task name that mise cannot read as an option.

 @param option - option or argument name for the message

 @param value - text under test

 @returns the value unchanged

 @throws CrateReleaseError when the value starts with a hyphen or has characters outside a task path

 @example
 ```ts
 parseTaskName({ option: '--lock-task', value: '//package/a/b.fuzz:lock:members' });
 ```
 */
export function parseTaskName({
  option,
  value,
}: TextInput): string {
  if ((!usesOnly({
    text: value,
    allowed: taskNameCharacters
  })) || value.startsWith('-'))
    throw refusal({
      option,
      expectation: 'a mise task such as //package/some/crate:lock:members that does not start with a hyphen',
      value
    });
  return value;
}

/**
 Accept a `User-Agent`: printable ASCII only, so no header can be split.

 @param option - option or argument name for the message

 @param value - text under test

 @returns the value unchanged

 @throws CrateReleaseError when the value is empty or holds a control or non-ASCII character

 @example
 ```ts
 parseUserAgent({ option: '--user-agent', value: 'crate-release-check' });
 ```
 */
export function parseUserAgent({
  option,
  value,
}: TextInput): string {
  if ((value === '')
    || Array.from(value)
      .some(function isUnprintable(character) {
      /**
       Code point of the character.
       */
        const code = character.codePointAt(0) ?? 0;
        return (code < firstPrintableCode) || (code > lastPrintableCode);
      }))
    throw refusal({
      option,
      expectation: 'non-empty printable ASCII text',
      value
    });
  return value;
}

/**
 Accept a workflow file name that `gh` cannot read as an option.

 @param option - option or argument name for the message

 @param value - text under test

 @returns the value unchanged

 @throws CrateReleaseError when the value starts with a hyphen or has other characters

 @example
 ```ts
 parseWorkflowName({ option: '--workflow', value: 'cargo-publish.yml' });
 ```
 */
export function parseWorkflowName({
  option,
  value,
}: TextInput): string {
  if ((!usesOnly({
    text: value,
    allowed: workflowNameCharacters
  })) || value.startsWith('-'))
    throw refusal({
      option,
      expectation: 'a workflow file name such as cargo-publish.yml that does not start with a hyphen',
      value
    });
  return value;
}

/**
 Accept a GitHub repository in `owner/name` form.

 @param option - option or argument name for the message

 @param value - text under test

 @returns the value unchanged

 @throws CrateReleaseError when the value is not exactly two plain parts separated by one slash

 @example
 ```ts
 parseRepository({ option: '--repo', value: 'Aquaticat/Monochromatic' });
 ```
 */
export function parseRepository({
  option,
  value,
}: TextInput): string {
  /**
   Owner and name, if the value has exactly those two parts.
   */
  const parts = value.split('/');
  if ((parts.length !== 2) || (!parts.every(function isPlain(part) {
    return usesOnly({
      text: part,
      allowed: repositoryPartCharacters
    }) && (!part.startsWith('-'));
  })))
    throw refusal({
      option,
      expectation: 'a repository such as owner/name',
      value
    });
  return value;
}

/**
 Accept a commit hash or prefix of it, and lower-case it as the GitHub API writes hashes.

 @param option - option or argument name for the message

 @param value - text under test

 @returns the hash in lower case

 @throws CrateReleaseError when the value is not seven to forty hexadecimal digits

 @example
 ```ts
 parseSha({ option: '<sha>', value: 'ABCDEF0' });
 ```
 */
export function parseSha({
  option,
  value,
}: TextInput): string {
  /**
   Lower-case form, which is also the form compared with the API.
   */
  const lowered = value.toLowerCase();
  if ((!usesOnly({
    text: lowered,
    allowed: hexadecimalCharacters
  })) || (lowered.length < minimumShaLength)
    || (lowered.length > maximumShaLength))
    throw refusal({
      option,
      expectation: `${String(minimumShaLength)} to ${String(maximumShaLength)} hexadecimal digits`,
      value
    });
  return lowered;
}

/**
 Accept a positive whole number written without a sign, leading zero or fraction.

 @param option - option or argument name for the message

 @param value - text under test

 @returns the number

 @throws CrateReleaseError when the text is not a positive whole number

 @example
 ```ts
 parseWholeNumber({ option: '--attempts', value: '45' });
 ```
 */
export function parseWholeNumber({
  option,
  value,
}: TextInput): number {
  if (!isPositiveWholeNumber({
    text: value,
    maximumDigits: maximumWholeNumberDigits
  }))
    throw refusal({
      option,
      expectation: 'a positive whole number',
      value
    });
  return Number(value);
}

/**
 Accept a name template: a single line of text that holds every required placeholder.

 @param option - option name for the message

 @param value - template under test

 @param required - placeholders the template must hold

 @returns the template unchanged

 @throws CrateReleaseError when the template is empty, holds a control character, or lacks a required placeholder

 @example
 ```ts
 parseTemplate({ option: '--tag-template', value: '{crate}-v{version}', required: ['{version}'] });
 ```
 */
export function parseTemplate({
  option,
  value,
  required,
}: TextInput & {
  readonly required: readonly string[];
}): string {
  if ((value === '') || hasControlCharacter(value)
    || (!required.every(function isHeld(placeholder) {
      return value.includes(placeholder);
    })))
    throw refusal({
      option,
      expectation: `one line of text that holds ${required.join(' and ')}`,
      value
    });
  return value;
}
