// oxlint-disable typescript/strict-boolean-expressions -- DOM string coercions and template-cloning indices
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core';

import { replicateElementAsContentOf, } from './client-replicate-element.ts';
import type { ExaSearchResult, } from './exa-fetch.ts';

/**
 * Populates a single result article element with data from an Exa search result.
 *
 * @param resultArticles - Live HTMLCollection of result container elements
 *
 * @param result - Exa search result with text, summary, and highlights content
 *
 * @param resultIndex - Zero-based index of this result in the search results array
 *
 * @example
 * ```ts
 * displayResult({ resultArticles, result, resultIndex: 0 });
 * ```
 */
export function displayResult({
  resultArticles,
  result,
  resultIndex,
}: {
  resultArticles: HTMLCollection;
  result: ExaSearchResult;
  resultIndex: number;
},): void {
  /**
   * Result article slot populated from `result` for this `resultIndex`.
   */
  const currentResultArticle = nonNullishOrThrow(
    resultArticles[resultIndex],
  );

  /**
   * Favicon image populated from `result.favicon` when present.
   */
  const favicon = nonNullishOrThrow(
    currentResultArticle.querySelector<HTMLImageElement>('.result__favicon',),
  );
  /**
   * Anchor populated from `result.url` and `result.title`.
   */
  const link = nonNullishOrThrow(
    currentResultArticle.querySelector<HTMLAnchorElement>('.result__link',),
  );
  /**
   * Time element populated from `result.publishedDate` when present.
   */
  const publishedDate = nonNullishOrThrow(
    currentResultArticle.querySelector<HTMLTimeElement>('.result__publishedDate',),
  );
  /**
   * Author element populated from `result.author` when present.
   */
  const author = nonNullishOrThrow(
    currentResultArticle.querySelector<HTMLElement>('.result__author',),
  );
  /**
   * Summary paragraph populated from `result.summary`.
   */
  const summary = nonNullishOrThrow(
    currentResultArticle.querySelector<HTMLParagraphElement>('.result__summary',),
  );
  /**
   * Text paragraph populated from `result.text`.
   */
  const text = nonNullishOrThrow(
    currentResultArticle.querySelector<HTMLParagraphElement>('.result__text',),
  );
  /**
   * Highlight list whose items are cloned and filled from `result.highlights`.
   */
  const highlights = nonNullishOrThrow(
    currentResultArticle.querySelector<HTMLUListElement>('.result__highlights',),
  );
  /**
   * Highlight item template cloned `result.highlights.length` times.
   */
  const firstHighlight = nonNullishOrThrow(
    currentResultArticle.querySelector<HTMLLIElement>('.result__highlight',),
  );
  /**
   * Image element populated from `result.image` when present.
   */
  const image = nonNullishOrThrow(
    currentResultArticle.querySelector<HTMLImageElement>('.result__image',),
  );

  if (result.favicon)
    favicon.src = result.favicon;

  link.href = result.url;
  link.textContent = result.title;

  if (result.publishedDate) {
    publishedDate.dateTime = result.publishedDate;
    publishedDate.textContent = result.publishedDate;
  }

  if (result.author)
    author.textContent = result.author;

  summary.textContent = result.summary;
  text.textContent = result.text;

  replicateElementAsContentOf({
    templateElement: firstHighlight,
    parentElement: highlights,
    targetCount: result.highlights
      .length,
  },);

  result.highlights
    .forEach(
    function populateHighlight(
      highlight: string,
      highlightIndex: number,
    ) {
      /**
       * Highlight slot at `highlightIndex` populated with the matching string.
       */
      const currentHighlight = nonNullishOrThrow(
        highlights.querySelector<HTMLElement>(`:nth-child(${highlightIndex + 1})`,),
      );
      currentHighlight.textContent = highlight;
    },
  );

  if (result.image)
    image.src = result.image;

  currentResultArticle.removeAttribute('hidden',);
}
