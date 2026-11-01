//region Photo reference
// WHICH IMAGES A PASSAGE SHOWS, so a stage can be handed the picture rather
// than only the markup that names it.
//
// The pipeline supplies the image to translators and judges, so a passage transcribed
// from one has a source that can be CHECKED rather than only preserved. The
// asset is already named in the markdown, so finding it is path work rather
// than a new corpus reader.
//
// ONE FORM, MEASURED. Across the pinned corpus the only image reference in any
// page is `<PhotoScroll photos={[ ... ]} />`, 173 occurrences, naming 380 assets
// of which 376 are `.webp` and 4 are `.jpg`. There are no markdown image
// references and no `<img>` elements, so this reader deliberately understands
// one construct rather than markdown's whole image vocabulary.
//
// TWO QUOTE MARKS, MEASURED LATER. At pin `a41fc607` the source pages write 199
// asset paths: 192 in single quotes across 47 entries and 7 in double quotes
// across 4 (`yulianNyanner`, `MTF_0615`, `Arita`, `BI4PBV`), and nothing in any
// third form. Until 2026-09-04 this reader took single-quoted strings only, and
// the first OpenRouter-alone BI4PBV pass (04:28 UTC) went through its picture
// stage in one millisecond: four pictures referenced, none gathered, none read,
// and no line saying so, because `assertVisualEvidenceComplete` asks this same
// reader what the slices show. An undercount here is the quiet kind.
//
// THE PLACEHOLDER IS THE ENTRY. Every reference is written `${path}/photos/…`,
// where `${path}` stands for the entry's own directory. One reference in the
// corpus writes `${path} /photos/…` with a stray space, so the prefix is matched
// with whitespace tolerated rather than as a fixed string: a reader that missed
// that one asset would report an entry as having fewer images than it shows.
//
// NO REGEX, per `RG1`: the rule is "inside a PhotoScroll element, take every
// quoted string, closed by the mark that opened it", which a scan states
// directly and in one pass.

/**
 Element that names images in this corpus.
 */
const ELEMENT_OPEN = '<PhotoScroll';

/**
 How an element's attributes end.
 */
const ELEMENT_CLOSE = '/>';

/**
 Quote marks the corpus writes asset paths in: most pages the first, four
 source pages the second.
 */
const QUOTE_MARKS = [
  '\'',
  '"',
] as const;

/**
 One of the marks a page may quote a path with.
 */
type QuoteMark = (typeof QUOTE_MARKS)[number];

/**
 Where the next quoted string opens, or that none opens before the limit.

 A NAMED OUTCOME rather than a nullish union, which this repository does not
 model absence with.

 @example
 ```ts
 const opening: QuoteOpening = { kind: 'opened', quote: '"', at: 12, };
 ```
 */
type QuoteOpening = {
  readonly kind: 'opened';

  /**
   Mark that opened it, which alone may close it.
   */
  readonly quote: QuoteMark;

  /**
   Offset of that mark.
   */
  readonly at: number;
} | {
  readonly kind: 'none';
};

/**
 Finds the nearest opening quote of either mark before a limit.

 EITHER MARK, NEAREST FIRST: a caption in double quotes beside paths in
 single quotes, or the reverse, must be read string by string in page order,
 or the caption's closing mark would be taken for a path's opening one.

 @param text - passage to read

 @param from - offset to search from

 @param limit - offset the element's attributes end at, exclusive

 @returns Nearest opening mark and which mark it is, or that none precedes
 the limit

 @example
 ```ts
 const opening = nextQuoteOpening({ text, from: 0, limit: text.length, },);
 ```
 */
function nextQuoteOpening(
  {
    text,
    from,
    limit,
  }: {
    readonly text: string;
    readonly from: number;
    readonly limit: number;
  },
): QuoteOpening {
  return QUOTE_MARKS.reduce(
    function nearer(
      best: QuoteOpening,
      quote: QuoteMark,
    ): QuoteOpening {
      /**
       Where this mark next occurs, if before the limit.
       */
      const at = text.indexOf(
        quote,
        from,
      );
      if ((at === (-1)) || (at >= limit))
        return best;
      if ((best.kind === 'opened') && (best.at <= at))
        return best;
      return {
        kind: 'opened',
        quote,
        at,
      };
    },
    { kind: 'none', },
  );
}

/**
 Directory every asset sits in, under the entry's own directory.
 */
const ASSET_DIRECTORY = '/photos/';

/**
 Placeholder standing for the entry's directory.
 */
const ENTRY_PLACEHOLDER = `\${path}`;

/**
 One image a passage shows.

 @example
 ```ts
 const shown: PhotoReference = { assetName: 'intro.webp', };
 ```
 */
export type PhotoReference = {
  /**
   File name within the entry's `photos` directory.
   */
  readonly assetName: string;
};

/**
 Reads every quoted string inside one element's attributes.

 @param text - passage to read

 @param from - offset of the element's opening

 @returns Quoted strings, and where the element ended

 @example
 ```ts
 const found = quotedWithin({ text, from, },);
 ```
 */
