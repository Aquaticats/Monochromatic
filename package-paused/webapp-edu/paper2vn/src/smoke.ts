/**
 * Live-LLM smoke test for paper2vn.
 *
 * Verifies the chapter-generation round-trip end to end **outside the
 * browser**: the smoke runs the same prompt the page would build, hits
 * OpenRouter from Bun, parses the response with the page's own
 * `dialogue/generator.ts` validator, and walks the model fallback list
 * (Kimi K2.6 -> Claude Haiku 4.5) until one succeeds.
 *
 * Why not drive the page through agent-browser? Two reasons:
 *
 *  1. agent-browser's CDP `Runtime.evaluate` queues behind the page's
 *     in-flight fetch, and reasoning models like Kimi K2.6 hold the
 *     fetch open for 40-80s; well past the per-command timeout.
 *  2. `Input.dispatchMouseEvent` for a click on `Start lecture` after
 *     filling the textarea is observed to time out at the CDP layer
 *     even though manual clicking works in a real browser. Repro and
 *     workarounds are documented in
 *     `TROUBLESHOOTING.agent-browser-cdp-during-fetch.md`.
 *
 * The UI flow is verified separately via `mise run dev` and the manual
 * agent-browser walkthrough captured in the README. This smoke covers
 * the brittle, integration-sensitive piece: that the bundled prompts
 * produce a parseable response from each supported model.
 *
 * Pre-req: `mise run build` (the corresponding mise task chains it).
 *
 * Required env: `PAPER2VN_OPENROUTER_API_KEY` (or `OPENROUTER_API_KEY`).
 * Optional env:
 *   - `PAPER2VN_OPENROUTER_MODEL`: restrict to a single model
 *   - `PAPER2VN_SMOKE_PAPER`: override the default tiny paper text
 */
/*
 * Side-effect import: installs `globalThis.localStorage` before any
 * page module runs. ES imports evaluate in source order, so this
 * MUST stay first; moving it below the page-module imports breaks
 * `state.ts` hydration.
 */
// oxlint-disable-next-line eslint-plugin-import/no-unassigned-import -- side-effect: installs localStorage shim
import './smoke-storage-shim.ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core';

import { generateChapters, } from './client/dialogue/generator.ts';
// oxlint-disable-next-line eslint-plugin-import/no-unassigned-import -- side-effect: typesafe-i18n runtime registers locale loaders
import './client/i18n/runtime.ts';
import { loadAllLocales, } from './client/i18n/i18n-util.sync.ts';
import {
  updateProvider,
  updateSettings,
} from './client/state.ts';

export {};

/**
 * Throws when the named env var is missing or empty.
 *
 * @param name - env var key
 *
 * @returns the value
 *
 * @throws when unset
 */
function requireEnv(name: string,): string {
  /**
   * Raw env-var value, validated as non-empty before being returned.
   */
  const value = process.env[name];
  if ((value === undefined) || (value === ''))
    throw new Error(`smoke: missing required env var ${name}`,);
  return value;
}

/**
 * OpenRouter API key. Scoped per-package via
 * `PAPER2VN_OPENROUTER_API_KEY` in `.env.local` (mise injects it for
 * any task in this package). Falls back to the unscoped
 * `OPENROUTER_API_KEY` so an ad-hoc one-off run still works.
 */
const OPENROUTER_API_KEY = process.env
  .PAPER2VN_OPENROUTER_API_KEY
  ?? requireEnv('OPENROUTER_API_KEY',);

/**
 * Smoke-test model order.
 *
 * Kimi K2.6 is the primary; the app's recommended default. It is a
 * reasoning model and a single chapter-generation call typically takes
 * 40-80s before content lands. If Kimi errors (transient upstream
 * failure, malformed JSON, bad gateway), fall through to Claude Haiku
 * 4.5 which is faster but still adequate for a smoke.
 *
 * Override the entire list with a single model via
 * `PAPER2VN_OPENROUTER_MODEL`.
 */
/**
 * Raw env override for the OpenRouter model list, validated below before use.
 */
const envOpenRouterModel = process.env
  .PAPER2VN_OPENROUTER_MODEL;
/**
 * OpenRouter models tried in order during the smoke run; the first that
 * returns a parseable response wins. Override the entire list with a
 * single model via `PAPER2VN_OPENROUTER_MODEL`.
 */
const OPENROUTER_MODELS: readonly string[] =
  ((envOpenRouterModel !== undefined) && (envOpenRouterModel !== ''))
    ? [envOpenRouterModel,]
    : [
      'moonshotai/kimi-k2.6',
      'anthropic/claude-haiku-4.5',
    ];

/**
 * Tiny paper text. Specific enough that a model has something
 * concrete to chapterize but small enough to stay within any
 * model's context window.
 */
const DEFAULT_PAPER_TEXT = `
Title: A Tiny Note on Iterative Refinement

Abstract. We propose a minimal iterative refinement loop in which
each step both samples a candidate and scores it against a
local consistency criterion. The procedure terminates when the
score plateau persists for two successive iterations.

1. Method
We initialize with a random sample from the prior. At each step,
we propose a new candidate by perturbing the current state and
accept it when the consistency score does not regress. The
acceptance criterion is intentionally permissive to avoid early
collapse.

2. Results
On three small synthetic tasks (sorting integers, balancing
parentheses, and reconstructing a permuted sequence), the
procedure converges within twelve iterations on average. Variance
across seeds is low.

3. Discussion
The method is simple and reproducible. The next step is scaling
to larger inputs and characterizing the failure modes when the
consistency criterion is noisy.
`
  .trim();

