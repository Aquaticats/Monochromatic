/**
 The source snapshot the native end-to-end suite compiles its executables from.
 */
import { copyWrapperSnapshot, } from './native-wrapper-snapshot.ts';

/**
 Copy the native crate and its path dependencies into the build context; the compile stage needs nothing else.
 Every copy settles before this returns or throws.

 @param context - build context directory receiving the repository-shaped snapshot

 @example
 ```ts
 await copyEndToEndSnapshot({ context: context.path });
 ```
 */
export async function copyEndToEndSnapshot({ context, }: {
  readonly context: string;
},): Promise<void> {
  await copyWrapperSnapshot({ context, });
}
