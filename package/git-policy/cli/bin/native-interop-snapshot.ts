/**
 The source snapshot the native interoperability runner builds its image from.
 */
import {
  join,
  resolve,
} from 'node:path';

import { copyWrapperSnapshot, } from './native-wrapper-snapshot.ts';

/**
 Copy the native crate, its path dependencies and the interoperability drivers into the build context.
 Every copy settles before this returns or throws.

 @param context - build context directory receiving the repository-shaped snapshot

 @example
 ```ts
 await copyInteropSnapshot({ context: context.path });
 ```
 */
export async function copyInteropSnapshot({ context, }: {
  readonly context: string;
},): Promise<void> {
  await copyWrapperSnapshot({
    context,
    additionalEntries: [
      {
        from: resolve('../../../clippy.toml'),
        to: join(
          context,
          'clippy.toml',
        ),
      },
      {
        from: resolve('native-interop'),
        to: join(
          context,
          'native-interop',
        ),
      },
    ],
  });
}
