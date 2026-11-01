# Hyperscript stable syntax deepening plan

## Purpose

This plan records the accepted design for deepening `package/hyperscript/core`.
Do not implement it while reading this document.
Use it as the source of truth for a future implementation pass.

The audience is a very smart forest monkey:
it understands hard problems,
but it has not lived inside this repository.
So this document explains the banana rules in plain language,
then gives exact module rules,
examples,
tests,
and migration notes.

## Source decision

Read this first:
`doc/decision/stable-syntax-modules.md`.

The key rule is:
shared deep Modules encode long-lived syntax and grammar policy.
They do not encode transient domain or tool policy.

That means:

- Deepen HTML,
  DOM,
  XML,
  and CSS syntax Modules.
- Do not create shared libvirt XML Modules.
- Do not create shared app page-shell Modules.
- Do not create shared app icon-set Modules.
- Keep domain and tool policy beside the caller that owns it.

Examples:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
// Good shared concern:
// XML escaping and valid XML names live in hXml.
```

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
// Bad shared concern:
// Libvirt domain defaults live in module-hyperscript.
```

XML may matter in 10 years.
Libvirt may not.
So XML syntax belongs in the shared Module.
Libvirt policy does not.

## Current package map

Current public Modules live under:

- `package/hyperscript/core/src/html/index.ts`
- `package/hyperscript/core/src/dom/index.ts`
- `package/hyperscript/core/src/xml/index.ts`
- `package/hyperscript/core/src/css/index.ts`
- `package/hyperscript/core/src/index.ts`

Current tests live under:

- `package/hyperscript/core/src/html/index.unit.test.ts`
- `package/hyperscript/core/src/xml/index.unit.test.ts`
- `package/hyperscript/core/src/css/index.unit.test.ts`

There is currently no `hDom` test file in `package/hyperscript/core`.
Future implementation should add one.

## Plain-language model

A plain `string` is loose banana mush.
It might be safe text.
It might be already-rendered HTML.
It might be attacker-controlled markup.
The type system cannot tell.

A branded string is a banana with a sticker.
The sticker says:
this string already crossed the syntax Seam.
Do not treat any random string as if it has that sticker.

For HTML:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
type HtmlFragment = string & HtmlBrand;
```

For XML:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
type XmlFragment = string & XmlBrand;
```

A helper named `unsafeHtml` is a danger door.
It does not make unsafe input safe.
It only says the caller intentionally bypassed escaping.

A helper named `unsafeXml` is the same danger door for XML.

There is no `unsafeCss` in this plan.
The CSS field is already named `raw`,
which is enough warning for now.

## Non-goals

Do not create these shared Modules:

- `renderHtmlShell`
- `renderPage`
- `libvirtXml`
- `domainXml`
- `closeIcon`
- `backIcon`
- app-specific icon factories

Those may exist locally inside app or tool packages.
They should not be promoted into `module-hyperscript` only because repeated markup exists.

Do not change the README rule that children carry output form without recording the change.
This plan preserves that spirit:
children carry rendered output,
not nested option objects.

## Cross-package documentation update

The current `package/hyperscript/core/README.md` says each factory module has no sibling cross-dependencies.
Future implementation will likely need stable shared syntax primitives.
For example,
`hDom` should accept `HtmlFragment` for raw `innerHTML`.
That means `hDom` needs the HTML fragment type.

Update the README when implementing.
The new rule should be:

- generic syntax factories may share stable syntax primitives,
  such as `HtmlFragment`;
- generic syntax factories must not import app,
  domain,
  or tool policy;
- no libvirt,
  page-shell,
  or app-icon policy belongs in `module-hyperscript`.

## File-splitting expectation

Do not cram everything into current `index.ts` files.
The repository has max-lines rules.
Split before fighting the linter.

Suggested split for HTML:

- `src/html/index.ts`
- `src/html/types.ts`
- `src/html/fragments.ts`
- `src/html/validate.ts`
- `src/html/escape.ts`
- `src/html/void-elements.ts`

Suggested split for DOM:

- `src/dom/index.ts`
- `src/dom/types.ts`
- `src/dom/validate.ts`

Suggested split for XML:

- `src/xml/index.ts`
- `src/xml/types.ts`
- `src/xml/fragments.ts`
- `src/xml/validate.ts`
- `src/xml/escape.ts`

Suggested split for CSS:

- keep current CSS split;
- move disallowed at-rule runtime constants near `properties.ts`,
  or into a small sibling file;
- keep type tests separate from runtime tests if that reads better.

## HTML accepted Interface

### Brand

`hHtml` returns `HtmlFragment`,
not plain `string`.

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
declare const htmlFragmentBrand: unique symbol;

export type HtmlFragment = string & {
  readonly [htmlFragmentBrand]: true;
};
```

The brand symbol should stay private.
The type should be exported.

Good:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
const child = hHtml({
  tag: 'strong',
  text: 'banana',
});

hHtml({
  tag: 'p',
  children: [child],
});
```

