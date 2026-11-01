import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

//region Page name link rendering
// WHICH ARCHIVE LINK RENDERS WHICH LINK OF THE ORIGINAL. Both documents carry
// a link under one href, and the archive's text for it is how the page renders
// the original's (`page-name-glossary.ts`).
//
// NOTHING MAKES AN HREF UNIQUE AMONG A PAGE'S LINKS. A page links one
// destination under a name in one sentence and under "here" in another, and a
// table keyed by href kept the last text the archive links it under, so every
// link of the original to that href read that one text as its rendering. Its
// own file because the glossary has no room left under the file-length limit.

/**
 One inline link as the pairing reads it: its text and where it points.
 */
type TextAtHref = {
  /**
   Link text.
   */
  readonly text: string;

  /**
   Destination.
   */
  readonly href: string;
};

/**
 One link of the original with the archive's text for it.

 @example
 ```ts
 const rendered: RenderedLink = { text: '咪咪', href: 'https://example.invalid/mimi', rendering: 'Mimi', };
 ```
 */
export type RenderedLink = TextAtHref & {
  /**
   Text the archive links the same href under, at this link's place.
   */
  readonly rendering: string;
};

/**
 Destination of a link, which both documents' links are gathered by.

 @param link - link read

 @returns Its href

 @example
 ```ts
 hrefOf({ text: 'Mimi', href: 'https://example.invalid/mimi', },); // 'https://example.invalid/mimi'
 ```
 */
function hrefOf(link: TextAtHref,): string {
  return link.href;
}

/**
 Pairs each link of the original with the archive's text for it, read off the
 archive's links to the same href.

 ONE TEXT, OR PLACE BY PLACE. Where every archive link to the href carries one
 text, that text renders every link of the original to it, however often
 either side links it. Where the archive links the href under more than one
 text, the original's links to it pair with the archive's in page order when
 both carry the same count, the rule the signatures and the headings pair
 under. A differing count leaves which text renders which link unreadable, so
 the link is left out rather than given whichever text the archive wrote last.

 @param sourceLinks - original's links, in page order

 @param archiveLinks - archive's links, in page order

 @returns Each link of the original whose rendering can be read, with it, in
 page order

 @example
 ```ts
 const rendered = renderedLinks({ sourceLinks: linksOf({ text: sourceText, },), archiveLinks: linksOf({ text: targetText, },), },);
 ```
 */
export function renderedLinks(
  {
    sourceLinks,
    archiveLinks,
  }: {
    readonly sourceLinks: readonly TextAtHref[];
    readonly archiveLinks: readonly TextAtHref[];
  },
): readonly RenderedLink[] {
  /**
   Archive's links by href, each list in page order.
   */
  const archiveByHref = Map.groupBy(
    archiveLinks,
    hrefOf,
  );
  /**
   Original's links by href, read for how often it links each.
   */
  const sourceByHref = Map.groupBy(
    sourceLinks,
    hrefOf,
  );
  /**
   Hrefs the archive links under more than one text, whose links pair by place.
   */
  const severalTexts = new Set<string>();
  for (const [href, partners,] of archiveByHref) {
    /**
     Distinct texts the archive links this href under.
     */
    const texts = new Set(partners.map(function textOf(partner,): string {
      return partner.text;
    },),);
    if (texts.size > 1)
      severalTexts.add(href,);
  }
  /**
   How many of the original's links to each href the walk has passed.
   */
  const passed = new Map<string, number>();
  return sourceLinks.flatMap(function toRendered(link,): readonly RenderedLink[] {
    /**
     Which of the original's links to this href this one is, from zero.
     */
    const place = passed.get(link.href,) ?? 0;
    passed.set(
      link.href,
      place + 1,
    );
    /**
     Archive's links to the same href, none when the archive lacks it.
     */
    const partners = archiveByHref.get(link.href,) ?? [];
    /**
     Whether the archive links the href under more than one text.
     */
    const byPlace = severalTexts.has(link.href,);
    /**
     Original's links to the href, which hold this link, since the grouping
     read the same list this walk goes through.
     */
    const sourcePartners = nonNullishOrThrow(sourceByHref.get(link.href,),);
    if (byPlace && (partners.length !== sourcePartners.length))
      return [];
    /**
     Archive's link at this link's place among the links to the href, or its
     first where it links the href under one text; absent only when the
     archive lacks the href.
     */
    const partner = partners[byPlace ? place : 0];
    if (partner === undefined)
      return [];
    return [{
      ...link,
      rendering: partner.text,
    },];
  },);
}

//endregion Page name link rendering
