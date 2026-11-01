/**
 Default-library read-only view member names, read from upstream declarations.
 
 @module
 */

import {
  isIdentifier,
  isInterfaceDeclaration,
  isMethodSignatureDeclaration,
  isPropertySignatureDeclaration,
} from 'typescript/ast/is';
import type {
  Program,
  Project,
} from 'typescript/sync';

/**
 Prefix TypeScript gives every default-library read-only collection view.
 */
export const READONLY_VIEW_INTERFACE_PREFIX = 'Readonly';

/**
 Basename prefix of every TypeScript default-library declaration file.
 */
const LIBRARY_FILE_PREFIX = 'lib.';

/**
 Basename suffix of every TypeScript default-library declaration file.
 */
const LIBRARY_FILE_SUFFIX = '.d.ts';

/**
 View member names per program snapshot, rebuilt when the snapshot changes.
 */
const viewMembersByProgram = new WeakMap<
  Program,
  ReadonlyMap<string, ReadonlySet<string>>
  >();

/**
 Collects member names declared on each default-library read-only view.
 
 TypeScript derives each read-only view from its mutable collection by removing
 the mutators, so the view's member list is upstream's own statement of which
 operations survive a holder that may not mutate the value. Reading it here
 keeps that partition upstream's rather than authoring one.
 
 Interfaces merge across library files, so every default-library file
 contributes: `ReadonlyArray` alone is declared in `lib.es5.d.ts`,
 `lib.es2015.core.d.ts`, `lib.es2015.iterable.d.ts` and more, and a partial
 scan would misread a later-declared member as a mutator.
 
 @param project - TypeScript project owning the program snapshot.
 
 @returns member names by view interface name.
 
 @example
 ```ts
 defaultLibraryViewMembers({ project }).get('ReadonlyArray');
 ```
 */
export function defaultLibraryViewMembers({
  project,
}: {
  readonly project: Project;
},): ReadonlyMap<string, ReadonlySet<string>> {
  /**
   Program snapshot keying the memoized scan.
   */
  const { program, } = project;
  /**
   Member names already scanned for this snapshot.
   */
  const memoized = viewMembersByProgram.get(program,);
  if (memoized !== undefined)
    return memoized;
  /**
   Accumulated member names by view interface name.
   */
  const viewMembers = new Map<string, Set<string>>();
  program.getSourceFileNames()
    .forEach(function scanCandidate(fileName,): void {
      // Cheap basename test before fetching: a whole-program fetch to classify
      // every file costs more than twice this scan on its own.
      /**
       Trailing path segment naming the candidate file.
       */
      const baseName = fileName.slice(fileName.lastIndexOf('/',) + 1,);
      if ((!baseName.startsWith(LIBRARY_FILE_PREFIX,))
        || (!baseName.endsWith(LIBRARY_FILE_SUFFIX,)))
        return;
      /**
       Candidate source file, absent when not part of the program.
       */
      const sourceFile = program.getSourceFile(fileName,);
      if ((sourceFile === undefined)
        || (!program.isSourceFileDefaultLibrary(sourceFile,)))
        return;
      sourceFile.forEachChild(function collectInterface(statement,): undefined {
        if ((!isInterfaceDeclaration(statement,))
          || (!statement.name
            .text
            .startsWith(READONLY_VIEW_INTERFACE_PREFIX,)))
          return undefined;
        /**
         Interface name identifying this view across library files.
         */
        const viewName = statement.name
          .text;
        /**
         Member names already collected for this view.
         */
        const members = viewMembers.get(viewName,)
          ?? new Set<string>();
        statement.members
          .forEach(function collectMember(member,): void {
            // Index signatures and call signatures carry no name and name no
            // operation, so only named members participate in the partition.
            if ((!isMethodSignatureDeclaration(member,))
              && (!isPropertySignatureDeclaration(member,)))
              return;
            if (!isIdentifier(member.name,))
              return;
            members.add(member.name
              .text,);
          },);
        viewMembers.set(
          viewName,
          members,
        );
        return undefined;
      },);
    },);
  viewMembersByProgram.set(
    program,
    viewMembers,
  );
  return viewMembers;
}