Output:

```html
<!-- doc/todo/hyperscript-stable-syntax-deepening.md -->
<p><strong>banana</strong></p>
```

Bad:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
const loose: string = '<strong>banana</strong>';

hHtml({
  tag: 'p',
  children: [loose],
});
```

Expected result:

```text
Type error: string is not HtmlFragment.
```

### Raw HTML escape hatch

Add `unsafeHtml(raw: string): HtmlFragment`.

Good,
but dangerous:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hHtml({
  tag: 'article',
  html: unsafeHtml('<p>Already rendered markdown.</p>'),
});
```

Output:

```html
<!-- doc/todo/hyperscript-stable-syntax-deepening.md -->
<article><p>Already rendered markdown.</p></article>
```

Bad:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hHtml({
  tag: 'article',
  html: '<p>Already rendered markdown.</p>',
});
```

Expected result:

```text
Type error: string is not HtmlFragment.
```

`unsafeHtml` must have scary TSDoc.
It should say that it does no escaping,
no validation,
and no sanitization.

### No `htmlText` helper

Do not add `htmlText`.

Use `tag: ''` with `text` for text fragments.

Good:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hHtml({
  tag: '',
  text: '<b>banana</b>',
});
```

Output:

```html
<!-- doc/todo/hyperscript-stable-syntax-deepening.md -->
&lt;b&gt;banana&lt;/b&gt;
```

Mixed inline content:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hHtml({
  tag: 'p',
  children: [
    hHtml({
      tag: '',
      text: 'Hello ',
    }),
    hHtml({
      tag: 'strong',
      text: 'banana',
    }),
    hHtml({
      tag: '',
      text: ' world.',
    }),
  ],
});
```

Output:

```html
<!-- doc/todo/hyperscript-stable-syntax-deepening.md -->
<p>Hello <strong>banana</strong> world.</p>
```

### Empty tag mode

`tag: ''` is text-fragment mode.
Only `text` is valid.

Good:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hHtml({
  tag: '',
  text: 'banana',
});
```

Output:

```html
<!-- doc/todo/hyperscript-stable-syntax-deepening.md -->
banana
```

Bad:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hHtml({
  tag: '',
  children: [hHtml({ tag: 'strong', text: 'banana' })],
});
```

Expected result:

```text
Type error: tag "" allows only text.
```

Bad:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hHtml({
  tag: '',
  attrs: { id: 'banana' },
  text: 'banana',
});
```

Expected result:

```text
Type error: tag "" allows only text.
```

Runtime should also reject invalid empty-tag shapes if TypeScript is bypassed.

### Mutually exclusive content modes

Normal non-empty elements may use exactly one content mode:

- no content;
- `text`;
- `html`;
- `children`.

Good text mode:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hHtml({
  tag: 'p',
  text: 'banana',
});
```

Output:

```html
<!-- doc/todo/hyperscript-stable-syntax-deepening.md -->
<p>banana</p>
```

Good HTML mode:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hHtml({
  tag: 'p',
  html: unsafeHtml('<strong>banana</strong>'),
});
```

Output:

```html
<!-- doc/todo/hyperscript-stable-syntax-deepening.md -->
<p><strong>banana</strong></p>
```

Good children mode:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hHtml({
  tag: 'p',
  children: [
    hHtml({ tag: '', text: 'Hello ' }),
    hHtml({ tag: 'strong', text: 'banana' }),
  ],
});
```

Output:

```html
<!-- doc/todo/hyperscript-stable-syntax-deepening.md -->
<p>Hello <strong>banana</strong></p>
```

Bad mixed mode:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hHtml({
  tag: 'p',
  text: 'Hello ',
  children: [hHtml({ tag: 'strong', text: 'banana' })],
});
```

Expected result:

```text
Type error: choose text or children, not both.
```

Bad mixed mode:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hHtml({
  tag: 'div',
  text: 'A',
  html: unsafeHtml('<span>B</span>'),
});
```

Expected result:

```text
Type error: choose text or html, not both.
```

Runtime should also reject multiple content modes if TypeScript is bypassed.

### Void element content

HTML void elements cannot have content.

Literal void tags should be type errors when content is present.
Dynamic tags should throw at runtime if they become void tags with content.

Good:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hHtml({
  tag: 'br',
});
```

Output:

```html
<!-- doc/todo/hyperscript-stable-syntax-deepening.md -->
<br>
```

Bad literal:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hHtml({
  tag: 'br',
  text: 'banana',
});
```

Expected result:

```text
Type error: void HTML element cannot have content.
```

Bad dynamic:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
const tag: string = getTag();

hHtml({
  tag,
  text: 'banana',
});
```

If `tag` is `br`,
runtime throws:

```text
Void HTML element cannot have content: br
```

Keep the existing `VOID_ELEMENTS` export for current callers.
Add whatever const tuple or type union is needed to make literal void-tag checks possible.

