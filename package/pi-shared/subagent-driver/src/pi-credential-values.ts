/**
 The credentials Pi keeps in its own files: its `auth.json` by the schema Pi documents, and any other file of its own by the names
 of the fields that hold a credential.

 Pi's `auth.json` holds one type-tagged credential per provider
(`dist/auth/types.d.ts` in pi-ai 1.5.2): an `api_key` credential whose secret is in `key`, and an `oauth` credential whose secrets
are in `access` and `refresh`. Those three names are read by the schema, whatever a generic name rule would say about them, so an
OAuth token is armed exactly as an API key is. Nothing else in that file is armed: `type` and `expires` are not secrets, and `env`
holds provider-scoped configuration such as an account identifier.

 Every other file this reads is configuration with a credential in it, so a string is armed when its own field's name marks a
 credential, which is the same rule the driver's environment uses.

 @module
 */

import { carriesCredentialName, } from './environment-without-credentials.ts';
import {
  isMaskableValue,
  type SecretValue,
} from './secret-value-masking.ts';

//region Constants

/**
 The names Pi's own credential schema gives its secrets, which are armed whatever a name rule would make of them.
 */
export const PI_CREDENTIAL_FIELD_NAMES: readonly string[] = [
  'key',
  'access',
  'refresh',
];

/**
 The file whose shape is Pi's credential schema rather than a provider configuration.
 */
const PI_CREDENTIAL_FILE_NAME = 'auth.json';

//endregion Constants

//region Reading one file

/**
 Says whether a field's name is one Pi's own credential schema names as a secret.

 @param field - the field's name

 @returns whether the schema puts a credential there

 @example
 ```ts
 const names = isPiCredentialField('access',); // true
 ```
 */
export function isPiCredentialField(field: string,): boolean {
  return PI_CREDENTIAL_FIELD_NAMES.includes(field,);
}

/**
 Walks a parsed file and returns every string that is a credential of it.

 @param fileName - the file's name, which says whether it follows Pi's credential schema

 @param parsed - the parsed file

 @returns the credentials, each named by the file and the fields it stands under

 @example
 ```ts
 const declared = valuesInPiFile({ fileName: 'auth.json', parsed: JSON.parse(text), },);
 ```
 */
export function valuesInPiFile({
  fileName,
  parsed,
  text,
}: {
  readonly fileName: string;
  readonly parsed: unknown;

  /**
   The file's own text, which spells a value it writes as JSON escapes where the value itself may not.
   */
  readonly text?: string;
},): readonly SecretValue[] {
  /**
   Whether this file is Pi's own credential store, whose secrets are named by the schema rather than by a rule.
   */
  const bySchema = fileName === PI_CREDENTIAL_FILE_NAME;
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
       The field this string stands in, which is what says whether it is a credential.
       */
      const field = fieldPath.at(-1,);
      /**
       Whether this field is one Pi's schema names, or one whose own name marks a credential.
       */
      const isCredential = (field !== undefined)
        && ((bySchema && isPiCredentialField(field,)) || carriesCredentialName({ name: field, },));
      if ((!isCredential) || (!isMaskableValue(part,))) {
        return;
      }
      found.push({
        name: `${fileName} ${fieldPath.join('.',)}`,
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
    part: parsed,
    fieldPath: [],
  },);
  // A file that spells a character as a JSON `\uXXXX` escape holds the value in that spelling too, and a tool that prints the file
  // prints it that way. The spelling is armed beside the decoded value, under the same name, so the marker still says where the
  // value stood.
  return [
    ...found,
    ...spellingsIn({
      ...(text === undefined ? {} : { text, }),
      values: found,
    },),
  ];
}

/**
 Reads one JSON string token as the value it spells.

 @param written - the token, quotes and all

 @returns the value it spells, or that it is not a string this code can read
 */
function decodedString({ written, }: { readonly written: string },): { readonly decoded: string } | {
  readonly notAString: true;
} {
  try {
    /**
     What the token decodes to.
     */
    const decoded: unknown = JSON.parse(written,);
    return ((typeof decoded) === 'string') ? { decoded, } : { notAString: true, };
  }
  catch (error) {
    if (!Error.isError(error,)) {
      throw error;
    }
    return { notAString: true, };
  }
}

/**
 Every spelling of an armed value that the file writes and the value itself does not.

 @param text - the file's own text

 @param values - the values the file declares

 @returns one value per spelling, named as the value it spells is named
 */
function spellingsIn({
  text,
  values,
}: {
  readonly text?: string;

  /**
   The values the file declares, by their own text.
   */
  readonly values: readonly SecretValue[];
},): readonly SecretValue[] {
  if (text === undefined) {
    return [];
  }
  /**
   The decoded text of each value the file declares, which is what a spelling decodes to.
   */
  const nameOfText = new Map(values.map(function toEntry(value,) {
    return [
      value.value,
      value.name,
    ] as const;
  },),);
  /**
   The spellings found.
   */
  const spellings: SecretValue[] = [];
  // A JSON string token is a shape, and the file is one credential file of this machine: the walk is over that file's own text,
  // anchored to quoted runs with no nested quantifier, so it is linear in the file.
  // oxlint-disable-next-line no-restricted-syntax/no-regex -- see the comment above.
  for (const match of text.matchAll(/"(?:[^"\\]|\\.)*"/gu,)) {
    /**
     The text between the quotes, as the file writes it.
     */
    const written = match[0]
      .slice(
      1,
      -1,
      );
    /**
     What that text decodes to, or that it is not a JSON string this code can read.
     */
    const read = decodedString({ written: match[0], },);
    if ('notAString' in read) {
      continue;
    }
    /**
     The value the token spells.
     */
    const { decoded, } = read;
    /**
     The name the value it spells is armed with.
     */
    const name = nameOfText.get(decoded,);
    if ((name !== undefined) && (written !== decoded)
      && (!nameOfText.has(written,))) {
      spellings.push({
        name,
        value: written,
      },);
    }
  }
  return spellings;
}

//endregion Reading one file