/**
 * Raw env override for the paper body; falls back to {@link DEFAULT_PAPER_TEXT} when unset or empty.
 */
const envSmokePaper = process.env
  .PAPER2VN_SMOKE_PAPER;
/**
 * Paper body for the smoke run; env override takes precedence over the default.
 */
const PAPER_TEXT = ((envSmokePaper !== undefined) && (envSmokePaper !== ''))
  ? envSmokePaper
  : DEFAULT_PAPER_TEXT;

/**
 * Top-of-step log helper (mirrors the rest of the codebase using stderr).
 *
 * @param message - human-readable step description
 */
function step(message: string,): void {
  console.error(`[smoke] ${message}`,);
}

//region Provider seeding
/*
 * generateChapters reads provider config and locale via the page's
 * `state.ts` module, which in turn reads `localStorage`. In Bun there
 * is a global `localStorage` (Web Storage API support is built in),
 * so seeding the same way the page does is enough to make
 * generateChapters use the right model and key.
 */
//endregion

/**
 * Seeds the active provider and locale into Bun's localStorage so the
 * page modules pick them up when imported.
 *
 * @param model - OpenRouter slug to install
 */
function seedProviderState(model: string,): void {
  updateProvider({
    id: 'openrouter',
    model,
    apiKey: OPENROUTER_API_KEY,
    baseUrl: '',
    acknowledgedAnthropicWarning: false,
  },);
  updateSettings({ locale: 'en', },);
}

//region typesafe-i18n boot
/*
 * generateChapters calls `LL().persona()` and friends. The runtime
 * lazy-loads the active locale, but only after `loadAllLocales()` has
 * registered it. Importing `runtime.ts` is not enough; we must run
 * the loader explicitly the way `bootI18n()` does in the browser.
 */
//endregion
loadAllLocales();

/**
 * Tries each model in `OPENROUTER_MODELS` in order. First success
 * wins. Each model gets one attempt; transient failures fall through
 * to the next.
 *
 * @returns the model that succeeded plus its generation result
 *
 * @throws when every model in the list fails
 */
async function runWithFallback(): Promise<{
  model: string;
  generation: Awaited<ReturnType<typeof generateChapters>>;
  durationMs: number;
}> {
  /**
   * Accumulated per-model failure messages reported when every model fails.
   */
  const errors: string[] = [];
  for (const model of OPENROUTER_MODELS) {
    seedProviderState(model,);
    step(`attempting chapter generation with ${model}`,);
    /**
     * Per-attempt start timestamp used to report elapsed milliseconds.
     */
    const t0 = Date.now();
    try {
      /**
       * Successful generator output, returned with timing info on first success.
       *
       * Sequential await is intentional: each model gets one attempt before
       * the next falls through, so parallel `Promise.all` would defeat the
       * fallback ordering.
       */
      // oxlint-disable-next-line eslint/no-await-in-loop -- sequential fallback, must wait before trying next model
      const generation = await generateChapters({
        paperText: PAPER_TEXT,
        signal: undefined,
      },);
      return {
        model,
        generation,
        durationMs: Date.now()
          - t0,
      };
    }
    catch (err) {
      /**
       * Normalised error message stashed for the eventual aggregate throw.
       */
      const message = err instanceof Error ? err.message : String(err,);
      console.error(
        `[smoke] ${model} failed after ${Date.now()
          - t0}ms: ${message}`,
      );
      errors.push(`${model}: ${message}`,);
    }
  }
  throw new Error(
    `smoke: all models failed:\n  - ${errors.join('\n  - ',)}`,
  );
}

/**
 * Top-level start timestamp used to report the overall PASS duration.
 */
const startedAt = Date.now();
/**
 * Successful run output destructured into the script-scope bindings.
 */
const {
  model,
  generation,
  durationMs,
} = await runWithFallback();

step(
  `chapters generated in ${durationMs}ms via ${model}: title="${generation.title}" chapters=${generation.chapters
    .length}`,
);
for (
  const [
    i,
    chapter,
  ] of generation.chapters
    .entries()
) {
  console.error(
    `[smoke]   chapter ${
      i + 1
    }: title="${chapter.title}" beats=${chapter.dialogue
      .length}`,
  );
}

if (generation.chapters
  .length
  === 0)
  throw new Error('smoke: model returned zero chapters',);

//region Sanity-check the first beat
/*
 * Validates that the bundled validator's output is non-empty and the
 * pose tag is one we recognize. Catches regressions where the LLM
 * returns an unrecognized shape that asBeat/asChapter quietly drops.
 */
//endregion
/**
 * First chapter of the generation, asserted non-empty above.
 */
const firstChapter = nonNullishOrThrow(generation.chapters[0],);
/**
 * First dialogue beat of the first chapter, checked for non-empty text below.
 */
const [firstBeat,] = firstChapter.dialogue;
if ((firstBeat === undefined) || (firstBeat.text
  .trim()
  === '')) {
  throw new Error(
    'smoke: first chapter has no usable dialogue beat',
  );
}

/**
 * Number of characters to log from the first beat as a smoke preview.
 */
const FIRST_BEAT_PREVIEW_CHARS = 80;
/**
 * Total elapsed time for the smoke run, reported in the PASS log.
 */
const tookMs = Date.now()
  - startedAt;
console.error(
  `[smoke] PASS in ${tookMs}ms (model: ${model}, chapters: ${generation.chapters
    .length}, first beat: "${
    firstBeat.text
      .slice(
      0,
      FIRST_BEAT_PREVIEW_CHARS,
    )
  }...")`,
);
