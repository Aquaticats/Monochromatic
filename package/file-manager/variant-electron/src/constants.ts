/**
 Shared constants for the sticky-flow Electron file manager.
 
 Geometry mirrors `package/file-manager/app/src/constants.rs` so the
 two prototypes and the original render the same pane grid and the audit doc
 can compare them pixel for pixel.
 
 @example
 ```ts
 console.log(PANE_WIDTH);
 ```
 
 @packageDocumentation
 */

/**
 Fixed pane width in pixels on the strip.
 
 @example
 ```ts
 console.log(PANE_WIDTH);
 ```
 */
export const PANE_WIDTH = 320;

/**
 Fixed pane height in pixels on the strip.
 
 @example
 ```ts
 console.log(PANE_HEIGHT);
 ```
 */
export const PANE_HEIGHT = 520;

/**
 Gap in pixels between adjacent panes, both across columns and down rows.
 
 @example
 ```ts
 console.log(PANE_GAP);
 ```
 */
export const PANE_GAP = 12;

/**
 Vertical stride of one grid row: pane height plus inter-pane gap.
 
 @example
 ```ts
 console.log(ROW_STRIDE);
 ```
 */
export const ROW_STRIDE: number = PANE_HEIGHT + PANE_GAP;

/**
 Initial top-level window width in pixels.
 
 @example
 ```ts
 console.log(DEFAULT_WINDOW_WIDTH);
 ```
 */
export const DEFAULT_WINDOW_WIDTH = 1_280;

/**
 Initial top-level window height in pixels.
 
 @example
 ```ts
 console.log(DEFAULT_WINDOW_HEIGHT);
 ```
 */
export const DEFAULT_WINDOW_HEIGHT = 800;

/**
 Environment variable pointing at a JSON file where boundary tests observe
 renderer state, following the electron-counter state-file convention.
 
 @example
 ```ts
 console.log(STATE_PATH_ENVIRONMENT_VARIABLE);
 ```
 */
export const STATE_PATH_ENVIRONMENT_VARIABLE = 'MONOCHROMATIC_FILE_MANAGER_ELECTRON_STATE_PATH';

/**
 Environment variable overriding the directory the first root pane lists.
 
 @example
 ```ts
 console.log(ROOT_DIRECTORY_ENVIRONMENT_VARIABLE);
 ```
 */
export const ROOT_DIRECTORY_ENVIRONMENT_VARIABLE = 'MONOCHROMATIC_FILE_MANAGER_ELECTRON_ROOT';

/**
 Environment variable that, when set, renders the debug tint (visible rail
 outlines) so screenshots can name every structural layer.
 
 @example
 ```ts
 console.log(DEBUG_TINT_ENVIRONMENT_VARIABLE);
 ```
 */
export const DEBUG_TINT_ENVIRONMENT_VARIABLE = 'MONOCHROMATIC_FILE_MANAGER_ELECTRON_DEBUG_TINT';

/**
 Window and document title of the app shell.
 
 @example
 ```ts
 console.log(APP_TITLE);
 ```
 */
export const APP_TITLE = 'Monochromatic File Manager Electron';
