/** Ambient type shim for `.css` files imported with `{ type: 'text' }` attribute. */
declare module '*.css' {
  const content: string;
  export default content;
}
