/**
 * E2e tests for paper2vn.
 *
 * Drives the built `dist/final/index.html` in a real browser via
 * Playwright. Tier 1 (UI) covers menu rendering, navigation, settings,
 * locale switching, the missing-key gate, save listing, and the lecture
 * runtime against a seeded save; no LLM required, fast and
 * deterministic. Tier 2 covers the live LLM round-trip and is skipped
 * when `PAPER2VN_OPENROUTER_API_KEY` is unset, so CI without a key
 * still runs the rest.
 *
 * Pre-req: `mise run //packages/webapp-edu/paper2vn:build`. The mise
 * task chains the build, so calling `mise run :test:e2e` is enough.
 *
 * State is seeded into the page via `addInitScript` before any module
 * runs. `state.ts` reads `localStorage` at module load to hydrate
 * provider, settings, and saves; the seed has to land first for the
 * page to come up in the desired state.
 */
import { join, } from 'node:path';
import { pathToFileURL, } from 'node:url';

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core';
import {
  expect,
  type Page,
  test,
} from '@playwright/test';

/**
 * Absolute file URL to the bundled paper2vn page. Resolved off
 * `import.meta.dirname` so the test does not depend on the cwd of the
 * Playwright invocation.
 */
const APP_URL = pathToFileURL(
  join(import.meta.dirname, '..', 'dist', 'final', 'index.html',),
)
  .href;

/** localStorage key prefix; mirrors `client/storage-keys.ts`. */
const KEY_PROVIDER = 'p2vn:provider';
const KEY_SETTINGS = 'p2vn:settings';
const KEY_SAVES = 'p2vn:saves';
const KEY_SAVE_PREFIX = 'p2vn:save:';

/** Locale codes supported by the page. Mirrors `types.ts`. */
type Locale = 'en' | 'zh' | 'ja' | 'ru';

/** Provider id type. Mirrors `types.ts`. */
type ProviderId = 'openrouter' | 'openai' | 'anthropic' | 'ollama';

/** Optional seed payload for `seedStorage`. */
type Seed = {
  /** Provider config (full or partial, merged into defaults inside the page). */
  provider?: {
    id?: ProviderId;
    model?: string;
    apiKey?: string;
    baseUrl?: string;
    acknowledgedAnthropicWarning?: boolean;
  };

  /** Settings overlay (locale, font scale, text speed, ...). */
  settings?: {
    locale?: Locale;
    textSpeed?: number;
    voiceEnabled?: boolean;
  };

  /** Optional save payloads. The first becomes the active save when seeded. */
  saves?: {
    id: string;
    label: string;
    paperTitle: string;
    paperText: string;
    chapters: {
      title: string;
      summary: string;
      dialogue: { text: string; pose?: 'neutral' | 'thinking' | 'happy'; }[];
    }[];
    chapterIndex?: number;
    beatIndex?: number;
  }[];
};

/**
 * Seeds the page's localStorage *before* any page module runs.
 *
 * `state.ts` hydrates from localStorage at module init. Setting these
 * via `page.evaluate` after navigation would race with module load;
 * `addInitScript` runs in the page context before any user script,
 * which is the only reliable seeding point.
 *
 * @param page - Playwright page handle
 * @param seed - state to install before navigation
 *
 * @example
 * ```ts
 * await seedStorage({ page, seed: { provider: { apiKey: 'sk-...' }, settings: { locale: 'zh' } } });
 * await page.goto(APP_URL);
 * ```
 */
async function seedStorage(
  { page, seed, }: { page: Page; seed: Seed; },
): Promise<void> {
  /*
   * Storage keys live in `client/storage-keys.ts` but the init script
   * runs in the page context with no module imports, so the keys are
   * passed in through the second arg. Anything captured by closure
   * here would refer to test-process locals that don't exist in the
   * page.
   */
  type InitArg = {
    keyProvider: string;
    keySettings: string;
    keySaves: string;
    keySavePrefix: string;
    seed: Seed;
  };
  await page.addInitScript(
    function install(arg: InitArg,): void {
      function writeJson({ key, value, }: { key: string; value: unknown; },): void {
        globalThis.localStorage.setItem(
          key,
          JSON.stringify(value,),
        );
      }
      const { keyProvider, keySettings, keySaves, keySavePrefix, seed: payload, } = arg;
      if (payload.provider !== undefined)
        writeJson({ key: keyProvider, value: payload.provider, },);
      if (payload.settings !== undefined)
        writeJson({ key: keySettings, value: payload.settings, },);
      if ((payload.saves !== undefined) && (payload.saves.length > 0)) {
        const summaries = payload.saves.map(function toSummary(s,) {
          return {
            id: s.id,
            label: s.label,
            paperTitle: s.paperTitle,
            updatedAt: new Date().toISOString(),
          };
        },);
        writeJson({ key: keySaves, value: summaries, },);
        for (const s of payload.saves) {
          writeJson({
            key: `${keySavePrefix}${s.id}`,
            value: {
              id: s.id,
              label: s.label,
              paperTitle: s.paperTitle,
              paperText: s.paperText,
              chapters: s.chapters,
              chapterIndex: s.chapterIndex ?? 0,
              beatIndex: s.beatIndex ?? 0,
              log: [],
              updatedAt: new Date().toISOString(),
            },
          },);
        }
      }
    },
    {
      keyProvider: KEY_PROVIDER,
      keySettings: KEY_SETTINGS,
      keySaves: KEY_SAVES,
      keySavePrefix: KEY_SAVE_PREFIX,
      seed,
    },
  );
}

