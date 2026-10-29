/**
 Minimal css-tree surface for the benchmark; the package ships no types and
 `@types/css-tree` lags a major version, so the benchmark declares exactly
 the two calls it makes.
 */
declare module 'css-tree' {
  /**
   Opaque css-tree AST handle; the benchmark only round-trips it.
   */
  type CssTreeNode = {
    readonly type: string;
  };

  /**
   Parses CSS into a css-tree AST.
   
   @param source - CSS text.
   
   @returns css-tree AST.
   */
  export function parse(source: string,): CssTreeNode;

  /**
   Serializes a css-tree AST back to CSS text.
   
   @param ast - css-tree AST from {@link parse}.
   
   @returns Compact CSS text.
   */
  export function generate(ast: CssTreeNode,): string;
}