### Tag name validation

Validate HTML tag names at runtime.
This protects dynamic names.

Use a small safe grammar,
not full browser parsing.
Recommended grammar:

- empty string is allowed only for text-fragment mode;
- non-empty tag starts with a lowercase ASCII letter;
- remaining characters are lowercase ASCII letters,
  digits,
  or hyphen;
- no spaces;
- no quotes;
- no equals signs;
- no slashes;
- no angle brackets;
- no control characters.

Good:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hHtml({
  tag: 'top-nav',
  attrs: { heading: 'Inbox' },
});
```

Output:

```html
<!-- doc/todo/hyperscript-stable-syntax-deepening.md -->
<top-nav heading="Inbox"></top-nav>
```

Bad:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hHtml({
  tag: 'div onclick="alert(1)"',
  text: 'banana',
});
```

Runtime throws:

```text
Invalid HTML tag name: div onclick="alert(1)"
```

Do not use a regular expression unless the linter-required regex justification is written.
A simple character scan is clearer and avoids regex policy friction.

### Attribute name validation

Validate HTML attribute names at runtime.
Attribute values are already escaped.
Attribute names must also be grammar-checked.

Recommended grammar:

- starts with a lowercase ASCII letter;
- remaining characters are lowercase ASCII letters,
  digits,
  or hyphen;
- allows `data-kind`;
- allows `aria-label`;
- no spaces;
- no quotes;
- no equals signs;
- no slashes;
- no angle brackets;
- no control characters.

Good:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hHtml({
  tag: 'input',
  attrs: {
    type: 'checkbox',
    'aria-label': 'Pick banana',
    'data-kind': 'ripe',
  },
});
```

Output:

```html
<!-- doc/todo/hyperscript-stable-syntax-deepening.md -->
<input type="checkbox" aria-label="Pick banana" data-kind="ripe">
```

Bad:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hHtml({
  tag: 'div',
  attrs: {
    'onclick="alert(1)"': 'banana',
  },
});
```

Runtime throws:

```text
Invalid HTML attribute name: onclick="alert(1)"
```

### Attribute values

HTML attribute values are `string | true`.

Good string attr:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hHtml({
  tag: 'input',
  attrs: {
    type: 'checkbox',
  },
});
```

Output:

```html
<!-- doc/todo/hyperscript-stable-syntax-deepening.md -->
<input type="checkbox">
```

Good true attr:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hHtml({
  tag: 'input',
  attrs: {
    type: 'checkbox',
    disabled: true,
  },
});
```

Output:

```html
<!-- doc/todo/hyperscript-stable-syntax-deepening.md -->
<input type="checkbox" disabled>
```

Bad false attr:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hHtml({
  tag: 'input',
  attrs: {
    disabled: false,
  },
});
```

Expected result:

```text
Type error: false is not assignable to string | true.
```

Runtime must also reject `false` if TypeScript is bypassed.

Conditional attr pattern:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hHtml({
  tag: 'input',
  attrs: {
    type: 'checkbox',
    ...(isDisabled ? { disabled: true } : {}),
  },
});
```

No numbers:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hHtml({
  tag: 'meter',
  attrs: {
    value: 3,
  },
});
```

Expected result:

```text
Type error: HTML attribute values are serialized text, not numbers.
```

Caller chooses the string form:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
const value = 3;

hHtml({
  tag: 'meter',
  attrs: {
    value: String(value),
  },
});
```

### No top-level `class`

Remove top-level `class` from `hHtml`.
Use `attrs.class`.

Good:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hHtml({
  tag: 'div',
  attrs: {
    class: 'card primary',
  },
  text: 'banana',
});
```

Output:

```html
<!-- doc/todo/hyperscript-stable-syntax-deepening.md -->
<div class="card primary">banana</div>
```

Bad:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hHtml({
  tag: 'div',
  class: 'card primary',
  text: 'banana',
});
```

Expected result:

```text
Type error: class is not an hHtml option.
```

### No top-level `style`

Remove top-level `style` from `hHtml`.
Use `attrs.style`.

Good:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hHtml({
  tag: 'div',
  attrs: {
    style: 'background-color:yellow;flex-direction:row',
  },
  text: 'banana',
});
```

Output:

```html
<!-- doc/todo/hyperscript-stable-syntax-deepening.md -->
<div style="background-color:yellow;flex-direction:row">banana</div>
```

Bad:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hHtml({
  tag: 'div',
  style: {
    backgroundColor: 'yellow',
  },
});
```

Expected result:

```text
Type error: style is not an hHtml option.
```

Delete or replace current camel-to-kebab tests.
That helper should not remain in `hHtml` after top-level style is removed.

## DOM accepted Interface

`hDom` mirrors the cleaned-up `hHtml` Interface where DOM semantics allow.

### Text node mode

`tag: ''` creates a `Text` node.
Only `text` is valid.

