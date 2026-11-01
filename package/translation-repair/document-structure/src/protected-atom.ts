//region Protected atoms
// The things a naturalness rewrite may reorder words around but must never
// change: destinations, identifiers, code, numbers, and foreign-language runs.
//
// They are compared as an ORDERED SEQUENCE, never as a multiset. A multiset
// admits swaps that pass the gate while changing what the text says: "3 cats
// and 5 dogs" becoming "5 cats and 3 dogs", two links exchanging destinations,
// two names exchanging positions. Order is what makes those fail.
//
// Numbers and foreign runs are NOT mdast nodes, so they are scanned out of text
// leaves only, never out of Markdown syntax. Scanning the raw source would find
// digits inside link destinations and inline code and report changes that never
// happened.

/**
 What kind of thing an atom is, kept on the atom so a gate failure can say
 what changed rather than only that something did.

 @example
 ```ts
 const kind: AtomKind = 'link-url';
 ```
 */
export type AtomKind =
  | 'link-url'
    | 'image-url'
    | 'reference'
    | 'footnote'
    | 'inline-code'
    | 'container-tag'
    | 'number'
    | 'foreign-run';

/**
 One thing a rewrite must carry through unchanged and in place.

 @example
 ```ts
 const atom: ProtectedAtom = { kind: 'number', value: '1,200', };
 ```
 */
export type ProtectedAtom = {
  /**
   What the atom is.
   */
  readonly kind: AtomKind;

  /**
   Exact bytes that must survive.
   */
  readonly value: string;
};

/**
 Separators that stay INSIDE a number when digits sit on both sides, so
 `1,200`, `3.5`, and `12:30` are each one token rather than several.

 A date like `2019-05-01` is deliberately three tokens: a hyphen is far more
 often prose punctuation than part of a number, and splitting there costs
 nothing because all three parts still have to survive in order.
 */
const NUMBER_SEPARATORS = '.,:';

/**
 Highest code point that fits in one UTF-16 unit; above it a character
 occupies a surrogate pair, which is why the scan advances by a computed
 width rather than by one.
 */
const BASIC_PLANE_MAX = 0xFF_FF;

/**
 Unicode blocks whose characters count as foreign-language content, each named
 so a reader can check a boundary against the standard without decoding hex.

 Punctuation blocks are deliberately absent, because corpus prose mixes CJK
 punctuation into English sentences and a rewrite is allowed to repunctuate.

 The supplementary range is why the scan walks code points rather than UTF-16
 units: a rare given name in Han Extension B is a surrogate pair, and reading
 it as two units would match neither half against any range here, silently
 leaving the one character most likely to be a person's name unprotected.
 */
const FOREIGN_BLOCKS = {
  kana: {
    first: 0x30_40,
    last: 0x30_FF,
  },
  hanExtensionA: {
    first: 0x34_00,
    last: 0x4D_BF,
  },
  han: {
    first: 0x4E_00,
    last: 0x9F_FF,
  },
  hangulSyllables: {
    first: 0xAC_00,
    last: 0xD7_AF,
  },
  hanCompatibility: {
    first: 0xF9_00,
    last: 0xFA_FF,
  },
  hanSupplementary: {
    first: 0x2_00_00,
    last: 0x3_23_AF,
  },
} as const;

/**
 Those blocks as a list, for the membership scan.
 */
const FOREIGN_RANGES: readonly {
  readonly first: number;
  readonly last: number;
}[] = Object.values(FOREIGN_BLOCKS,);

/**
 Digit blocks, named for the same reason the foreign blocks are: a reader
 checking whether full-width digits are covered should not have to decode hex.
 */
const DIGIT_BLOCKS = {
  ascii: {
    first: 0x00_30,
    last: 0x00_39,
  },
  fullwidth: {
    first: 0xFF_10,
    last: 0xFF_19,
  },
} as const;

/**
 Whether a code point is a digit in either the ASCII or the full-width form.

 @param codePoint - code point under test

 @returns Whether it reads as a digit

 @example
 ```ts
 isDigit(0x0037,);
 ```
 */