//region Tier 1: pure UI tests

test.describe('menu screen', () => {
  test('renders app name and primary buttons', async ({ page, },) => {
    await page.goto(APP_URL,);
    const screen = page.locator('section[data-screen="menu"]',);
    await expect(screen,).toBeVisible();
    await expect(screen.locator('h1',),).toHaveText('paper2vn',);
    await expect(
      screen.locator('button',).filter({ hasText: 'Start', },),
    )
      .toBeVisible();
    await expect(
      screen.locator('button',).filter({ hasText: 'Saves', },),
    )
      .toBeVisible();
    await expect(
      screen.locator('button',).filter({ hasText: 'Settings', },),
    )
      .toBeVisible();
  });

  test('Start navigates to select-topic', async ({ page, },) => {
    await page.goto(APP_URL,);
    await page
      .locator('section[data-screen="menu"] button',)
      .filter({ hasText: 'Start', },)
      .click();
    await expect(page.locator('section[data-screen="select-topic"]',),)
      .toBeVisible();
  });

  test('Settings navigates to settings screen', async ({ page, },) => {
    await page.goto(APP_URL,);
    await page
      .locator('section[data-screen="menu"] button',)
      .filter({ hasText: 'Settings', },)
      .click();
    await expect(page.locator('section[data-screen="settings"]',),)
      .toBeVisible();
  });

  test('Saves navigates to saves screen', async ({ page, },) => {
    await page.goto(APP_URL,);
    await page
      .locator('section[data-screen="menu"] button',)
      .filter({ hasText: 'Saves', },)
      .click();
    await expect(page.locator('section[data-screen="saves"]',),)
      .toBeVisible();
  });
});

test.describe('locale switching', () => {
  test('Chinese locale renders zh menu strings', async ({ page, },) => {
    await seedStorage({ page, seed: { settings: { locale: 'zh', }, }, },);
    await page.goto(APP_URL,);
    const menu = page.locator('section[data-screen="menu"]',);
    await expect(menu,).toBeVisible();
    await expect(
      menu.locator('button',).filter({ hasText: '开始', },),
    )
      .toBeVisible();
    await expect(
      menu.locator('button',).filter({ hasText: '存档', },),
    )
      .toBeVisible();
    await expect(
      menu.locator('button',).filter({ hasText: '设置', },),
    )
      .toBeVisible();
  });

  test('switching language in settings re-renders the screen', async ({ page, },) => {
    await page.goto(APP_URL,);
    await page
      .locator('section[data-screen="menu"] button',)
      .filter({ hasText: 'Settings', },)
      .click();
    const settings = page.locator('section[data-screen="settings"]',);
    await expect(settings,).toBeVisible();

    /*
     * The first <select> on the settings screen is the language picker
     * (markup order is fixed in `settings.ts`). Switching it triggers
     * `navigate('settings')` which re-mounts the screen with the new
     * locale; the back button text flips to Chinese.
     */
    await settings.locator('select',).first().selectOption('zh',);
    await expect(
      page
        .locator('section[data-screen="settings"] button',)
        .filter({ hasText: '返回', },),
    )
      .toBeVisible();
  });
});