Good:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
const node = hDom({
  tag: '',
  text: '<b>banana</b>',
});
```

Expected DOM result:

```html
<!-- doc/todo/hyperscript-stable-syntax-deepening.md -->
&lt;b&gt;banana&lt;/b&gt;
```

The returned value is a `Text` node,
not an `HTMLElement`.

Bad:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hDom({
  tag: '',
  attrs: { id: 'banana' },
  text: 'banana',
});
```

Expected result:

```text
Type error: tag "" allows only text.
```

### Mutually exclusive content modes

`hDom` allows exactly one content mode:

- no content;
- `text`;
- `html`;
- `children`.

Good text mode:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hDom({
  tag: 'p',
  text: 'banana',
});
```

Good raw HTML mode:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hDom({
  tag: 'p',
  html: unsafeHtml('<strong>banana</strong>'),
});
```

Bad raw HTML mode:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hDom({
  tag: 'p',
  html: '<strong>banana</strong>',
});
```

Expected result:

```text
Type error: string is not HtmlFragment.
```

Good children mode:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hDom({
  tag: 'p',
  children: [
    hDom({ tag: '', text: 'Hello ' }),
    hDom({ tag: 'strong', text: 'banana' }),
  ],
});
```

Expected DOM result:

```html
<!-- doc/todo/hyperscript-stable-syntax-deepening.md -->
<p>Hello <strong>banana</strong></p>
```

Bad mixed mode:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hDom({
  tag: 'div',
  text: 'A',
  html: unsafeHtml('<span>B</span>'),
});
```

Expected result:

```text
Type error: choose text or html, not both.
```

### Children are nodes only

Do not let `children` accept strings.
Use text-node mode.

Bad:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hDom({
  tag: 'p',
  children: ['hello'],
});
```

Expected result:

```text
Type error: string is not Node.
```

Good:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hDom({
  tag: 'p',
  children: [hDom({ tag: '', text: 'hello' })],
});
```

### Attributes

`hDom` attrs are `string | true`,
same as `hHtml`.

Good:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hDom({
  tag: 'input',
  attrs: {
    type: 'checkbox',
    disabled: true,
  },
});
```

Expected DOM effect:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
input.setAttribute('type', 'checkbox');
input.setAttribute('disabled', '');
```

Bad:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hDom({
  tag: 'input',
  attrs: {
    disabled: false,
  },
});
```

Expected result:

```text
Type error and runtime error if bypassed.
```

`hDom` should validate HTML tag names and attribute names with the same grammar as `hHtml`.

### No top-level `class` or `style`

Bad:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hDom({
  tag: 'button',
  class: 'primary',
});
```

Expected result:

```text
Type error: class is not an hDom option.
```

Good:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hDom({
  tag: 'button',
  attrs: {
    class: 'primary',
    style: 'inline-size:100%',
  },
  text: 'banana',
});
```

### Void content

Mirror `hHtml`.
Literal void tags with content are type errors.
Dynamic void tags with content throw at runtime.

Bad:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hDom({
  tag: 'br',
  text: 'banana',
});
```

Expected result:

```text
Type error: void HTML element cannot have content.
```

### Events

Keep event listener support in `hDom`.
Events are a DOM concern,
not an HTML string concern.

Good:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hDom({
  tag: 'button',
  attrs: { type: 'button' },
  text: 'Save',
  on: {
    click: handleSave,
  },
});
```

The event map should remain typed for known DOM events.
Do not add events to `hHtml`.

## SVG DOM plan

This is stable syntax,
not app icon policy.
The package should support creating SVG namespace nodes without `innerHTML`.

Recommended shape:
add an `hSvg` Adapter exported from the same package.
It may live under the DOM area,
for example `src/dom/svg.ts`,
or under `src/svg/index.ts` if that keeps the code clearer.

Do not add app-specific icon Modules.
Do not add `backIcon` or `closeIcon`.
Callers can define their own local icon helpers.

Good future usage:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
const closeGlyph = hSvg({
  tag: 'svg',
  attrs: {
    viewBox: '0 0 48 48',
    fill: 'none',
    stroke: 'currentColor',
    'stroke-width': '4',
  },
  children: [
    hSvg({
      tag: 'line',
      attrs: {
        x1: '14',
        y1: '14',
        x2: '34',
        y2: '34',
      },
    }),
    hSvg({
      tag: 'line',
      attrs: {
        x1: '34',
        y1: '14',
        x2: '14',
        y2: '34',
      },
    }),
  ],
});

hDom({
  tag: 'button',
  attrs: {
    class: 'close',
    'aria-label': 'Close',
  },
  children: [closeGlyph],
});
```

Expected DOM shape:

```html
<!-- doc/todo/hyperscript-stable-syntax-deepening.md -->
<button class="close" aria-label="Close">
  <svg viewBox="0 0 48 48" fill="none" stroke="currentColor" stroke-width="4">
    <line x1="14" y1="14" x2="34" y2="34"></line>
    <line x1="34" y1="14" x2="14" y2="34"></line>
  </svg>
