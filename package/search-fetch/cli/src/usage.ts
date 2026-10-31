/**
 Usage text for the search-fetch CLI.

 The option lines print from the same option names `args.ts` enforces, and the
 verb lines print the shared tool specs' own prompt snippets, so help output and
 enforced behavior cannot drift apart.

 @module
 */

import {
  webFetchToolSpec,
  webSearchToolSpec,
} from '@monochromatic-dev/search-fetch-core/ts/tool-spec.ts';

import {
  fetchVerb,
  programName,
  searchVerb,
  type SearchFetchCliOptionName,
  type SearchFetchVerb,
} from './constants.ts';

//region Constants

/**
 Help text for one accepted flag.
 */
type SearchFetchCliOptionHelp = {
  /**
   Placeholder shown after the flag when it takes a value.
   */
  readonly valueLabel?: string;
  /**
   One-line summary of what the flag does.
   */
  readonly summary: string;
};

/**
 Help text keyed by exactly the option names the parser enforces; TypeScript
 rejects a help entry missing an enforced flag or naming an unenforced one.
 */
const searchFetchCliOptionHelp: Record<SearchFetchCliOptionName, SearchFetchCliOptionHelp> = {
  'include-domains': {
    valueLabel: '<a.com,b.com>',
    summary: 'Comma-separated domain allow-list in one flag; entries are trimmed and empty entries dropped.',
  },
  'from-date': {
    valueLabel: '<iso-date>',
    summary: 'ISO date passed through exactly as the shared tool schema accepts it, as a start date.',
  },
  'to-date': {
    valueLabel: '<iso-date>',
    summary: 'ISO date passed through exactly as the shared tool schema accepts it, as an end date.',
  },
  'help': {
    summary: 'Print usage to stdout and exit 0 (verb usage after a verb, global usage otherwise).',
  },
  'version': {
    summary: 'Print the @monochromatic-dev/search-fetch-cli package version to stdout and exit 0.',
  },
};

/**
 Grammar note printed with every usage text.
 */
const grammarNote = `Everything after -- is data: those words join with single spaces into the one ${
  searchVerb
} query or ${
  fetchVerb
} URL argument, and are never options. Pass the argument in exactly one form: one positional before --, or words after --.
Supplying both forms together is unsupported and outside the contract.`;

/**
 Exit-code note printed with every usage text.
 */
const exitCodeNote = 'Exit codes: 0 success and explicit --help or --version, 1 provider failure or deadline expiry, 2 usage error, configuration error, or blocklist refusal.';

//endregion Constants

//region Public API

/**
 Build the global usage text printed by `--help` and on misuse.

 @returns usage text naming both verbs and every accepted flag

 @example
 ```ts
 console.log(globalUsageText());
 ```
 */
export function globalUsageText(): string {
  return [
    'Usage:',
    `  ${programName} ${searchVerb} <query> [options]`,
    `  ${programName} ${searchVerb} [options] -- <words...>`,
    `  ${programName} ${fetchVerb} <url> [options]`,
    `  ${programName} ${fetchVerb} [options] -- <words...>`,
    `  ${programName} --help`,
    `  ${programName} --version`,
    '',
    grammarNote,
    '',
    optionsSection(),
    '',
    exitCodeNote,
  ].join('\n',);
}

/**
 Build the verb usage text printed by `search-fetch <verb> --help`.

 @param verb - verb whose forms and shared tool description to print

 @returns usage text for one verb

 @example
 ```ts
 console.log(verbUsageText(searchVerb));
 ```
 */
export function verbUsageText(verb: SearchFetchVerb,): string {
  /**
   Shared tool description for the dispatched tool.
   */
  const promptSnippet = verb === searchVerb
    ? webSearchToolSpec.promptSnippet
    : webFetchToolSpec.promptSnippet;
  /**
   Argument placeholder for the verb's positional form.
   */
  const argumentLabel = verb === searchVerb
    ? '<query>'
    : '<url>';
  return [
    'Usage:',
    `  ${programName} ${verb} ${argumentLabel} [options]`,
    `  ${programName} ${verb} [options] -- <words...>`,
    '',
    promptSnippet,
    '',
    grammarNote,
    '',
    optionsSection(),
    '',
    exitCodeNote,
  ].join('\n',);
}

//endregion Public API

//region Helpers

/**
 Build the options section shared by both usage texts.

 @returns flag lines generated from the enforced option names

 @example
 ```ts
 optionsSection();
 ```
 */
function optionsSection(): string {
  return [
    'Options:',
    ...Object
      .entries(searchFetchCliOptionHelp,)
      .map(function optionLine(entry: readonly [
        string,
        SearchFetchCliOptionHelp,
      ],): string {
        /**
         Flag name and help text for the entry.
         */
        const [name, help,] = entry;
        return help.valueLabel === undefined
          ? `  --${name}  ${help.summary}`
          : `  --${name} ${help.valueLabel}  ${help.summary}`;
      },),
  ].join('\n',);
}

//endregion Helpers
