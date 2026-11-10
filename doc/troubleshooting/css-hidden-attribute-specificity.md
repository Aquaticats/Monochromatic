# Browser UA `[hidden]` rule loses to author rules of equal specificity placed later in source order

## Symptom

An element whose `hidden` HTML attribute is set keeps painting on
screen.
 The DOM reports the attribute correctly,
 JS that introspects
the element confirms the state,
 and yet the box is still visible:

```js
el.hidden = true;
console.log(el.hidden,); // true
console.log(getComputedStyle(el,).display,); // "grid"  (expected: "none")
```

Discovered in paper2vn's chapter-card overlay
(`.chapter-card { display: grid }`):
 a beat advance set
`chapterCard.hidden = true` and the overlay stayed visible until a
later beat that swapped the display state again.

## Root cause

The HTML `hidden` attribute has no intrinsic styling priority.
 Every
modern browser exposes it through a user-agent stylesheet rule:

```css
[hidden] {
  display: none;
}
```

The attribute selector `[hidden]` has specificity `(0, 1, 0)`.
 So does
any class selector such as `.chapter-card`.
 Under CSS cascade rules,
when origin and specificity tie,
 the rule that appears later in source
order wins.

The user agent stylesheet is the earliest origin layer (well before
author styles),
 but inside the author origin the UA rule no longer
participates.
 Most projects mirror the UA rule inside their reset
stylesheet (so it survives a CSS framework's reset),
 and place the
reset near the top of the build output.
 Any layout rule loaded after
the reset with equal specificity beats it.
 The `hidden` attribute
stays `true`,
 the DOM reflects the state,
 but the rule that sets
`display: grid` is the cascade winner.

## Verification

Version under test:

- Firefox ESR 140 (June 2025;
   project's baseline browser,
   see
  `PHILOSOPHY.browser-support.md`)
- Chromium 126+
- Safari 18+

All three browsers exhibit the same cascade behaviour because the
specificity tie-breaker is defined by the CSS Cascade Level 5 spec,
 not
the engine.

Minimal reproduction:

```html
<style>
[hidden] {
  display: none;
}
.card {
  display: grid;
}
</style>
<div
  class='card'
  hidden>
  still visible
</div>
```

`getComputedStyle(document.querySelector('.card')).display` returns
`"grid"`.
 Swap the rules so `.card` comes first and `[hidden]` after,
and the same query returns `"none"`.

## Verified workarounds

### Place `[hidden]` after layout rules in source order

In paper2vn's `package/webapp-edu/paper2vn/src/styles.ts`,
 the
`[hidden]` rule lives at the very end of the stylesheet:

```ts
$({
  rule: '.chapter-card',
  decls: { display: 'grid', /* ... */ },
}),
// ... all other layout rules ...
$({
  rule: '[hidden]',
  decls: { display: 'none' },
}),
```

Tradeoff:
 every future contributor must continue to place new layout
rules **above** the `[hidden]` block.
 The constraint is one line of
convention;
 a stylesheet linter cannot generally enforce "this rule
must be last" without project-specific configuration.
 Acceptable
because the stylesheet is a single file with a clear top-down flow.

### `@layer reset { [hidden] { display: none } }` (alternative)

Wrapping the reset in a CSS layer makes it always lose to **unlayered**
rules,
 the opposite of what is needed.
 The correct application is to
put **layout rules** in a layer and leave `[hidden]` unlayered:

```css
@layer layout {
  .card {
    display: grid;
  }
}
[hidden] {
  display: none;
}
```

Tradeoff:
 every layout rule in the codebase needs to move into the
layer;
 mixed authoring (some layered,
 some unlayered) re-introduces
the source-order trap.
 The codebase does not currently use `@layer`
elsewhere,
 so adopting it for this single concern would be
inconsistent.
 Worth revisiting when the project adopts layers more
broadly.

## What does not work

- `[hidden] { display: none !important }`:
   project conventions ban
  `!important` (see `AGENTS.md` "CSS" section).
   Importance correctly
  overrides specificity,
   but the maintenance debt of allowing one
  `!important` opens the door to more.
- Increasing the selector specificity (`html [hidden]`,
   `body[data-x]
  [hidden]`):
   defeats the symmetry the UA stylesheet relies on;
   every
  future author rule has to track the inflated specificity to stay
  out of the way.
- Setting `display: none` from JS (`el.style.display = 'none'`):
  works because inline styles always beat author rules,
   but couples
  the layout decision to the JS code path that flips `hidden`.
   The
  attribute alone should be the single source of truth.

## Why we do not file this upstream

The behaviour is a documented feature of the CSS cascade,
 not a
browser defect.
 Walking the 5 constraints:

1.  **Is it really upstream's fault?
    ** No. CSS Cascade Level 5
    specifies source-order tie-breaking;
     every engine implements it
    identically.
2.  **Can upstream fix it?
    ** No. The specifier behaviour is part of the
    spec;
     changing it would break every existing site.
3.  **Are they supporting this use case?
    ** Yes.
     `[hidden]` is a
    standard attribute and the UA rule is correctly applied;
     user
    stylesheets override it by spec.
4.  **Will they likely fix it?
    ** N/A.
5.  **Have we prototyped a minimal fix?
    ** N/A.
     The fix is at our
    boundary (source order in `styles.ts`).

Decision:
 no upstream report.
 The cascade rule is correct;
 the
remediation is on our side.
