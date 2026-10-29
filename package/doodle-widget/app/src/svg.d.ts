/** Ambient type shim for `.svg` files imported with `{ type: 'text' }` attribute. */
declare module '*.svg' {
  /** Raw SVG markup as text */
  const content: string;
  export default content;
}
