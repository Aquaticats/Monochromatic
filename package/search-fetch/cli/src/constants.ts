/**
 Program constants shared by argument parsing, usage text, and exit handling.

 The option table here is the single spelling of the flag grammar: `args.ts`
 enforces these names with `parseArgs`, and `usage.ts` prints from the same
 keys, so the printed text and the enforced behavior cannot drift.

 @module
 */

//region Program identity

/**
 Command name of the built bin, matching the `bin` entry of package.json.
 */
export const programName = 'search-fetch' as const;

/**
 Verb running the shared web search tool.
 */
export const searchVerb = 'search' as const;

/**
 Verb running the shared web fetch tool.
 */
export const fetchVerb = 'fetch' as const;

/**
 Both verbs in dispatch order.
 */
export const searchFetchVerbs: readonly SearchFetchVerb[] = [
  searchVerb,
  fetchVerb,
];

/**
 Verb names the CLI dispatches to one shared tool each.
 */
export type SearchFetchVerb = typeof searchVerb | typeof fetchVerb;

//endregion Program identity

//region Exit codes

/**
 Exit status of a successful run and of explicit `--help` or `--version` output.
 */
export const successExitCode = 0 as const;

/**
 Exit status of a caller or configuration mistake: usage error, configuration
 error, or blocklist refusal. The only non-standard code, so the bin entry sets
 `process.exitCode` explicitly for it alone; a failed run exits 1 through a
 thrown error.
 */
export const usageExitCode = 2 as const;

/**
 Settled statuses `runSearchFetchCli` returns; a provider or deadline failure
 leaves the function by throwing instead, and takes the standard exit status 1.
 */
export type SearchFetchCliSettledStatus = typeof successExitCode | typeof usageExitCode;

//endregion Exit codes

//region Flag grammar

/**
 Long-form option descriptors for `node:util` `parseArgs`, strict mode.

 Every flag is single-valued: repeating one is a usage error the parser layer
 reports, never last-wins.
 */
export const searchFetchParseArgsOptions: SearchFetchParseArgsOptions = {
  'include-domains': { type: 'string', },
  'from-date': { type: 'string', },
  'to-date': { type: 'string', },
  'help': { type: 'boolean', },
  'version': { type: 'boolean', },
};

/**
 Shape of the enforced long-form option table: one entry per accepted flag,
 carrying the exact `parseArgs` descriptor that name enforces.
 */
export type SearchFetchParseArgsOptions = {
  readonly 'include-domains': { readonly type: 'string'; };
  readonly 'from-date': { readonly type: 'string'; };
  readonly 'to-date': { readonly type: 'string'; };
  readonly 'help': { readonly type: 'boolean'; };
  readonly 'version': { readonly type: 'boolean'; };
};

/**
 Names of every accepted long-form flag.
 */
export type SearchFetchCliOptionName = keyof SearchFetchParseArgsOptions;

//endregion Flag grammar
