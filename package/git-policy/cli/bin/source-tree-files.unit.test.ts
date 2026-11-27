import {
  mkdir,
  mkdtempDisposable,
  symlink,
  writeFile,
} from 'node:fs/promises';
import { homedir, } from 'node:os';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { sourceTreeFiles, } from './source-tree-files.ts';

/**
 Name of the file a control writes where it needs one that is certainly not a source of anything.
 */
const realFileName = 'a.rs';

/**
 Paths of the files one control found, relative to the folder it walked.

 @param options - the folder walked and what the walk answered

 @returns the answer, with every path under the walked folder removed

 @example
 ```ts
 relativeTo({ root: '/f/tree', files: ['/f/tree/a.rs'] }); // => ['a.rs']
 ```
 */
function relativeTo({
  root,
  files,
}: {
  readonly root: string;
  readonly files: readonly string[];
},): readonly string[] {
  return files.map(function underRoot(file) {
    return file.slice(root.length + 1,);
  },);
}

await describe({
  name: sourceTreeFiles.name,
  children: [
    //region A link back into the tree

    it({
      name: 'reads a folder a link points back into once, and never through the link',
      fn: async () => {
        /**
         Folder this control walks, which is the control's own and goes with it.
         */
        await using root = await mkdtempDisposable(join(
          homedir(),
          'temp',
          'agent',
          'source-tree-files-',
        ));
        /**
         Folder whose files a `uses/` link of the other folder points back at.
         */
        const used = join(
          root.path,
          'used',
        );
        /**
         Folder holding the link a source tree would hold as its `uses/` folder.
         */
        const using = join(
          root.path,
          'using',
        );
        await mkdir(
          used,
          { recursive: true, },
        );
        await mkdir(using,);
        await writeFile(
          join(
            root.path,
            realFileName,
          ),
          'fn top() {}\n',
        );
        await writeFile(
          join(
            used,
            'used_by_the_other_folder.rs',
          ),
          'fn used() {}\n',
        );
        await symlink(
          join(
            '..',
            'used',
          ),
          join(
            using,
            'uses',
          ),
          'dir',
        );
        /**
         What the walk answered.
         */
        const files = await sourceTreeFiles({ root: root.path, });
        expect(relativeTo({
          root: root.path,
          files,
        },),).toEqual([
          realFileName,
          'used/used_by_the_other_folder.rs',
        ],);
      },
    },),

    it({
      name: 'ends on a link that points at the folder holding it',
      fn: async () => {
        /**
         Folder this control walks.
         */
        await using root = await mkdtempDisposable(join(
          homedir(),
          'temp',
          'agent',
          'source-tree-files-loop-',
        ));
        await mkdir(
          join(
            root.path,
            'nested',
          ),
          { recursive: true, },
        );
        await writeFile(
          join(
            root.path,
            realFileName,
          ),
          'fn top() {}\n',
        );
        await writeFile(
          join(
            root.path,
            'nested',
            'under_it.rs',
          ),
          'fn nested() {}\n',
        );
        // A recursive read that descends links does not end on this folder, which is why no control runs the old walk
        // against it: the link points at the folder that holds it.
        await symlink(
          '.',
          join(
            root.path,
            'loop',
          ),
          'dir',
        );
        /**
         What the walk answered.
         */
        const files = await sourceTreeFiles({ root: root.path, });
        expect(relativeTo({
          root: root.path,
          files,
        },),).toEqual([
          realFileName,
          'nested/under_it.rs',
        ],);
      },
    },),

    it({
      name: 'leaves out a link to a file, so no file is read through it',
      fn: async () => {
        /**
         Folder this control walks.
         */
        await using root = await mkdtempDisposable(join(
          homedir(),
          'temp',
          'agent',
          'source-tree-files-file-link-',
        ));
        /**
         Folder holding the file the link points at.
         */
        const real = join(
          root.path,
          'real',
        );
        await mkdir(
          real,
          { recursive: true, },
        );
        await writeFile(
          join(
            real,
            realFileName,
          ),
          'fn real() {}\n',
        );
        await symlink(
          join(
            'real',
            realFileName,
          ),
          join(
            root.path,
            'linked.rs',
          ),
          'file',
        );
        /**
         What the walk answered.
         */
        const files = await sourceTreeFiles({ root: root.path, });
        expect(relativeTo({
          root: root.path,
          files,
        },),).toEqual([`real/${realFileName}`,],);
      },
    },),

    //endregion A link back into the tree
  ],
},);
