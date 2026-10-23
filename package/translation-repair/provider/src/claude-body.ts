import type { ChatMessage, } from '@monochromatic-dev/module-llm-type/ts';

import type { ReadableResponseFormat, } from './anthropic-tool.ts';
import {
  speakingTurns,
  systemTextOf,
} from './anthropic-request.ts';
import type { VisionMessage, } from './chat-contract.ts';
import type { ClaudeModelInfo, } from './claude-catalog.ts';
import { claudeSchemaOf, } from './claude-schema.ts';

//region Claude body
// THE BODY ONE CLAUDE CALL SENDS, member by member from the owner's brief of 2026-10-22 and the probes it cites.
// What it leaves out is as much of the contract as what it sends: no `temperature`, `top_p` or `top_k` (any
// non-default value is refused with HTTP 400 on both models, and a request has no field to carry one), no assistant
// prefill (400 as well), no forced `tool_choice`, no `budget_tokens` and no way to turn thinking off.

/**
 Refusal raised when a conversation ends on an assistant turn, which the API reads as a prefill and answers with 400
 on both models.

 A THROW RATHER THAN A DATA OUTCOME, because it is a prompt built wrong in our own code and not a thing a model did.

 @example
 ```ts
 throw new ClaudePrefillError();
 ```
 */
export class ClaudePrefillError extends Error {
  /**
   Declares this message safe to forward: it names a structural rule and quotes no message.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds the failure.

   @example
   ```ts
   new ClaudePrefillError();
   ```
   */
  public constructor() {
    super(
      'The conversation ends on an assistant turn, which Sonnet 5.5 and Haiku 5.5 refuse as a prefill (HTTP 400); '
        + 'end it on a user turn',
    );
    this.name = 'ClaudePrefillError';
  }
}

/**
 Builds the whole body of one Messages call.

 @param card - the Claude card the call is for, whose wire model the body carries

 @param messages - conversation as the caller built it, system message included

 @param responseFormat - structured-output constraint, omitted for free text

 @param maxTokens - ceiling for the call

 @returns Body to serialise

 @throws {@link ClaudePrefillError} when the conversation ends on an assistant turn

 @throws {@link EmptyConversationError} where the messages cannot form a conversation

 @throws {@link ClaudeSchemaKeywordError} when the response format carries a keyword the structured-output feature
 refuses, before any request is made

 @example
 ```ts
 const body = buildClaudeBody({ card, messages, maxTokens: 1_000, },);
 ```
 */
