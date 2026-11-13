/**
 This provider's passing token-rate limit,
 and the text pi's own automatic retry takes in its place.

 The provider answers a request that outstrips its per-second or per-minute
 token allowance with status `429` and OpenAI's `insufficient_quota` code,
 and pi reads that code as a spent budget:
 `isRetryableAssistantError` in `@earendil-works/pi-ai` 1.5.2
 (`dist/utils/retry.js`) tests `NON_RETRYABLE_PROVIDER_LIMIT_ERROR_PATTERN`
 before the list that holds `429`,
 and the first list carries `insufficient_quota` and `quota exceeded`
 for OpenAI's billing answer and for budgets nothing refills inside a turn.
 So pi settles on the provider's first answer and the run ends on it.

 The provider's own error-code page
 (`https://www.alibabacloud.com/help/en/model-studio/error-code`,
 read 2026-11-13) gives that answer as
 `429 - Throttling.AllocationQuota/insufficient_quota`,
 whose cause is
 "Token consumption per second or minute (TPS/TPM) triggered rate limiting",
 so it passes by itself and pi's retry is the layer that answers 429s
 (`package/pi-shared/subagent-driver/README.md`, "Which layer retries what",
 owner decision of 2026-11-11).

 What this module recognizes, and what it rewrites, stays inside this provider:
 the same `insufficient_quota` code is OpenAI's own spent-credit answer,
 which stays final because its wording never matches here.
 The provider's answers that must stay final look alike,
 and the message keeps them apart rather than the code:
 `Free allocated quota exceeded.` is a spent free quota under the same status
 and code,
 and `PrepaidBillOverdue` and `PostpaidBillOverdue` are unsettled bills.

 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';
import type {
  AssistantMessage,
  AssistantMessageDiagnostic,
} from '@earendil-works/pi-ai';
import type {
  ForeignBorrowed,
} from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

//region Documented answers

/**
 Status this provider sends its passing token-rate limit under.
 */
export const QWEN_TOKEN_PLAN_TOKEN_RATE_LIMIT_STATUS = 429;

/**
 OpenAI-compatible error code this provider sends that limit under.
 */
export const QWEN_TOKEN_PLAN_TOKEN_RATE_LIMIT_CODE = 'insufficient_quota';

/**
 Messages the provider's error-code page gives for that limit,
 folded to lower case and without the closing period,
 so a message the provider extends with its own details still reads as the
 documented one.

 Each is anchored at the start of the message,
 which is what keeps `Free allocated quota exceeded.` out:
 it holds the words `allocated quota exceeded` but opens with `free`.
 */
const PASSING_TOKEN_RATE_LIMIT_MESSAGE_STARTS: readonly string[] = [
  'allocated quota exceeded, please increase your quota limit',
  'you exceeded your current quota, please check your plan and billing details',
];

/**
 Characters a three-digit HTTP status takes at the start of an answer pi
 formatted from a provider.
 */
const HTTP_STATUS_TEXT_LENGTH = 3;

/**
 Separator pi writes between that status and the provider's body
 (`formatProviderError` in `@earendil-works/pi-ai` 1.5.2
 `dist/utils/error-body.js`, which composes `"<status>: <body>"`).
 */
const STATUS_SEPARATOR = ': ';

/**
 Diagnostic type this module writes onto a message whose error text pi retries,
 so a reader of the run finds the provider's own answer there.
 */
const DIAGNOSTIC_TYPE =
  'qwen-token-plan-individual.passing-token-rate-limit';

//endregion Documented answers

//region The text pi retries

/**
 The error text pi's own automatic retry takes in place of the provider's
 passing limit.

 It names the status and the limit in words `isRetryableAssistantError` accepts,
 and holds none of the words that function tests first:
 the provider's body cannot ride along,
 because `insufficient_quota` and `quota exceeded` are in its not-retried list.

 It also tells a reader where the provider's own answer went,
 because this text is what the user and a run's summary see.
 */
export const QWEN_TOKEN_PLAN_TOKEN_RATE_LIMIT_RETRIED_ERROR_MESSAGE: string =
  '429: token rate limit reached: the token allowance this provider gives a second or a minute, which passes by itself. Pi retries it, and the provider\'s own answer is carried in this message\'s diagnostics and in the log.';

//endregion The text pi retries

//region Reading a body

/**
 A parsed JSON object,
 the only body shape this provider's answers take.
 */
type ReadonlyJsonObject = Readonly<Record<string, unknown>>;

/**
 Sentinel for text that is not a JSON object.

 An absence that travels as a value keeps the reader free of a nullish union,
 and the reader tells it apart by its type before reading any field.
 */
