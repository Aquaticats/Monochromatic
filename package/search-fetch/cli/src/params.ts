/**
 Assembled CLI parameters validated against the shared tool schemas.

 The CLI parses a strict flag grammar, then feeds the assembled params through
 the tolerant shared Valibot specs, so one schema owns the accepted shape and a
 value that cannot fit it becomes a usage error here.

 @module
 */

import * as v from 'valibot';
import {
  LinkupWebFetchParametersSchema,
  LinkupWebSearchParametersSchema,
} from '@monochromatic-dev/search-fetch-core/ts/tool-spec.ts';

import { SearchFetchCliUsageError, } from './errors.ts';

//region Public API

/**
 Validate assembled search params against the shared web_search schema.

 @param rawParams - assembled raw params exactly as the CLI parsed them

 @throws SearchFetchCliUsageError when the assembled params fail the shared schema

 @example
 ```ts
 validateSearchParams({ rawParams: { query: 'docs' } });
 ```
 */
export function validateSearchParams(
  { rawParams, }: {
    readonly rawParams: unknown;
  },
): void {
  /**
   Shared-schema parse of the assembled params.
   */
  const parsed = v.safeParse(
    LinkupWebSearchParametersSchema,
    rawParams,
  );
  if (!parsed.success)
    throw new SearchFetchCliUsageError({
      message: `Search parameters do not fit the shared web_search schema: ${schemaIssueText(parsed.issues,)}`,
    },);
}

/**
 Validate assembled fetch params against the shared web_fetch schema.

 @param rawParams - assembled raw params exactly as the CLI parsed them

 @throws SearchFetchCliUsageError when the assembled params fail the shared schema

 @example
 ```ts
 validateFetchParams({ rawParams: { url: 'https://example.com' } });
 ```
 */
export function validateFetchParams(
  { rawParams, }: {
    readonly rawParams: unknown;
  },
): void {
  /**
   Shared-schema parse of the assembled params.
   */
  const parsed = v.safeParse(
    LinkupWebFetchParametersSchema,
    rawParams,
  );
  if (!parsed.success)
    throw new SearchFetchCliUsageError({
      message: `Fetch parameters do not fit the shared web_fetch schema: ${schemaIssueText(parsed.issues,)}`,
    },);
}

//endregion Public API

//region Helpers

/**
 Render shared-schema issues into one diagnostic naming the refused values.

 @param issues - schema issues reported by the shared parse

 @returns one-line issue summary

 @example
 ```ts
 schemaIssueText([{ message: 'Invalid type: Expected string' }]);
 ```
 */
function schemaIssueText(issues: readonly v.BaseIssue<unknown>[],): string {
  return issues
    .map(function issueText(issue,) {
      return issue.message;
    },)
    .join('; ',);
}

//endregion Helpers
