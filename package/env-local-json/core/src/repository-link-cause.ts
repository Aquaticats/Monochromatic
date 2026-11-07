/**
 The cause of a missing store path when the path goes through the repository's link to the 860 drive: the plain message of
 the repository link check (`package/repository-link/check`, plan item W10), never a guess.

 @module
 */

import { realpath, } from 'node:fs/promises';
import {
  dirname,
  basename,
  join,
  relative,
  isAbsolute,
  sep,
} from 'node:path';

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  checkRepositoryLink,
  identityPathFor,
  readIdentity,
  realProbe,
  type LinkIdentity,
} from '@monochromatic-dev/repository-link-check/ts';

/**
 Logger for the cause lookup; names paths only.
 */
const causeLogger = tagged({ tag: 'envLocalJsonLinkCause', },);

/**
 Resolves every link of a path as far as it exists and appends the rest as written, so a path through a link whose target is
 absent still lands under the link's own spelling.

 @param path - absolute path

 @returns the path with its existing part resolved

 @example
 ```ts
 const resolved = await resolveAsFarAsItExists({ path: '/home/u/link/store.json' });
 ```
 */
async function resolveAsFarAsItExists({ path, }: { readonly path: string; },): Promise<string> {
  try {
    return await realpath(path,);
  }
  catch (error) {
    causeLogger.debug(`${path} does not resolve, resolving its parent instead: ${caughtValueText(error,)}`,);
    /**
     Parent of the path, whose own resolution carries the part that exists.
     */
    const parent = dirname(path,);
    if (parent === path)
      return path;
    return join(
      await resolveAsFarAsItExists({ path: parent, },),
      basename(path,),
    );
  }
}

/**
 Whether a path lies under a root, by whole path segments.

 @param path - the path

 @param root - the root

 @returns true when the path is the root or below it

 @example
 ```ts
 isUnder({ path: '/a/b/c', root: '/a/b' }); // true
 ```
 */
function isUnder({
  path,
  root,
}: {
  readonly path: string;
  readonly root: string;
},): boolean {
  /**
   Path from the root to the path.
   */
  const between = relative(
    root,
    path,
  );
  return (between === '') || ((!between.startsWith(`..${sep}`,)) && (between !== '..')
    && (!isAbsolute(between,)));
}

/**
 Whether a path lies under the repository the identity names: under its link path (the link's parent resolved), under its
 approved root, or under either with the existing part resolved.

 @param identity - the recorded identity

 @param path - absolute path

 @returns true when the path is under the repository

 @example
 ```ts
 await underRepository({ identity, path });
 ```
 */
async function underRepository({
  identity,
  path,
}: {
  readonly identity: LinkIdentity;
  readonly path: string;
},): Promise<boolean> {
  /**
   The path with its existing part resolved.
   */
  const resolved = await resolveAsFarAsItExists({ path, },);
  /**
   Each recorded root, spelled with its parent resolved so a link that dangles is still where the path goes through.
   */
  const roots = await Promise.all([
    identity.linkPath,
    identity.canonicalRoot,
  ].filter(function recorded(root,): boolean {
    return root !== '';
  },)
    .map(async function spelledRoot(root,): Promise<{
      readonly root: string;
      readonly spelled: string
    }> {
      return {
        root,
        spelled: join(
          await resolveAsFarAsItExists({ path: dirname(root,), },),
          basename(root,),
        ),
      };
    },),);
  return roots.some(function isPathUnder({
    root,
    spelled,
  },): boolean {
    return isUnder({
      path,
      root,
    },) || isUnder({
      path: resolved,
      root: spelled,
    },);
  },);
}

/**
 The plain cause of a store path that cannot be used, when the path is under the recorded repository and the repository link
 check says the repository is not usable.

 @param path - absolute store path that failed

 @param home - home directory that holds the identity file

 @returns the check's plain message, or the empty text when there is no identity file, the path is elsewhere, the repository
 is usable, or the identity file cannot be read

 @example
 ```ts
 const cause = await repositoryLinkCause({ path, home });
 ```
 */
export async function repositoryLinkCause({
  path,
  home,
}: {
  readonly path: string;
  readonly home: string;
},): Promise<string> {
  try {
    /**
     The recorded identity, when there is one.
     */
    const read = await readIdentity(identityPathFor(home,),);
    if (!read.found)
      return '';
    if (!await underRepository({
      identity: read.identity,
      path,
    },))
      return '';
    /**
     The check's answer for the recorded repository.
     */
    const answer = await checkRepositoryLink({
      read,
      probe: realProbe,
    },);
    return answer.ready ? '' : answer.message;
  }
  catch (error) {
    causeLogger.warn(`the repository link identity of ${home} could not be used: ${caughtValueText(error,)}`,);
    return '';
  }
}
