/**
 Error thrown when a CSS source string cannot be parsed under the strict
 structure rules of this package.
 
 Carries the byte offset where parsing failed so callers can point at the
 problem. Thrown rather than returned, per the workspace error policy.
 Tokenizer-level failures (bad strings, bad urls, unterminated comments) and
 structure-level failures (unclosed blocks, selector preludes without blocks)
 both surface as this class.
 
 @example
 ```ts
 try {
   parseCss({ source: '.btn {' as StringCss });
 } catch (error) {
   if (error instanceof CssParseError) console.error(error.offset);
 }
 ```
 */
export class CssParseError extends Error {
  /**
   Zero-based byte offset into the source where parsing failed.
   */
  readonly offset: number;

  /**
   @param message - Human-readable failure description.
   
   @param offset - Byte offset into source where parsing failed.
   */
  constructor({
    message,
    offset,
  }: {
    readonly message: string;
    readonly offset: number;
  },) {
    super(`${message} (at offset ${String(offset,)})`,);
    this.name = 'CssParseError';
    this.offset = offset;
  }
}
