# @monochromatic-dev/pnpm-override-shim-katex

Not published (`private: true`);
it exists only as a `link:` target inside this workspace.

Throwing shim for [`katex`](https://www.npmjs.com/package/katex).
Every member of its default export throws `KatexNotInstalledError`,
and nothing ever renders.

## Why katex is not installed

`pnpm-workspace.yaml` overrides `'micromark-extension-math>katex'`
because every katex release that `micromark-extension-math@3` accepts (`^0.16.0`)
is affected by GHSA-238p-pmpm-9mq7 (CVE-2026-103923).
The fixed release,
`0.18.2`,
is outside that range,
and the latest `micromark-extension-math` still asks for `^0.16.0`.
The workspace uses only the syntax extension through `remark-math`;
it never renders math to HTML with katex.

## Why an import must still resolve

`micromark-extension-math`'s root `index.js` re-exports `mathHtml` from `lib/html.js`,
and `lib/html.js` runs `import katex from 'katex'` and reads `katex.renderToString` when the module loads.
Dropping katex altogether (`'-'`) therefore made every unbundled load of `remark-math` throw
`ERR_MODULE_NOT_FOUND: Cannot find package 'katex'`.
Built bundles were unaffected because rolldown drops `lib/html.js` (`sideEffects: false`).

This shim stands in so the import resolves,
the way `package/pnpm-override/shim-node-domexception` does for its package.
Unlike that shim,
it cannot be API-identical,
because the real katex is what the advisory rules out:
reading a member is allowed,
and calling or constructing one throws.

## Members

- `render`,
  `renderToString`,
  `__parse`,
  `__renderToDomTree`,
  `__renderToHTMLTree`,
  `__setFontMetrics`,
  `__defineSymbol`,
  `__defineFunction`,
  `__defineMacro`:
  functions that throw `KatexNotInstalledError` when called.
- `ParseError`:
  a class whose constructor throws `KatexNotInstalledError`.
- `KatexNotInstalledError`:
  the error class;
  its `member` field names the katex member that was reached.
- `KatexOptions` (type only):
  an open interface,
  because `micromark-extension-math`'s `HtmlOptions` extends it
  and,
  once `katex` resolves to this package,
  TypeScript no longer falls back to `@types/katex`.

## Using real katex

Do not point the override at a real katex until a release outside the advisory range exists in a version
that `micromark-extension-math` accepts,
or until `mathHtml` is needed and the advisory is judged against that use.
The message of `KatexNotInstalledError` repeats the override name and the advisory so a failure explains itself.

See `doc/troubleshooting/dependencies.md` for the audit trail
and `doc/dependency-blocklist.md` for when to reach for a shim package versus a generic stub or removal.
