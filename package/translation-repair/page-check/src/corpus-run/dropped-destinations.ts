import type {
  Root,
  RootContent,
} from 'mdast';
import {
  type DeepReadonlyData,
  isAutolinkLiteral,
  maskHtmlComments,
  maskInvisibleLines,
  splitFrontMatter,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  isAsciiLetter,
  isTagWhitespace,
} from '@monochromatic-dev/translation-repair-markup/ts';
import { parseBodyTolerant, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { nextSchemeStart, } from '@monochromatic-dev/translation-repair-reference/ts';
import {
  judgeDestinationRenderings,
  sameAddress,
} from './destination-renderings.ts';

//region Dropped destinations
// WHAT THE SOURCE LINKS TO THAT THE PAGE NO LONGER DOES.
//
// The naturalness lane protects link destinations as ordered atoms, so a
// rewrite cannot drop one. Nothing protected them across the other two ways a
// slice ships: the repair lane keeping an archive sentence that never carried
// the link, and the contest preferring the incumbent. The 2026-08-26 reading
// (`doc/audit/translation-repair-output-reading-20260826.md`) found a page
// that had lost a source hyperlink exactly that way, with nothing having
// noticed. This is the document-level check that notices.
//
// TWO READERS, UNIONED. Markdown destinations come off the tree the pipeline
// itself parses (front matter split, invisible lines and HTML comments masked,
// strict MDX with the plain-markdown downgrade), which also covers reference
// definitions; bare runs come off a linear scan for the two web schemes, which
// also covers front matter and HTML attributes, over the text with its HTML
// comments masked (class forty-six, shi_Yumiaoya1, 2026-09-17: the original
// carried a profile link inside a comment, every stage masks comments, and the
// page was refused for dropping a destination no reader could follow). A
// destination is a string; two
// spellings of one address that differ only by a trailing slash are treated as
// the same address, because that difference changes nothing a reader can
// follow.
//
// AN EXPLICIT DESTINATION IS FOLLOWED AS WRITTEN. Only an address prose ran
// into, a bare run the scanner reads or an autolink literal the tree builds,
// is cut at its first stopper and where the parse's trail rule begins its
// trail (sentence punctuation, a character reference, a closing bracket); a destination
// written between parentheses, angle brackets or after a definition's label
// ends where its author ended it. Cut too, two explicit links that differ only
// past a full-width comma read as one, and a page keeping either was taken
// for keeping both. The scanner still reads such a destination's address off
// the raw text and stops at that comma, so the union can carry the explicit
// destination beside the scanner's shorter run of it, on every side alike.
//
// THE EMPTY STRING IS NO DESTINATION. A link written with nothing between its
// parentheses names nowhere a reader could follow, so neither side carries
// one for it and no page owes it; recorded, it compared equal to every other
// destination the trimming emptied, and `traceDroppedDestinations` found it in
// every slice, the shipped text included, since every text holds the empty
// string. A destination that carries something never reads as the empty
// string: an explicit one stands as written, and the cut of an address prose
// ran into never empties it (`literalAddress`).
//
// THE SITE'S OWN GRAMMAR IS NOT THIS ONE. The corpus repo compiles a page with
// MDX 3 and remark-math after rewriting HTML comments into JSX comments
// (`scripts/build.ts`, `scripts/mdx.ts` there); reconciling the
// two is open. For destinations the difference does not matter: a link is a link under
// both, and the bare-run scan catches what either tree would not.
//
// THE ARCHIVE'S RENDERING COUNTS. Where the archive rendered a reference
// another way than the original, the page owes one rendering from either
// side (`destination-renderings.ts`, the owner's decision of 2026-09-04);
// a reader that compared the page to the source alone refused the luxuanwen3
// page every slice had passed. Without an archive the source alone governs.
//
// A DROPPED DESTINATION IS STRUCTURED EVIDENCE. Production checks this result
// before writing and pauses as an invariant if stage-local translation failed
// to restore a source destination. Reporting tools may still read all fields.

/**
 Parsed page, read-only.
 */
type ReadonlyMdastRoot = DeepReadonlyData<Root>;

/**
 Any node of a parsed page, read-only.
 */
type ReadonlyMdastContent = DeepReadonlyData<RootContent>;

/**
 Characters other than whitespace that end a bare run: Markdown and HTML
 delimiters, and the full-width punctuation Chinese prose sets a link off
 with. Whitespace ends one too, every character the parse reads as whitespace
 (`endsRun`). A closing parenthesis ends a run only where it balances no
 opening one of the run (`addressEnd`). A closing square bracket is no
 stopper: the parse keeps one inside an address and ends the address at it
 only as its trail rule reads one (`readTrail`).
 */
const RUN_STOPPERS: ReadonlySet<string> = new Set([
  ')',
  '>',
  '<',
  '"',
  '\'',
  '`',
  '）',
  '（',
  '】',
  '【',
  '，',
  '。',
  '、',
  '《',
  '》',
  '「',
  '」',
  '；',
  '：',
],);

/**
 Trailing characters a run sheds, since sentence punctuation and the marks
 emphasis closes with follow a link more often than they belong to one: the
 marks the parse's trail rule reads as no part of an address
 (`micromark-extension-gfm-autolink-literal` 2.1.0, `tokenizeTrail`), less
 the quotation marks, which end a run as stoppers. The rule's character
 reference and bracket forms are read by `readTrail`.
 */
const RUN_TRAILERS: ReadonlySet<string> = new Set([
  '.',
  ',',
  ';',
  ':',
  '!',
  '?',
  '*',
  '_',
  '~',
],);

/**
 What after a closing square bracket ends the parse's trail there, as the
 text's end and whitespace do: an opening parenthesis or bracket.
 */
const BRACKET_ENDERS: ReadonlySet<string> = new Set([
  '(',
  '[',
],);

/**
 Whether a character ends a bare run: whitespace as the parse reads it, or
 one of `RUN_STOPPERS`.

 WHITESPACE IS THE PARSE'S. The autolink literal's path and trail
 (`micromark-extension-gfm-autolink-literal` 2.1.0, `tokenizePath` and
 `tokenizeTrail`) end at `markdownLineEndingOrSpace` or `unicodeWhitespace`,
 the ECMAScript `\s` over one UTF-16 unit, which is the whitespace the strict
 grammar steps over inside a tag, so one predicate answers both
 (`isTagWhitespace`). The scan once ended a run at the four ASCII ones alone,
 so an address before a no-break or an ideographic space ran on into the
 words after it, where the parse ends it.

 @param character - one UTF-16 unit, empty past the text's end

 @returns True where a run stops

 @example
 ```ts
 endsRun({ character: '<', },); // true
 ```
 */
function endsRun({ character, }: { readonly character: string; },): boolean {
  return isTagWhitespace({ character, },) || RUN_STOPPERS.has(character,);
}

/**
 Characters a trail may begin at, which the scan reads the trail of before it
 decides the address goes on.
 */
const TRAIL_STARTS: ReadonlySet<string> = new Set([
  ...RUN_TRAILERS,
  '&',
  ']',
],);

/**
 What the check found on both sides.

 @example
 ```ts
 const check: DestinationCheck = droppedDestinations({ sourceText, pageText, },);
 ```
 */
export type DestinationCheck = {
  /**
   Distinct destinations the source carries, in first-seen order.
   */
  readonly source: readonly string[];

  /**
   Distinct destinations the page carries, in first-seen order.
   */
  readonly page: readonly string[];

  /**
   Source destinations the page does not carry.
   */
  readonly dropped: readonly string[];

  /**
   Telemetry in scorecard-stable wording, empty unless the strict grammar
   downgraded a side to plain markdown.
   */
  readonly findings: readonly string[];
};

/**
 Bare web addresses in the text.

 ONE LINEAR PASS. `nextSchemeStart` searches forward from the cursor for where
 a scheme starts, the run is read from there to its first stopper, and the
 cursor resumes at that stopper: nothing behind the cursor is searched again,
 so the pass costs time in proportion to the text. This summary said so while
 the scan searched for each scheme apart and read to the text's end, at every
 run, for a scheme the text lacks (`scheme-start-scan.ts` holds the
 measurement).

 COVERS WHAT THE TREE CANNOT: front matter and HTML attributes. A Markdown
 destination shows up here too, because its
 address starts with a scheme like any other; the union dedupes it.

 @param text - text scanned

 @returns Runs in the order found, repeats kept

 @throws Error when a run consumed nothing, which no text produces: a run
 opens where `nextSchemeStart` found a scheme, and no scheme opens on a
 stopper

 @example
 ```ts
 const runs = scanUrlRuns({ text: 'see https://example.org/a, then https://example.org/b', },);
 ```
 */
export function scanUrlRuns({ text, }: { readonly text: string; },): readonly string[] {
  /**
   Runs found so far.
   */
  const runs: string[] = [];

  /**
   Cursor, advanced past every run read.
   */
  let at = 0;
  while (at < text.length) {
    /**
     Where the next run starts, the text's length when no scheme remains.
     */
    const start = nextSchemeStart({
      text,
      from: at,
    },);
    if (start === text.length)
      break;

    /**
     End of the run, exclusive: the first stopper after the scheme, a closing
     parenthesis that balances one the run opened not being one.
     */
    const end = addressEnd({
      text,
      from: start,
    },);

    runs.push(text.slice(
      start,
      end,
    ),);
    // The run always consumes at least its scheme, so the scan advances to
    // its end. LOUD IF IT EVER DOES NOT: a scheme opening on a stopper would
    // leave the cursor where it stood and this loop running for ever (ledger
    // M113).
    if (end === start)
      throw new Error(
        'unreachable: a web address run that consumed nothing, though nextSchemeStart answers only where a '
          + 'scheme starts and no scheme opens on a character that ends a run',
      );
    at = end;
  }
  return runs;
}

/**
 What the parse's trail rule makes of the characters from an offset on.

 @example
 ```ts
 const trail: TrailReading = { ends: true, stop: 5, };
 ```
 */
type TrailReading = {
  /**
   Whether the characters are the trail of the address: only what the rule
   reads as trailing, then an end. The address stops where they begin.
   */
  readonly ends: boolean;

  /**
   Offset the rule read up to: where it met an end, or the first character
   that made the characters part of the address.
   */
  readonly stop: number;
};

/**
 Reads the trail of an address from an offset, as the parse reads it
 (`micromark-extension-gfm-autolink-literal` 2.1.0, `tokenizeTrail`).

 THE TRAIL IS A RUN OF THREE KINDS OF ITEM: a mark of sentence punctuation or
 emphasis or a closing parenthesis; a character reference, `&` and ASCII
 letters and `;` (one that is not well formed is no item and is part of the
 address, so `&amp` before a space and `&#35;` stay in it); and `]`, which
 ends the trail where whitespace, `(`, `[` or the text's end follows it and
 is otherwise one more item. The run is the trail only when an end follows
 it: whitespace, `<` or any other stopper of the scanner, or the text's end.
 Anything else makes every character of it part of the address.

 @param text - text holding the address

 @param from - offset of the first character that may start the trail

 @returns Whether the characters from `from` are the trail, and where the
 reading stopped

 @example
 ```ts
 const trail = readTrail({ text: 'a&amp;).b', from: 1, },); // { ends: false, stop: 8 }
 ```
 */
function readTrail(
  {
    text,
    from,
  }: {
    readonly text: string;
    readonly from: number;
  },
): TrailReading {
  for (let at = from; at < text.length;) {
    /**
     Character under the reading.
     */
    const character = text.charAt(at,);
    if (RUN_TRAILERS.has(character,) || (character === ')')) {
      at += 1;
      continue;
    }
    if (character === '&') {
      /**
       Offset past the reference's letters.
       */
      let letters = at + 1;
      while (isAsciiLetter({ character: text.charAt(letters,), },))
        letters += 1;
      if ((letters === (at + 1)) || (text.charAt(letters,) !== ';')) {
        return {
          ends: false,
          stop: letters,
        };
      }
      at = letters + 1;
      continue;
    }
    if (character === ']') {
      /**
       Offset of the character after the bracket, which decides whether the
       bracket ends the trail or is one more item of it.
       */
      const after = at + 1;
      if ((after >= text.length)
        || BRACKET_ENDERS.has(text.charAt(after,),)
        || isTagWhitespace({ character: text.charAt(after,), },)) {
        return {
          ends: true,
          stop: after,
        };
      }
      at = after;
      continue;
    }
    return {
      ends: endsRun({ character, },),
      stop: at,
    };
  }
  return {
    ends: true,
    stop: text.length,
  };
}

/**
 Where an address ends in a text: its first stopper from an offset, or the
 text's end.

 A CLOSING PARENTHESIS THAT BALANCES AN OPENING ONE OF THE SAME ADDRESS IS
 NOT A STOPPER, as the parse reads it (`micromark-extension-gfm-autolink-literal`
 2.1.0, `tokenizePath`: a `)` is part of the path while fewer have closed than
 opened). `Tabby_(cat)` is one address, so a page writing it bare and a source
 linking it explicitly name one destination; a `)` that balances none, as the
 one closing a parenthesis set around the address, still ends it.

 THE TRAIL ENDS THE ADDRESS WHERE IT BEGINS, as the parse's trail rule reads it
 (`readTrail`): `(see Tabby_(cat).)` holds `Tabby_(cat)` and `(cat.)` holds
 `(cat`, and a character reference or a closing square bracket ends an
 address as it ends the parse's. A run that turned out to be no trail is not
 looked at again: one pass over the characters.

 @param text - text holding the address

 @param from - offset of the address's first character

 @returns Offset of the first stopper at or after `from`, exclusive end of the address

 @example
 ```ts
 const end = addressEnd({ text: 'https://example.org/a\uff0c', from: 0, },);
 ```
 */
function addressEnd(
  {
    text,
    from,
  }: {
    readonly text: string;
    readonly from: number;
  },
): number {
  for (let at = from, noTrailBefore = from, open = 0; at < text.length;) {
    /**
     Character under the scan.
     */
    const character = text.charAt(at,);
    if (character === '(') {
      open += 1;
      at += 1;
      continue;
    }
    if ((character === ')') && (open > 0)) {
      open -= 1;
      at += 1;
      continue;
    }
    if (TRAIL_STARTS.has(character,) && (at >= noTrailBefore)) {
      /**
       What the parse's trail rule makes of the characters from here.
       */
      const trail = readTrail({
        text,
        from: at,
      },);
      if (trail.ends)
        return at;
      noTrailBefore = trail.stop;
    }
    if (endsRun({ character, },))
      return at;
    at += 1;
  }
  return text.length;
}

/**
 Address as a reader would follow it where prose ran into it: cut at the first
 stopper and where the trail begins.

 A GFM autolink literal runs until whitespace, so in Chinese prose it swallows
 the full-width comma or stop after the address; the scanner never does, and
 the two readers must agree on the address or the union counts one link twice.

 ONLY AN AUTOLINK LITERAL COMES HERE, whose destination the parse writes as the
 address it read when that opens with its scheme, and with `http://` or
 `mailto:` put before a `www.` address or an email address
 (`mdast-util-gfm-autolink-literal`). It opens on a letter, which the cut
 never removes, so it is never emptied. An explicit destination never comes
 here: emptied, `.`, `..` and a destination opening on a stopper read as one
 empty string (ledger B139), and cut, two that differ past a stopper read as
 one.

 @param url - address as the autolink literal produced it

 @returns Address ending where the scanner's address ends in the same text

 @throws Error when the cut leaves nothing, which no address opening on a
 scheme's letter allows

 @example
 ```ts
 const clean = literalAddress({ url: 'https://example.org/a\uff0c', },);
 ```
 */
function literalAddress({ url, }: { readonly url: string; },): string {
  /**
   Where the address ends in the literal.
   */
  const end = addressEnd({
    text: url,
    from: 0,
  },);
  if (end === 0) {
    throw new Error(
      'unreachable: the cut at the first stopper and the trail left nothing of an autolink literal, '
        + 'though one opens on the letter of a scheme',
    );
  }
  return url.slice(
    0,
    end,
  );
}

/**
 Link, image and definition destinations off the tree the pipeline parses,
 each as written except an autolink literal, which is cut where prose ran
 into it, and an empty destination left out.

 @param text - page or source text, front matter included

 @returns Destinations in document order, none of them empty, and the
 downgrade finding when the strict grammar refused the body

 @example
 ```ts
 const { urls, findings, } = markdownDestinations({ text, },);
 ```
 */
export function markdownDestinations(
  { text, }: { readonly text: string; },
): {
  readonly urls: readonly string[];
  readonly findings: readonly string[];
} {
  /**
   Front matter and body apart, as `parseDocument` splits them.
   */
  const split = splitFrontMatter({ text, },);

  /**
   Body with the lines that show nothing masked, as `parseDocument` does.
   */
  const { masked: unwelded, } = maskInvisibleLines({ text: split.body, },);

  /**
   Body with HTML comments masked to whitespace, as `parseDocument` does.
   */
  const { masked, } = maskHtmlComments({ text: unwelded, },);

  /**
   Tree and any downgrade finding, under the pipeline's own grammar.
   */
  const parsed = parseBodyTolerant({
    body: masked,
    bodyOffset: split.bodyOffset,
  },);

  /**
   Parsed body, read-only.
   */
  const root: ReadonlyMdastRoot = parsed.root;

  /**
   Destinations in document order.
   */
  const urls: string[] = [];

  /**
   Nodes still to visit, top of the stack first, so the walk is document order.
   */
  const pending: ReadonlyMdastContent[] = [...root.children,].toReversed();
  // Node under visit, until the stack is empty.
  for (let node = pending.pop(); node !== undefined; node = pending.pop()) {
    if ((node.type === 'link')
      || (node.type === 'image')
      || (node.type === 'definition')) {
      /**
       Where this node leads: an autolink literal, an address prose ran into
       (`isAutolinkLiteral`, the test the footnote relabel makes), cut where
       the prose resumed, anything else as written, empty only where it was
       written with no destination.
       */
      const destination = ((node.type === 'link') && isAutolinkLiteral(node,))
        ? literalAddress({ url: node.url, },)
        : node.url;
      // AN EMPTY DESTINATION IS NOT READ: it names nowhere a reader could
      // follow, so no page owes it.
      if (destination !== '')
        urls.push(destination,);
    }
    if ('children' in node) {
      /**
       Children in document order, pushed reversed so the first is visited first.
       */
      const children = [...node.children,];
      pending.push(...children.toReversed(),);
    }
  }
  return {
    urls,
    findings: parsed
      .findings
      .map(function named(finding,): string {
        return `destinations-${finding.kind}`;
      },),
  };
}

/**
 Every destination a text carries, from both readers, deduped in first-seen
 order.

 @param text - page or source text

 @param side - which side, for the finding when the strict grammar downgraded

 @returns Destinations and any finding

 @example
 ```ts
 const { urls, findings, } = collectDestinations({ text, side: 'source', },);
 ```
 */
export function collectDestinations(
  {
    text,
    side,
  }: {
    readonly text: string;
    readonly side: 'source' | 'page' | 'archive';
  },
): {
  readonly urls: readonly string[];
  readonly findings: readonly string[];
} {
  /**
   Tree destinations and any downgrade finding.
   */
  const parsed = markdownDestinations({ text, },);

  /**
   Text with its HTML comments masked to whitespace, since a link inside a
   comment is rendered nowhere and owed by no page.
   */
  const { masked: uncommented, } = maskHtmlComments({ text, },);

  /**
   Both readers' output, tree first so a definition precedes its bare run.
   */
  const combined = [
    ...parsed.urls,
    ...scanUrlRuns({ text: uncommented, },),
  ];

  /**
   Addresses already kept, compared with the trailing slash shed.
   */
  const seen = new Set<string>();

  /**
   Distinct destinations in first-seen order.
   */
  const urls = combined.filter(function firstSeen(url,): boolean {
    /**
     Address compared, trailing slash shed.
     */
    const key = sameAddress({ url, },);
    if (seen.has(key,))
      return false;
    seen.add(key,);
    return true;
  },);

  return {
    urls,
    findings: parsed
      .findings
      .map(function sided(finding,): string {
        return `${finding} (${side})`;
      },),
  };
}

/**
 Source destinations the published page does not carry, an archive's
 rendering of one accepted in its place.

 @param sourceText - whole source page

 @param pageText - whole published page

 @param archiveText - whole archive page before the run, whose renderings the
 page may keep; absent when the page is judged against the source alone

 @returns Both sides' destinations, the dropped ones, and any finding

 @example
 ```ts
 const check = droppedDestinations({ sourceText, pageText, archiveText, },);
 if (check.dropped.length > 0) l.warn(`dropped destinations: ${check.dropped.join(', ',)}`,);
 ```
 */
export function droppedDestinations(
  {
    sourceText,
    pageText,
    archiveText,
  }: {
    readonly sourceText: string;
    readonly pageText: string;
    readonly archiveText?: string;
  },
): DestinationCheck {
  /**
   What the source carries.
   */
  const source = collectDestinations({
    text: sourceText,
    side: 'source',
  },);

  /**
   What the page carries.
   */
  const page = collectDestinations({
    text: pageText,
    side: 'page',
  },);

  /**
   What the archive carried before the run, nothing when there is none.
   */
  const archive = (archiveText === undefined)
    ? {
      urls: [],
      findings: [],
    }
    : collectDestinations({
      text: archiveText,
      side: 'archive',
    },);

  /**
   What the page owes and lacks, the archive's renderings counted.
   */
  const verdict = judgeDestinationRenderings({
    source: source.urls,
    page: page.urls,
    archive: archive.urls,
  },);

  return {
    source: source.urls,
    page: page.urls,
    dropped: verdict.dropped,
    findings: [
      ...source.findings,
      ...page.findings,
      ...archive.findings,
      ...verdict.findings,
    ],
  };
}

//endregion Dropped destinations
