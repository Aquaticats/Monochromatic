/**
 Comment attached to a JSONC node.
 
 A node carries at most one `JsoncComment`. When several `//` or block comments
 stack against the same node, the parser merges them into a single comment of
 type `mixed` whose `text` joins the bodies with newlines. The body is stored
 untrimmed so markers like `//region` survive a round-trip.
 
 @example
 ```ts
 const inline: JsoncComment = { type: 'inline', text: ' a name' };
 const merged: JsoncComment = { type: 'mixed', text: 'region config\n a name' };
 ```
 */
export type JsoncComment = {
  /**
   `inline` for a `//` line comment, `block` for a `/* *\/` comment, `mixed`
   when stacked comments of differing kinds were merged into one.
   */
  readonly type: 'inline' | 'block' | 'mixed';
  /**
   Comment body with delimiters removed and surrounding whitespace preserved,
   so `//region` and leading-space conventions are not lost.
   */
  readonly text: string;
};
