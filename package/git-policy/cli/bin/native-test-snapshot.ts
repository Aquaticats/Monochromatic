/**
 The source snapshot the native wrapper gate builds its test image from.
 */
import {
  join,
  resolve,
} from 'node:path';

import { repositorySettingsFileName, } from './native-test-settings-file.ts';
import { copyWrapperSnapshot, } from './native-wrapper-snapshot.ts';

/**
 Copy the native crate and its path dependencies into the build context, plus the repository files its tests read.
 Every copy settles before this returns or throws, so removing the context afterwards never races an in-flight copy.

 @param context - build context directory receiving the repository-shaped snapshot

 @example
 ```ts
 await copySourceSnapshot({ context: context.path });
 ```
 */
export async function copySourceSnapshot({ context, }: {
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
      // The configuration control in `src/native/config/config_repository_tests.rs` reads the repository's own file.
      {
        from: resolve('../../../', repositorySettingsFileName(),),
        to: join(
          context,
          repositorySettingsFileName(),
        ),
      },
    ],
  });
}
