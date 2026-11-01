import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { openRouterChunksOf, } from './openrouter-chunk-scan.ts';

//region OpenRouter error finish
// A GENERATION THE PROVIDER MARKED FAILED, delivered as a whole stream. The
// gateway normalizes every upstream's stop reason to one of `stop`, `length`,
// `tool_calls`, `content_filter` and `error` (its API reference, read
// 2026-09-08), and the mid-stream failure it documents writes a top-level
// `error` object beside `finish_reason: "error"` and closes without `[DONE]`,
// which `openrouter-stream-error.ts` reads. The wire also carries a second
// shape that reference does not draw: a stream that reasons for tens of
// seconds, writes no content, closes its choice with `finish_reason: "error"`
// and no error object, and then sends `[DONE]`. Seven such replies since
// 2026-09-03 (GLM-5.3-Flash four times on Together; gpt-oss-120b on Together;
// deepseek-v4-pro-0813 on CoreWeave after 8284 reasoning characters and 24 s
// in the eleventh hakureico pass, 2026-09-08 18:08 UTC), each billed at cost
// 0 and each read by the reply ladder as the model's unparseable answer:
// `schema-mismatch (content is not valid JSON: Unexpected end of JSON input
// (model stopped with finish_reason=error)) raw="", voice lost`. THE
// EIGHTEENTH CLASS: the provider said the call failed and the pipeline
// counted the failure as the model's vote.
//
// READ AS A PROVIDER FAILURE, so the call rides the retry ladder under its
// own name, the way the documented shape already does.

/**
 Field carrying the choices on every chunk.
 */
const CHOICES_KEY = 'choices';

/**
 Field on a choice naming why generation stopped, normalized by the gateway.
 */
const FINISH_REASON_KEY = 'finish_reason';

/**
 Normalized stop reason the gateway writes when the upstream failed.
 */
const ERROR_FINISH = 'error';

/**
 Field on a choice carrying the upstream's own spelling of the stop reason.
 */
const NATIVE_FINISH_REASON_KEY = 'native_finish_reason';

/**
 Whether a choice stopped on an error finish, and how the upstream named it.

 @example
 ```ts
 const reading: ErrorFinishReading = { found: true, nativeReason: 'upstream_error', };
 ```
 */
export type ErrorFinishReading =
  | {
    readonly found: true;

    /**
     Upstream's own stop reason, when the gateway forwarded one.
     */
    readonly nativeReason?: string;
  }
    | { readonly found: false; };

/**
 Reading given when no choice stopped on an error finish.
 */
export const ERROR_FINISH_ABSENT: ErrorFinishReading = { found: false, };

/**
 Every choice of every chunk, in arrival order.

 @param bodyText - whole drained `text/event-stream` body

 @returns Choice objects the chunks carried

 @example
 ```ts
 const choices = choicesOf({ bodyText, },);
 ```
 */
function choicesOf(
  { bodyText, }: { readonly bodyText: string; },
): readonly Readonly<Record<string, unknown>>[] {
  return openRouterChunksOf({ bodyText, },)
    .flatMap(function chunkChoices(chunk,): readonly Readonly<Record<string, unknown>>[] {
      /**
       Whatever sits at the field, of unknown type until checked.
       */
      const choices = chunk[CHOICES_KEY];
      if (!Array.isArray(choices,))
        return [];
      return choices.filter(function isChoice(choice: unknown,): choice is Readonly<Record<string, unknown>> {
        return isJsonRecord(choice,);
      },);
    },);
}

/**
 Reads whether a stream's choice stopped on an error finish.

 THE FIRST SUCH CHOICE WINS, as the error-chunk reader's first object does:
 the gateway closes the choice once.

 @param bodyText - whole drained `text/event-stream` body

 @returns That a choice stopped on an error finish with the upstream's own
 reason when forwarded, or that none did

 @example
 ```ts
 const reading = openRouterErrorFinishOf({ bodyText: reply.bodyText, },);
 ```
 */
export function openRouterErrorFinishOf(
  { bodyText, }: { readonly bodyText: string; },
): ErrorFinishReading {
  /**
   Choices whose stop reason is the gateway's error finish.
   */
  const failed = choicesOf({ bodyText, },)
    .filter(function stoppedOnError(choice,): boolean {
      return choice[FINISH_REASON_KEY] === ERROR_FINISH;
    },);

  /**
   First failed choice, or none.
   */
  const [first,] = failed;
  if (first === undefined)
    return ERROR_FINISH_ABSENT;

  /**
   Upstream's own spelling of the stop, of unknown type until checked.
   */
  const native = first[NATIVE_FINISH_REASON_KEY];

  return {
    found: true,
    // Conditional spread keeps an unnamed native reason absent.
    ...((((typeof native) === 'string') && (native !== ''))
      ? { nativeReason: native, }
      : {}),
  };
}

//endregion OpenRouter error finish
