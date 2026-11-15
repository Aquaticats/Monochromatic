/**
 Which files a review reads, by the extension of their name: the code of a change, and the kinds beside it that a review leaves
 out.

 The kind is read from the extension rather than from a file's content, because a review is decided from a diff: the ranges are
 asked which paths they touched, and nothing in that answer says what a file holds. The lists are measured against what this
 repository tracks rather than reasoned from memory. On 2026-11-14 the tree this change started from named 23845 files in 89
 extensions: 16053 of them in the 24 code extensions below, and 7792 in the 64 left-out ones or in no extension at all. The same
 measurement on the tree this change leaves names 23847, the two more being this module and its test, which are `.ts` files a
 review reads. The check kept beside this change's report lists every extension with its count and the kind this module gives it,
 and the report says why each kind is read or left out.

 A test or a fixture is not a kind here. Which file is one is the rule in `material-of-a-review.ts` (`_tests.rs` and `.test.ts`
 suffixes, `test_support` and `stand-in-` paths, and a folder named `tests`), because a test is written in the language of the
 code it tests and no extension tells it apart. That rule is asked before this one, so a `.rs` file under `tests/` is a test and
 not the code it looks like.

 An extension in neither list is a kind this writer does not know. `whatAReviewDoesWithAFile` answers
 `reads-it-as-code-of-an-unlisted-kind` for it, so a review reads it instead of dropping it, and the line the brief writer
 prints names the file, because a change nobody listed must not go unread. That answer is also why a `.cjs` source and the
 `.d.cts` types beside it are read: both were in no list until this change, so a range that changed them, their test and their
 note was answered `nothing to review` on 2026-11-14, and that branch stood unreviewed until someone wrote its brief by hand.

 @module
 */

/**
 What a review does with a file, read from the extension of the file's name: it reads the file as code, it leaves the file out,
 or it reads the file as code of a kind its lists do not name.

 A file a review leaves out holds what a review cannot judge as the change: a document, data a program reads, what a tool wrote, a
 picture, a sound or a font. A file of a kind no list names is one whose extension this writer does not know: a review reads it as
 code rather than drop it, because a language nobody listed here yet is more likely new code in this repository than a kind a
 review has no use for.
 */
export type WhatAReviewDoesWithAFile =
  | 'reads-it-as-code'
    | 'leaves-it-out'
    | 'reads-it-as-code-of-an-unlisted-kind';

/**
 The extensions whose files a review reads as the code of a change: 16053 of the 23845 files this repository tracked on
 2026-11-14, in 24 extensions of the languages its packages are written in and of the markup those packages render.

 A language is listed when a file of it holds a program, a page, a style or a template that runs or is interpreted, which is what
 a reviewer judges. An extension shared by code and data is not listed here on the strength of one of its files: `.json`, `.toml`
 and `.yaml` are left out by this module and read only where a name in `material-of-a-review.ts` decides a build, while `.html`,
 `.css` and `.xml` are listed because the files this repository holds in them are pages, styles, manifests and layouts that run.
 */
const CODE_FILE_EXTENSION_GROUPS: readonly (readonly string[])[] = [
  // TypeScript and JavaScript, the repository's own languages: 10654 `.ts` files, and the 124 module files beside them.
  [
    '.ts',
    '.mts',
    '.cts',
    '.mjs',
    '.cjs',
    '.js',
  ],
  // Rust: 4520 files.
  ['.rs',],
  // Kotlin: 313 `.kt` files, and the 5 `.kts` Gradle task scripts a build runs.
  [
    '.kt',
    '.kts',
  ],
  // The UI languages of the desktop programs: 43 `.slint` files, and 8 `.qml` files of the Qt variant and its bench.
  [
    '.slint',
    '.qml',
  ],
  // A page, a style and the markup a program reads: 163 `.html`, 55 `.css` and 149 `.xml` files, among them Android's manifests
  // and layouts, an IDE plugin's descriptor, and the design pages a UI was chosen from.
  [
    '.html',
    '.css',
    '.xml',
  ],
  // C and C++: one reproduction a troubleshooting record kept, and the Qt variant's log writer.
  [
    '.c',
    '.cpp',
  ],
  // The scripts a task, a test harness or a Windows-only path runs: 2 `.sh`, 1 `.ps1`, and 2 `.bat` files, both of them
  // Gradle's wrapper, which a review reads because a batch file is a program.
  [
    '.sh',
    '.ps1',
    '.bat',
  ],
  // SQL, which decides what a database holds: 7 migrations, a schema and a fixture; the template a build fills to write a
  // shipped file; R8's rules program for the Android build; OpenTofu's infrastructure code; and the 2 SPIR-V shader programs a
  // troubleshooting record reproduced.
  [
    '.sql',
    '.hbs',
    '.pro',
    '.tf',
    '.spvasm',
  ],
];

/**
 Every extension whose files a review reads as code, as one set to ask.
 */
export const CODE_FILE_EXTENSIONS: ReadonlySet<string> = new Set(CODE_FILE_EXTENSION_GROUPS.flat(),);

/**
 The extensions whose files a review leaves out, grouped by what such a file holds: 7792 of the 23845 files this repository
 tracked on 2026-11-14, in 64 extensions and in the names that carry none.

 A name that carries no extension is left out too, and none of the 159 such files this repository tracked was code: they are
 licences, settings, fuzz seeds beside a fuzz target, generated wrapper scripts, and container files named by
 `material-of-a-review.ts`'s build-file words, which are read by name. An extension is listed here only when every file of it this
 repository holds is something a review cannot judge as the change.
 */