const NOT_A_JSON_OBJECT: unique symbol = Symbol(
  'answer body that is not a JSON object',
);

/**
 Whether one character is a decimal digit.

 @param character - single character to read

 @returns `true` for `0` to `9`

 @example
 ```typescript
 isDecimalDigit({ character: '4' }); // true
 ```
 */
function isDecimalDigit({
  character,
}: {
  readonly character: string;
},): boolean {
  return (character >= '0') && (character <= '9');
}

/**
 Whether one text is exactly three decimal digits, the shape of the status pi
 writes in front of an answer.

 @param text - text to read

 @returns `true` for a three-digit status

 @example
 ```typescript
 isThreeDecimalDigits({ text: '429' }); // true
 ```
 */
function isThreeDecimalDigits({
  text,
}: {
  readonly text: string;
},): boolean {
  if (text.length !== HTTP_STATUS_TEXT_LENGTH)
    return false;
  for (const character of text) {
    if (!isDecimalDigit({ character, },))
      return false;
  }
  return true;
}

/**
 Whether one parsed value is a JSON object,
 which is what this provider's answers carry after the status.

 @param value - parsed JSON value

 @returns `true` for an object that is not an array

 @example
 ```typescript
 if (isJsonObject(parsed)) {
   // parsed's fields can be read by name here
 }
 ```
 */
function isJsonObject(value: unknown,): value is ReadonlyJsonObject {
  if ((typeof value) !== 'object')
    return false;
  if (value === null)
    return false;
  return !Array.isArray(value,);
}

/**
 Read a JSON object out of an answer's body.

 A body cut short, or one that is not JSON at all, is no answer this module can
 read, so the caller keeps the answer it would have had.

 @param text - body pi wrote after the status

 @param l - logger tagged for this reading

 @returns the parsed object, or {@link NOT_A_JSON_OBJECT}

 @example
 ```typescript
 const body = jsonObjectIn({ text: '{"code":"insufficient_quota"}', l });
 ```
 */
function jsonObjectIn({
  text,
  l,
}: {
  readonly text: string;
  readonly l: Logger;
},): ReadonlyJsonObject | typeof NOT_A_JSON_OBJECT {
  try {
    /**
     Value the body parsed to, before its shape is known.
     */
    const parsed: unknown = JSON.parse(
      text,
    );
    if (!isJsonObject(parsed,))
      return NOT_A_JSON_OBJECT;
    return parsed;
  }
  catch (error) {
    l.debug(
      `the answer's body did not parse as JSON, so the answer stands as pi wrote it: ${caughtValueText(error,)}`,
    );
    return NOT_A_JSON_OBJECT;
  }
}

/**
 Whether the body names this provider's quota code,
 reading the two fields the provider fills with it.
 A body that names the code in one field and something else in the other is not
 the answer this module reads,
 and a body that names neither is not read as this limit either.

 @param body - parsed body

 @returns `true` when at least one field carries the code and neither disagrees

 @example
 ```typescript
 if (answeredUnderTheQuotaCode({ body })) {
   // the body carries this provider's quota code
 }
 ```
 */
function answeredUnderTheQuotaCode({
  body,
}: {
  readonly body: ReadonlyJsonObject;
},): boolean {
  /**
   `code` and `type`, the two fields this provider fills with the code.
   */
  const {
    code,
    type,
  } = body;
  if (((typeof code) !== 'string') && ((typeof type) !== 'string'))
    return false;
  if (((typeof code) === 'string')
    && (code !== QWEN_TOKEN_PLAN_TOKEN_RATE_LIMIT_CODE))
    return false;
  if (((typeof type) === 'string')
    && (type !== QWEN_TOKEN_PLAN_TOKEN_RATE_LIMIT_CODE))
    return false;
  return true;
}

/**
 Whether one character is a letter or a digit,
 read without a pattern so a message is scanned a character at a time.

 @param character - single character to read,
 empty when the text ended before it

 @returns `true` for a letter or a digit

 @example
 ```typescript
 isLetterOrDigit({ character: 'x' }); // true
 isLetterOrDigit({ character: ' ' }); // false
 ```
 */
function isLetterOrDigit({
  character,
}: {
  readonly character: string;
},): boolean {
  if (character.length === 0)
    return false;
  if (isDecimalDigit({ character, },))
    return true;
  // A letter is the only character whose case changes, which is what keeps a
  // message that merely extends the documented words with a word out.
  return character.toLowerCase() !== character.toUpperCase();
}

