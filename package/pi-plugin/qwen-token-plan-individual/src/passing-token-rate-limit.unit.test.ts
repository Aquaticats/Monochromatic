/**
 Tests for recognizing this provider's passing token-rate limit in the error
 text pi formatted, and for the text pi's own automatic retry takes in its
 place.

 The recorded answer is the one fifteen runs ended on 2026-11-13, with its
 request id replaced by a placeholder: the provider's own words, status and
 code, nothing else changed.

 Every case is decided by `isPassingTokenRateLimitAnswer`, and the acceptance of
 the rewritten text is decided by pi's own `isRetryableAssistantError`, so a pi
 upgrade that changes the classifier fails here rather than in a run.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  isRetryableAssistantError,
  type AssistantMessage,
} from '@earendil-works/pi-ai';
import type {
  Logger,
} from '@monochromatic-dev/logger-core/ts';
import {
  isPassingTokenRateLimitAnswer,
  messagePiRetriesInPlaceOfPassingTokenRateLimit,
  QWEN_TOKEN_PLAN_TOKEN_RATE_LIMIT_CODE,
  QWEN_TOKEN_PLAN_TOKEN_RATE_LIMIT_RETRIED_ERROR_MESSAGE,
  QWEN_TOKEN_PLAN_TOKEN_RATE_LIMIT_STATUS,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Discards one log message.
 */
function discardLogMessage(): void {}

/**
 Silent logger these tests run under,
 so a decision is measured by its answer and never by its records.
 */
function fixtureLogger(): Logger {
  return {
    debug: discardLogMessage,
    error: discardLogMessage,
    fatal: discardLogMessage,
    flush: function flush(): Promise<void> {
      return Promise.resolve();
    },
    info: discardLogMessage,
    trace: discardLogMessage,
    warn: discardLogMessage,
  };
}

/**
 The answer fifteen review runs ended on 2026-11-13, as pi wrote it:
 the provider's message, status and code, with the request id replaced.
 */
const RECORDED_PASSING_LIMIT_ANSWER = `429: {"message":"Allocated quota exceeded, please increase your quota limit. For details, see: https://www.alibabacloud.com/help/en/model-studio/error-code#token-limit","id":"00000000-0000-4000-8000-000000000000","type":"insufficient_quota","code":"insufficient_quota"}`;

/**
 Answers this provider sends that must stay final, each with the status and
 code the provider's error-code page gives it.
 */
const FINAL_ANSWERS: readonly string[] = [
  `429: {"message":"Free allocated quota exceeded.","type":"insufficient_quota","code":"insufficient_quota"}`,
  `429: {"message":"The prepaid bill is overdue.","type":"PrepaidBillOverdue","code":"PrepaidBillOverdue"}`,
  `429: {"message":"The postpaid bill is overdue.","type":"PostpaidBillOverdue","code":"PostpaidBillOverdue"}`,
];

/**
 Build one answer text the way pi composes it: the status, a separator, and the
 provider's body.

 @param status - status pi writes in front

 @param body - body the provider sent

 @returns the text one assistant message would carry
 */
function answerText({
  status,
  body,
}: {
  readonly status: number;
  readonly body: string
},): string {
  return `${String(status,)}: ${body}`;
}

/**
 Build the JSON body one answer carries,
 naming the limit the way the provider does.

 @param message - the body's `message` field

 @param code - the body's `code` and `type` fields

 @returns the body as the provider sends it, as text
 */
function bodyText({
  message,
  code = QWEN_TOKEN_PLAN_TOKEN_RATE_LIMIT_CODE,
}: {
  readonly message: string;
  readonly code?: string
},): string {
  return JSON.stringify({
    message,
    id: '00000000-0000-4000-8000-000000000000',
    type: code,
    code,
  },);
}

/**
 The two messages the provider's error-code page gives for the passing limit,
 each extended with the provider's own details as the recorded answer does.

 @returns the answer texts a passing limit arrives as
 */
function documentedPassingAnswers(): string[] {
  return [
    RECORDED_PASSING_LIMIT_ANSWER,
    answerText({
      status: QWEN_TOKEN_PLAN_TOKEN_RATE_LIMIT_STATUS,
      body: bodyText({
        message:
          'You exceeded your current quota, please check your plan and billing details.',
      },),
    },),
    answerText({
      status: QWEN_TOKEN_PLAN_TOKEN_RATE_LIMIT_STATUS,
      body: bodyText({
        message:
          'ALLOCATED QUOTA EXCEEDED, PLEASE INCREASE YOUR QUOTA LIMIT.',
      },),
    },),
  ];
}

/**
 Whether one answer text is this provider's passing limit.

 @param errorMessage - answer text under test

 @returns the decision under test
 */
function readsAsPassingLimit(errorMessage: string,): boolean {
  return isPassingTokenRateLimitAnswer({
    errorMessage,
    logger: fixtureLogger(),
  },);
}