export function buildClaudeBody(
  {
    card,
    messages,
    responseFormat,
    maxTokens,
  }: {
    readonly card: ClaudeModelInfo;
    readonly messages: readonly (ChatMessage | VisionMessage)[];
    readonly responseFormat?: ReadableResponseFormat;
    readonly maxTokens: number;
  },
): Readonly<Record<string, unknown>> {
  /**
   Turns the conversation holds outside the system prompt.
   */
  const turns = speakingTurns({ messages, },);
  if (turns.at(-1,)
    ?.role
    === 'assistant')
    throw new ClaudePrefillError();
  /**
   Every system message, joined into the one field this protocol takes.
   */
  const instruction = systemTextOf({ messages, },);
  return {
    model: card.wireModel,
    max_tokens: maxTokens,
    stream: true,
    // THINKING IS ALWAYS ADAPTIVE, AND NOTHING HERE TURNS IT OFF OR BUDGETS IT. The scoped rule, owner 2026-10-22
    // ("I'm only against thinking params on 3rd tier providers", "Anthropic counts as first rate", "we should never
    // disable thinking on anything"): thinking parameters are cleared on Anthropic, DeepSeek direct, Xiaomi direct and
    // NeuralWatt and banned on Hyper, Synthetic, OpenRouter and Bedrock. So this body sends `adaptive` and never
    // `disabled` or `between_tools` (which turns thinking off on Sonnet 5.5), and never `budget_tokens`, which both
    // models answer with HTTP 400.
    //
    // THE SUMMARY IS SHOWN (`display: 'summarized'`) SO THE STREAM CARRIES `thinking_delta` EVENTS WHILE THE MODEL
    // THINKS. The default on both models is `omitted`, under which each thinking block streams with an empty text and
    // only a signature (the bundled API skill, `shared/model-migration.md`: "the default streams empty-text thinking
    // blocks (long pause before output)", and "set `display: "summarized"` to restore visible progress during
    // thinking"). The stream idle guard resets its window on generated text only, and an empty delta is not text
    // (`stream-idle-guard.ts`), so under the default a long think looked like a stalled stream, was cut at the first
    // content window and was sent again, billing the thinking tokens each time. `display` changes what is returned and
    // not what is generated or billed ("thinking happens and is billed the same under every setting", same file), and
    // the reader drops the summary text (`anthropic-completion.ts`), so the answer is unchanged.
    //
    // THE 120 S IDLE WINDOW WAS CHECKED AGAINST SIX LIVE CALLS (R2 of the second review of the Claude route, kept in
    // `package/translation-repair/doc/claude-route-probe-figures-and-review-handles.md`), the lead's probe of
    // 2026-10-22, 22:02 to 22:03 UTC: the longest pause between stream events (`maxGap` in `reportStreamProgress`) was
    // 40, 112, 1,990, 3,429, 3,278 and 3,934 ms, the thinking calls 1,990 to 3,934 ms, and the longest call ran 38 s.
    // Summaries arrive in batches, and the longest pause seen is 3.9 s against the guard's 120,000 ms, so the window
    // stands. SIX SHORT-PROMPT CALLS ARE NOT A BOUND: the Claude `maxGap` lines of the first corpus pass are to be read
    // against 120 s, and the window changed only if one comes close.
    thinking: {
      type: 'adaptive',
      display: 'summarized',
    },
    // THE STABLE PREFIX IS THE SYSTEM FIELD. Every call of a stage opens on the same instruction (house policy, the
    // rules, the reply block) and differs in the sheet it shows, which is the user turn, so one ephemeral breakpoint
    // on the last system block caches what hundreds of calls repeat. The default time to live of five minutes stands,
    // with no `ttl` member: the brief asks for 5 minutes, priced at the 5-minute write rate on the card. A prefix under
    // 512 tokens on either model is not cached, silently, and costs nothing extra (the bundled API skill,
    // `shared/prompt-caching.md`, minimum cacheable prefix for Sonnet 5.5 and Haiku 5.5).
    ...((instruction === '')
      ? {}
      : {
        system: [{
          type: 'text',
          text: instruction,
          cache_control: { type: 'ephemeral', },
        },],
      }),
    messages: turns,
    // STRUCTURED OUTPUT BY `output_config.format`, NEVER BY A FORCED TOOL. The parent session's probe of 2026-10-22
    // (`sonnet-route-probe.out`): Sonnet 5.5 answered `output_config.format` with adaptive thinking (192 thinking
    // tokens, a right answer) and refused a forced `tool` with HTTP 400 ("tool_choice: type \"tool\" and \"any\" are
    // not supported for this model."). Haiku 5.5 under a forced `tool` or `any` returned no thinking tokens on 12 of 12
    // calls and the wrong answer on 6 of 6 hard ones, against 0 of 12 wrong with `format` or auto
    // (`haiku-forced-tool-probe*.out`). The schema goes through narrowed to what the feature accepts and nothing more
    // (`claude-schema.ts`): every object closed with `additionalProperties: false` and the unsupported bounds dropped,
    // which the reply guards of the stages still enforce.
    // `effort` is the card's, one level per effort card; a card that names none sends none, which is the model's own
    // default (Sonnet high, Haiku medium), the neutral reading of the owner's "Why don't you bench?".
    ...((responseFormat === undefined) && (card.effort === undefined)
      ? {}
      : {
        output_config: {
          ...((card.effort === undefined) ? {} : { effort: card.effort, }),
          ...((responseFormat === undefined)
            ? {}
            : {
              format: {
                type: 'json_schema',
                schema: claudeSchemaOf({
                  schema: responseFormat.json_schema
                    .schema,
                },),
              },
            }),
        },
      }),
  };
}

//endregion Claude body