test.describe('select-topic gating', () => {
  test('clicking Start lecture without a key shows the missing-key error', async ({ page, },) => {
    await page.goto(APP_URL,);
    await page
      .locator('section[data-screen="menu"] button',)
      .filter({ hasText: 'Start', },)
      .click();
    const screen = page.locator('section[data-screen="select-topic"]',);
    await screen
      .locator('textarea',)
      .fill('Some paper text with content.',);
    await screen
      .locator('button',)
      .filter({ hasText: 'Start lecture', },)
      .click();
    await expect(screen.locator('p.error',),).toContainText('No API key',);
  });

  test('clicking Start lecture with empty input falls back to selectTopicHint', async ({ page, },) => {
    /*
     * With a key configured, the empty-input branch is the only one
     * that surfaces the muted hint as an error; the page short-
     * circuits before any LLM call would happen.
     */
    await seedStorage({
      page,
      seed: {
        provider: {
          id: 'openrouter',
          model: 'anthropic/claude-haiku-4.5',
          apiKey: 'fake-key-for-empty-input-branch',
          baseUrl: '',
          acknowledgedAnthropicWarning: false,
        },
      },
    },);
    await page.goto(APP_URL,);
    await page
      .locator('section[data-screen="menu"] button',)
      .filter({ hasText: 'Start', },)
      .click();
    await page
      .locator('section[data-screen="select-topic"] button',)
      .filter({ hasText: 'Start lecture', },)
      .click();
    await expect(
      page.locator('section[data-screen="select-topic"] p.error',),
    )
      .toContainText('Upload a',);
  });
});

test.describe('settings screen', () => {
  test('Anthropic provider surfaces the browser-direct warning', async ({ page, },) => {
    await page.goto(APP_URL,);
    await page
      .locator('section[data-screen="menu"] button',)
      .filter({ hasText: 'Settings', },)
      .click();
    const settings = page.locator('section[data-screen="settings"]',);

    /*
     * Provider select is the second <select> (language is first). Use
     * it to switch to anthropic and watch the warning paragraph
     * appear.
     */
    await settings.locator('select',).nth(1,).selectOption('anthropic',);
    await expect(
      page.locator('section[data-screen="settings"]',),
    )
      .toContainText('Anthropic blocks browser CORS by default',);
    await expect(
      page.locator('section[data-screen="settings"]',),
    )
      .toContainText('I understand, enable browser-direct mode',);
  });

  test('typed API key persists across navigations', async ({ page, },) => {
    await page.goto(APP_URL,);
    await page
      .locator('section[data-screen="menu"] button',)
      .filter({ hasText: 'Settings', },)
      .click();
    const settings = page.locator('section[data-screen="settings"]',);
    await settings.locator('input[type="password"]',).fill('sk-test-persist',);

    // Round-trip: settings -> menu -> settings -> field still populated
    await settings.locator('button',).filter({ hasText: 'Back', },).click();
    await page
      .locator('section[data-screen="menu"] button',)
      .filter({ hasText: 'Settings', },)
      .click();
    await expect(
      page.locator('section[data-screen="settings"] input[type="password"]',),
    )
      .toHaveValue('sk-test-persist',);
  });
});

test.describe('saves screen', () => {
  test('empty saves shows noSaves copy', async ({ page, },) => {
    await page.goto(APP_URL,);
    await page
      .locator('section[data-screen="menu"] button',)
      .filter({ hasText: 'Saves', },)
      .click();
    await expect(page.locator('section[data-screen="saves"]',),)
      .toContainText('No saves yet',);
  });

  test('seeded save appears in the saves list', async ({ page, },) => {
    await seedStorage({
      page,
      seed: {
        saves: [
          {
            id: 'fixture-save-1',
            label: 'A Tiny Note on Iterative Refinement',
            paperTitle: 'A Tiny Note on Iterative Refinement',
            paperText: 'Abstract. Sample paper text.',
            chapters: [
              {
                title: 'Overview',
                summary: 'Overview of the paper.',
                dialogue: [
                  { text: 'Welcome, Master.', pose: 'happy', },
                  { text: 'Today we discuss iterative refinement.', pose: 'neutral', },
                ],
              },
            ],
          },
        ],
      },
    },);
    await page.goto(APP_URL,);
    await page
      .locator('section[data-screen="menu"] button',)
      .filter({ hasText: 'Saves', },)
      .click();
    await expect(page.locator('section[data-screen="saves"]',),)
      .toContainText('A Tiny Note on Iterative Refinement',);
  });
});