</button>
```

`hSvg` should use `document.createElementNS` with the SVG namespace.
It should not use `innerHTML`.

`hSvg` attrs should be string-only unless a real SVG grammar case justifies `true`.
SVG is XML-like;
valueless boolean attributes are not the same as HTML boolean attributes.

`hSvg` content modes should be simple:

- no content;
- `text`;
- `children`.

Do not add raw SVG string insertion in the first pass.
If a future caller genuinely needs raw SVG,
that should be a separate decision with tests.

## XML accepted Interface

### Brand

`hXml` returns `XmlFragment`,
not plain `string`.

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
declare const xmlFragmentBrand: unique symbol;

export type XmlFragment = string & {
  readonly [xmlFragmentBrand]: true;
};
```

Good:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
const title = hXml({
  tag: 'title',
  text: 'banana',
});

hXml({
  tag: 'entry',
  children: [title],
});
```

Output:

```xml
<!-- doc/todo/hyperscript-stable-syntax-deepening.md -->
<entry><title>banana</title></entry>
```

Bad:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
const loose: string = '<title>banana</title>';

hXml({
  tag: 'entry',
  children: [loose],
});
```

Expected result:

```text
Type error: string is not XmlFragment.
```

### Raw XML escape hatch

Add `unsafeXml(raw: string): XmlFragment`.

Good,
but dangerous:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hXml({
  tag: 'entry',
  raw: unsafeXml('<title>banana</title>'),
});
```

Output:

```xml
<!-- doc/todo/hyperscript-stable-syntax-deepening.md -->
<entry><title>banana</title></entry>
```

Bad:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hXml({
  tag: 'entry',
  raw: '<title>banana</title>',
});
```

Expected result:

```text
Type error: string is not XmlFragment.
```

### Empty tag text-fragment mode

`tag: ''` plus `text` returns escaped XML text.
Only `text` is valid.

Good:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hXml({
  tag: '',
  text: 'x < y & z',
});
```

Output:

```xml
<!-- doc/todo/hyperscript-stable-syntax-deepening.md -->
x &lt; y &amp; z
```

Bad:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hXml({
  tag: '',
  attrs: { id: 'banana' },
  text: 'banana',
});
```

Expected result:

```text
Type error: tag "" allows only text.
```

### Mutually exclusive content modes

Normal XML elements may use exactly one content mode:

- no content;
- `text`;
- `raw`;
- `children`.

Good text mode:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hXml({
  tag: 'title',
  text: 'banana',
});
```

Output:

```xml
<!-- doc/todo/hyperscript-stable-syntax-deepening.md -->
<title>banana</title>
```

Good raw mode:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hXml({
  tag: 'entry',
  raw: unsafeXml('<title>banana</title>'),
});
```

Good children mode:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hXml({
  tag: 'entry',
  children: [
    hXml({ tag: 'title', text: 'banana' }),
  ],
});
```

Bad mixed mode:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hXml({
  tag: 'entry',
  text: 'A',
  children: [hXml({ tag: 'title', text: 'B' })],
});
```

Expected result:

```text
Type error: choose text or children, not both.
```

### XML self-closing

Keep the current XML self-closing rule.
If there is no content,
self-close.

Good:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hXml({
  tag: 'link',
  attrs: { href: 'https://example.com' },
});
```

Output:

```xml
<!-- doc/todo/hyperscript-stable-syntax-deepening.md -->
<link href="https://example.com" />
```

If `children: []` is supplied,
treat it as no content and self-close.
This preserves current behaviour.

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hXml({
  tag: 'items',
  children: [],
});
```

Output:

```xml
<!-- doc/todo/hyperscript-stable-syntax-deepening.md -->
<items />
```

### XML names

Validate XML tag names and attribute names at runtime.
Use a safe subset of XML names,
not the full Unicode XML grammar.

Recommended name grammar:

- one name part,
  or two name parts separated by exactly one colon;
- each part starts with ASCII letter or underscore;
- remaining characters are ASCII letters,
  digits,
  underscore,
  hyphen,
  or period;
- no spaces;
- no quotes;
- no equals signs;
- no slashes;
- no angle brackets;
- no empty parts;
- no more than one colon.

Good:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hXml({
  tag: 'atom:title',
  attrs: {
    'xml:lang': 'en',
  },
  text: 'banana',
});
```

Output:

```xml
<!-- doc/todo/hyperscript-stable-syntax-deepening.md -->
<atom:title xml:lang="en">banana</atom:title>
```

Good namespace declaration:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hXml({
  tag: 'feed',
  attrs: {
    'xmlns:atom': 'http://www.w3.org/2005/Atom',
  },
});
```

