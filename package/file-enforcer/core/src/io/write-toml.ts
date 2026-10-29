import {
  parseTomlEdit,
  tomlSet,
  tomlStringify,
  type TomlValueInput,
} from '@monochromatic-dev/toml-edit-core/ts';
import type { Path, } from '../types.ts';
import {
  ABSENT_FILE_CONTENT,
  overwrite,
  readExisting,
} from './write.ts';

/**
 Updates a single key in an existing TOML file, preserving comments and
 unmutated whitespace byte-identically (splice mode).
 
 Reads the destination via the shared read cache, applies one {@link tomlSet}, then
 routes the resulting text through {@link overwrite} so content-skip, cache
 update, and write-time tracking are inherited.
 
 Throws when the destination does not exist; create-from-empty is a different
 operation. To create an empty TOML and write keys into it, compose
 {@link emptyTomlEdit} + {@link tomlSet} + {@link tomlStringify} + {@link overwrite} from
 `\@monochromatic-dev/toml-edit-core` directly.
 
 @param dest - Path to the existing TOML file
 
 @param path - Sequence of key segments (and numeric array-of-tables indices)
 
 @param value - JS value to write at the path
 
 @param manifestPath - Optional staleness manifest path override forwarded to {@link overwrite}
 
 @throws Error when dest does not exist
 
 @throws Error from {@link parseTomlEdit} when the existing file is not valid TOML
 
 @example
 ```ts
 await overwriteTomlKey({
   dest: './pkg.toml',
   path: ['package', 'version',],
   value: '1.2.3',
 },);
 ```
 */
export async function overwriteTomlKey(
  {
    dest,
    path,
    value,
    manifestPath,
  }: {
    readonly dest: string;
    readonly manifestPath?: string;
    readonly path: Path;
    readonly value: TomlValueInput;
  },
): Promise<void> {
  /**
   Current file content; ABSENT_FILE_CONTENT when the file does not exist
   */
  const existing = await readExisting(dest,);
  if (existing === ABSENT_FILE_CONTENT)
    throw new Error(`overwriteTomlKey: ${dest} does not exist`,);
  /**
   Parsed TOML state for the existing content
   */
  const edit = parseTomlEdit({ source: existing, },);
  /**
   Fresh state with the pending edit recorded
   */
  const edited = tomlSet({
    edit,
    path,
    value,
  },);
  /**
   Updated source text; splice mode keeps unmutated regions byte-identical
   */
  const newContent = tomlStringify({ edit: edited, },);
  await overwrite(manifestPath === undefined
    ? {
      dest,
      content: newContent,
    }
    : {
      dest,
      content: newContent,
      manifestPath,
    },);
}
