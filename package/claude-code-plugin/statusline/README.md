# statusline

Claude Code status line package showing model name,
 effort level,
 context window token usage,
 API rate limit warnings,
 and a context-aware activity word.

## What it displays

Model info appears first,
followed by a fixed-width token counter,
rate limit indicators when approaching limits,
and an activity word extracted from conversation context.

**Normal usage** (rate limits comfortable):

```text
Opus     51,045/1,000,000    Searching
```

**Approaching a limit** (session window at 72% used,
 28% remaining):

```text
Opus     51,045/1,000,000    28% left (1h23m)    Refactoring
```

**Both tiers constrained**:

```text
Opus     51,045/1,000,000    28% left (1h23m) · 12% left (3d2h)    Compiling
```

**Projected to exceed** (60% used in the first 2h of a 5h window;
 remaining looks fine but burn rate extrapolates to 150%):

```text
Opus     51,045/1,000,000    40% left →150% (3h)    Refactoring
```

The `→150%` marker shows the extrapolated end-of-window usage.
 The segment renders in red whenever projection exceeds 100%,
 even if remaining capacity is above the normal 50% threshold.

The token counter is always 7 characters wide (`TTT,OOO` format) so the display never shifts.

## Activity word

Replaces Claude Code's built-in spinner verbs,
 which pick random words from a flat list
