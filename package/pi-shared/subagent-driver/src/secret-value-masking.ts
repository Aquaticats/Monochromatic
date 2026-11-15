/**
 Replacing a value of this machine's secret environment, where it stands, with a marker that names it.

 A value reaches a model in more shapes than the one a file holds: a command prints it as JSON, an encoder prints it as base64, a
 URL prints it percent-encoded, and all of those print without anyone asking for the value. Each of those shapes is a form of the
 value, and each is replaced here with a marker that names the variable, so an agent reads what was taken and goes on without it.

 What this does not cover, and why: a transformation of a *container* of the value (base64 of a whole file, hex, gzip, base64
 wrapped at column 76, a substitution cipher) is not a form of the value, and matching it needs a decoder in front of every matcher
 rather than one pattern per value. The shapes this repository has seen a value leak through are a command's own output and a file
 printed with `cat` (`doc/troubleshooting/secret-values-reaching-a-model.md` beside this package), and those print the value itself.

 @module
 */

/**
 Says whether a value is one this masking replaces.

 A value shorter than the floor is not armed, because such a string appears in ordinary text by chance. What a value looks like
 decides nothing: a provider key may begin with `/`, and a source that says a field is a credential is believed over the shape of
 what stands in it. A variable whose value is a path is skipped by its own name in `secret-values-the-driver-holds.ts`, where that
 judgement has the name to make it with.

 @param value - the value read from a source

 @returns whether a marker stands in its place

 @example
 ```ts
 const maskable = isMaskableValue('ghp_example',);
 ```
 */
export function isMaskableValue(value: string,): boolean {
  return value.length >= MINIMUM_MASKED_VALUE_LENGTH;
}

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
 How many passes of replacement a text gets before this masking refuses it. One pass replaces every form, a second proves the
 markers changed nothing, and a third is room for a marker that held a form its construction did not foresee.
 */
const MAXIMUM_MASKING_PASSES = 3;

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
   The length of the longest form, which is how many characters a reader of a stream must hold back between two reads so a
   value split across them is still matched.
   */
  readonly longestFormLength: number;

  /**
   How many values are masked, which is what a marker can name.
   */
  readonly valueCount: number;
};

/**
 What a writer of a run's own files needs of the masking: the text of a record, and the strings inside one.
 */
export type TextMasking = {
  readonly maskText: (text: string) => string;
  readonly maskDeeply: <Value>(value: Value) => MaskedDeeply<Value>;

  /**
   The length of the longest form, which is how many characters a writer of a stream holds back between two writes.
   */
  readonly longestFormLength: number;
};

/**
 The masking a run whose values are not known yet writes with, which changes nothing.
 */
