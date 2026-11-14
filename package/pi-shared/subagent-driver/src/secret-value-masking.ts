/**
 Replacing a value of this machine's secret environment, where it stands, with a marker that names it.

 A value reaches a model in more shapes than the one a file holds: a command prints it as JSON, an encoder prints it as base64, a
 URL prints it percent-encoded, and all of those print without anyone asking for the value. Each of those shapes is a form of the
 value, and each is replaced here with a marker that names the variable, so an agent reads what was taken and goes on without it.

 What this does not cover, and why: a transformation of a *container* of the value (base64 of a whole file, hex, gzip, base64
 wrapped at column 76, a substitution cipher) is not a form of the value, and matching it needs a decoder in front of every matcher
 rather than one pattern per value. The shapes this repository has seen a value leak through are a command's own output and a file
 printed with `cat` (`doc/troubleshooting/secret-values-reaching-a-model.md`), and those print the value itself.

 @module
 */

/**
 What opens the marker that stands where a value was.
 */
export const MASKED_VALUE_OPENING = '[masked secret value of ';

/**
 What closes that marker.
 */
export const MASKED_VALUE_CLOSING = ']';

/**
 The shortest value that is masked.

 A value shorter than this is not masked, because a two-letter or three-letter string appears in ordinary text by chance, and
 replacing it corrupts every tool result that happens to hold it while hiding nothing. Every value the repository's own secret
 environment declares is at least ten characters (`keep/measure-store-value-lengths.ts` in the report's scratch folder), so the
 floor stands below the shortest of them with room, and a value shorter than it is named as a limit of this masking rather than
 guessed at.
 */
export const MINIMUM_MASKED_VALUE_LENGTH = 8;

/**
 The characters a regular expression's grammar reserves, which a form of a value has to escape to be matched literally.
 */
const RESERVED_BY_THE_PATTERN_GRAMMAR: ReadonlySet<string> = new Set('. * + ? ^ $ { } ( ) | [ ] \\'.split(' ',),);

/**
 One value of this machine's secret environment, and what a marker says in its place.
 */
export type SecretValue = {
  /**
   What the marker names: the variable, or the file and the field, that declared the value.
   */
  readonly name: string;

  /**
   The value itself. It is never logged, written, summarised or thrown in a message.
   */
  readonly value: string;
};

/**
 What one walk of a value found.
 */
export type MaskedDeeply<Value> = {
  /**
   The value, with every string in it masked. It is the value handed in when nothing changed.
   */
  readonly value: Value;

  /**
   Whether anything was masked.
   */
  readonly changed: boolean;
};

/**
 The masking of every value of this machine's secret environment.
 */
export type SecretValueMasking = {
  /**
   The text with every value in it replaced by its marker.
   
   @throws SecretValueMaskingError when a matched form has no name, which cannot happen and is never passed off as masked text
   */
  readonly maskText: (text: string) => string;

  /**
   Every string inside a value masked, so a whole message or a whole transcript is masked in one call. The shape a value has is
   the shape it keeps: this replaces text with text and touches nothing else, so a caller can hand in a message and read one back.
   */
  readonly maskDeeply: <Value>(value: Value) => MaskedDeeply<Value>;

  /**
   How many forms are looked for, which is what a test measures the cost of.
   */
  readonly patternCount: number;

  /**
   How many values are masked, which is what a marker can name.
   */
  readonly valueCount: number;
};

/**
 A form that was matched and could not be named. Reported rather than passed off as masked text.
 */
export class SecretValueMaskingError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'SecretValueMaskingError';
}

/**
 The marker that stands where a value was.

 @param name - what the marker names

 @returns the marker

 @example
 ```ts
 const text = maskedValueMarker('MISE_GITHUB_TOKEN',);
 ```
 */
export function maskedValueMarker(name: string,): string {
  return `${MASKED_VALUE_OPENING}${name}${MASKED_VALUE_CLOSING}`;
}

//region The forms a command prints a value in

/**
 The base64 forms of a value: standard and URL-safe alphabet, with and without the padding an encoder adds.

 `base64` on this machine writes one line for a value shorter than 57 bytes and wraps a longer one at 76 columns, so a wrapped
 form is not among these and is named as a limit of this masking.

 @param value - the value

 @returns the forms, each still at least as long as the value
 */
function base64FormsOf(value: string,): readonly string[] {
  /**
   The value's bytes, which is what an encoder encodes.
   */
  const bytes = Buffer.from(
    value,
    'utf8',
  );
  /**
   Both alphabets, padded.
   */
  const padded = [
    bytes.toString('base64',),
    bytes.toString('base64url',),
  ];
  return [
    ...padded,
    ...padded.map(function withoutPadding(form,) {
      // An encoder pads with `=` and only at the end, so removing every one of them removes the padding.
      return form.replaceAll(
        '=',
        '',
      );
    },),
  ];
}

/**
 Every form of one value that a command prints without being asked to hide anything: the value itself, the value as JSON escapes
 it inside a string, the value as a URL percent-encodes it, and the value as base64 encodes it.

 @param value - the value

 @returns the distinct forms, in no particular order
 */
function formsOfOneValue(value: string,): readonly string[] {
  /**
   The value inside a JSON string, without the quotes `JSON.stringify` adds around it.
   */
  const jsonEscaped = JSON.stringify(value,)
    .slice(
    1,
    -1,
    );
  return [
    value,
    jsonEscaped,
    encodeURIComponent(value,),
    ...base64FormsOf(value,),
  ];
}

