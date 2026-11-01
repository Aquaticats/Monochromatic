/**
 Options katex's `renderToString` and `render` accept.
 Only `micromark-extension-math`'s `HtmlOptions` extends it; nothing here ever renders,
 so the fields are open instead of copied from katex.
 */
interface KatexOptions {
  displayMode?: boolean;
  throwOnError?: boolean;
  [option: string]: unknown;
}

/**
 Thrown by every member of the shim; carries the katex member that was reached.
 */
declare class KatexNotInstalledError extends Error {
  readonly member: string;
  constructor(member: string,);
}

/**
 Stands in for katex's `ParseError`; constructing it throws {@link KatexNotInstalledError}.
 */
declare class ParseError extends Error {
  constructor();
}

declare function throwingMember(...args: unknown[]): never;

declare const katex: {
  KatexNotInstalledError: typeof KatexNotInstalledError;
  ParseError: typeof ParseError;
  __defineFunction: typeof throwingMember;
  __defineMacro: typeof throwingMember;
  __defineSymbol: typeof throwingMember;
  __parse: typeof throwingMember;
  __renderToDomTree: typeof throwingMember;
  __renderToHTMLTree: typeof throwingMember;
  __setFontMetrics: typeof throwingMember;
  render: typeof throwingMember;
  renderToString: typeof throwingMember;
};

declare namespace katex {
  export type { KatexOptions, };
}

export = katex;
