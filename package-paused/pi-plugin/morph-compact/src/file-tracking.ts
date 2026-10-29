/**
 File tracking utilities.
 Reimplemented because pi doesn't export computeFileLists or formatFileOperations.
 */

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type { FileOperations, } from '@earendil-works/pi-coding-agent';

/**
 Derive read/modified file lists from cumulative file operations.
 Mirrors pi's internal `computeFileLists` which is not exported.
 
 @param fileOps - cumulative file operations extracted from tool calls
 
 @returns sorted lists of read-only and modified file paths
 
 @example
 ```typescript
 const { readFiles, modifiedFiles } = computeFileLists(fileOps);
 ```
 */
export function computeFileLists(
  fileOps: ForeignBorrowed<FileOperations>,
): {
  readFiles: string[];
  modifiedFiles: string[];
} {
  /**
   Union of edits and writes used to subtract from the read list.
   */
  const modified = new Set([
    ...fileOps.edited,
    ...fileOps.written,
  ],);
  /**
   Read-only entries with mutations stripped to avoid duplication.
   */
  const readFiles = [...fileOps.read,]
    .filter(function isNotModified(file,) {
      return !modified.has(file,);
    },)
    .toSorted();
  /**
   Sorted modifications surface deterministically in summaries.
   */
  const modifiedFiles = [...modified,].toSorted();
  return {
    readFiles,
    modifiedFiles,
  };
}

/**
 Format file operations as XML appended to compaction summary.
 Mirrors pi's internal `formatFileOperations` which is not exported.
 
 @param readFiles - read-only file paths
 
 @param modifiedFiles - written or edited file paths
 
 @returns XML string with `<read-files>` and `<modified-files>` sections, or empty string
 
 @example
 ```typescript
 const xml = formatFileOperations({ readFiles: ["foo.ts"], modifiedFiles: ["bar.ts"] });
 // "<read-files>\nfoo.ts\n</read-files>\n\n<modified-files>\nbar.ts\n</modified-files>"
 ```
 */
export function formatFileOperations({
  readFiles,
  modifiedFiles,
}: {
  readonly readFiles: readonly string[];
  readonly modifiedFiles: readonly string[];
},): string {
  /**
   Accumulates per-category XML fragments before joining.
   */
  const sections: string[] = [];
  if (readFiles.length
    > 0)
    sections.push(`<read-files>\n${readFiles.join('\n',)}\n</read-files>`,);
  if (modifiedFiles.length
    > 0)
    sections.push(`<modified-files>\n${modifiedFiles.join('\n',)}\n</modified-files>`,);
  if (sections.length
    === 0)
    return '';
  return `\n\n${sections.join('\n\n',)}`;
}