Bad:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hXml({
  tag: 'atom::title',
  text: 'banana',
});
```

Runtime throws:

```text
Invalid XML tag name: atom::title
```

### XML attrs are strings only

Do not add `true` boolean attrs to XML.
XML attributes have explicit values.

Good:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hXml({
  tag: 'option',
  attrs: {
    enabled: 'true',
  },
});
```

Bad:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hXml({
  tag: 'option',
  attrs: {
    enabled: true,
  },
});
```

Expected result:

```text
Type error: XML attrs are strings.
```

## CSS accepted Interface

### No CSS output brand yet

Do not add `CssFragment` in this pass.
Do not add `unsafeCss`.

`raw: string` remains allowed.
The field name already tells the human it is raw.

Good:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hCss({
  rule: '.card',
  raw: 'display:flex;gap:1rem',
});
```

Output:

```css
/* doc/todo/hyperscript-stable-syntax-deepening.md */
.card{display:flex;gap:1rem}
```

Do not change this to:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hCss({
  rule: '.card',
  raw: unsafeCss('display:flex;gap:1rem'),
});
```

There is no `unsafeCss` in this plan.

### Type-policy tests first

`hCss` is already the deepest current Module.
Its Depth is mostly in TypeScript types.
The tests must cover that Interface.

Good declaration:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
const good: CssDeclarations = {
  display: 'flex',
  gap: cssRem(1),
  color: 'currentColor',
  '--color-fg': 'oklch(0.2 0 0)',
};
```

Bad physical dimension:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
const bad: CssDeclarations = {
  width: cssRem(10),
};
```

Expected result:

```text
Type error: width is disallowed, use inline-size.
```

Bad raw length string:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
const bad: CssDeclarations = {
  gap: '1rem',
};
```

Expected result:

```text
Type error: use cssRem(1).
```

Bad named color:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
const bad: CssDeclarations = {
  color: 'red',
};
```

Expected result:

```text
Type error: named colors are banned.
```

Good special color keyword:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
const good: CssDeclarations = {
  color: 'currentColor',
};
```

### Do not redesign constructors first

Do not redesign these just because they accept strings:

- `cssCalc(expr: string)`
- `cssColorFn({ space, channels })`
- `cssOklchFrom({ l, c, h, a })`
- `cssAnchor(side: string)`
- `cssCommaList(values)`
- `cssCompounded(values)`

That would be speculative.
First write type-policy tests for the existing Interface.
Then tighten only leaks proven by tests or caller evidence.

### Disallowed at-rules

The type policy says some at-rules are disallowed.
Runtime must also reject known disallowed at-rules.

Known disallowed at-rules from current source:

- `charset`
- `font-palette-values`

Good:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hCss({
  at: 'media',
  params: '(prefers-color-scheme: dark)',
  children: [
    hCss({
      rule: ':root',
      decls: {
        color: 'currentColor',
      },
    }),
  ],
});
```

Output:

```css
/* doc/todo/hyperscript-stable-syntax-deepening.md */
@media (prefers-color-scheme: dark){:root{color:currentColor}}
```

Bad literal:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hCss({
  at: 'charset',
  params: '"UTF-8"',
});
```

Expected result:

```text
Type error: charset is disallowed.
```

Bad dynamic:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
const at: string = 'charset';

hCss({
  at,
  params: '"UTF-8"',
});
```

Runtime throws:

```text
Disallowed CSS at-rule: charset
```

The current runtime test that expects `@charset "UTF-8";` should be removed or inverted.
The policy wins over the old permissive runtime test.

## Type-shape sketch

This sketch is not final code.
It records the intended Interface.
The future implementer may choose better names internally.

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
type HtmlAttrValue = string | true;
type HtmlAttrs = Readonly<Record<string, HtmlAttrValue>>;

type NoHtmlContent = {
  readonly text?: never;
  readonly html?: never;
  readonly children?: never;
};

type HtmlTextContent = {
  readonly text: string;
  readonly html?: never;
  readonly children?: never;
};

type HtmlRawContent = {
  readonly html: HtmlFragment;
  readonly text?: never;
  readonly children?: never;
};

type HtmlChildrenContent = {
  readonly children: readonly HtmlFragment[];
  readonly text?: never;
  readonly html?: never;
};

type HtmlContent =
  | NoHtmlContent
  | HtmlTextContent
  | HtmlRawContent
  | HtmlChildrenContent;
```

For literal void tags,
use conditional types so content is rejected.
Do not rely on `Exclude<string, HtmlVoidTag>` alone,
because `string` still includes all strings.

Possible shape:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
type HtmlOptions<TTag extends string> = TTag extends ''
  ? HtmlTextFragmentOptions
  : TTag extends HtmlVoidTag
    ? HtmlVoidElementOptions<TTag>
    : HtmlNormalElementOptions<TTag>;

export function $(
  options: HtmlTextFragmentOptions,
): HtmlFragment;

