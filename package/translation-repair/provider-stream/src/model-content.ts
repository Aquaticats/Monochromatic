import type { ExtractedCompletion, } from './completion-shape.ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import { refusalText, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Model content handling
// Deterministic handling of what models write: fence unwrapping, thinking-block
// splitting (thinking dominates output tokens on these models), tolerant JSON
// parse attempts, and usage formatting for log lines.

/**
 Strips one wrapping markdown code fence when present,
 because models wrap JSON in fences despite instructions.
 Single linear pass over fence positions; inner text is returned trimmed.

 @param text - model content possibly wrapped in a fence

 @returns Inner text when fenced, trimmed input otherwise

 @example
 ```ts
 stripCodeFence({ text: '```json\n{"a":1}\n```', },);
 ```
 */
export function stripCodeFence({ text, }: { readonly text: string; },): string {
  /**
   Input without surrounding whitespace so fence detection sees column zero.
   */
  const trimmed = text.trim();
  if (!trimmed.startsWith('```',))
    return trimmed;

  /**
   End of the opening fence line (language tag included).
   */
  const openingEnd = trimmed.indexOf('\n',);
  if (openingEnd === (-1))
    return trimmed;

  /**
   Start of the closing fence; must sit after the opening line.
   */
  const closingStart = trimmed.lastIndexOf('```',);
  if (closingStart <= openingEnd)
    return trimmed;

  return trimmed
    .slice(
    openingEnd + 1,
    closingStart,
    )
    .trim();
}

/**
 Opening tag of an embedded thinking block.
 */
const THINK_OPEN = '<think>';

/**
 Closing tag of an embedded thinking block.
 */
const THINK_CLOSE = '</think>';

/**
 Splits an embedded thinking block off the answer.
 This provider delivers reasoning in a separate field, spelled
 `reasoning_content` on some models and `reasoning` on others, measured
 2026-08-21 and counted per model in `stream-delta-scan.ts`. Either way it does
 not reach content. But per-model chat templates may still embed `<think>`
 blocks in content, and a tight token cap truncates mid-thinking;
 both cases must be handled deterministically.

 @param text - model content possibly opening with a thinking block

 @returns Answer after the block, and whether thinking never closed

 @example
 ```ts
 const { answer, truncatedThinking, } = stripThinkBlock({ text: reply.text, },);
 ```
 */
export function stripThinkBlock({ text, }: { readonly text: string; },): {
  readonly answer: string;
  readonly truncatedThinking: boolean;
} {
  /**
   Input without leading whitespace so tag detection sees column zero.
   */
  const trimmed = text.trimStart();
  if (!trimmed.startsWith(THINK_OPEN,)) {
    return {
      answer: text,
      truncatedThinking: false,
    };
  }

  /**
   Position of the closing tag; absence means output died mid-thought.
   */
  const closeAt = trimmed.indexOf(THINK_CLOSE,);
  if (closeAt === (-1)) {
    return {
      answer: '',
      truncatedThinking: true,
    };
  }

  return {
    answer: trimmed.slice(closeAt + THINK_CLOSE.length,),
    truncatedThinking: false,
  };
}

/**
 Parse attempt over model-written JSON;
 failure is data because model content defects are ordinary.

 @param text - fence-stripped model content

 @returns Parsed value, or a detail naming the refusal's class and quoting none of the text

 @example
 ```ts
 const attempt = parseModelJson({ text: stripped, },);
 ```
 */
export function parseModelJson({ text, }: { readonly text: string; },):
  | {
    readonly parsed: true;
    readonly value: unknown;
  }
    | {
      readonly parsed: false;
      readonly detail: string;
    }
  {
  try {
    return {
      parsed: true,
      value: JSON.parse(text,),
    };
  }
  catch (error) {
    // THE CLASS, NOT THE MESSAGE: V8 quotes the opening of the text it refused,
    // and this text is a model's reply or a provider's body, which a detail
    // that reaches a log line or a stored finding would copy there.
    return {
      parsed: false,
      detail: refusalText({ error, },),
    };
  }
}

/**
 Formats the token-usage suffix of a completion log line.

 @param extracted - completion whose usage the log line reports

 @returns Usage suffix, empty when the server reported none

 @example
 ```ts
 rl.debug(`done${formatUsageNote({ extracted, },)}`,);
 ```
 */
export function formatUsageNote(
  { extracted, }: { readonly extracted: ExtractedCompletion; },
): string {
  if (extracted.usage === undefined)
    return '';

  /**
   Component token counts pulled out for the log line.
   */
  const {
    prompt_tokens: promptTokens,
    completion_tokens: completionTokens,
  } = extracted.usage;

  return `, ${String(promptTokens,)}+${String(completionTokens,)} ${
    wordForCount({
      count: promptTokens + completionTokens,
      one: 'token',
      many: 'tokens',
    },)
  }`;
}

//endregion Model content handling
