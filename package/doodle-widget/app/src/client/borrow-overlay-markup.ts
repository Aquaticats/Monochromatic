/**
 Puts an element's markup back when the scope that borrowed it ends.

 Multi-page exports borrow the live SVG overlay as scratch space: each page is
 rasterized through the same element, so the markup the reader sees is replaced
 for the duration. Restoration written as a bare statement after the work runs
 only on the paths that reach it, and rasters, blob reads, and PDF composition
 all reject; the reader is then left looking at another page's background while
 the editor believes its own page is active.

 A borrowed-markup handle restores during scope exit and during exception
 unwinding, so every path that leaves the scope puts the original markup back.

 @module
 */

//region Types

/**
 Anything whose inner markup can be read and replaced, which a DOM element and a
 test fixture both satisfy.
 */
export type MarkupHolder = {
  innerHTML: string;
};

//endregion Types

/**
 Snapshots an element's markup and returns a handle that puts it back on dispose.

 The snapshot is taken here, before the caller mutates anything, so a partial
 mutation by a rejecting rasterizer is undone exactly as a completed run is.

 @param overlay - element whose markup the caller replaces

 @returns handle whose `Symbol.dispose` restores {@link MarkupHolder.innerHTML}

 @example
 ```ts
 using borrowed = borrowOverlayMarkup({ overlay });
 for (const page of pages)
   await renderPageCanvas({ svgBackground: page.svgBackground, overlay });
 ```
 */
export function borrowOverlayMarkup({
  overlay,
}: {
  readonly overlay: MarkupHolder;
},): { readonly [Symbol.dispose]: () => void; } {
  /**
   Markup the reader currently sees, put back when the borrowing scope ends.
   */
  const borrowedMarkup = overlay.innerHTML;
  return {
    [Symbol.dispose](): void {
      overlay.innerHTML = borrowedMarkup;
    },
  };
}
