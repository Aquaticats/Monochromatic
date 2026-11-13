/**
 Reading as much of a JSON document as the text holds, so a document a caller keeps only a cut copy of still answers with every
 member that is whole.

 The feed keeps 500 characters of each tool result and a marker for the rest (`record-of-pi-run.ts`), so a `process` tool answer
 longer than that is cut in the middle of a string. `JSON.parse` refuses such a text and throws, and on 2026-11-13 that throw
 ended two runs as `driver-failed` after their agents had finished (`optique-two-remaining-copies-prototype-1` and
 `commit-fails-within-a-bound-and-says-why-when-its-service-does-not-answer-2`, issue 6295).

 What this reads is what the text holds of the document `JSON.parse` would have read: a member whose value is complete before the
 cut is read, and the member the cut falls inside is dropped with everything after it. Nothing is repaired, guessed at or invented.

 @module
 */

/**
 How deep a document may nest before this reads nothing more of it, so a cut copy holding a long run of `[` cannot spend the stack
 of the process that reads it.
 */
const DEEPEST_NESTING = 64;

/**
 The answer of a read that reached the end of the text inside a value.
 */
const INCOMPLETE: unique symbol = Symbol('the text ends inside this value');

/**
 Where one read has got to, and the text it reads.
 */
type Reading = {
  /**
   The text, which may be a document cut anywhere.
   */
  readonly shown: string;

  /**
   Where this read is in the text.
   */
  at: number;

  /**
   Whether this read has already abandoned a value the text ended inside, which is what makes the document an incomplete one.
   */
  truncated: boolean;
};

/**
 What reading a document as far as it goes answered.
 */
export type DocumentAsFarAsItGoes = {
  /**
   The document's value, holding every member that was whole, or nothing when the text held no value at all.
   */
  readonly value: unknown;

  /**
   Whether the text held the whole document.
   */
  readonly complete: boolean;
};

/**
 Reads as much of a JSON document as the text holds.

 @param shown - the text, which may be a document cut anywhere

 @returns every member that is whole, and whether the text held the whole document

 @example
 ```ts
 const read = asMuchJsonAsTheTextHolds({ shown: '{"details":{"id":"proc_7adb","name":"buil', },);
 ```
 */
export function asMuchJsonAsTheTextHolds({ shown, }: { readonly shown: string },): DocumentAsFarAsItGoes {
  /**
   Where this read is in the text.
   */
  const reading: Reading = {
    shown,
    at: 0,
    truncated: false,
  };
  /**
   The document's first value, or that the text ends inside it.
   */
  const value = readValueIn({
    reading,
    depth: 0,
  },);
  skipSpaces({ reading, },);
  return {
    value: (value === INCOMPLETE) ? undefined : value,
    complete: (!reading.truncated) && (value !== INCOMPLETE)
      && (reading.at === shown.length),
  };
}

/**
 Moves past the spaces and line endings between two values.

 @param reading - where the read is

 @example
 ```ts
 skipSpaces({ reading, },);
 ```
 */
function skipSpaces({ reading, }: { readonly reading: Reading },): void {
  while (reading.at
    < reading.shown
      .length) {
    /**
     The character at this position.
     */
    const character = reading.shown
      .charAt(reading.at,);
    if ((character !== ' ') && (character !== '\n')
      && (character !== '\r')
      && (character !== '\t')) {
      return;
    }
    reading.at += 1;
  }
}

/**
 Reads one value, whatever kind it is.

 @param reading - where the read is

 @param depth - how deep in the document this value sits

 @returns the value, or that the text ends inside it

 @example
 ```ts
 const value = readValueIn({ reading, depth: 0, },);
 ```
 */
function readValueIn({
  reading,
  depth,
}: {
  readonly reading: Reading;
  readonly depth: number;
},): unknown {
  skipSpaces({ reading, },);
  if ((reading.at
    >= reading.shown
      .length) || (depth > DEEPEST_NESTING)) {
    reading.truncated = true;
    return INCOMPLETE;
  }
  /**
   The character the value starts with.
   */
  const character = reading.shown
    .charAt(reading.at,);
  if (character === '{') {
    return readObjectIn({
      reading,
      depth,
    },);
  }
  if (character === '[') {
    return readArrayIn({
      reading,
      depth,
    },);
  }
  if (character === '"') {
    return readStringIn({ reading, },);
  }
  return readWordIn({ reading, },);
}

/**
 Reads one string, which is the value of a member or a key.

 @param reading - where the read is, at the opening quote

 @returns the string's characters, or that the text ends inside it

 @example
 ```ts
 const text = readStringIn({ reading, },);
 ```
 */
function readStringIn({ reading, }: { readonly reading: Reading },): unknown {
  /**
   The characters between the quotes, as the document spells them.
   */
  const characters: string[] = [];
  reading.at += 1;
  while (reading.at
    < reading.shown
      .length) {
    /**
     The character at this position.
     */
    const character = reading.shown
      .charAt(reading.at,);
    reading.at += 1;
    if (character === '"') {
      return characters.join('',);
    }
    if (character === '\\') {
      if (reading.at
        >= reading.shown
          .length) {
        reading.truncated = true;
        return INCOMPLETE;
      }
      characters.push(
        character,
        reading.shown
          .charAt(reading.at,),
      );
      reading.at += 1;
      continue;
    }
    characters.push(character,);
  }
  reading.truncated = true;
  return INCOMPLETE;
}