test.describe('lecture screen with seeded save', () => {
  /** Two-chapter fixture used to verify advance/regress and chapter card display. */
  const lectureFixture: Seed = {
    settings: {
      // Max-out the typewriter so reveal lands within Playwright's
      // default expect timeout instead of the 25-char/sec default.
      textSpeed: 240,
    },
    saves: [
      {
        id: 'lecture-fixture',
        label: 'Lecture Fixture Paper',
        paperTitle: 'Lecture Fixture Paper',
        paperText: 'Abstract. The fixture paper for lecture-screen e2e tests.',
        chapters: [
          {
            title: 'Chapter A',
            summary: 'First chapter summary.',
            dialogue: [
              { text: 'First beat of chapter A.', pose: 'neutral', },
              { text: 'Second beat of chapter A.', pose: 'thinking', },
            ],
          },
          {
            title: 'Chapter B',
            summary: 'Second chapter summary.',
            dialogue: [
              { text: 'First beat of chapter B.', pose: 'happy', },
            ],
          },
        ],
      },
    ],
  };

  test('Load button on saves screen takes the user to the lecture', async ({ page, },) => {
    await seedStorage({ page, seed: lectureFixture, },);
    await page.goto(APP_URL,);
    await page
      .locator('section[data-screen="menu"] button',)
      .filter({ hasText: 'Saves', },)
      .click();
    await page
      .locator('section[data-screen="saves"] button',)
      .filter({ hasText: 'Load', },)
      .click();

    const lecture = page.locator('section[data-screen="lecture"]',);
    await expect(lecture,).toBeVisible();
    await expect(lecture.locator('.dialogue-text',),)
      .toContainText('First beat of chapter A',);
    await expect(lecture.locator('.chapter-card h2',),)
      .toHaveText('Chapter A',);
  });

  test('arrow-right advances to the next beat and hides the chapter card', async ({ page, },) => {
    await seedStorage({ page, seed: lectureFixture, },);
    await page.goto(APP_URL,);
    await page
      .locator('section[data-screen="menu"] button',)
      .filter({ hasText: 'Saves', },)
      .click();
    await page
      .locator('section[data-screen="saves"] button',)
      .filter({ hasText: 'Load', },)
      .click();
    const lecture = page.locator('section[data-screen="lecture"]',);

    // Wait for first beat reveal to settle.
    await expect(lecture.locator('.dialogue-text',),)
      .toContainText('First beat of chapter A',);

    // First press cancels typewriter (already done) so triggers advance.
    await page.keyboard.press('ArrowRight',);

    await expect(lecture.locator('.dialogue-text',),)
      .toContainText('Second beat of chapter A',);
    // Chapter card hides for non-first beats.
    await expect(lecture.locator('.chapter-card',),).toBeHidden();
  });

  test('advancing past last beat of chapter A jumps to chapter B card', async ({ page, },) => {
    await seedStorage({ page, seed: lectureFixture, },);
    await page.goto(APP_URL,);
    await page
      .locator('section[data-screen="menu"] button',)
      .filter({ hasText: 'Saves', },)
      .click();
    await page
      .locator('section[data-screen="saves"] button',)
      .filter({ hasText: 'Load', },)
      .click();
    const lecture = page.locator('section[data-screen="lecture"]',);
    await expect(lecture.locator('.dialogue-text',),)
      .toContainText('First beat of chapter A',);

    // First beat -> second beat
    await page.keyboard.press('ArrowRight',);
    await expect(lecture.locator('.dialogue-text',),)
      .toContainText('Second beat of chapter A',);

    // Second beat -> chapter B's first beat (chapter card re-shows)
    await page.keyboard.press('ArrowRight',);
    await expect(lecture.locator('.dialogue-text',),)
      .toContainText('First beat of chapter B',);
    await expect(lecture.locator('.chapter-card h2',),)
      .toHaveText('Chapter B',);
  });

  test('Menu toolbar button returns to the menu screen', async ({ page, },) => {
    await seedStorage({ page, seed: lectureFixture, },);
    await page.goto(APP_URL,);
    await page
      .locator('section[data-screen="menu"] button',)
      .filter({ hasText: 'Saves', },)
      .click();
    await page
      .locator('section[data-screen="saves"] button',)
      .filter({ hasText: 'Load', },)
      .click();
    await expect(page.locator('section[data-screen="lecture"]',),)
      .toBeVisible();

    await page
      .locator('section[data-screen="lecture"] .stage-controls button',)
      .filter({ hasText: 'Menu', },)
      .click();
    await expect(page.locator('section[data-screen="menu"]',),).toBeVisible();
  });
});

//endregion Tier 1

//region Tier 2: live LLM round-trip

/**
 * API key for the live-LLM tier. Pulled from the same env vars as the
 * Bun smoke test (`smoke.ts`) so a single `.env.local` setting drives
 * both. When unset, all tests below skip with a reason rather than
 * fail; CI without a key still runs the rest of the suite.
 */
