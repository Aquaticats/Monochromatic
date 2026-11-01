/**
 Ambient program files that apply to every source without an import.
 
 @module
 */

import type { SourceFile, } from 'typescript/ast';

/**
 Tests whether a program file can change what a source sees without that source importing it.
 
 Global scripts apply to every file, and so do module files that augment a module or the
 global scope or declare an ambient module. A script is any file the parser did not mark as
 an external module, whatever its extension, so a non-module source with a top-level
 declaration counts as much as a declaration file does. Library files are scripts, with one
 module member that augments the global scope, so the rule covers them without asking the
 program which file is a default library. JSON modules cannot declare anything global.
 The `export as namespace` form exposes a module as a global to scripts, so a declaration
 file carrying it counts too.
 
 @param fileName - Program file path.
 
 @param sourceFile - Decoded program file.
 
 @returns whether the file belongs to the ambient surface.
 
 @example
 ```ts
 const ambient = isAmbientProgramFile({ fileName, sourceFile });
 ```
 */
export function isAmbientProgramFile({
  fileName,
  sourceFile,
}: {
  readonly fileName: string;
  readonly sourceFile: SourceFile;
},): boolean {
  if (fileName.endsWith('.json',))
    return false;
  /**
   Module and global augmentations the file declares.
   */
  const augmentations = sourceFile.moduleAugmentations;
  /**
   Ambient modules the file declares.
   */
  const ambientModules = sourceFile.ambientModuleNames;
  /**
   File text searched for the global-exposing export form.
   */
  const { text, } = sourceFile;
  return (sourceFile.externalModuleIndicator === undefined)
    || (augmentations.length > 0)
    || (ambientModules.length > 0)
    || (sourceFile.isDeclarationFile && text.includes('export as namespace',));
}