/**
 Reads one number, `true`, `false` or `null`.

 @param reading - where the read is

 @returns the value, or that the text ends inside it

 @example
 ```ts
 const value = readWordIn({ reading, },);
 ```
 */
function readWordIn({ reading, }: { readonly reading: Reading },): unknown {
  /**
   Where this word starts, which is where it is read from again.
   */
  const start = reading.at;
  while (reading.at
    < reading.shown
      .length) {
    /**
     The character at this position.
     */
    const character = reading.shown
      .charAt(reading.at,);
    if (' \n\r\t,]}'.includes(character,)) {
      return wordOf({ word: reading.shown
        .slice(
        start,
        reading.at,
        ), },);
    }
    reading.at += 1;
  }
  // A word the text ends inside may be half a number, so it is no value: nothing is read from a guess.
  reading.truncated = true;
  return INCOMPLETE;
}

/**
 Reads one word of a document as the value it spells.

 @param word - the word, whose characters a value is made of

 @returns the value, or nothing when the word spells no value this reads

 @example
 ```ts
 const value = wordOf({ word: 'null', },);
 ```
 */
function wordOf({ word, }: { readonly word: string },): unknown {
  if (word === 'null') {
    return null;
  }
  if (word === 'true') {
    return true;
  }
  if (word === 'false') {
    return false;
  }
  if (word === '') {
    return undefined;
  }
  /**
   The number the word spells, which is nothing for a word that is no number.
   */
  const number = Number(word,);
  return Number.isFinite(number,) ? number : undefined;
}

/**
 Reads one object, keeping every member whose value is whole.

 @param reading - where the read is, at the opening brace

 @param depth - how deep in the document this object sits

 @returns the members that are whole, or that the text ends inside the object's first value

 @example
 ```ts
 const details = readObjectIn({ reading, depth: 1, },);
 ```
 */
function readObjectIn({
  reading,
  depth,
}: {
  readonly reading: Reading;
  readonly depth: number;
},): unknown {
  /**
   The members read so far, in the document's own order.
   */
  const members: [
    string,
    unknown
  ][] = [];
  reading.at += 1;
  // Reading on until the object closes, or the document ends inside it.
  for (;;) {
    skipSpaces({ reading, },);
    if (reading.at
      >= reading.shown
        .length) {
      reading.truncated = true;
      break;
    }
    if (reading.shown
      .charAt(reading.at,)
      === '}') {
      reading.at += 1;
      break;
    }
    if ((members.length > 0) && (!tookComma({ reading, },))) {
      break;
    }
    /**
     The member's key.
     */
    const key = readStringIn({ reading, },);
    skipSpaces({ reading, },);
    if ((key === INCOMPLETE) || (reading.shown
      .charAt(reading.at,)
      !== ':')) {
      reading.truncated = true;
      break;
    }
    reading.at += 1;
    /**
     The member's value, which the document's end may fall inside.
     */
    const value = readValueIn({
      reading,
      depth: depth + 1,
    },);
    if (value === INCOMPLETE) {
      reading.truncated = true;
      break;
    }
    members.push([
      String(key,),
      value,
    ],);
  }
  return Object.fromEntries(members,);
}

/**
 Reads one array, keeping every element that is whole.

 @param reading - where the read is, at the opening bracket

 @param depth - how deep in the document this array sits

 @returns the elements that are whole, or that the text ends inside the array's first value

 @example
 ```ts
 const processes = readArrayIn({ reading, depth: 2, },);
 ```
 */
function readArrayIn({
  reading,
  depth,
}: {
  readonly reading: Reading;
  readonly depth: number;
},): unknown {
  /**
   The elements read so far.
   */
  const elements: unknown[] = [];
  reading.at += 1;
  // Reading on until the array closes, or the document ends inside it.
  for (;;) {
    skipSpaces({ reading, },);
    if (reading.at
      >= reading.shown
        .length) {
      reading.truncated = true;
      break;
    }
    if (reading.shown
      .charAt(reading.at,)
      === ']') {
      reading.at += 1;
      break;
    }
    if ((elements.length > 0) && (!tookComma({ reading, },))) {
      break;
    }
    /**
     The element, which the document's end may fall inside.
     */
    const element = readValueIn({
      reading,
      depth: depth + 1,
    },);
    if (element === INCOMPLETE) {
      reading.truncated = true;
      break;
    }
    elements.push(element,);
  }
  return elements;
}

/**
 Moves past the comma between two members or two elements.

 @param reading - where the read is

 @returns whether a comma was there to move past

 @example
 ```ts
 const took = tookComma({ reading, },);
 ```
 */
function tookComma({ reading, }: { readonly reading: Reading },): boolean {
  skipSpaces({ reading, },);
  if (reading.shown
    .charAt(reading.at,)
    !== ',') {
    return false;
  }
  reading.at += 1;
  return true;
}