//endregion The forms a command prints a value in

//region Escaping for the pattern

/**
 Escapes text so a regular expression matches it literally.

 @param text - text to escape

 @returns the text with every character the pattern grammar reserves escaped

 @example
 ```ts
 const pattern = escapedForRegExp('a.b',);
 ```
 */
function escapedForRegExp(text: string,): string {
  return Array.from(text,)
    .map(function escapeOne(character,) {
      return RESERVED_BY_THE_PATTERN_GRAMMAR.has(character,) ? `\\${character}` : character;
    },)
    .join('',);
}

//endregion Escaping for the pattern

/**
 Starts the masking of every value of this machine's secret environment.

 One pass looks for every form at once. The pattern is compiled from forms that are escaped literals, so it holds no grammar a
 value could contribute and cannot backtrack; the text it is given is one tool result or one transcript, both bounded by Pi, and
 every match is replaced once.

 @param values - the values, each with what a marker names in its place

 @returns the masking, which is never given a value to log or return

 @example
 ```ts
 const masking = createSecretValueMasking({ values, },);
 const masked = masking.maskText('MISE_GITHUB_TOKEN=ghp_example',);
 ```
 */
export function createSecretValueMasking({
  values,
}: {
  readonly values: readonly SecretValue[];
},): SecretValueMasking {
  /**
   The name each form is masked with, so a marker says which variable was taken.
   */
  const nameOfForm = new Map<string, string>();
  for (const {
    name,
    value,
  } of values) {
    if (value.length < MINIMUM_MASKED_VALUE_LENGTH) {
      continue;
    }
    for (const form of formsOfOneValue(value,)) {
      if (!nameOfForm.has(form,)) {
        nameOfForm.set(
          form,
          name,
        );
      }
    }
  }
  /**
   The forms longest first, so a form that holds another is matched before that other one.
   */
  const formsLongestFirst = [...nameOfForm.keys(),].toSorted(function longerFirst(
    left,
    right,
  ) {
    return right.length - left.length;
  },);
  /**
   One pass over the text for every form, rather than one pass per form.
   */
  // oxlint-disable-next-line no-restricted-syntax/no-regex -- one alternation of escaped literals is what makes this one pass; see the comment above.
  const anyForm = (formsLongestFirst.length === 0) ? undefined : new RegExp(
    formsLongestFirst.map(escapedForRegExp,)
      .join('|',),
    'gu',
  );

  /**
   Replaces every form in a text with its marker.

   @param text - the text to mask

   @returns the text with every value in it replaced by its marker
   */
  function maskText(text: string,): string {
    if ((anyForm === undefined) || (text === '')) {
      return text;
    }
    return text.replace(
      anyForm,
      function replaceWithItsMarker(form,) {
        /**
         The name the marker says.
         */
        const name = nameOfForm.get(form,);
        if (name === undefined) {
          throw new SecretValueMaskingError('a matched form of a secret value has no name to put in its marker',);
        }
        return maskedValueMarker(name,);
      },
    );
  }

  /**
   Masks every string inside a value, keeping the parts nothing was masked in.

   @param value - the value to mask

   @returns the value with every string in it masked, and whether anything was masked
   */
  function maskDeeplyOfUnknown(value: unknown,): MaskedDeeply<unknown> {
    if ((typeof value) === 'string') {
      /**
       The string after masking.
       */
      const masked = maskText(value,);
      return {
        value: masked,
        changed: masked !== value,
      };
    }
    if (Array.isArray(value,)) {
      /**
       The parts after masking, and whether any of them changed.
       */
      const parts = value.map(function maskPart(part,) {
        return maskDeeplyOfUnknown(part,);
      },);
      /**
       Whether any part held a value.
       */
      const changed = parts.some(function didChange(part,) {
        return part.changed;
      },);
      return {
        value: changed ? parts.map(function valueOfPart(part,) {
          return part.value;
        },) : value,
        changed,
      };
    }
    if (((typeof value) === 'object') && (value !== null)) {
      /**
       The object's fields after masking.
       */
      const fields = Object.entries(value,)
        .map(function maskField([key, field,],) {
          return [
            key,
            maskDeeply(field,),
          ] as const;
        },);
      /**
       Whether any field held a value.
       */
      const changed = fields.some(function didChange([, part,],) {
        return part.changed;
      },);
      return {
        value: changed ? Object.fromEntries(fields.map(function valueOfField([key, part,],) {
          return [
            key,
            part.value,
          ];
        },)) : value,
        changed,
      };
    }
    return {
      value,
      changed: false,
    };
  }

  /**
   Masks every string inside a value, whose shape is the shape it keeps.

   @param value - the value to mask

   @returns the value with every string in it masked, and whether anything was masked
   */
  function maskDeeply<Value>(value: Value,): MaskedDeeply<Value> {
    // Masking replaces a string with a string and keeps every other part exactly as it was, so the shape handed in is the shape
    // that comes back. TypeScript cannot check a rewrite of a structure against the structure's own type, and this is the one
    // place where the shape is carried over rather than narrowed.
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- masking keeps the shape it was given; see the comment above.
    return maskDeeplyOfUnknown(value,) as MaskedDeeply<Value>;
  }

  return {
    maskText,
    maskDeeply,
    valueCount: values.length,
    patternCount: formsLongestFirst.length,
  };
}
