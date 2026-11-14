/**
 What reading one source of this machine's secret values needs: the shapes a caller reports a source with, and the reading of the
 two kinds of file a value stands in beside the checkout's store.

 Pi's own files and the shell start-up files are read here, and neither stops a run when it cannot be read: each is a place a value
 may stand, and a run that says the place could not be read is a run that has said which of its values are not masked.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import type { EnvLocalJsonEnvironment, } from '@monochromatic-dev/env-local-json-core/ts';
import {
  carriesCredentialName,
  VARIABLES_KEPT_DESPITE_CREDENTIAL_NAME,
} from './environment-without-credentials.ts';
import { errorCodeOf, } from './error-code-of.ts';
import {
  MINIMUM_MASKED_VALUE_LENGTH,
  type SecretValue,
} from './secret-value-masking.ts';

//region Constants

/**
 The folder Pi keeps its own configuration in, under the home folder.
 */
const PI_AGENT_FOLDER = join(
  '.pi',
  'agent',
);

/**
 Pi's own files that hold a provider key in clear, which the rules file names beside the repository's store.
 */
const PI_CREDENTIAL_FILE_NAMES: readonly string[] = [
  'auth.json',
  'models.json',
];

/**
 The shell start-up files a terminal exports a token in, read from the home folder.
 */
const SHELL_START_FILE_NAMES: readonly string[] = [
  '.bashrc',
  '.bash_profile',
  '.profile',
];

/**
 An assignment of a shell start-up file: the variable's name, then its value as written.

 A regular expression is what reads one line of a shell file into a name and a value: the alternative is a shell parser, and a
 value may hold spaces, quotes and `=` of its own. The pattern is anchored at the line's start, has no nested quantifier, and each
 line is one line of a file this machine owns, so matching it is linear in the line and cannot backtrack.
 */
const SHELL_ASSIGNMENT_PATTERN =
  // oxlint-disable-next-line no-restricted-syntax/no-regex -- reading `NAME=value` from one line needs a pattern; see the comment above.
  /^\s*(?:export\s+|declare\s+-[a-zA-Z]*x[a-zA-Z]*\s+)?(?<name>[A-Za-z_][A-Za-z0-9_]*)=(?<written>.*)$/u;

/**
 What marks a value in a shell start-up file as computed when the shell loads, which this code cannot know.
 */
const SHELL_EXPANSION_MARKERS: readonly string[] = [
  '$',
  '`',
];

/**
 The name every error of the environment store reader starts with. Those are built for a reader and never hold a value, so their
 message is what a run's own message says when one of them stops the run.
 */
const SAFE_SECRET_SOURCE_ERROR_NAME_PREFIX = 'EnvLocalJson';

//endregion Constants

//region Types

/**
 One source the values came from, and how many it gave.
 */
export type SecretValueSourceReport = {
  /**
   What the source is, named for a reader: the store's file name, Pi's file, a shell start-up file, the age identity, or the
   driver's own environment.
   */
  readonly source: string;

  /**
   How many values it gave, which is a count and never a value.
   */
  readonly count: number;
};

/**
 A source that holds values and could not be read, so a run says which of its values are not masked.
 */
export type UnreadSecretValueSource = {
  readonly source: string;

  /**
   Why it could not be read, in words that never hold a value.
   */
  readonly cause: string;
};

/**
 What reading one group of sources found.
 */
export type ValuesAndTheirSources = {
  readonly values: readonly SecretValue[];
  readonly sources: readonly SecretValueSourceReport[];
  readonly unread: readonly UnreadSecretValueSource[];
};

/**
 The text of a file, or why it is not there to read.
 */
export type TextOfAFile =
  | { readonly text: string }
    | { readonly absent: true }
    | { readonly cause: string };

//endregion Types

//region Failures

/**
 Why a source failed, in words that never hold a value.

 The environment store reader's own errors are built for a reader and hold none, so their message is used as it stands. Any other
 error may hold a fragment of the file it was reading, so only its name is used.

 @param error - what the read threw

 @returns the cause, for a message or a feed line

 @example
 ```ts
 const cause = plainCauseOf(new SyntaxError('...'),);
 ```
 */
export function plainCauseOf(error: unknown,): string {
  if (!Error.isError(error,)) {
    return 'the read failed with something that is not an error';
  }
  return error.name
    .startsWith(SAFE_SECRET_SOURCE_ERROR_NAME_PREFIX,)
    ? error.message
    : `the read failed with ${error.name}`;
}