export const MASKING_NOTHING: TextMasking = {
  longestFormLength: 0,
  maskText: function maskNoText(text,) {
    return text;
  },
  maskDeeply: function maskNoValue<Value>(value: Value,): MaskedDeeply<Value> {
    return {
      value,
      changed: false,
    };
  },
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

/**
 The marker that stands where a value was when what a marker would name is itself a value: an index into the values, which says
 nothing a value could hold.

 @param index - the value's place in the list, counting from one

 @returns the marker

 @example
 ```ts
 const text = maskedValueAliasMarker(3,);
 ```
 */
export function maskedValueAliasMarker(index: number,): string {
  return `${MASKED_VALUE_OPENING}#${String(index,)}${MASKED_VALUE_CLOSING}`;
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
   The values this masking replaces, each with the marker that stands in its place. A value shorter than the floor is not among
   them: a two-letter or three-letter string appears in ordinary text by chance, and replacing it corrupts every tool result that
   happens to hold it while hiding nothing.
   */
  const arms: {
    readonly forms: readonly string[];
    readonly marker: string
  }[] = [];
  /**
   The marker each form is replaced with.
   */
  const markerOfForm = new Map<string, string>();
  /**
   The forms already armed by an earlier value, so a value that repeats another's value is armed once.
   */
  const alreadyArmed = new Set<string>();
  /**
   The forms longest first, so a form that holds another is matched before that other one.
   */
  const formsLongestFirst: string[] = [];
  for (const [index, {
    name,
    value,
  },] of values.entries()) {
    if (value.length < MINIMUM_MASKED_VALUE_LENGTH) {
      continue;
    }
    /**
     The forms this value is printed in.
     */
    const forms = formsOfOneValue(value,)
      .filter(function isNotAlreadyArmed(form,) {
        return !alreadyArmed.has(form,);
      },);
    arms.push({
      forms,
      // A marker names the variable in the plain form. When that name would itself hold an armed value or one of its forms, the
      // marker says the value's place in the list instead, so a marker never discloses what another marker replaced.
      marker: maskedValueMarker(JSON.stringify(name,)
        .slice(
        1,
        -1,
        ),),
    },);
    for (const form of forms) {
      alreadyArmed.add(form,);
      formsLongestFirst.push(form,);
    }
  }
  formsLongestFirst.sort(function longerFirst(
    left,
    right,
  ) {
    return right.length - left.length;
  },);
  /**
   One pattern of every form at once, which is what looks for all of them in one pass.
   */
  // oxlint-disable-next-line no-restricted-syntax/no-regex -- one alternation of escaped literals is what makes this one pass; see the comment above.
  const anyForm = (formsLongestFirst.length === 0) ? undefined : new RegExp(
    formsLongestFirst.map(escapedForRegExp,)
      .join('|',),
    'gu',
  );
  /**
   The same pattern without the flag that makes a regular expression remember where it stopped, which is what a question about a
   text asks.
   */
  // oxlint-disable-next-line no-restricted-syntax/no-regex -- the same alternation of escaped literals as the pass above, asked as a question.
  const anyFormIn = (formsLongestFirst.length === 0) ? undefined : new RegExp(
    formsLongestFirst.map(escapedForRegExp,)
      .join('|',),
    'u',
  );

  /**
   Says whether a text holds a form of an armed value.

   @param text - the text to ask about

   @returns whether an armed value, in any of its forms, stands in it
   */
  function holdsAForm(text: string,): boolean {
    return (anyFormIn === undefined) ? false : anyFormIn.test(text);
  }

  for (const [index, arm,] of arms.entries()) {
    /**
     The marker that names this value in the plain form.
     */
    const named = arm.marker;
    /**
     The marker that names it by its place, which holds no name a value could be.
     */
    const aliased = maskedValueAliasMarker(index + 1,);
    /**
     The marker this arm uses.
     */
    const marker = holdsAForm(named,) ? aliased : named;
    if (holdsAForm(marker,)) {
      throw new SecretValueMaskingError('a marker for a secret value would hold a form of an armed value, so no marker names it',);
    }
    for (const form of arm.forms) {
      markerOfForm.set(
        form,
        marker,
      );
    }
  }

  /**
   Replaces every form in a text with its marker, and proves the answer holds none.

   @param text - the text to mask

   @returns the text with every value in it replaced by its marker

   @throws SecretValueMaskingError when a text still holds a form of an armed value after the passes this masking allows
   */
  function maskText(text: string,): string {
    if ((!holdsAForm(text,)) || (anyForm === undefined)) {
      return text;
    }
    /**
     The text each pass is handed, and how many passes have run, in holders so the loop names its own state.
     */
    const state: {
      text: string;
      taken: number
    } = {
      text,
      taken: 0,
    };
    while (state.taken < MAXIMUM_MASKING_PASSES) {
      state.taken += 1;
      /**
       The text with every form this pass found replaced.
       */
      const replaced = state.text
        .replace(
        anyForm,
        function replaceWithItsMarker(form,) {
          /**
           The marker this form's value stands behind.
           */
          const marker = markerOfForm.get(form,);
          if (marker === undefined) {
            throw new SecretValueMaskingError('a matched form of a secret value has no marker to put in its place',);
          }
          return marker;
        },
        );
      if (!holdsAForm(replaced,)) {
        return replaced;
      }
      // A marker that is the very text it replaced cannot converge, and one that still holds a form would be a marker built
      // wrongly; either way another pass changes nothing, so the loop stops here and the answer is refused below.
      if (replaced === state.text) {
        break;
      }
      state.text = replaced;
    }
    throw new SecretValueMaskingError('a text still holds a form of a secret value after masking it',);
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
    longestFormLength: formsLongestFirst[0]
      ?.length
      ?? 0,
  };
}
