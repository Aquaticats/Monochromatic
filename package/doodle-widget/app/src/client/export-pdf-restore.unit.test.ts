/**
 The live SVG overlay comes back after a failed multi-page PDF export.

 Every page is rasterized through the one overlay element the reader is looking
 at, so the export replaces its markup per page. Restoration written after the
 page loop only runs when the loop finishes, and a raster, a blob read, or the
 PDF composition can reject; the reader is then left on another page's
 background while the editor still calls its own page active. The case below
 switches to the second page, fails the first page's raster after the mutation,
 and reads the markup back.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  exportAsPdf,
  initPages,
  switchToPage,
} from '../../dist/final/node/index.mjs';

//region Constants

/**
 Markup of the page the reader has open, and what the export must put back.
 */
const ACTIVE_PAGE_MARKUP = '<svg id="active-page"><rect width="1" height="1"/></svg>';

/**
 Markup of the page the failing export rasterizes first, which differs from
 {@link ACTIVE_PAGE_MARKUP} so a leaked mutation is visible.
 */
const EXPORTED_PAGE_MARKUP = '<svg id="another-page"><circle r="1"/></svg>';

/**
 Failure the page raster reports, so the assertion names the exact error that
 carried the export out of the loop.
 */
const RASTER_FAILURE = 'raster refused this page (fixture)';

/**
 Page size the fixtures report, in CSS pixels, matching no letter dimension so
 the export's scale conversion stays a visible number.
 */
const PAGE_SIZE_PX = 400;

//endregion Constants

//region Fixtures

/**
 Overlay stand-in: the export reads and replaces `innerHTML`, and finds no SVG
 child so no overlay rasterization runs.
 */
type FakeOverlay = {
  innerHTML: string;
  querySelector: () => null;
};

/**
 Text layer stand-in holding no inputs, so serialization contributes nothing.
 */
type FakeTextLayer = {
  querySelectorAll: () => readonly never[];
  replaceChildren: () => void;
};

/**
 Canvas stand-in for the redraw a page switch runs before the export.
 */
type FakeContext = {
  clearRect: () => void;
};

/**
 Builds the overlay stand-in.

 @returns overlay whose markup the caller sets

 @example
 ```ts
 const overlay = fakeOverlay();
 ```
 */
function fakeOverlay(): FakeOverlay {
  return {
    innerHTML: '',
    querySelector: () => null,
  };
}

/**
 Builds the text layer stand-in.

 @returns layer that serializes to no entries

 @example
 ```ts
 const textLayer = fakeTextLayer();
 ```
 */
function fakeTextLayer(): FakeTextLayer {
  return {
    querySelectorAll: () => [],
    replaceChildren: () => undefined,
  };
}

/**
 Builds the container stand-in whose size read fails, which is the first
 failure-capable operation after the overlay mutation.

 @returns container whose `clientWidth` read throws {@link RASTER_FAILURE}

 @example
 ```ts
 const container = failingContainer();
 ```
 */
function failingContainer(): { readonly clientWidth: number; } {
  return {
    get clientWidth(): number {
      throw new Error(RASTER_FAILURE,);
    },
  };
}

/**
 Opens the page at [index] so the active page's markup and the exporting page's
 markup differ.

 @param overlay - overlay stand-in the page switch rewrites

 @param textLayer - text layer stand-in the page switch serializes

 @param index - page to open

 @example
 ```ts
 openPage({ overlay, textLayer, index: 1 });
 ```
 */
function openPage(
  {
    overlay,
    textLayer,
    index,
  }: {
    readonly overlay: FakeOverlay;
    readonly textLayer: FakeTextLayer;
    readonly index: number;
  },
): void {
  switchToPage({
    index,
    ctx: { clearRect: () => undefined, } as unknown as CanvasRenderingContext2D,
    cw: PAGE_SIZE_PX,
    ch: PAGE_SIZE_PX,
    overlay: overlay as unknown as HTMLElement,
    textLayer: textLayer as unknown as HTMLDivElement,
  },);
}

//endregion Fixtures

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: exportAsPdf.name,
      children: [
        it({
          name: 'puts the active page markup back when a page raster fails',
          fn: async () => {
            /**
             Overlay the export borrows, which holds the page the reader opened.
             */
            const overlay = fakeOverlay();
            /**
             Text layer the page snapshot serializes.
             */
            const textLayer = fakeTextLayer();
            initPages({
              backgrounds: [
                EXPORTED_PAGE_MARKUP,
                ACTIVE_PAGE_MARKUP,
              ],
              overlay: overlay as unknown as HTMLElement,
            },);
            openPage({
              overlay,
              textLayer,
              index: 1,
            },);
            /**
             Markup the reader sees when the export starts.
             */
            const activeMarkup = overlay.innerHTML;

            const failure = expect(exportAsPdf({
              container: failingContainer() as unknown as HTMLDivElement,
              overlay: overlay as unknown as HTMLDivElement,
              drawCanvas: {} as unknown as HTMLCanvasElement,
              textLayer: textLayer as unknown as HTMLDivElement,
            },),).rejects;
            await failure.toThrow(RASTER_FAILURE,);
            expect(overlay.innerHTML,).toBe(activeMarkup,);
          },
        },),
      ],
    },),
  ],
},);