with no relation to what Claude is actually doing
([anthropics/claude-code#33057](https://github.com/anthropics/claude-code/issues/33057)).

The statusline extracts a gerund from the newest assistant prose and thinking summaries
at the end of the session transcript instead.
Since Claude's prose naturally contains gerunds that describe its current activity
("Let me search for...",
 "I'll try refactoring..."),
the displayed word reflects what Claude was most recently talking about.

### How it works

1. The statusline JSON payload includes `transcript_path`
2. `openAsBlob` reads only the final 64KB of the transcript (no full-file load)
3. The tail is split into lines;
    the first line is dropped when the read began mid-file,
    because it is cut off
4. Lines are visited newest first;
    only lines containing `"type":"assistant"` are passed to `JSON.parse`,
    so only a handful of entries are ever parsed
5. Only `text` and `thinking` content blocks of assistant entries are scanned
    (the thinking signature is not);
    tool calls,
    tool results,
    attachments,
    and user entries are ignored
6. A single-pass scanner reports every word that ends in `-ing` between word boundaries,
    including hyphenated compounds such as `type-checking`
7. A noise filter removes non-activity words (pronouns,
    prepositions,
    adjectives,
    phase-implying verbs)
8. The last surviving match of the newest entry that has one is capitalized and displayed
9. When no gerund is found,
    the word from an earlier render of the same session is shown instead;
    see Memory below
10. Falls back to "Thinking" when the session has never produced a gerund or the transcript is unavailable

### Memory

The statusline command is a new process on every render,
 so the last word is kept on disk:
 `~/.claude/statusline-activity/<session id>.txt`,
 where the session id is the transcript file name without `.jsonl`.
A freshly extracted word replaces the stored one,
 and the file is only written when the word changed.
A stored file that is not exactly one word the extractor could produce is ignored,
 and an unwritable directory never blocks rendering.
Files are tiny and are not pruned.

### Noise filter categories

- **Phase-implying**:
   "beginning",
   "completing",
   "finishing",
   "stopping";
  sound wrong at arbitrary points in processing
- **Too generic**:
   "asking",
   "doing",
   "getting",
   "making",
   "working";
  uninformative as status words
- **Pronouns/determiners**:
   "something",
   "nothing",
   "anything",
   "everything"
- **Prepositions/conjunctions**:
   "according",
   "during",
   "including",
   "regarding"
- **Adjectives**:
   "interesting",
   "existing",
   "surprising",
   "confusing"
- **Not gerunds**:
   "string",
   "king",
   "ring",
   "spring";
   root contains "-ing"
- **Common filler verbs**:
   "being",
   "needing",
   "using",
   "thinking"

### Required settings

Disable the built-in spinner verbs and tips so they don't compete with the statusline word:

```json
{
  "spinnerVerbs": { "mode": "replace", "verbs": [] },
  "spinnerTipsEnabled": false
}
```

### Limitations

- Only the final 64KB is read.
  When the newest assistant text sits behind more than that
   (for example a very large tool result),
   it is not seen and the remembered word is shown.
- The word reflects the most recent gerund in assistant prose and thinking summaries,
  which may lag behind the current activity by one or more tool calls.
- At the start of a session (empty transcript),
   the fallback "Thinking" is shown.

## Model name

The model family name (Opus,
 Sonnet,
 Haiku) is shown after the activity word.
Version and context size are stripped when they match the current defaults:

- **Opus**:
   latest 4.8,
   default 1M context
- **Sonnet**:
   latest 4.6,
   default 200K context
- **Haiku**:
   latest 4.5,
   default 200K context

Non-default values are kept:
 Sonnet with 1M context shows `Sonnet (1M)`,
an older model shows `Opus 4.5`.

## Effort level

When the effort level is below "high" (the default),
 a yellow symbol appears after the model name:

- **low**:
   `Opus ○`
- **medium**:
   `Opus ◐`
- **high**:
   no indicator (default,
   nothing extra shown)
- **max**:
   `Opus ◉`

The symbols match Claude Code's built-in effort indicators.

The effort level is read from `~/.claude/settings.json` (`effortLevel` field)
because the statusline JSON payload does not include it yet
([anthropics/claude-code#40261](https://github.com/anthropics/claude-code/issues/40261)).
Changes made via `/effort` or `/fast` that write to settings are picked up on the next statusline refresh.
In-session toggles that skip the settings file will not be reflected.

## Rate limit indicators

Rate limit warnings use a "remaining + time-to-reset" framing.
Instead of "72% used,
 resets at 4pm,
" the status line shows "28% left (1h23m).
"
This answers the natural question:
 "How much can I do before it refills?
"

Indicators appear when either condition holds:

- Remaining capacity drops to 50% or below,
   **or**
- Current burn rate,
   extrapolated to the window reset,
   exceeds 100%.

When everything is comfortable and burn rate is sustainable,
 nothing extra is shown;
 no news is good news.

### Projection

Each tier's window has a fixed duration (5 hours for `five_hour`,
 7 days for `seven_day`).
Elapsed time is recovered from `windowSize - (resets_at - now)`,
burn rate is `used_percentage / elapsed`,
and the projected end-of-window usage is `burn_rate * windowSize`.

When projection exceeds 100%,
 the segment renders in red and appends a `→Z%` marker
showing the extrapolated total (e.g.,
 `40% left →150% (3h)`).
Projection is suppressed until usage reaches 5%,
 since extrapolating from a near-zero
sample at the start of a window produces unstable estimates.

Two tiers are tracked from the statusline JSON payload:

- **Session** (`rate_limits.five_hour`):
   5-hour rolling window
- **Week** (`rate_limits.seven_day`):
   7-day rolling window

If both tiers are constrained,
 both appear separated by a centered dot.
If only one is constrained,
 only that one is shown.
Data is only available for Pro/Max subscribers after the first API response in a session.

## Color thresholds

**Context window** (token counter):

- **Default** (no color):
   under 100,000 tokens
- **Yellow**:
   100,000 or more
- **Pink/magenta**:
   200,000 or more
- **White**:
   900,000 or more (near context limit)

**Rate limit remaining**:

- **Green**:
   more than 25% remaining
- **Yellow**:
   10-25% remaining
- **Red**:
   10% or less remaining,
   **or** projection exceeds 100% at any remaining level

## Token calculation

Sums all token types from the current API state:
`input_tokens + cache_creation_input_tokens + cache_read_input_tokens + output_tokens`
from `context_window.current_usage`.
This matches the built-in token counter (`SBH` in Claude Code's source),
 which includes output tokens.
Note that `used_percentage` excludes output tokens,
 so this count is slightly higher.

The count may briefly differ from the built-in counter during active tool use or subagent work.
The built-in reads usage from the last message in the conversation,
while `context_window.current_usage` comes from the statusline JSON payload.
These data sources can reference different API calls until the next assistant response settles them.

Claude Code debounces statusline script invocations at 300ms.
This is built into Claude Code itself and cannot be configured from the script side.

## Runtime

The package builds a Node CLI at `dist/final/node/statusline.mjs`.
Runtime code uses Node standard-library APIs only and has no Bun dependency.

## Installation

Build the package with `mise run //package/claude-code-plugin/statusline:build`,
then add the built command to `~/.claude/settings.json`:

```json
{
  "statusLine": {
    "type": "command",
    "command": "node /path/to/packages/claude-code-plugin/statusline/dist/final/node/statusline.mjs"
  },
  "spinnerVerbs": { "mode": "replace", "verbs": [""] },
  "spinnerTipsEnabled": false
}
```

The package also exposes a private workspace bin named `claude-code-statusline`.

### Why this is not a Claude Code plugin

Claude Code plugins cannot contribute a `statusLine`.
The plugins-reference "File locations" table states that a plugin's bundled `settings.json`
supports only the `agent` and `subagentStatusLine` keys;
`statusLine` is user-scope only and must live in `~/.claude/settings.json`.
`subagentStatusLine` is a separate feature (it formats rows in the subagent panel)
and is not a substitute for the main status line.
That is why this directory has no `.claude-plugin/plugin.json`
and is not listed in the repo's `marketplace.json`.