/**
 Whether a message is one the provider's page documents for the passing limit,
 read from the start of the message and ended at a word boundary.

 @param message - the body's own `message` text

 @returns `true` when the message opens with a documented one

 @example
 ```typescript
 namesPassingRateLimitedTokens({ message: 'Allocated quota exceeded, please increase your quota limit. For details, see: …' }); // true
 ```
 */
function namesPassingRateLimitedTokens({
  message,
}: {
  readonly message: string;
},): boolean {
  /**
   Message folded once, so case alone never decides an answer.
   */
  const folded = message.trim()
    .toLowerCase();
  return PASSING_TOKEN_RATE_LIMIT_MESSAGE_STARTS.some(
    function startsWithDocumentedMessage(documented,) {
      if (!folded.startsWith(documented,))
        return false;
      return !isLetterOrDigit({
        character: folded.charAt(documented.length,),
      },);
    },
  );
}

/**
 Whether one assistant message's error text is this provider's passing
 token-rate limit, the answer pi's own automatic retry should take.

 A `429` alone is not enough, and neither is the code alone:
 the answer also has to be this provider's documented wording for a limit on
 tokens a second or a minute, read out of a JSON body.
 Everything else passes through untouched,
 so a plan that is really spent,
 a credential that is refused and a bill that is overdue still end a run at
 once.

 @param errorMessage - error text one assistant message carries

 @param logger - package logger, re-tagged for this reading

 @returns `true` when the text is the provider's passing limit

 @example
 ```typescript
 if (isPassingTokenRateLimitAnswer({ errorMessage, logger })) {
   // pi retries this answer instead of settling on it
 }
 ```
 */
export function isPassingTokenRateLimitAnswer({
  errorMessage,
  logger,
}: {
  readonly errorMessage: string;
  readonly logger: Logger;
},): boolean {
  /**
   Logger tagged for this reading, so a record names the test that decided.
   */
  const l = tagged({
    tag: isPassingTokenRateLimitAnswer.name,
    l: logger,
  },);
  /**
   First characters of the text, where pi writes a status and nowhere else.
   */
  const statusText = errorMessage.slice(
    0,
    HTTP_STATUS_TEXT_LENGTH,
  );
  if (!isThreeDecimalDigits({ text: statusText, },))
    return false;
  if (!errorMessage.startsWith(
    STATUS_SEPARATOR,
    HTTP_STATUS_TEXT_LENGTH,
  ))
    return false;
  if (Number(statusText,) !== QWEN_TOKEN_PLAN_TOKEN_RATE_LIMIT_STATUS)
    return false;
  /**
   Body the provider sent, which pi wrote after the status.
   */
  const body = jsonObjectIn({
    text: errorMessage.slice(
      HTTP_STATUS_TEXT_LENGTH + STATUS_SEPARATOR.length,
    ),
    l,
  },);
  if ((typeof body) === 'symbol')
    return false;
  if (!answeredUnderTheQuotaCode({ body, },))
    return false;
  /**
   Message the body's own `message` field carries.
   */
  const { message, } = body;
  if ((typeof message) !== 'string')
    return false;
  return namesPassingRateLimitedTokens({ message, },);
}

//endregion Reading a body

//region Rewriting the message

/**
 The message pi retries in place of one that ended on the provider's passing
 limit:
 the same message with the error text pi accepts,
 and the provider's own answer kept whole in a diagnostic,
 which is a field pi stores and shows but never classifies.

 @param message - message that ended on the provider's passing limit

 @param providerAnswer - error text the provider's answer arrived as, whole

 @returns the message pi's own automatic retry takes

 @example
 ```typescript
 const retried = messagePiRetriesInPlaceOfPassingTokenRateLimit({ message, providerAnswer });
 ```
 */
export function messagePiRetriesInPlaceOfPassingTokenRateLimit({
  message,
  providerAnswer,
}: {
  readonly message: ForeignBorrowed<AssistantMessage>;
  readonly providerAnswer: string;
},): AssistantMessage {
  /**
   Diagnostic carrying the provider's answer word for word.
   */
  const diagnostic: AssistantMessageDiagnostic = {
    type: DIAGNOSTIC_TYPE,
    timestamp: Date.now(),
    error: { message: providerAnswer, },
    details: {
      status: QWEN_TOKEN_PLAN_TOKEN_RATE_LIMIT_STATUS,
      code: QWEN_TOKEN_PLAN_TOKEN_RATE_LIMIT_CODE,
    },
  };
  return {
    ...message,
    errorMessage: QWEN_TOKEN_PLAN_TOKEN_RATE_LIMIT_RETRIED_ERROR_MESSAGE,
    diagnostics: [
      ...(message.diagnostics ?? []),
      diagnostic,
    ],
  };
}

//endregion Rewriting the message
