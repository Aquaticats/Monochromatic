/**
 * Paper-parsing dispatch.
 *
 * Routes by extension and mime type, delegating to the per-format
 * extractor. Returns the unified plain-text representation that the
 * dialogue generator feeds into the LLM.
 */
import { BYTES_PER_MIB, } from '@monochromatic-dev/const-core';

import { extractPdf, } from './pdf.ts';
import { extractText, } from './text.ts';

/**
 * Maximum accepted upload size in mebibytes, mirroring the original UI.
 */
const MAX_MIB = 30;

/**
 * Maximum accepted file size in bytes.
 */
const MAX_BYTES = MAX_MIB * BYTES_PER_MIB;

/**
 * Extracts text from a paper file.
 *
 * @param file - browser `File`
 *
 * @returns plain text content
 *
 * @throws when the file is too large or the format is unsupported
 *
 * @example
 * ```ts
 * const file = inputEl.files[0];
 * const text = await extractPaperText(file);
 * // text is plaintext content; PDFs go through pdf.ts, .md/.txt through text.ts
 * ```
 */
export async function extractPaperText(file: File,): Promise<string> {
  if (file.size
    > MAX_BYTES)
    throw new Error(`paper too large: ${file.size} bytes (max ${MAX_BYTES})`,);
  /**
   * Lowercased file name so extension checks stay case-insensitive.
   */
  const name = file.name
    .toLowerCase();
  /**
   * Whether the upload should be routed to the PDF extractor.
   */
  const isPdf = name.endsWith('.pdf',)
    || (file.type
      === 'application/pdf');
  /**
   * Whether the upload should be routed to the plain-text extractor.
   */
  const isText = name.endsWith('.txt',)
    || name
    .endsWith('.md',)
    || file
    .type
    .startsWith('text/',);
  if (isPdf)
    return await extractPdf(file,);
  if (isText)
    return await extractText(file,);
  throw new Error(`unsupported file type: ${file.type} (${file.name})`,);
}