/**
 One assistant message that ended on an error text.

 @param errorMessage - text the message carries

 @returns a complete message, as the stream would settle it
 */
function assistantMessageFixture({
  errorMessage,
}: {
  readonly errorMessage: string
},): AssistantMessage {
  return {
    role: 'assistant',
    content: [],
    api: 'openai-completions',
    provider: 'qwen-token-plan-individual',
    model: 'qwen4-max-preview',
    usage: {
      input: 0,
      output: 0,
      cacheRead: 0,
      cacheWrite: 0,
      totalTokens: 0,
      cost: {
        input: 0,
        output: 0,
        cacheRead: 0,
        cacheWrite: 0,
        total: 0,
      },
    },
    stopReason: 'error',
    errorMessage,
    timestamp: Date.now(),
  };
}

//endregion Fixtures

//region Recognizing the passing limit

await describe({
  name: 'passing token-rate limit recognition',
  children: [
    it({
      name: 'reads the recorded answer as the passing limit',
      fn: async () => {
        expect(readsAsPassingLimit(RECORDED_PASSING_LIMIT_ANSWER,),).toBe(true,);
      },
    },),
    it({
      name: 'reads both documented messages, and reads them in any letter case',
      fn: async () => {
        for (const answer of documentedPassingAnswers())
          expect(readsAsPassingLimit(answer,),).toBe(true,);
      },
    },),
    it({
      name: 'leaves every answer that must stay final untouched',
      fn: async () => {
        for (const answer of FINAL_ANSWERS)
          expect(readsAsPassingLimit(answer,),).toBe(false,);
      },
    },),
    it({
      name: 'leaves the documented words inside a longer sentence untouched',
      fn: async () => {
        const answer = answerText({
          status: QWEN_TOKEN_PLAN_TOKEN_RATE_LIMIT_STATUS,
          body: bodyText({
            message:
              'Your free tier ended. Allocated quota exceeded, please increase your quota limit.',
          },),
        },);
        expect(readsAsPassingLimit(answer,),).toBe(false,);
      },
    },),
    it({
      name: 'leaves a documented message extended by a letter untouched',
      fn: async () => {
        const answer = answerText({
          status: QWEN_TOKEN_PLAN_TOKEN_RATE_LIMIT_STATUS,
          body: bodyText({
            message: 'Allocated quota exceeded, please increase your quota limitX',
          },),
        },);
        expect(readsAsPassingLimit(answer,),).toBe(false,);
      },
    },),
    it({
      name: 'leaves a body that is not JSON untouched, even with the documented words in it',
      fn: async () => {
        const answer = answerText({
          status: QWEN_TOKEN_PLAN_TOKEN_RATE_LIMIT_STATUS,
          body:
            'Allocated quota exceeded, please increase your quota limit. For details, see: https://www.alibabacloud.com/help/en/model-studio/error-code#token-limit',
        },);
        expect(readsAsPassingLimit(answer,),).toBe(false,);
      },
    },),
    it({
      name: 'leaves a body cut short untouched, whether the cut falls inside the message or after it',
      fn: async () => {
        /**
         The body the provider sent, whole.
         */
        const complete = bodyText({
          message: 'Allocated quota exceeded, please increase your quota limit.',
        },);
        /**
         Where the documented message ends inside the body,
         so a cut after it keeps every word the matcher would read.
         */
        const afterMessage = complete.indexOf('quota limit.',) + 'quota limit.'.length;
        for (const cutAt of [Math.floor(complete.length / 2,), afterMessage,])
          expect(
            readsAsPassingLimit(answerText({
              status: QWEN_TOKEN_PLAN_TOKEN_RATE_LIMIT_STATUS,
              body: complete.slice(0, cutAt,),
            },),),
          ).toBe(false,);
      },
    },),
    it({
      name: 'leaves a JSON body that is not an object untouched',
      fn: async () => {
        const answer = answerText({
          status: QWEN_TOKEN_PLAN_TOKEN_RATE_LIMIT_STATUS,
          body: '["Allocated quota exceeded, please increase your quota limit."]',
        },);
        expect(readsAsPassingLimit(answer,),).toBe(false,);
      },
    },),
    it({
      name: 'leaves the documented words under any other status untouched',
      fn: async () => {
        for (const status of [400, 401, 402, 403, 500,])
          expect(
            readsAsPassingLimit(answerText({
              status,
              body: bodyText({
                message:
                  'Allocated quota exceeded, please increase your quota limit.',
              },),
            },),),
          ).toBe(false,);
      },
    },),
    it({
      name: 'leaves a 429 with other words untouched',
      fn: async () => {
        expect(
          readsAsPassingLimit(answerText({
            status: QWEN_TOKEN_PLAN_TOKEN_RATE_LIMIT_STATUS,
            body: bodyText({
              message: 'Too many requests.',
              code: 'Throttling.RateQuota',
            },),
          },),),
        ).toBe(false,);
      },
    },),
    it({
      name: 'leaves the documented message untouched when the code disagrees',
      fn: async () => {
        for (const code of ['Throttling.AllocationQuota', 'AllocationQuota.FreeTierOnly',])
          expect(
            readsAsPassingLimit(answerText({
              status: QWEN_TOKEN_PLAN_TOKEN_RATE_LIMIT_STATUS,
              body: bodyText({
                message:
                  'Allocated quota exceeded, please increase your quota limit.',
                code,
              },),
            },),),
          ).toBe(false,);
      },
    },),
    it({
      name: 'leaves a body that names no code untouched',
      fn: async () => {
        const answer = answerText({
          status: QWEN_TOKEN_PLAN_TOKEN_RATE_LIMIT_STATUS,
          body: JSON.stringify({
            message: 'Allocated quota exceeded, please increase your quota limit.',
          },),
        },);
        expect(readsAsPassingLimit(answer,),).toBe(false,);
      },
    },),
    it({
      name: 'reads a body whose only code field is `type`',
      fn: async () => {
        const answer = answerText({
          status: QWEN_TOKEN_PLAN_TOKEN_RATE_LIMIT_STATUS,
          body: JSON.stringify({
            message: 'Allocated quota exceeded, please increase your quota limit.',
            type: QWEN_TOKEN_PLAN_TOKEN_RATE_LIMIT_CODE,
          },),
        },);
        expect(readsAsPassingLimit(answer,),).toBe(true,);
      },
    },),
    it({
      name: 'leaves text with no status in front untouched',
      fn: async () => {
        for (const errorMessage of [
          bodyText({
            message: 'Allocated quota exceeded, please increase your quota limit.',
          },),
          `4290: ${bodyText({
            message: 'Allocated quota exceeded, please increase your quota limit.',
          },)}`,
          `HTTP 429: ${bodyText({
            message: 'Allocated quota exceeded, please increase your quota limit.',
          },)}`,
        ])
          expect(readsAsPassingLimit(errorMessage,),).toBe(false,);
      },
    },),
  ],
},);

