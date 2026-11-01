'use strict';

/**
 Names the one fact every thrown message repeats, so the message cannot drift from the override it describes.
 */
const OVERRIDE_KEY = 'micromark-extension-math>katex';

/**
 Thrown by every member of this shim: katex is deliberately not installed in this workspace.
 
 The class exists so a caller (or a test) can tell "someone reached for katex rendering" apart from any other
 failure. It carries the member that was reached, never a rendered result.
 */
class KatexNotInstalledError extends Error {
  /**
   @param {string} member Name of the `katex` member that was reached, so the message points at the exact call.
   */
  constructor(member,) {
    super(
      `katex.${member} is not available: this workspace does not install katex. `
        + `The pnpm-workspace.yaml override '${OVERRIDE_KEY}' replaces it with the workspace shim `
        + '@monochromatic-dev/pnpm-override-shim-katex (package/pnpm-override/shim-katex) because every katex release in the range '
        + 'micromark-extension-math@3 accepts (^0.16.0) is affected by GHSA-238p-pmpm-9mq7. '
        + "The shim exists only so that micromark-extension-math's lib/html.js, which its root re-exports "
        + "and which runs `import katex from 'katex'` when the module loads, keeps resolving. "
        + 'Rendering math to HTML (mathHtml) is therefore unsupported here; '
        + 'see doc/troubleshooting/dependencies.md before reaching for it.',
    );
    this.name = 'KatexNotInstalledError';
    this.member = member;
  }
}

/**
 Builds a member that satisfies the shape katex exposes (a callable) but throws when called.
 Reading the member never throws: `lib/html.js` reads `katex.renderToString` while the module loads.
 
 @param {string} member Name of the `katex` member the returned function stands in for.
 @returns {(...args: unknown[]) => never} A function that throws {@link KatexNotInstalledError}.
 */
function throwingMember(member,) {
  return {
    [member]() {
      throw new KatexNotInstalledError(member,);
    },
  }[member];
}

/**
 Stands in for katex's `ParseError` class: reading it is fine, constructing it throws.
 */
class ParseError {
  constructor() {
    throw new KatexNotInstalledError('ParseError',);
  }
}

module.exports = {
  KatexNotInstalledError,
  ParseError,
  __defineFunction: throwingMember('__defineFunction',),
  __defineMacro: throwingMember('__defineMacro',),
  __defineSymbol: throwingMember('__defineSymbol',),
  __parse: throwingMember('__parse',),
  __renderToDomTree: throwingMember('__renderToDomTree',),
  __renderToHTMLTree: throwingMember('__renderToHTMLTree',),
  __setFontMetrics: throwingMember('__setFontMetrics',),
  render: throwingMember('render',),
  renderToString: throwingMember('renderToString',),
};
