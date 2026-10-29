/**
 Shard identifier sanitisation.
 
 @example
 ```ts
 sanitizeShardTag('src/io/glob.ts');
 // 'src__io__glob.ts'
 ```
 */

/**
 Characters allowed in shard tags (container names, file names).
 */
const ALLOWED = new Set(
  'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789._-',
);

/**
 Converts one path into a tag safe for container names and file names.
 
 Slashes become double underscores; any other disallowed character
 becomes a single underscore. Linear scan, no regex.
 
 @param path - Package-relative path.
 
 @returns Sanitised tag.
 
 @example
 ```ts
 sanitizeShardTag('src/io/glob.ts');
 // 'src__io__glob.ts'
 ```
 */
export function sanitizeShardTag(path: string,): string {
  /**
   Sanitised characters accumulated by the scan.
   */
  const safe: string[] = [];

  for (const character of path) {
    if (character === '/')
      safe.push('__',);
    else
      safe.push(ALLOWED.has(character,) ? character : '_',);
  }

  return safe.join('',);
}