/**
 Says whether a failure is a path that is not there, which is not a source that could not be read.

 @param error - what the read threw

 @returns whether the operating system was asked for a path it does not have

 @example
 ```ts
 const missing = isMissingPathFailure({ error, },);
 ```
 */
export function isMissingPathFailure({ error, }: { readonly error: unknown },): boolean {
  return errorCodeOf(error,) === 'ENOENT';
}

//endregion Failures

//region Value rules

/**
 Says whether a value is one this masking replaces.

 A value shorter than the floor appears in ordinary text by chance, and a value that is an absolute path is where a secret is kept
 rather than a secret, so masking it would hide an ordinary file name from an agent and protect nothing.

 @param value - the value read from a source

 @returns whether a marker stands in its place

 @example
 ```ts
 const maskable = isMaskableValue('ghp_example',);
 ```
 */
export function isMaskableValue(value: string,): boolean {
  return (value.length >= MINIMUM_MASKED_VALUE_LENGTH) && (!value.startsWith('/',));
}

/**
 Says whether a variable's name is one the credential rule keeps on purpose, so the rule that takes a name from Pi's environment
 does not make this code mask a value that was judged worth keeping.

 @param name - the variable's name

 @returns whether the allow list holds it

 @example
 ```ts
 const kept = isKeptDespiteItsName('XAUTHORITY',);
 ```
 */
export function isKeptDespiteItsName(name: string,): boolean {
  return VARIABLES_KEPT_DESPITE_CREDENTIAL_NAME.some(function isThisOne(kept,) {
    return kept.name === name;
  },);
}

/**
 Keeps the first of two values that are the same, so a marker names the source that is most use to an agent reading it.

 @param values - every value read, in source order

 @returns the values, each once

 @example
 ```ts
 const once = withoutRepeatedValues([{ name: 'A', value: 'x', }, { name: 'B', value: 'x', },],);
 ```
 */
export function withoutRepeatedValues(values: readonly SecretValue[],): readonly SecretValue[] {
  /**
   The values already kept.
   */
  const kept = new Set<string>();
  return values.filter(function isNew(value,) {
    if (kept.has(value.value,)) {
      return false;
    }
    kept.add(value.value,);
    return true;
  },);
}

//endregion Value rules

//region Reading one file

/**
 Reads a file's text, saying which of the ways it can fail happened.

 @param path - the file

 @returns its text, that it is not there, or why it could not be read

 @example
 ```ts
 const read = await textOfAFile({ path: '/home/a/.pi/agent/auth.json', },);
 ```
 */
export async function textOfAFile({ path, }: { readonly path: string },): Promise<TextOfAFile> {
  try {
    return { text: await readFile(
      path,
      'utf8',
    ), };
  }
  catch (error) {
    if (isMissingPathFailure({ error, },)) {
      return { absent: true, };
    }
    return { cause: plainCauseOf(error,), };
  }
}

/**
 Reads a JSON file, saying which of the ways it can fail happened.

 @param path - the file

 @returns what it parsed to, that it is not there, or why it could not be read

 @example
 ```ts
 const read = await readJSONFile({ path: '/home/a/.pi/agent/auth.json', },);
 ```
 */
export async function readJSONFile({ path, }: { readonly path: string },): Promise<
  | { readonly parsed: unknown }
    | { readonly absent: true }
    | { readonly cause: string }
  > {
  /**
   The file's text, or why there is none.
   */
  const read = await textOfAFile({ path, },);
  if ('absent' in read) {
    return { absent: true, };
  }
  if ('cause' in read) {
    return { cause: read.cause, };
  }
  try {
    return { parsed: JSON.parse(read.text,), };
  }
  catch (error) {
    return { cause: `${path} is not JSON (${plainCauseOf(error,)})`, };
  }
}

//endregion Reading one file

//region Pi's own credential files

/**
 Walks a parsed file and returns every string that stands under a field whose own name marks a credential.

 A field's name is what says whether its value is a key: `models.json` holds provider names, model identifiers and base URLs
 beside the keys, and masking those would hide what an agent needs to work while protecting nothing.

 @param name - what a marker names the file as

 @param value - the parsed file

 @returns the values under credential-named fields

 @example
 ```ts
 const declared = valuesUnderCredentialFields({ name: 'auth.json', value: JSON.parse(text), },);
 ```
 */