const LEFT_OUT_FILE_EXTENSION_GROUPS: readonly (readonly string[])[] = [
  // A document a person reads: 3211 notes, 36 pages of the site's own text, 518 text files (among them the licences), 77 patches,
  // one PDF, and one outline of a feed's subscriptions.
  [
    '.md',
    '.mdx',
    '.txt',
    '.patch',
    '.pdf',
    '.opml',
  ],
  // What a tool wrote rather than what a person keeps: 35 lockfiles, 21 recorded outputs of a merge, 43 `.theirs`, 43 `.ours`, 43
  // `.base`, 41 `.status` and 8 `.resolved` files beside them, 9 `.stderr` files of an expected error, and 3 built jars.
  [
    '.lock',
    '.out',
    '.theirs',
    '.ours',
    '.base',
    '.status',
    '.resolved',
    '.stderr',
    '.jar',
  ],
  // Data a program reads, or a setting for one: 1091 `.json`, 564 `.toml`, 33 `.yml`, 24 `.jsonc`, 21 `.zip`, 115 `.gz`, 19
  // `.dict` and the rest of this group, which hold dependencies, tables, schemas, a fuzzer's dictionary, service units, desktop
  // entries and a database file.
  [
    '.json',
    '.jsonc',
    '.toml',
    '.yaml',
    '.yml',
    '.properties',
    '.conf',
    '.dict',
    '.jsonl',
    '.db',
    '.db-wal',
    '.desktop',
    '.service',
    '.timer',
    '.socket',
    '.plist',
    '.theme',
    '.webmanifest',
    '.code-workspace',
    '.example',
    '.origin',
    '.sha256',
    '.pub',
    '.RuleSetProvider',
    '.list',
    '.codepoints',
    '.zip',
    '.gz',
  ],
  // A picture, a sound or a font, which a reviewer looks at or hears rather than reads as a change: 1549 `.png`, 20 `.jpg`, 13
  // `.svg`, 9 `.avif`, 14 `.woff2`, 6 `.ttf`, 2 `.ico`, 2 `.opus`, 2 `.m4a`, and one each of `.icns`, `.mp3`, `.ogg`, `.flac` and
  // `.wav`.
  [
    '.png',
    '.jpg',
    '.avif',
    '.svg',
    '.ico',
    '.icns',
    '.woff2',
    '.ttf',
    '.mp3',
    '.ogg',
    '.m4a',
    '.flac',
    '.opus',
    '.wav',
  ],
  // The suffix a container file's own name carries: `Containerfile.release`, `Containerfile.windows-gnu`, `Containerfile.icons`,
  // `Containerfile.sign`, `Containerfile.alpine`, `native-test-base.Containerfile` and `native-test-base.Dockerfile`. The suffix
  // names a variant of a build, not a kind of content, and `material-of-a-review.ts` reads each of those files by its name.
  [
    '.Containerfile',
    '.Dockerfile',
    '.release',
    '.windows-gnu',
    '.icons',
    '.sign',
    '.alpine',
  ],
];

/**
 Every extension whose files a review leaves out, as one set to ask.
 */
export const LEFT_OUT_FILE_EXTENSIONS: ReadonlySet<string> = new Set(LEFT_OUT_FILE_EXTENSION_GROUPS.flat(),);

/**
 Reads the extension a path's own name ends with, as the lists above name it.

 A git path separates its folders with `/` alone, so the name is what follows the last `/`. A leading dot makes a hidden name
 rather than an extension (`.gitignore` carries none), and a trailing dot carries none either.

 @param path - a path from the top of the checkout

 @returns the extension including its dot, or an empty string when the name carries none

 @example
 ```ts
 const extension = extensionOfAPath({ path: 'package/a/src/index.d.cts', },);
 ```
 */
export function extensionOfAPath({ path, }: { readonly path: string }): string {
  /**
   The file's own name, which the extension belongs to.
   */
  const name = path.slice(path.lastIndexOf('/',) + 1,);
  /**
   Where the name's last dot sits.
   */
  const dot = name.lastIndexOf('.',);
  return ((dot > 0) && (dot < (name.length - 1)))
    ? name.slice(dot,)
    : '';
}

/**
 Says what a review does with a file, which is what the extension of its name says about it.

 A name that carries no extension is left out, and that is measured too: of the 159 such files this repository tracked on
 2026-11-14, none was code, because they are licences, settings, fuzz seeds beside a fuzz target, generated wrapper scripts, and
 container files that `material-of-a-review.ts` reads by name instead.

 @param path - a path from the top of the checkout

 @returns the kind of change a review reads the file as, or that it leaves the file out

 @example
 ```ts
 const kind = whatAReviewDoesWithAFile({ path: 'package/a/src/gate.rs', },);
 ```
 */
export function whatAReviewDoesWithAFile({ path, }: { readonly path: string },): WhatAReviewDoesWithAFile {
  /**
   The extension the lists are asked about.
   */
  const extension = extensionOfAPath({ path, },);
  if (extension === '') {
    return 'leaves-it-out';
  }
  if (CODE_FILE_EXTENSIONS.has(extension,)) {
    return 'reads-it-as-code';
  }
  return LEFT_OUT_FILE_EXTENSIONS.has(extension,)
    ? 'leaves-it-out'
    : 'reads-it-as-code-of-an-unlisted-kind';
}