//endregion Recognizing the passing limit

//region The text pi retries, and the message it carries

await describe({
  name: 'passing token-rate limit rewriting',
  children: [
    it({
      name: 'pi leaves the recorded answer alone and takes the rewritten one',
      fn: async () => {
        const recorded = assistantMessageFixture({
          errorMessage: RECORDED_PASSING_LIMIT_ANSWER,
        },);
        // Positive control: the classifier this package has to satisfy does
        // refuse the text the provider sent, and does take the text written here.
        expect(isRetryableAssistantError(recorded,),).toBe(false,);
        expect(
          isRetryableAssistantError(
            assistantMessageFixture({
              errorMessage: QWEN_TOKEN_PLAN_TOKEN_RATE_LIMIT_RETRIED_ERROR_MESSAGE,
            },),
          ),
        ).toBe(true,);
      },
    },),
    it({
      name: 'the rewritten message carries the provider answer whole in a diagnostic',
      fn: async () => {
        const retried = messagePiRetriesInPlaceOfPassingTokenRateLimit({
          message: assistantMessageFixture({
            errorMessage: RECORDED_PASSING_LIMIT_ANSWER,
          },),
          providerAnswer: RECORDED_PASSING_LIMIT_ANSWER,
        },);
        expect(retried.errorMessage,).toBe(
          QWEN_TOKEN_PLAN_TOKEN_RATE_LIMIT_RETRIED_ERROR_MESSAGE,
        );
        expect(retried.errorMessage,).not.toContain(QWEN_TOKEN_PLAN_TOKEN_RATE_LIMIT_CODE,);
        expect(retried.diagnostics,).toHaveLength(1,);
        expect(retried.diagnostics?.[0]?.error?.message,).toBe(
          RECORDED_PASSING_LIMIT_ANSWER,
        );
        expect(retried.diagnostics?.[0]?.details,).toEqual({
          status: QWEN_TOKEN_PLAN_TOKEN_RATE_LIMIT_STATUS,
          code: QWEN_TOKEN_PLAN_TOKEN_RATE_LIMIT_CODE,
        },);
        expect(isRetryableAssistantError(retried,),).toBe(true,);
      },
    },),
    it({
      name: 'a diagnostic already on the message stays beside the new one',
      fn: async () => {
        const message = assistantMessageFixture({
          errorMessage: RECORDED_PASSING_LIMIT_ANSWER,
        },);
        const earlier = {
          type: 'provider.stream',
          timestamp: 0,
          details: { note: 'kept', },
        };
        const retried = messagePiRetriesInPlaceOfPassingTokenRateLimit({
          message: {
            ...message,
            diagnostics: [earlier,],
          },
          providerAnswer: RECORDED_PASSING_LIMIT_ANSWER,
        },);
        expect(retried.diagnostics,).toHaveLength(2,);
        expect(retried.diagnostics?.[0],).toBe(earlier,);
      },
    },),
  ],
},);

//endregion The text pi retries, and the message it carries