export function valuesUnderCredentialFields({
  name,
  value,
}: {
  readonly name: string;
  readonly value: unknown;
},): readonly SecretValue[] {
  /**
   The values found so far.
   */
  const found: SecretValue[] = [];

  /**
   Walks one part, keeping the field path a marker names.

   @param part - the part of the file being walked

   @param fieldPath - the fields this part stands under
   */
  function walk({
    part,
    fieldPath,
  }: {
    readonly part: unknown;
    readonly fieldPath: readonly string[];
  },): void {
    if ((typeof part) === 'string') {
      /**
       The field this string stands in.
       */
      const field = fieldPath.at(-1,);
      if ((field === undefined) || (!carriesCredentialName({ name: field, },))
        || (!isMaskableValue(part,))) {
        return;
      }
      found.push({
        name: `${name} ${fieldPath.join('.',)}`,
        value: part,
      },);
      return;
    }
    if (Array.isArray(part,)) {
      part.forEach(function walkPart(
        element,
        index,
      ) {
        walk({
          part: element,
          fieldPath: [
            ...fieldPath,
            String(index,),
          ],
        },);
      },);
      return;
    }
    if (((typeof part) === 'object') && (part !== null)) {
      for (const [key, field] of Object.entries(part,)) {
        walk({
          part: field,
          fieldPath: [
            ...fieldPath,
            key,
          ],
        },);
      }
    }
  }

  walk({
    part: value,
    fieldPath: [],
  },);
  return found;
}

/**
 Reads one of Pi's own credential files.

 @param home - the home folder Pi's agent folder is in

 @param fileName - the file's name inside that folder

 @returns the values it declares, that it is not there, or why it could not be read

 @example
 ```ts
 const read = await valuesInOnePiCredentialFile({ home: '/home/a', fileName: 'auth.json', },);
 ```
 */
async function valuesInOnePiCredentialFile({
  home,
  fileName,
}: {
  readonly home: string;
  readonly fileName: string;
},): Promise<
  | { readonly values: readonly SecretValue[] }
    | { readonly absent: true }
    | { readonly cause: string }
  > {
  /**
   What the file parsed to, or why there is nothing to parse.
   */
  const read = await readJSONFile({ path: join(
    home,
    PI_AGENT_FOLDER,
    fileName,
  ), },);
  if ('absent' in read) {
    return { absent: true, };
  }
  if ('cause' in read) {
    return { cause: read.cause, };
  }
  return { values: valuesUnderCredentialFields({
    name: fileName,
    value: read.parsed,
  },), };
}

/**
 Reads the provider keys Pi keeps in clear in its own files, naming each by the file and the field it stands in.

 @param home - the home folder Pi's agent folder is in

 @returns the values, what each file gave, and the files that could not be read

 @example
 ```ts
 const read = await valuesInPiCredentialFiles({ home: '/home/a', },);
 ```
 */
export async function valuesInPiCredentialFiles({ home, }: { readonly home: string },): Promise<ValuesAndTheirSources> {
  /**
   Every value of every file that could be read.
   */
  const values: SecretValue[] = [];
  /**
   What each file gave.
   */
  const sources: SecretValueSourceReport[] = [];
  /**
   The files that hold keys and could not be read.
   */
  const unread: UnreadSecretValueSource[] = [];
  for (const fileName of PI_CREDENTIAL_FILE_NAMES) {
    /* oxlint-disable eslint/no-await-in-loop -- one file at a time, so the ones a reader has to be told about are named in the
       order they are read. See doc/lint/sequential-awaits.md. */
    /**
     What this file gave, or why there was nothing to read.
     */
    const read = await valuesInOnePiCredentialFile({
      fileName,
      home,
    },);
    /* oxlint-enable eslint/no-await-in-loop */
    if ('absent' in read) {
      continue;
    }
    if ('cause' in read) {
      unread.push({
        source: fileName,
        cause: read.cause,
      },);
      continue;
    }
    values.push(...read.values,);
    sources.push({
      source: fileName,
      count: read.values
        .length,
    },);
  }
  return {
    values,
    sources,
    unread,
  };
}

//endregion Pi's own credential files

//region A shell start-up file of this machine

/**
 Takes the quotes a shell file writes around a value, when the value wears a matching pair.

 @param written - the value as the assignment wrote it

 @returns the value the shell would export

 @example
 ```ts
 const value = unquote({ written: '"ghp_example"', },);
 ```
 */
