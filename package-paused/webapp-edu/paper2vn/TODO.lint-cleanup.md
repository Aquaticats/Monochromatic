# TODO: lint cleanup

`mise run //packages/webapp-edu/paper2vn:lint:oxlint` reports 50 errors and 163 warnings as of 2026-05-13.
The build, the type check, and the seeded-save lecture flow all pass.

## Resolved in this round

- 22 `typescript-eslint(no-unsafe-type-assertion)` errors;
  refactored `screens/settings.ts` to capture typed element refs
  and `addEventListener` post-creation;
  added `coerceLocale`/`coerceProviderId` helpers;
  changed `lecture.ts` timer type to `ReturnType<typeof setTimeout>`.
  The remaining JSON-parse / pdfjs casts have `oxlint-disable-next-line` directives
  with justification per AGENTS.md.

## Remaining counts

```text
50 errors, including module-root `let`, function-root `let`, `avoid-new`,
missing destructured params, missing querySelector generics, no-promise-catch,
non-null assertions, max-lines, new-cap, init-declarations, and no-magic-numbers.
163 warnings, mostly TSDoc and stylistic cleanup.
```

## Categories

### tsdoc(require-example, require-param, require-returns, require-tsdoc)

Add missing `@example`, `@param`, `@returns` tags and TSDoc blocks.
The screen modules and `dom.ts` have most of the gaps.
AGENTS.md mandates these on every declaration.

### eslint(no-magic-numbers)

Lift literals to named constants. Hot spots:

- `state.ts`: `textSpeed: 40`, `voiceVolume/bgmVolume: 0.3`, `autoAdvanceDelayMs: 1600`, font scale range bounds
- `select-topic.ts`: random-id seed `1e6`
- `parse/index.ts`: `30 * 1024 * 1024` byte cap
- `dialogue/generator.ts`: `60_000` paper text budget
- `tts.ts`: voice-pick fallbacks
- `settings.ts`: range bounds (`0.75`, `1.5`, `0.05`, `120`, `5000`, `100`, `5`)

### eslint(new-cap)

`LL()`, `i18nObject(...)`, `initI18n(...)` factory calls flagged
because their names start with uppercase.
Either rename in the wrapper or configure `new-cap`
to allow these specific identifiers.

### eslint(init-declarations)

`let foo;` declarations in `state.ts` and `lecture.ts` need explicit initializers (typically `undefined`).

### eslint(max-lines)

`screens/lecture.ts` and `screens/settings.ts` exceed the line limit. Split into sub-modules:

- `lecture.ts` -> `lecture/runtime.ts` (typewriter + auto), `lecture/toolbar.ts`, `lecture/ask-panel.ts`
- `settings.ts` -> `settings/display.ts`, `settings/provider.ts`

### structural errors added since the first cleanup pass

- `state.ts` has module-root `let` state that must move into containers or a named helper shape.
- `lecture.ts` has `avoid-new`, function-root `let`, querySelector generic, and no-promise-catch violations.
- Some event handlers still need single destructured object parameters.
- Remaining non-null assertions need runtime narrowing helpers.

### stylistic (`object-property-per-line`, `type-property-per-line`, `tuple-per-line`)

Mostly auto-fixable formatting; `dprint` should handle this once configured for the package.

### Remaining tsdoc/structural issues

- `multiline-blocks`: some `/** */` comments have layout issues (3)
- `valid-types`: TSDoc does not allow TypeScript type syntax in tags (2)
- `numeric-separators-style`: `_` separators on numbers >= 5 digits (6)

## Suggested order

1. Split `lecture.ts` and `settings.ts` to clear `max-lines` first; smaller files lint faster on iteration.
2. Lift magic numbers to named constants: knocks out `no-magic-numbers` and most `numeric-separators-style`.
3. Add missing TSDoc: volume work, but mechanical.
4. Configure `new-cap` allowlist for typesafe-i18n factories.
5. Fix `init-declarations` and stylistic remainder.

## Verification expected after cleanup

- `mise run lint:types` -- already green
- `mise run lint:oxlint` -- 0 errors, 0 warnings
- `mise run build` -- already green; `dist/final/index.html` mounts in a browser
