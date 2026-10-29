/**
 Ambient type declaration for `wawoff2`, the Fontello packaging of Google's
 WOFF2 codec compiled to WebAssembly. The package ships without TypeScript
 types, so this declaration documents only the call shape used by
 `src/build/subset-fonts.ts` (and only `decompress`, since the re-encode
 step goes through `woff2-encode-wasm`).
 */
declare module 'wawoff2' {
  /**
   Decompresses a WOFF2 buffer to the underlying SFNT (TrueType/OpenType)
   bytes. The returned value is a `Uint8Array` (Node `Buffer` is a subclass).
   */
  function decompress(
    input: Uint8Array,
  ): Promise<Uint8Array>;

  /**
   Compresses an SFNT buffer to WOFF2. Not used by `subset-fonts.ts` (the
   re-encode goes through `woff2-encode-wasm`), but declared for
   completeness so future callers see the available surface.
   */
  function compress(
    input: Uint8Array,
  ): Promise<Uint8Array>;

  const wawoff2: {
    decompress: typeof decompress;
    compress: typeof compress;
  };

  export default wawoff2;
  export {
    compress,
    decompress,
  };
}
