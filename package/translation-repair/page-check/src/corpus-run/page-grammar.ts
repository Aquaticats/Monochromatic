import {
  FrontMatterParseError,
  MarkdownParseError,
  refusalText,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  type ParseFinding,
  parseDocument,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Page grammar
// A PAGE THE GRAMMAR REFUSES cannot ship, whatever every slice floor said of
// its slices. Every slice is read under the strict MDX grammar before it is
// admitted, but the would-ship reading runs AFTER those floors, so a rewrite
// applied there reaches the page unread. On 2026-09-06 the yulianNyanner page
// carried a JSX string literal in curly quotes, `{[“…”]}`, because the
// typography restoration at the would-ship reading treated the quotes of a
// component line as prose; the page compiled nowhere, and nothing refused it.
//
// The floor reads the assembled page the way every document is read,
// `parseDocument`, whose strict parse falls back to plain markdown and records
// the fall as an `mdx-downgraded` finding. One such finding on a would-ship
// page is a page the site cannot build.

/**
 Refusal when a would-ship page does not parse under the MDX grammar.

 @example
 ```ts
 throw new UnparseablePageError({ entryId: 'Cat', refusal: 'at 3:12 (mdx-jsx)', },);
 ```
 */
export class UnparseablePageError extends Error {
  /**
   Message contains the entry id and the strict parser's refusal site, which
   `MdxParseError` builds from positions and rule names alone.
   */
  readonly messageNamesOnly: true = true;

  /**
   Entry whose page failed the invariant.
   */
  readonly entryId: string;

  /**
   Where the grammar stopped, as the strict parser reported it.
   */
  readonly refusal: string;

  /**
   @param entryId - affected entry

   @param refusal - strict parser's refusal site, positions and rule names only
   */
  public constructor(
    {
      entryId,
      refusal,
    }: {
      readonly entryId: string;
      readonly refusal: string;
    },
  ) {
    super(
      `entry ${entryId} would ship a page the MDX grammar refuses to parse; ${refusal}`,
    );
    this.name = 'UnparseablePageError';
    this.entryId = entryId;
    this.refusal = refusal;
  }
}

/**
 Falls from the strict grammar a text took, which `parseDocument` records rather than raises.

 @param pageText - page or fragment, front matter included when it has one

 @returns Downgrade findings in source order, none for a text that parsed strictly

 @throws Whatever `parseDocument` raises, which is not a downgrade

 @example
 ```ts
 const [downgrade,] = downgradesOf({ pageText, },);
 ```
 */
function downgradesOf({ pageText, }: { readonly pageText: string; },): readonly ParseFinding[] {
  return parseDocument({ text: pageText, },)
    .parseFindings
    .filter(function isDowngrade(finding,): boolean {
      return finding.kind === 'mdx-downgraded';
    },);
}

/**
 What the strict MDX grammar says of a page or of a piece of one.

 @example
 ```ts
 const reading: PageGrammarReading = { kind: 'refused', refusal: 'at 3:12 (mdx-jsx)', };
 ```
 */
export type PageGrammarReading =
  | {
    /**
     The text parses strictly.
     */
    readonly kind: 'parses';
  }
    | {
    /**
     The text does not parse strictly, so the site could not build a page that holds it.
     */
      readonly kind: 'refused';

    /**
     The refusal site and rule as the parser reported them, positions and rule names only.
     */
      readonly refusal: string;
    };

/**
 Reads a page, or a piece of one, under the strict MDX grammar.

 READS THREE KINDS OF REFUSAL AS ONE ANSWER. A strict parse that fell back to plain markdown is the usual one. A text
 nested past the bound the plain markdown fallback keeps, or a front matter fence whose YAML does not parse, raises its
 own error out of `parseDocument` and not a downgrade finding; each of those is a text the site cannot build as well,
 so each answers "refused" here and the caller chooses what to do about it. Any other error is a fault in the code and
 propagates.

 @param pageText - page or fragment, front matter included when it has one

 @returns Whether the text parses strictly and, where it does not, the grammar's refusal

 @throws Whatever `parseDocument` raises other than the refusals it answers

 @example
 ```ts
 const reading = readPageGrammar({ pageText: '她说 { 然后走了。\n', },);
 ```
 */
export function readPageGrammar({ pageText, }: { readonly pageText: string; },): PageGrammarReading {
  try {
    /**
     First fall from the strict grammar, if the text took one.
     */
    const [downgrade,] = downgradesOf({ pageText, },);
    if (downgrade === undefined)
      return { kind: 'parses', };
    return {
      kind: 'refused',
      refusal: downgrade.detail,
    };
  }
  catch (error) {
    if ((error instanceof MarkdownParseError) || (error instanceof FrontMatterParseError)) {
      return {
        kind: 'refused',
        refusal: refusalText({ error, },),
      };
    }
    throw error;
  }
}

/**
 Refuses a would-ship page the strict MDX grammar cannot parse.

 @param entryId - entry about to publish

 @param pageText - whole would-ship page, front matter included

 @throws {@link UnparseablePageError} when the strict parse of the page fell
 back to plain markdown

 @example
 ```ts
 assertPageParses({ entryId: 'Cat', pageText, },);
 ```
 */
export function assertPageParses(
  {
    entryId,
    pageText,
  }: {
    readonly entryId: string;
    readonly pageText: string;
  },
): void {
  /**
   First fall from the strict grammar, if the page took one.
   */
  const [downgrade,] = downgradesOf({ pageText, },);
  if (downgrade === undefined)
    return;

  throw new UnparseablePageError({
    entryId,
    refusal: downgrade.detail,
  },);
}

//endregion Page grammar
