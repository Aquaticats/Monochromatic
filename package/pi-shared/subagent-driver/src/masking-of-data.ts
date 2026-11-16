/**
 What masking a value as data is: the value goes through the one serialisation Pi uses, the text is masked, and the answer is what
 that text parses back to.

 Pi writes and sends JSON, so masking what the value serialises to is one answer for everything an object can do: a key, a method,
 an accessor, a cycle. What comes back is plain data or the answer that it could not be masked, and never the object that went in.

 @module
 */

//region Types

/**
 What masking a value as data found: the data it became, or that it could not be masked and nothing of it is handed on.
 */
export type MaskedData =
  | { readonly value: unknown }
    | {
      readonly withheld: true;

    /**
     What made the value impossible to mask: it is not data that serialises, the masking itself failed, or masking it did not
     leave the same shape of data behind.
     */
      readonly cause: 'not-data' | 'not-maskable' | 'shape-changed'
    };

/**
 Why a value could not be masked.
 */
export type WithheldCause = 'not-data' | 'not-maskable' | 'shape-changed';

/**
 A value that was withheld, with the reason.
 */
export type Withheld = {
  readonly withheld: true;
  readonly cause: WithheldCause;
};

//endregion Types

//region The steps

/**
 A value as the text Pi will write it as.

 @param value - the value

 @returns the text, or that the value has no serialisation at all
 */
function serialise({ value, }: { readonly value: unknown },): { readonly text: string } | Withheld {
  try {
    /**
     The value as Pi's own serialisation writes it.
     */
    const text = JSON.stringify(value,);
    // A value with no serialisation at all, such as a function, would be written as nothing whatever the masking did.
    return ((text === undefined) || (text === ''))
      ? {
        withheld: true,
        cause: 'not-data',
      }
      : { text, };
  }
  catch (error) {
    if (!Error.isError(error,)) {
      throw error;
    }
    return {
      withheld: true,
      cause: 'not-data',
    };
  }
}

/**
 A text with every value in it masked.

 @param text - the text

 @param maskText - the masking

 @returns the masked text, or that the masking refused it
 */
function maskedText({
  text,
  maskText,
}: {
  readonly text: string;
  readonly maskText: (text: string) => string;
},): { readonly text: string } | Withheld {
  try {
    return { text: maskText(text,), };
  }
  catch (error) {
    if (!Error.isError(error,)) {
      throw error;
    }
    return {
      withheld: true,
      cause: 'not-maskable',
    };
  }
}

/**
 A text parsed back as data.

 @param text - the text

 @returns the data, or that the text is not data any more
 */
function parsedJSON({ text, }: { readonly text: string },): { readonly value: unknown } | Withheld {
  try {
    return { value: JSON.parse(text,), };
  }
  catch (error) {
    if (!Error.isError(error,)) {
      throw error;
    }
    return {
      withheld: true,
      cause: 'shape-changed',
    };
  }
}

/**
 Says whether two parsed values have the same shape: arrays as long, objects with as many keys, the same fields of primitives.

 Masking replaces text with text, so the only way a masked value loses data is a shape that changed: two keys that became one
 marker, a string that swallowed a delimiter. A value whose masked form has a different shape is not handed on.

 @param left - the value before masking

 @param right - the value after masking

 @returns whether the two have the same shape

 @example
 ```ts
 const same = sameShape({ left: { a: 1, }, right: { b: 2, }, },); // true
 ```
 */
function sameShape({
  left,
  right,
}: {
  readonly left: unknown;
  readonly right: unknown;
},): boolean {
  if (Array.isArray(left,) || Array.isArray(right,)) {
    return Array.isArray(left,) && Array.isArray(right,)
      && (left.length === right.length)
      && left.every(function samePart(
        part,
        index,
      ) {
        return sameShape({
          left: part,
          right: right[index],
        },);
      },);
  }
  if (((typeof left) === 'object') && (left !== null)
    && ((typeof right) === 'object')
    && (right !== null)) {
    /**
     The fields of the original.
     */
    const leftFields = Object.entries(left,);
    /**
     The fields of the masked one.
     */
    const rightFields = Object.entries(right,);
    return (leftFields.length === rightFields.length)
      // A key may be masked, so the keys themselves are expected to differ: what must not differ is how many fields the object
      // has and what each field holds, taken in the order both sides keep them.
      && leftFields.every(function sameField(
        [, field,],
        index,
      ) {
        return sameShape({
          left: field,
          right: rightFields[index]?.[1],
        },);
      },);
  }
  // A primitive keeps its type and may change its text, which is what masking is.
  return (typeof left) === (typeof right);
}

//endregion The steps

//region The answer

/**
 Masks a value as the data it serialises to, which is what Pi will serialise.

 @param value - the value to mask

 @param maskText - the masking of a text

 @returns the data it became, or that it could not be masked

 @example
 ```ts
 const masked = maskDataOf({ value: { credential: 'x', }, maskText, },);
 ```
 */
export function maskDataOf({
  value,
  maskText,
}: {
  readonly value: unknown;
  readonly maskText: (text: string) => string;
},): MaskedData {
  /**
   The value as the text Pi will write, or why it has none.
   */
  const serialised = serialise({ value, },);
  if ('withheld' in serialised) {
    return serialised;
  }
  /**
   The value as it went in, parsed again, which is what the masked shape is compared with.
   */
  const original = parsedJSON({ text: serialised.text, },);
  if ('withheld' in original) {
    return original;
  }
  /**
   The text with every value in it masked, or why the masking refused it.
   */
  const masked = maskedText({
    text: serialised.text,
    maskText,
  },);
  if ('withheld' in masked) {
    return masked;
  }
  /**
   What the masked text parses back to, or why it parses to nothing.
   */
  const parsed = parsedJSON({ text: masked.text, },);
  if ('withheld' in parsed) {
    return parsed;
  }
  // Two keys that masked to the same marker would leave one of them, which would lose data silently; a shape that changed for any
  // other reason is the same answer.
  return sameShape({
    left: original.value,
    right: parsed.value,
  },) ? { value: parsed.value, } : {
    withheld: true,
    cause: 'shape-changed',
  };
}

//endregion The answer