function quotedWithin(
  {
    text,
    from,
  }: {
    readonly text: string;
    readonly from: number;
  },
): {
  readonly quoted: readonly string[];
  readonly ended: number;
} {
  /**
   Where this element's attributes stop, or the end of the passage when the
   element is never closed.
   */
  const closeAt = text.indexOf(
    ELEMENT_CLOSE,
    from,
  );

  /**
   End of the region to read, exclusive.
   */
  const limit = (closeAt === (-1)) ? text.length : closeAt;

  /**
   Strings found so far.
   */
  const quoted: string[] = [];

  /**
   Cursor, walking quote to quote.
   */
  const at = { offset: from, };

  while (at.offset < limit) {
    /**
     Opening quote of the next string, of either mark.
     */
    const opening = nextQuoteOpening({
      text,
      from: at.offset,
      limit,
    },);
    if (opening.kind === 'none')
      break;

    /**
     Its closing quote: the same mark, so the other mark inside a path is
     part of the path.
     */
    const closed = text.indexOf(
      opening.quote,
      opening.at + 1,
    );
    if ((closed === (-1)) || (closed >= limit))
      break;

    quoted.push(text.slice(
      opening.at + 1,
      closed,
    ),);
    at.offset = closed + 1;
  }

  return {
    quoted,
    ended: (closeAt === (-1)) ? text.length : (closeAt + ELEMENT_CLOSE.length),
  };
}

/**
 What one quoted attribute string turned out to name.

 A NAMED OUTCOME rather than a nullish union, which this repository does not
 model absence with.

 @example
 ```ts
 const read: AssetNameRead = { kind: 'asset', assetName: 'intro.webp', };
 ```
 */
type AssetNameRead = {
  readonly kind: 'asset';

  /**
   File name within the entry's photos directory.
   */
  readonly assetName: string;
} | {
  readonly kind: 'not-an-asset';
};

/**
 Turns one quoted asset path into the file name it names.

 TOLERATES WHITESPACE AFTER THE PLACEHOLDER, because one reference in the
 corpus writes `${path} /photos/…`. Reading it as a different prefix would
 report that entry as showing one image fewer than it does.

 @param quoted - quoted string from a photo element

 @returns Asset file name, or a note that the string names something else

 @example
 ```ts
 const name = assetNameOf({ quoted: '${path}/photos/intro.webp', },);
 ```
 */
function assetNameOf({ quoted, }: { readonly quoted: string; },): AssetNameRead {
  if (!quoted.startsWith(ENTRY_PLACEHOLDER,))
    return { kind: 'not-an-asset', };

  /**
   Everything after the placeholder, whose leading whitespace is incidental.
   */
  const rest = quoted.slice(ENTRY_PLACEHOLDER.length,)
    .trimStart();
  if (!rest.startsWith(ASSET_DIRECTORY,))
    return { kind: 'not-an-asset', };

  /**
   File name, which must not itself be a path.
   */
  const assetName = rest.slice(ASSET_DIRECTORY.length,);
  if ((assetName === '') || assetName.includes('/',))
    return { kind: 'not-an-asset', };
  return {
    kind: 'asset',
    assetName,
  };
}

/**
 Every image one passage shows, in the order it shows them.

 @param text - passage to read

 @returns Images it names, empty when it names none

 @example
 ```ts
 const shown = photoReferences({ text: slice.target.text, },);
 ```
 */
export function photoReferences({ text, }: { readonly text: string; },): readonly PhotoReference[] {
  /**
   References found so far.
   */
  const found: PhotoReference[] = [];

  /**
   Cursor, walking element to element.
   */
  const at = { offset: 0, };

  while (at.offset < text.length) {
    /**
     Next photo element.
     */
    const opened = text.indexOf(
      ELEMENT_OPEN,
      at.offset,
    );
    if (opened === (-1))
      break;

    /**
     Its quoted attribute strings.
     */
    const within = quotedWithin({
      text,
      from: opened + ELEMENT_OPEN.length,
    },);

    for (const quoted of within.quoted) {
      /**
       What this string names.
       */
      const read = assetNameOf({ quoted, },);
      if (read.kind === 'asset')
        found.push({ assetName: read.assetName, },);
    }
    at.offset = Math.max(
      within.ended,
      opened + ELEMENT_OPEN.length,
    );
  }

  return found;
}

/**
 Where an entry's asset sits within the corpus.

 @param entryId - corpus entry

 @param assetName - file name within its photos directory

 @returns Repository-relative path

 @example
 ```ts
 const path = photoPath({ entryId: 'Tabby', assetName: 'intro.webp', },);
 ```
 */
export function photoPath(
  {
    entryId,
    assetName,
  }: {
    readonly entryId: string;
    readonly assetName: string;
  },
): string {
  return `people/${entryId}${ASSET_DIRECTORY}${assetName}`;
}

//endregion Photo reference
