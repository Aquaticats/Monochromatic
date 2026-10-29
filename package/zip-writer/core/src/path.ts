/**
 Path validation rules for ZIP entry names.
 
 @module
 */

/**
 Reject paths that produce malformed or ambiguous ZIP entries.
 
 Allowed: forward-slash delimited, non-empty, UTF-8 representable,
 no NUL byte, no `..` segment, no leading `/`, no backslash. These
 constraints reflect what every well-behaved ZIP reader accepts.
 
 @param path - File path inside the archive
 
 @throws When the path is invalid
 
 @example
 ```ts
 validatePath('manifest.json',);    // ok
 validatePath('data/blob.bin',);    // ok
 validatePath('../escape.txt',);    // throws
 validatePath('/absolute',);        // throws
 ```
 */
export function validatePath(path: string,): void {
  if (path.length
    === 0)
    throw new Error('zip-writer: path must be non-empty',);
  if (path.includes('\0',))
    throw new Error(`zip-writer: path contains NUL byte: ${JSON.stringify(path,)}`,);
  if (path.includes('\\',))
    throw new Error(`zip-writer: backslash not allowed in path (use \`/\`): ${path}`,);
  if (path.startsWith('/',)) {
    throw new Error(
      `zip-writer: leading slash not allowed (use relative paths): ${path}`,
    );
  }
  for (const segment of path.split('/',)) {
    if (segment === '..') {
      throw new Error(
        `zip-writer: parent-directory segment \`..\` not allowed: ${path}`,
      );
    }
  }
}