export function $<const TTag extends string>(
  options: HtmlOptions<TTag>,
): HtmlFragment;
```

The actual implementation must include type tests proving these examples fail:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hHtml({ tag: 'br', text: 'banana' });
hHtml({ tag: '', attrs: { id: 'x' }, text: 'banana' });
hHtml({ tag: 'p', text: 'A', children: [] });
hHtml({ tag: 'p', children: ['loose string'] });
hHtml({ tag: 'p', html: '<b>x</b>' });
```

## Runtime validation sketch

Runtime validation should defend the Seam when TypeScript is bypassed.

HTML runtime checks:

- invalid tag name throws;
- invalid attr name throws;
- attr value `false` throws;
- attr value number throws;
- void tag with content throws;
- `tag: ''` with non-text fields throws;
- multiple content modes throw.

XML runtime checks:

- invalid tag name throws;
- invalid attr name throws;
- `tag: ''` with non-text fields throws;
- multiple content modes throw.

CSS runtime checks:

- known disallowed at-rule throws.

Use custom error classes if that matches nearby package style.
At minimum,
throw errors with exact messages tests can assert.

## Test plan

Use the Interface as the test surface.
Do not write tests against private helpers unless a helper has its own exported Interface.

### HTML tests

Replace the current camel-to-kebab-only HTML test coverage.
Cover:

- `hHtml` returns a branded value accepted by `children`;
- plain string rejected by `children` in type tests;
- plain string rejected by `html` in type tests;
- `unsafeHtml` accepted by `html`;
- `tag: ''` escapes text;
- `tag: ''` rejects attrs,
  html,
  and children;
- text mode escapes text;
- html mode preserves unsafe fragment;
- children mode concatenates fragments;
- mixed content modes reject at type level and runtime;
- void tag with content rejects at type level and runtime;
- dynamic void tag with content throws;
- invalid tag name throws;
- invalid attr name throws;
- attr `true` emits valueless attr;
- attr `false` throws at runtime;
- attr number throws at runtime if bypassed;
- top-level `class` no longer type-checks;
- top-level `style` no longer type-checks.

### DOM tests

Add DOM tests for `hDom`.
Because `hDom` uses real `document`,
prefer browser tests through the existing Playwright pattern used by `package/dom/core`.
Read that package before adding the task.

Cover:

- `tag: ''` returns a `Text` node;
- text node preserves raw text as text,
  not markup;
- children reject strings at type level;
- children accept `Node[]`;
- `html` accepts `HtmlFragment`;
- `html` rejects plain string at type level;
- content modes are mutually exclusive;
- attrs are `string | true`;
- attr `true` calls `setAttribute(name, '')`;
- attr `false` throws at runtime if bypassed;
- top-level `class` no longer type-checks;
- top-level `style` no longer type-checks;
- event listeners still work;
- void content rejects.

### SVG tests

If `hSvg` is added,
cover:

- created node namespace is `http://www.w3.org/2000/svg`;
- nested SVG children have SVG namespace;
- SVG attrs serialize correctly;
- SVG node can be a child of an `hDom` HTML node;
- no `innerHTML` is needed for close/back glyph examples.

### XML tests

Update XML tests to cover:

- `hXml` returns `XmlFragment`;
- children reject plain string at type level;
- `raw` rejects plain string at type level;
- `unsafeXml` accepted by `raw`;
- `tag: ''` escapes XML text;
- `tag: ''` rejects attrs,
  raw,
  and children;
- content modes are mutually exclusive;
- self-closing remains for no content;
- `children: []` self-closes;
- namespaced names still work;
- invalid XML names throw;
- XML attrs remain string-only.

### CSS tests

Add type-policy tests for:

- allowed keyword values;
- allowed `CssValue` constructors;
- custom properties;
- disallowed physical properties;
- disallowed shorthand properties;
- raw length strings rejected where constructor should be used;
- named colors rejected;
- `currentColor` allowed;
- disallowed at-rule literals rejected;
- dynamic disallowed at-rule throws at runtime.

## Type-test mechanics

The repository currently has runtime unit tests in `module-hyperscript`.
Future implementer must choose a type-test mechanism that actually fails the build when type expectations drift.

Acceptable patterns:

- `.ts` files included by `lint:types` with `@ts-expect-error` for rejected examples;
- dedicated type test files if the repo already has that convention;
- a package-level type test task added through `mise.toml` if needed.

Do not rely on comments in Markdown as tests.
The examples in this document are explanatory only.

## Migration strategy

Do this in small commits.
The future implementer should commit after each meaningful change.

Recommended order:

1. Add shared `HtmlFragment` type and `unsafeHtml`.
2. Change `hHtml` return type,
   `html`,
   and `children` to `HtmlFragment`.
3. Add `tag: ''` text-fragment mode for `hHtml`.
4. Make `hHtml` content modes mutually exclusive.
5. Remove top-level `class` and `style` from `hHtml`.
6. Add HTML name and attr validation.
7. Add void-content type and runtime checks.
8. Update hHtml callers.
9. Mirror accepted rules in `hDom`.
10. Add DOM tests.
11. Add SVG namespace Adapter.
12. Add `XmlFragment` and `unsafeXml`.
13. Apply XML content and validation rules.
14. Add CSS type-policy tests.
15. Runtime reject disallowed at-rules.
16. Update README and any usage docs.

