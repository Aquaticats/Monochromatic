/**
 * Shared constants for editord client and server.
 */

import { BYTES_PER_KIB, } from '@monochromatic-dev/const-core';

/**
 * Maximum file size in bytes before large-file degradation kicks in.
 * Files exceeding this threshold trigger a client-side warning toast,
 * skip LSP activation (no diagnostics, hover, completions, etc.),
 * and truncate syntax highlighting.
 *
 * @example
 * A 150 KiB minified bundle triggers all large-file guards;
 * a 50 KiB source file is handled normally.
 */
export const FILE_SIZE_WARNING_THRESHOLD: number = 100 * BYTES_PER_KIB;