function unquote({ written, }: { readonly written: string },): string {
  /**
   The value without the spaces an assignment may end with.
   */
  const trimmed = written.trimEnd();
  for (const quote of [
    '"',
    "'",
  ]) {
    if ((trimmed.length >= 2) && (trimmed.startsWith(quote,))
      && (trimmed.endsWith(quote,))) {
      return trimmed.slice(
        1,
        -1,
      );
    }
  }
  return trimmed;
}

/**
 Says whether a value is written out in the file rather than computed when the shell loads.

 @param value - the value as the shell would export

 @returns whether this code can know the value

 @example
 ```ts
 const written = isWrittenOut({ value: 'ghp_example', },);
 ```
 */
function isWrittenOut({ value, }: { readonly value: string },): boolean {
  return !SHELL_EXPANSION_MARKERS.some(function appearsIn(marker,) {
    return value.includes(marker,);
  },);
}

/**
 Reads the assignments of one shell start-up file that export a credential-named variable, as written.

 A value the shell computes (a variable reference, a command substitution) is skipped: this code cannot know what it becomes,
 and a wrong guess would mask ordinary text. A value written in quotes is unquoted, because that is what the shell exports.

 @param path - the start-up file

 @param fileName - what a marker names the file as

 @returns the values it exports, that it is not there, or why it could not be read

 @example
 ```ts
 const read = await valuesInOneShellStartFile({ path: '/home/a/.bashrc', fileName: '.bashrc', },);
 ```
 */
async function valuesInOneShellStartFile({
  path,
  fileName,
}: {
  readonly path: string;
  readonly fileName: string;
},): Promise<
  | { readonly values: readonly SecretValue[] }
    | { readonly absent: true }
    | { readonly cause: string }
  > {
  /**
   The file's text, or why there is none.
   */
  const read = await textOfAFile({ path, },);
  if ('absent' in read) {
    return { absent: true, };
  }
  if ('cause' in read) {
    return { cause: read.cause, };
  }
  /**
   The values the file exports.
   */
  const values: SecretValue[] = [];
  for (const line of read.text
    .split('\n',)) {
    /**
     The assignment this line is, if it is one.
     */
    const assignment = SHELL_ASSIGNMENT_PATTERN.exec(line,);
    if (assignment === null) {
      continue;
    }
    /**
     The variable's name and its value as written.
     */
    const [, name, written,] = assignment;
    if ((name === undefined) || (written === undefined)
      || (!carriesCredentialName({ name, },))) {
      continue;
    }
    /**
     The value without the quotes a shell file writes around it.
     */
    const unquoted = unquote({ written, },);
    if ((!isMaskableValue(unquoted,)) || (!isWrittenOut({ value: unquoted, },))) {
      continue;
    }
    values.push({
      name: `${fileName} ${name}`,
      value: unquoted,
    },);
  }
  return { values, };
}

/**
 Reads every shell start-up file of the home folder that exports a credential-named variable.

 @param home - the home folder

 @returns the values, what each file gave, and the files that could not be read

 @example
 ```ts
 const read = await valuesInShellStartFiles({ home: '/home/a', },);
 ```
 */
export async function valuesInShellStartFiles({ home, }: { readonly home: string },): Promise<ValuesAndTheirSources> {
  /**
   Every value the start-up files export.
   */
  const values: SecretValue[] = [];
  /**
   What each file gave.
   */
  const sources: SecretValueSourceReport[] = [];
  /**
   The files that could not be read.
   */
  const unread: UnreadSecretValueSource[] = [];
  for (const fileName of SHELL_START_FILE_NAMES) {
    /* oxlint-disable eslint/no-await-in-loop -- one file at a time, so the ones a reader has to be told about are named in the
       order they are read. See doc/lint/sequential-awaits.md. */
    /**
     What this file exports, or why there was nothing to read.
     */
    const read = await valuesInOneShellStartFile({
      fileName,
      path: join(
        home,
        fileName,
      ),
    },);
    /* oxlint-enable eslint/no-await-in-loop */
    if ('absent' in read) {
      continue;
    }
    if ('cause' in read) {
      unread.push({
        source: fileName,
        cause: read.cause,
      },);
      continue;
    }
    values.push(...read.values,);
    sources.push({
      source: fileName,
      count: read.values
        .length,
    },);
  }
  return {
    values,
    sources,
    unread,
  };
}

//endregion A shell start-up file of this machine

/**
 The environment type this package reads: the driver's own, read-only, which is what the store reader asks a caller for.
 */
export type EnvironmentOfThisMachine = EnvLocalJsonEnvironment;