If a step becomes too large,
split again.
Do not widen the scope to page shells,
libvirt Modules,
or app icon Modules.

## Caller migration examples

### HTML class migration

Before:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hHtml({
  tag: 'div',
  class: 'card',
  text: 'banana',
});
```

After:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hHtml({
  tag: 'div',
  attrs: {
    class: 'card',
  },
  text: 'banana',
});
```

### HTML style migration

Before:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hHtml({
  tag: 'div',
  style: {
    flexDirection: 'row',
  },
});
```

After:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hHtml({
  tag: 'div',
  attrs: {
    style: 'flex-direction:row',
  },
});
```

### HTML mixed text migration

Before:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hHtml({
  tag: 'p',
  text: 'Hello ',
  children: [hHtml({ tag: 'strong', text: 'banana' })],
});
```

After:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hHtml({
  tag: 'p',
  children: [
    hHtml({ tag: '', text: 'Hello ' }),
    hHtml({ tag: 'strong', text: 'banana' }),
  ],
});
```

### HTML raw migration

Before:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hHtml({
  tag: 'article',
  html: renderedMarkdown,
});
```

After:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hHtml({
  tag: 'article',
  html: unsafeHtml(renderedMarkdown),
});
```

Only use `unsafeHtml` when the caller really owns the trust decision.
If the value is user text,
use `text` or `tag: ''` text-fragment mode instead.

### DOM string child migration

Before:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hDom({
  tag: 'p',
  children: ['banana'],
});
```

After:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hDom({
  tag: 'p',
  children: [hDom({ tag: '', text: 'banana' })],
});
```

### DOM SVG migration

Before:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
const button = hDom({
  tag: 'button',
  attrs: { 'aria-label': 'Close' },
});

button.innerHTML = '<svg viewBox="0 0 48 48"><line x1="14" y1="14" x2="34" y2="34" /></svg>';
```

After:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hDom({
  tag: 'button',
  attrs: { 'aria-label': 'Close' },
  children: [
    hSvg({
      tag: 'svg',
      attrs: { viewBox: '0 0 48 48' },
      children: [
        hSvg({
          tag: 'line',
          attrs: {
            x1: '14',
            y1: '14',
            x2: '34',
            y2: '34',
          },
        }),
      ],
    }),
  ],
});
```

### XML raw migration

Before:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hXml({
  tag: 'entry',
  raw: renderedXml,
});
```

After:

```ts
// doc/todo/hyperscript-stable-syntax-deepening.md
hXml({
  tag: 'entry',
  raw: unsafeXml(renderedXml),
});
```

## Verification commands for future implementation

Before running commands,
read root `mise.toml` and package `mise.toml`.
Use `mise run` tasks.
Do not run raw `tsc`,
raw `bun test`,
or direct package scripts.

Expected checks after TypeScript edits:

```sh
# doc/todo/hyperscript-stable-syntax-deepening.md
mise run //package/hyperscript/core:lint:types
```

Expected package lint:

```sh
# doc/todo/hyperscript-stable-syntax-deepening.md
mise run //package/hyperscript/core:lint
```

For DOM browser tests,
copy the existing Playwright task pattern from `package/dom/core`.
Do not invent a direct Playwright command if a mise task already exists.

After docs edits,
run markdown lint on touched docs:

```sh
# doc/todo/hyperscript-stable-syntax-deepening.md
mise run lint:markdown -- doc/todo/hyperscript-stable-syntax-deepening.md
```

## Completion criteria

The future implementation is not complete until:

- `hHtml` uses `HtmlFragment` exactly as described;
- `hDom` mirrors the accepted HTML rules;
- SVG namespace creation no longer requires caller `innerHTML` for ordinary inline SVG;
- `hXml` uses `XmlFragment` exactly as described;
- `hCss` type policy has executable tests;
- known disallowed CSS at-rules are rejected at runtime;
- all package callers compile;
- README design decisions are updated;
- package lint passes;
- package type lint passes;
- tests cover the public Interfaces,
  not private helper details.

## Final monkey rules

- Plain string is loose banana mush.
- `HtmlFragment` is HTML banana with a sticker.
- `XmlFragment` is XML banana with a sticker.
- `unsafeHtml` and `unsafeXml` are danger doors.
- `raw` in CSS is already a danger label,
  so no `unsafeCss` yet.
- `class` and `style` are just attributes.
- `false` is not an attribute banana.
- Numbers are not HTML attribute bananas.
- Empty tag means text fragment.
- One content bowl per element.
- Void HTML elements have no stomach.
- XML can self-close when it has no content.
- Stable syntax policy can be shared.
- Temporary tool policy stays in the caller forest.
