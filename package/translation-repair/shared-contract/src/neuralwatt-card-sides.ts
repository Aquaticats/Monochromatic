import type { NeuralWattCard, } from './model-card.ts';

//region NeuralWatt card sides
// NEURALWATT'S SIDE OF THE TWO CARDS IT SERVES, 2026-10-20, kept beside
// `model-cards.ts` rather than in it because that table sits at the package's
// line budget. The figures are `GET https://api.neuralwatt.com/v1/models`
// (public, fetched 2026-10-20): both models list a context of 1,048,560 and
// image input on the standard lane the owner's free daily energy covers
// (`https://docs.neuralwatt.com/billing/daily-free-energy`).
//
// NO HOLD CHANGES WITH THESE SIDES. The holds on both cards were recorded
// before NeuralWatt joined and stay as they were: the owner unseated
// GLM-5.3-Flash as a judge on 2026-09-02, and no transcription of a picture
// has been measured on any stack of DeepSeek V4.1 Flash.

/**
 NeuralWatt's side of the GLM-5.3-Flash card.

 The listing states no output ceiling for this model (`max_output_tokens`
 null), so the context window stands in and the measured completion cap on the
 card applies as it does everywhere else.

 @example
 ```ts
 const side = GLM_5_3_FLASH_ON_NEURALWATT;
 ```
 */
export const GLM_5_3_FLASH_ON_NEURALWATT: NeuralWattCard = {
  id: 'glm-5.3-flash',
  readsImages: true,
  contextLength: 1_048_560,
  maxOutputLength: 1_048_560,
  // LOW ON A PICTURE CALL, 2026-10-21 (XingZ6016). The listing's reasoning is
  // mandatory with `supported_efforts` `max`, `high`, `low` and `default_effort`
  // `max`, so a call that sends no effort reasons at the deepest level, and on a
  // dense 2400 by 2616 screenshot that spent all 18316 completion tokens of the
  // card's cap on 40652 characters of reasoning and wrote no answer. `low` is the
  // shallowest level the model has, hence the one that leaves the most of an
  // unchanged cap for the transcript. NO PICTURE CALL HAS BEEN MEASURED AT ANY
  // LEVEL: the choice rests on the one failure at `max`, on the listing's
  // ordering, and on the text probe named here, which is the nearest measurement and
  // is not a picture call. A transcription is not a deduction, so the depth is
  // not what the reading needs. THE PROBE of 2026-10-22, a keyed run whose raw
  // numbers are recorded in `package/translation-repair/provider/README.md`
  // ("Thinking-parameter evidence"): reasoning tokens of this model over three
  // repeats on one digit-sum question, text only. No effort sent: 3947, 558, 1144. `low`: 143,
  // 116, 118. `high`: 169, 222, 182. `max`: 555, 579, 355. `low` kept thinking
  // on (every repeat reasoned), which complies with the owner's "we should
  // never disable thinking on anything" of 2026-10-22, and answered 2 of 3
  // right where `high` and `max` and the unset default answered 3 of 3. LIMITS:
  // one question, three repeats, no picture, so the 2 of 3 is a signal to
  // watch and says nothing of a rate.
  pictureReasoningEffort: 'low',
  // LOW ON A TEXT CALL TOO, 2026-10-21 (zheermao22, ledger B807). The owner's
  // lift covers NeuralWatt text calls, and a text call at the default `max` ran
  // to the 18316-token cap with 0 content characters in 11 of 288 text calls
  // over the pass logs from zheermao19 to zheermao22 (4 in zheermao22 alone,
  // 29 percent of that run's NeuralWatt energy). The uncapped calls do not sit
  // well under the cap either: p50 2125 tokens, p90 7573, p99 15731, largest
  // 18262. DeepSeek V4.1 Flash, at its own default `high`, capped on 12 of 819
  // text calls, so `high` is not trusted to cure it here, and `high` is untested
  // on real text calls (the 2026-10-22 probe ran it on one digit-sum question
  // only). `low` STAYS, because the runaways at the default `max` are the
  // failure and `low` spent the fewest reasoning tokens of the four settings in
  // the probe (116 to 143, against 169 to 222 at `high` and 355 to 579 at
  // `max`) while still thinking. THE LIMITS ARE THE PROBE'S: one question, three
  // repeats, no picture call, and the unset default spent 558 to 3947 across
  // the same three repeats, a spread wider than any gap between settings. The
  // energy matrix the meter estimates from was drawn on the defaults, so the
  // hold per call may now be larger than the call uses.
  textReasoningEffort: 'low',
};

/**
 NeuralWatt's side of the DeepSeek V4.1 Flash card, whose listing states a
 maximum output of 393,216 tokens.

 THE SERVED ID IS ALREADY THE 1017 WEIGHTS. The listing's `huggingface_id`
 for `deepseek-v4.1-flash` reads `deepseek-ai/DeepSeek-V4.1-Flash-1017`
 (`GET https://api.neuralwatt.com/v1/models`, 2026-10-20), so the owner's
 migration to the 1017 weights of the same day changed nothing on this side.

 NO REASONING EFFORT IS NAMED HERE, picture or text (ledger B807). The repo
 records this model's default (`high`, `neuralwatt-catalog.ts`) and that its
 reasoning is on by default, and records no `supported_efforts` list for it;
 the GLM list in ledger B719 is that model's alone. A call that sends an
 effort the model does not list is a guess at a provider's reply, so none is
 sent. It capped on 12 of 819 text calls at the default.

 @example
 ```ts
 const side = DEEPSEEK_V4_1_FLASH_ON_NEURALWATT;
 ```
 */
export const DEEPSEEK_V4_1_FLASH_ON_NEURALWATT: NeuralWattCard = {
  id: 'deepseek-v4.1-flash',
  readsImages: true,
  contextLength: 1_048_560,
  maxOutputLength: 393_216,
};

//endregion NeuralWatt card sides