function isDigit(codePoint: number,): boolean {
  return ((codePoint
    >= DIGIT_BLOCKS.ascii
      .first) && (codePoint
        <= DIGIT_BLOCKS.ascii
          .last))
    || ((codePoint
      >= DIGIT_BLOCKS.fullwidth
        .first) && (codePoint
          <= DIGIT_BLOCKS.fullwidth
            .last));
}

/**
 Whether a code point falls in a foreign-language run.

 @param codePoint - code point under test

 @returns Whether it belongs to a protected script

 @example
 ```ts
 isForeign('猫'.codePointAt(0,) ?? 0,);
 ```
 */
function isForeign(codePoint: number,): boolean {
  return FOREIGN_RANGES.some(function within(range,) {
    return (codePoint >= range.first) && (codePoint <= range.last);
  },);
}

/**
 Scans one text leaf for the atoms that are characters rather than nodes.

 One linear pass over code points, emitting a token whenever a run ends.
 Separators join a number only when a digit follows, which is what keeps the
 period ending a sentence out of the number before it.

 @param text - decoded text of one mdast text leaf

 @returns Number and foreign-run atoms in the order they appear

 @example
 ```ts
 const atoms = scanTextAtoms({ text: 'she was 17 in 2019', },);
 ```
 */
export function scanTextAtoms({ text, }: { readonly text: string; },): readonly ProtectedAtom[] {
  /**
   Atoms in appearance order.
   */
  const atoms: ProtectedAtom[] = [];

  /**
   Cursor over the leaf, in UTF-16 units, shared with the flush so a run can
   be sliced once at its end rather than rebuilt per character.
   */
  const cursor = { at: 0, };

  /**
   Where the run currently open starts, and which kind it is.
   */
  const run = {
    kind: 'none' as 'none' | 'number' | 'foreign',
    start: 0,
  };

  /**
   Closes the open run, if any, into an atom.

   A number run always ends on a digit: a separator is only ever appended
   when a digit follows it, and that digit is appended on the next step. So
   nothing here has to strip a trailing separator.

   @example
   ```ts
   flush();
   ```
   */
  function flush(): void {
    if (run.kind === 'none')
      return;
    atoms.push({
      kind: run.kind === 'number' ? 'number' : 'foreign-run',
      value: text.slice(
        run.start,
        cursor.at,
      ),
    },);
    run.kind = 'none';
  }

  // Walked by CODE POINT rather than by UTF-16 unit or by spread. Han
  // Extension B and beyond live above the basic plane, so `charAt` would split
  // one character into two surrogates and read neither as foreign; spreading
  // the string would be correct here but breaks grapheme clusters in general.
  // `codePointAt` plus an explicit width is right for both.
  while (cursor.at < text.length) {
    /**
     Code point at the cursor, present because the cursor is in range.
     */
    const codePoint = text.codePointAt(cursor.at,);
    if (codePoint === undefined)
      throw new Error(`unreachable: the cursor stands at offset ${String(cursor.at,)} of a leaf whose length is ${String(text.length,)}, where a code point begins`,);

    /**
     UTF-16 units this code point occupies.
     */
    const width = codePoint > BASIC_PLANE_MAX ? 2 : 1;

    /**
     This code point as its own string, for accumulating the run.
     */
    const character = String.fromCodePoint(codePoint,);

    /**
     Code point after this one, absent at the end of the leaf.
     */
    const nextPoint = (cursor.at + width) < text.length
      ? text.codePointAt(cursor.at + width,)
      : undefined;
    if (isDigit(codePoint,)) {
      if (run.kind !== 'number') {
        flush();
        run.start = cursor.at;
      }
      run.kind = 'number';
    }
    else if (isForeign(codePoint,)) {
      if (run.kind !== 'foreign') {
        flush();
        run.start = cursor.at;
      }
      run.kind = 'foreign';
    }
    else if (
      (run.kind === 'number')
        && NUMBER_SEPARATORS.includes(character,)
        && (nextPoint !== undefined)
        && isDigit(nextPoint,)
    ) {
      // A separator inside a number continues the run; nothing to record.
    }
    else {
      flush();
    }
    cursor.at += width;
  }
  flush();
  return atoms;
}

//endregion Protected atoms
