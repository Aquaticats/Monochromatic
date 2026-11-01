/**
 Toolbar event handler setup for the doodle widget.
 
 Wires up color picker, size slider, clear, export, and upload
 controls to their respective actions.
 */

import { setSvgBackground, } from './background.ts';
import {
  setStrokeColor,
  setStrokeWidth,
} from './drawing-config.ts';
import { clearStrokes, } from './drawing.ts';
import { exportAsPdf, } from './export-pdf.ts';
import { exportAsPng, } from './export-png.ts';
import { exportAsSvg, } from './export-svg.ts';
import type {
  ExportDeps,
  ExportFormat,
} from './export.ts';
import { clearTextEntries, } from './text.ts';

/**
 Valid export formats, keyed for the isExportFormat membership test
 */
const EXPORT_FORMATS: Record<ExportFormat, true> = {
  pdf: true,
  png: true,
  svg: true,
};

/**
 Checks whether a string is a valid export format.
 
 @param value - string to check
 
 @returns `true` if value is a recognized export format
 */
function isExportFormat(value: string,): value is ExportFormat {
  return value in EXPORT_FORMATS;
}

/**
 Runs the exporter matching the format.
 
 {@link exportAsPdf} and {@link exportAsPng} rasterize the canvas
 asynchronously, so they are awaited; {@link exportAsSvg} serializes
 vector markup synchronously, so it is invoked directly. A uniform
 `Promise<void>`-returning interface keeps the dispatch free of a
 `void | Promise<void>` union.
 
 @param format - validated export format
 
 @param deps - shared {@link ExportDeps} export dependencies
 */
async function runExport({
  format,
  deps,
}: {
  readonly format: ExportFormat;
  readonly deps: ExportDeps;
},): Promise<void> {
  if (format === 'pdf') {
    await exportAsPdf(deps,);
    return;
  }
  if (format === 'png') {
    await exportAsPng(deps,);
    return;
  }
  exportAsSvg(deps,);
}

/**
 Dependencies for toolbar handler setup.
 */
type ToolbarHandlerDeps = {
  /**
   Color picker input
   */
  readonly colorPicker: HTMLInputElement;
  /**
   Stroke width slider
   */
  readonly sizeSlider: HTMLInputElement;
  /**
   Clear button
   */
  readonly clearBtn: HTMLButtonElement;
  /**
   Export button
   */
  readonly exportBtn: HTMLButtonElement;
  /**
   Export format dropdown
   */
  readonly formatSelect: HTMLSelectElement;
  /**
   Upload trigger button
   */
  readonly uploadBtn: HTMLButtonElement;
  /**
   Hidden file input
   */
  readonly uploadInput: HTMLInputElement;
  /**
   Page element (coordinate reference for export sizing)
   */
  readonly page: HTMLDivElement;
  /**
   SVG overlay element
   */
  readonly svgOverlay: HTMLDivElement;
  /**
   Drawing canvas element
   */
  readonly drawCanvas: HTMLCanvasElement;
  /**
   Text layer element
   */
  readonly textLayer: HTMLDivElement;
  /**
   Canvas 2D rendering context
   */
  readonly ctx: CanvasRenderingContext2D;
  /**
   Returns current canvas dimensions
   */
  readonly getCanvasSize: () => {
    cw: number;
    ch: number;
  };
  /**
   Resizes and redraws the canvas
   */
  readonly sizeCanvas: () => void;
  /**
   Pushes current state to undo history after a completed action
   */
  readonly pushSnapshot: () => void;
};

/**
 Attaches event listeners for all toolbar controls.
 
 @param deps - toolbar elements and shared state accessors, see {@link ToolbarHandlerDeps}
 
 @mutates deps - `clearBtn.addEventListener`, `colorPicker.addEventListener`, `exportBtn.addEventListener`, `sizeSlider.addEventListener`, `uploadBtn.addEventListener`, and `uploadInput.addEventListener` change event targets and retain handlers; `uploadInput.click` can dispatch retained listeners.
 
 @example
 ```ts
 setupToolbarHandlers(deps);
 ```
 */
export function setupToolbarHandlers(deps: ToolbarHandlerDeps,): void {
  /**
   Destructured up front so each handler closure captures the same handles.
   */
  const {
    colorPicker,
    sizeSlider,
    clearBtn,
    exportBtn,
    formatSelect,
    uploadBtn,
    uploadInput,
    page,
    svgOverlay,
    drawCanvas,
    textLayer,
    ctx,
    getCanvasSize,
    sizeCanvas,
    pushSnapshot,
  } = deps;

  colorPicker.addEventListener(
    'input',
    function handleColorChange(): void {
      setStrokeColor(colorPicker.value,);
    },
  );

  sizeSlider.addEventListener(
    'input',
    function handleSizeChange(): void {
      setStrokeWidth(Number(sizeSlider.value,),);
    },
  );

  clearBtn.addEventListener(
    'click',
    function handleClear(): void {
      /**
       Canvas dimensions captured per click so the clear region matches the current layout.
       */
      const {
        cw,
        ch,
      } = getCanvasSize();
      clearStrokes();
      clearTextEntries();
      ctx.clearRect(
        0,
        0,
        cw,
        ch,
      );
      pushSnapshot();
    },
  );

  exportBtn.addEventListener(
    'click',
    function handleExportClick(): void {
      /**
       Selected export format from the dropdown
       */
      const format = formatSelect.value;
      if (!isExportFormat(format,))
        return;
      void runExport({
        format,
        deps: {
          container: page,
          overlay: svgOverlay,
          drawCanvas,
          textLayer,
        },
      },);
    },
  );

  uploadBtn.addEventListener(
    'click',
    function handleUploadClick(): void {
      uploadInput.click();
    },
  );

  /**
   Processes a user-selected SVG background file.
   
   Reads the file as text and renders it in the SVG overlay layer
   via {@link setSvgBackground}.
   
   @param file - uploaded SVG file
   */
  async function processBackgroundFile(file: File,): Promise<void> {
    clearStrokes();
    clearTextEntries();
    /**
     Raw SVG markup read from the uploaded file
     */
    const svgMarkup = await file.text();
    setSvgBackground({
      svgMarkup,
      overlay: svgOverlay,
    },);
    sizeCanvas();
    pushSnapshot();
  }

  uploadInput.addEventListener(
    'change',
    function handleFileChange(): void {
      /**
       Selected file from the upload input
       */
      const file = uploadInput.files
        ?.item(0,)
        ?? null;
      if (file === null)
        return;
      void processBackgroundFile(file,);
    },
  );
}