const OPENROUTER_API_KEY: string | undefined =
  (((process.env.PAPER2VN_OPENROUTER_API_KEY !== undefined)
      && (process.env.PAPER2VN_OPENROUTER_API_KEY !== ''))
    ? process.env.PAPER2VN_OPENROUTER_API_KEY
    : undefined)
    ?? (((process.env.OPENROUTER_API_KEY !== undefined)
        && (process.env.OPENROUTER_API_KEY !== ''))
      ? process.env.OPENROUTER_API_KEY
      : undefined);

/**
 * Live-model order. Matches `smoke.ts` so the e2e and Bun smoke share
 * the fallback policy. Override with `PAPER2VN_OPENROUTER_MODEL` to
 * force a single model for a faster local run.
 */
const LIVE_MODEL = ((process.env.PAPER2VN_OPENROUTER_MODEL !== undefined)
    && (process.env.PAPER2VN_OPENROUTER_MODEL !== ''))
  ? process.env.PAPER2VN_OPENROUTER_MODEL
  : 'anthropic/claude-haiku-4.5';

/** Tiny paper text mirroring `smoke.ts`. */
const LIVE_PAPER_TEXT = `
Title: A Tiny Note on Iterative Refinement

Abstract. We propose a minimal iterative refinement loop in which
each step both samples a candidate and scores it against a
local consistency criterion. The procedure terminates when the
score plateau persists for two successive iterations.

1. Method
We initialize with a random sample from the prior. At each step,
we propose a new candidate by perturbing the current state and
accept it when the consistency score does not regress.

2. Results
On three small synthetic tasks, the procedure converges within
twelve iterations on average.
`
  .trim();

test.describe('live LLM round-trip', () => {
  test('paste paper -> Start lecture -> first dialogue beat appears', async ({ page, },) => {
    test.skip(
      OPENROUTER_API_KEY === undefined,
      'PAPER2VN_OPENROUTER_API_KEY (or OPENROUTER_API_KEY) not set',
    );
    /*
     * Reasoning models can take 40-80s end to end. Anthropic Haiku
     * 4.5 is faster but still pays JSON-mode round-trip latency.
     * 240s gives headroom for the slower upstream cases without
     * making the suite hostile when things are healthy.
     */
    test.setTimeout(240_000,);

    /*
     * Surface page-side errors so a failing LLM round-trip pinpoints
     * itself in test logs instead of looking like a Playwright timeout
     * with no signal.
     */
    page.on(
      'console',
      function onConsole(msg,): void {
        const t = msg.type();
        if ((t === 'error') || (t === 'warning'))
          console.error(`[page:${t}] ${msg.text()}`,);
      },
    );
    page.on(
      'pageerror',
      function onPageError(err,): void {
        console.error(`[pageerror] ${err.message}`,);
      },
    );
    page.on(
      'requestfailed',
      function onRequestFailed(request,): void {
        console.error(
          `[requestfailed] ${request.method()} ${request.url()} ${
            request.failure()?.errorText ?? ''
          }`,
        );
      },
    );

    await seedStorage({
      page,
      seed: {
        provider: {
          id: 'openrouter',
          model: LIVE_MODEL,
          apiKey: nonNullishOrThrow(OPENROUTER_API_KEY,),
          baseUrl: '',
          acknowledgedAnthropicWarning: false,
        },
        settings: {
          textSpeed: 240,
        },
      },
    },);
    await page.goto(APP_URL,);

    await page
      .locator('section[data-screen="menu"] button',)
      .filter({ hasText: 'Start', },)
      .click();

    const screen = page.locator('section[data-screen="select-topic"]',);
    await screen.locator('textarea',).fill(LIVE_PAPER_TEXT,);
    await screen
      .locator('button',)
      .filter({ hasText: 'Start lecture', },)
      .click();

    // Wait for the page to enter the generating state so we know
    // start() ran past the early-return checks before letting the
    // longer LLM round-trip wait kick in.
    await expect(screen.locator('p',).filter({ hasText: 'Generating', },),)
      .toBeVisible({ timeout: 10_000, },);

    /*
     * The page transitions from select-topic -> lecture once
     * `generateChapters` resolves. Wait for the lecture screen rather
     * than the generating-status text; the latter is replaced
     * synchronously when the resolution lands.
     */
    const lecture = page.locator('section[data-screen="lecture"]',);
    await expect(lecture,).toBeVisible({ timeout: 240_000, },);

    const dialogue = lecture.locator('.dialogue-text',);
    await expect(dialogue,).not.toBeEmpty();

    const text = (await dialogue.textContent()) ?? '';
    expect(text.trim().length,).toBeGreaterThan(0,);
  });
});

//endregion Tier 2
