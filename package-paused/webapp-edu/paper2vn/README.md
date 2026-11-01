# paper2vn

Open-source visual-novel-style paper reader.
Upload a paper, chat with a configurable persona who walks you through it chapter by chapter,
and ask questions in-character.

Inspired by paper2galgame; OSS, no backend, no login, no telemetry.

## Status: development paused

Active development is paused pending repo-wide work. `mise run //packages/webapp-edu/paper2vn:lint` currently reports 5
errors from `no-restricted-syntax/no-regex` across `src/client/llm/{anthropic,ollama,openai-compatible}.ts`,
`src/client/dialogue/generator.ts`, and `src/client/parse/pdf.ts`.
The refactor (string-API replacements for the trailing-slash strip and fence-strip,
scoped disables for the PDF normalisers) is deferred. Resume by completing the no-regex sweep documented in
`HANDOVER.no-regex.md`.

## What it does

- Accepts a paper as PDF, TXT, or Markdown (max 30 MB) or pasted text
- Splits the paper into chapters and generates persona-narrated dialogue per chapter via an LLM
- Plays the dialogue in a visual-novel layout with a configurable speaker sprite, advance/back/auto/log/hide controls
- Lets the reader interrupt with free-form questions, answered in-character against the paper
- Persists progress to `localStorage` save slots; settings tweak font size, text speed, voice and BGM volume, language,
auto-advance delay
- Speaks dialogue aloud using the browser's Web Speech API when available

## Architecture

Single self-contained HTML file: the build assembles CSS, the client JavaScript bundle, the placeholder sprite pack,
and i18n strings into one file you can drop on any static host or open from disk.

### Inputs

- `src/client/main.ts`: SPA entry, mounts the screen router
- `src/client/screens/*.ts`: one module per screen (menu, select-topic, lecture, settings, saves)
- `src/client/llm/`: provider abstraction; ships OpenRouter, OpenAI, Anthropic, and Ollama adapters
- `src/client/parse/`: file-to-text extractors (PDF via pdfjs, TXT, MD)
- `src/client/i18n/`: string tables per locale and the persona-prompt translations
- `src/assets/sprites/`: pluggable sprite manifest plus the bundled placeholder pack

### Outputs

- `dist/client/main.js`: bundled client (tsdown, minified, no name mangling)
- `dist/final/index.html`: the self-contained app

## Running

```sh
mise run //packages/webapp-edu/paper2vn:build
```

Open `dist/final/index.html` in a browser.

The app prompts for an LLM provider and API key on first use. Keys live only in `localStorage`;
nothing leaves the browser except calls to the configured provider.

## Tests

Three layers cover the package, each with a separate mise task.

### `test:e2e`: UI-only browser tests (default)

```sh
mise run //packages/webapp-edu/paper2vn:test:e2e
```

Runs `src/paper2vn.e2e.test.ts` in real Chromium and Firefox via Playwright (in podman).
Covers menu rendering and navigation, locale switching, the missing-key gate, the Anthropic browser-direct warning,
save listing, the lecture runtime against a seeded save, and keyboard advance. Around 30 tests;
finishes in roughly 10 seconds.

The live-LLM tier inside the same file uses `test.skip(!env, reason)` so those tests skip silently here;
no API key is threaded into the container.

### `test:e2e:llm`: UI tests + live LLM round-trip

```sh
mise run //packages/webapp-edu/paper2vn:test:e2e:llm
```

Same suite, but the package task forwards `PAPER2VN_OPENROUTER_API_KEY` (and `OPENROUTER_API_KEY` /
`PAPER2VN_OPENROUTER_MODEL`) into the podman container so the gated tests run.
Mise auto-injects `PAPER2VN_OPENROUTER_API_KEY` from `.env.local`.

The live tier drives the full flow: paste paper into the textarea, click Start lecture, wait for the lecture screen,
assert the first dialogue beat is non-empty. Defaults to `anthropic/claude-haiku-4.5` (fast, ~6 s);
override with `PAPER2VN_OPENROUTER_MODEL=<slug>` to point at a different OpenRouter model.

### `smoke`: Bun-only LLM round-trip

```sh
PAPER2VN_OPENROUTER_API_KEY=sk-or-... mise run //packages/webapp-edu/paper2vn:smoke
```

Imports the page's own `dialogue/generator.ts` from Bun and calls OpenRouter directly. Useful as a faster,
browser-free check that the prompt + parser still produce valid chapters end-to-end.
Tries `moonshotai/kimi-k2.6` first, falls back to `anthropic/claude-haiku-4.5`; expect 40-80 s for Kimi.
Override the model list with `PAPER2VN_OPENROUTER_MODEL=<slug>` and the paper text with `PAPER2VN_SMOKE_PAPER=<text>`.
Fails on missing key, all-models-failed, zero chapters, or empty first beat.

### Why both Bun and browser

The Bun smoke validates the prompt-to-parsed-chapters pipeline without the cost of spinning up a headless browser,
which makes it a good pre-commit check while developing prompts.
The Playwright e2e suite is the only thing that actually verifies the page works end-to-end in the runtime users
target; the prompt-build step inside the browser hit a
[`typesafe-i18n` regex bug](../../../doc/troubleshooting/typesafe-i18n-regex-redos.md) that didn't reproduce under Bun,
and the kind of bug that only shows up in the browser is the kind only the browser tests will catch.

### LLM providers and CORS

Browsers enforce CORS on cross-origin requests. Each supported provider behaves differently:

- **OpenRouter**: full CORS support, no extra setup, recommended default
- **OpenAI**: CORS-permissive on `/v1/chat/completions`, works direct from the browser
- **Anthropic**: requires the `anthropic-dangerous-direct-browser-access: true` request header
and the `dangerouslyAllowBrowser` SDK flag; the app surfaces this as an explicit warning in settings
- **Ollama (local)**: requires `OLLAMA_ORIGINS=*` (or the page's origin) to be set when starting `ollama serve`,
otherwise CORS blocks the request

## Sprite packs

Sprites are loaded from a JSON manifest:

```json
{
  "name": "Placeholder",
  "license": "CC0",
  "characters": {
    "ruka": {
      "displayName": "Ruka",
      "poses": {
        "neutral": "data:image/svg+xml;base64,...",
        "thinking": "data:image/svg+xml;base64,...",
        "happy": "data:image/svg+xml;base64,..."
      }
    }
  }
}
```

The bundled pack is a CSS-art SVG placeholder. To swap in a licensed pack,
drop a `manifest.json`-conformant pack and the build script picks it up via the `PAPER2VN_SPRITE_PACK` env var.

## Limitations

- DOCX is not supported in MVP
- Web Speech voices vary by browser/OS; fluency in non-English languages depends on installed system voices
- LLM-generated dialogue is non-deterministic and may misrepresent the paper.
The Ask flow grounds answers in the parsed text but is not a substitute for reading the original

## License

LGPL-3.0-or-later (code).
The bundled sprite pack uses its own license: see `src/assets/sprites/manifest.json`.
