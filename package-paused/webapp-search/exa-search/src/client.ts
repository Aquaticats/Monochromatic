import { prompt, } from '@monochromatic-dev/dom-core/ts/prompt.ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core';
import * as v from 'valibot';

import { displayResult, } from './client-display-result.ts';
import {
  apiKey,
  baseUrl,
  changeApiKeyButton,
  costDollarsSpan,
  numResults,
  numResultsInput,
  numTotalSearches,
  processingParagraph,
  resultArticles,
  resultsSection,
  searchForm,
  searchInput,
} from './client-dom.ts';
import { searchExa, } from './exa-fetch.ts';

searchForm.addEventListener(
  'submit',
  function onSearch(event,) {
    event.preventDefault();
    void (async function onSearchAsync(): Promise<void> {
      try {
        numTotalSearches.setValue(numTotalSearches.getValue()
          + 1,);
        resultsSection.setAttribute(
          'hidden',
          'true',
        );
        resultsSection.querySelectorAll<HTMLElement>(':scope > *',)
          .forEach(
          function hide(result,) {
            result.setAttribute(
              'hidden',
              'true',
            );
          },
        );

        processingParagraph.removeAttribute('hidden',);

        /**
         * Exa search response feeding the cost display and per-result population.
         */
        const results = await searchExa({
          apiKey: apiKey.getValue(),
          baseUrl,
          query: searchInput.value
            .trim(),
          options: {
            type: 'auto',
            numResults: numResults.getValue(),
            contents: {
              text: true,
              summary: true,
              subpages: 1,
              extras: {
                links: 1,
                imageLinks: 1,
              },
              highlights: true,
            },
          },
        },);

        processingParagraph.setAttribute(
          'hidden',
          'true',
        );

        costDollarsSpan.textContent = String(results.costDollars
          ?.total
          ?? 0,);

        results.results
          .forEach(function forEachResult(
          result,
          resultIndex,
        ) {
          displayResult({
            resultArticles,
            result,
            resultIndex,
          },);
        },);

        resultsSection.removeAttribute('hidden',);
      }
      catch (error: unknown) {
        console.error(
          'search failed',
          error,
        );
      }
    })();
  },
);

/**
 * Validator for the Change API Key prompt result.
 *
 * The prompt now resolves `''` on OK-with-empty (aligned with `window.prompt`),
 * which would slip past `nonNullishOrThrow` and overwrite the stored key with
 * an empty string, breaking the next page load when the module-init valibot
 * pipeline (`client-dom.ts`) parses localStorage and requires a uuid. Running
 * the same `v.string()` + `v.uuid()` check here rejects empty and malformed
 * input before the observable setter writes to localStorage.
 */
const apiKeySchema = v.pipe(
  v.string(),
  v.uuid(),
);

changeApiKeyButton.addEventListener(
  'click',
  function onChangeApiKey() {
    void (async function promptForNewApiKey(): Promise<void> {
      try {
        /**
         * Raw prompt response validated against `apiKeySchema` before observable assignment.
         */
        const inputApiKey = nonNullishOrThrow(
          await prompt({ message: 'Change api key', },),
        );
        apiKey.setValue(v.parse(
          apiKeySchema,
          inputApiKey,
        ),);
      }
      catch (error: unknown) {
        console.error(
          'api key change failed',
          error,
        );
      }
    })();
  },
);

numResultsInput.addEventListener(
  'input',
  function setNewNumResults() {
    numResults.setValue(Number(numResultsInput.value,),);
  },
);

export {};
