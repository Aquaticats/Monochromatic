import { realpath, } from 'node:fs/promises';

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

/**
 What the filesystem says when a path is resolved to its real place and cannot be.

 @param path - path whose resolution the filesystem refuses

 @returns Text of the refusal, which is what the audit logs of the failure

 @throws {@link Error} when the path resolves, since the case then holds no refusal to compare with

 @example
 ```ts
 // "ENOTDIR: not a directory, realpath '/cats/file/below'"
 const said = await refusalSaid({ path: '/cats/file/below', },);
 ```
 */
export async function refusalSaid({ path, }: { readonly path: string; },): Promise<string> {
  try {
    await realpath(path,);
  }
  catch (error) {
    return caughtValueText(error,);
  }
  throw new Error(`${path} resolves, so the case holds no refusal of the filesystem to compare with`,);
}
