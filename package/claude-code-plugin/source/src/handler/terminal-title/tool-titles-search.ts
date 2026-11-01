/**
 Claude Code title entries for search, web, notebook, LSP, and discovery tools.
 
 @module
 */

import {
  fieldTitleEntry,
  inputTitleEntry,
  pathTitleEntry,
  textTitleEntry,
  TOOL_TITLE_TEXT_MISSING,
  type ToolTitleEntry,
  type ToolTitleInput,
} from '@monochromatic-dev/agent-harness-shared-terminal-title/ts';

//region AskUserQuestion helpers

/**
 Checks whether title formatter result is text-missing sentinel.
 
 @param value - because formatter result can be title text or sentinel
 
 @returns whether value is text-missing sentinel
 
 @example
 ```ts
 isToolTitleTextMissing(TOOL_TITLE_TEXT_MISSING);
 // true
 ```
 */
function isToolTitleTextMissing(
  value: string | typeof TOOL_TITLE_TEXT_MISSING,
): value is typeof TOOL_TITLE_TEXT_MISSING {
  return ((typeof value) === 'symbol')
    && (value === TOOL_TITLE_TEXT_MISSING);
}

/**
 Tests whether object owns a question field.
 
 @param value - Runtime question candidate.
 
 @returns whether question field exists directly on candidate.
 
 @mutates value - `Object.hasOwn` may invoke caller-owned proxy hooks.
 
 @example
 ```ts
 hasOwnQuestion({ question: 'Continue?' });
 ```
 */
function hasOwnQuestion(value: object,): value is { readonly question: unknown; } {
  return Object.hasOwn(
    value,
    'question',
  );
}

/**
 Extracts first question text from AskUserQuestion input.
 
 @param input - because AskUserQuestion nests text inside questions array
 
 @returns first question string or text-missing sentinel when input shape is absent
 
 @example
 ```ts
 firstQuestionText({ questions: [{ question: 'Continue?' }] });
 // 'Continue?'
 ```
 */
function firstQuestionText(input: ToolTitleInput,): string | typeof TOOL_TITLE_TEXT_MISSING {
  /**
   Candidate questions value from tool input.
   */
  const { questions, } = input;
  if (!Array.isArray(questions,))
    return TOOL_TITLE_TEXT_MISSING;
  /**
   Questions narrowed to unknown entries before destructuring.
   */
  const unknownQuestions: readonly unknown[] = questions;
  /**
   First question candidate.
   */
  const [first,] = unknownQuestions;
  if ((first === null) || ((typeof first) !== 'object'))
    return TOOL_TITLE_TEXT_MISSING;
  if (!hasOwnQuestion(first,))
    return TOOL_TITLE_TEXT_MISSING;
  /**
   Question text from first question object.
   */
  const { question, } = first;
  if ((typeof question) === 'string')
    return question;
  return TOOL_TITLE_TEXT_MISSING;
}

//endregion AskUserQuestion helpers

//region URL helpers

/**
 Formats URL host for title text.
 
 @param url - because WebFetch titles should show destination host
 
 @returns URL hostname or generic URL text when parsing fails
 
 @example
 ```ts
 webFetchValue('https://example.com/a');
 // 'example.com'
 ```
 */
function webFetchValue(url: string,): string {
  if (!URL.canParse(url,))
    return 'URL';
  return new URL(url,).hostname;
}

//endregion URL helpers

/**
 Built-in tool names held in the search registry segment.
 */
type SearchToolTitleName =
  | 'WebSearch'
    | 'AskUserQuestion'
    | 'WebFetch'
    | 'NotebookEdit'
    | 'LSP'
    | 'Skill'
    | 'ToolSearch';

/**
 Title entries for search, web, notebook, LSP, skill, and discovery tools.
 */
const SEARCH_TOOL_TITLES: Record<SearchToolTitleName, ToolTitleEntry> = {
  WebSearch: textTitleEntry({
    field: 'query',
    labels: {
      pre: 'Searching web for',
      post: 'Searched web for',
    },
    fallback: {
      pre: 'Searching web',
      post: 'Searched web',
    },
  },),
  AskUserQuestion: inputTitleEntry({
    fallback: {
      pre: 'Asking question',
      post: 'Asked question',
    },
    format({
      input,
      tense,
    }): string | typeof TOOL_TITLE_TEXT_MISSING {
      /**
       First question text from nested input.
       */
      const question = firstQuestionText(input,);
      if (isToolTitleTextMissing(question,))
        return TOOL_TITLE_TEXT_MISSING;
      return `${tense === 'pre' ? 'Asking' : 'Asked'}: ${question}`;
    },
  },),
  WebFetch: fieldTitleEntry({
    field: 'url',
    fallback: {
      pre: 'Fetching URL',
      post: 'Fetched URL',
    },
    format({
      value,
      tense,
    }): string {
      return `${tense === 'pre' ? 'Fetching' : 'Fetched'} ${webFetchValue(value,)}`;
    },
  },),
  NotebookEdit: pathTitleEntry({
    field: 'notebook_path',
    labels: {
      pre: 'Editing notebook',
      post: 'Edited notebook',
    },
    noun: 'notebook',
  },),
  LSP: textTitleEntry({
    field: 'operation',
    labels: {
      pre: 'Running LSP',
      post: 'Finished LSP',
    },
    fallback: {
      pre: 'Running LSP query',
      post: 'Finished LSP query',
    },
  },),
  Skill: textTitleEntry({
    field: 'skill',
    labels: {
      pre: 'Running skill',
      post: 'Finished skill',
    },
    fallback: {
      pre: 'Running skill',
      post: 'Finished skill',
    },
  },),
  ToolSearch: textTitleEntry({
    field: 'query',
    labels: {
      pre: 'Discovering tools for',
      post: 'Discovered tools for',
    },
    fallback: {
      pre: 'Discovering tools',
      post: 'Discovered tools',
    },
  },),
};

export { SEARCH_TOOL_TITLES, };
