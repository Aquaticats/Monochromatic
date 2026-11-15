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

 A test or a fixture is not a kind of extension, but it is a rule this module keeps, and it is measured the same way: a test is
 written in the language of the code it tests, so its extension says `code`, and which names and folders mark it is counted here
 with the extensions: of the 24010 files the tree held on 2026-11-15, 5513 are tests and 1545 are fixtures. A file only tests read is material
 beside them, whatever its extension, so the fixture lists name folders and suffixes rather than extensions, and
 `whatAChangeToTestsReads` says what a review of a change to tests does with a file whose folder no list names. The two rules cannot be apart: a kind read as code needs its tests known as tests, or a change to tests
 alone is reviewed as though it were the change. The test rule is asked first, so a `.rs` file under `tests/` is a test and not
 the code it looks like.

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

/**
 The name suffixes a test's own file name ends with, which mark it whatever folder it sits in: 1810 `_tests.rs` files, 3371
 `.test.ts` files (most of them `.unit.test.ts`), and 143 `Test.kt` files. The other suffixes are
 the same families in a flavour this repository has none of yet, listed so the first one is a test rather than code.

 A test is marked by the name it carries because a test is written in the language of the code it tests: its extension says
 `code`, and a review that read it as code would judge a test as the change itself. `.bench.ts` is not here: a benchmark is a
 program a reviewer judges, and no brief asks for a break to be planted in one.
 */
export const TEST_FILE_NAME_SUFFIXES: readonly string[] = [
  '_tests.rs',
  '.test.ts',
  '.test.mts',
  '.test.cts',
  '.test.mjs',
  '.test.cjs',
  '.test.js',
  'Test.kt',
  'Tests.kt',
];

/**
 The folder names at any depth whose files are tests whatever their extension: `tests` (178 files, cargo's integration tests
 among them), `test` (147, the Kotlin and Android `src/test` layout and one browser harness's folder), and `androidTest` (7). A
 folder's own name marks its files because cargo puts a package's integration tests under `tests/`, beside the crate rather than
 inside it, so no suffix of such a file tells it from shipped code.
 */
export const TEST_FOLDER_NAMES: ReadonlySet<string> = new Set([
  'tests',
  'test',
  'androidTest',
  '__tests__',
],);

/**
 The folder names at any depth whose files are material only tests read: `fixture` (24 files), `fixtures` (27), `test-fixture`
 (539, whole packages that exist to be parsed or linted by a test), `golden` (83), and `dictionary` (19 fuzz dictionaries).
 `test_support`, `testdata`, `snapshots`, `__snapshots__` and `expected` name no tracked file today, and are listed so the first
 one is a fixture rather than data.

 `seed` is not one of these names: two folders carry it, and one of them, `package-paused/webapp-forge/seed`, is a template
 package whose `src/cli.ts` is a program. The fuzz seed corpora are fixtures by the path part below instead.
 */
export const FIXTURE_FOLDER_NAMES: ReadonlySet<string> = new Set([
  'fixture',
  'fixtures',
  'test-fixture',
  'test_support',
  'testdata',
  'golden',
  'snapshots',
  '__snapshots__',
  'expected',
  'dictionary',
],);

/**
 The parts of a path that mark material only tests read: 13 `stand-in-` files, 17 whose name holds `.fixture.`, 1071 holding
 `-fixture` (the fixture modules of the e2e suites and of the packages that record answers for their tests, which the
 `test-fixture` folders are already among), and the 145 files of a fuzz target's `seed/` corpus under a `*.fuzz` package.

 A part is matched anywhere in the path because a fixture is named for what it records rather than for where it sits, and a file
 named `commit-shape-log-fixture.ts` is a fixture in any folder.
 */
export const FIXTURE_PATH_PARTS: readonly string[] = [
  'stand-in-',
  '.fixture.',
  '-fixture',
  '.fuzz/seed/',
];

/**
 The name suffixes of material only tests read: the 193 files of the repository's two merge corpora (43 `.theirs`, 43 `.ours`, 43
 `.base`, 41 `.status`, 21 `.out` and 8 `.resolved`), 19 fuzz dictionaries, and the 9 `.stderr` files a UI test compares against.
 `.snap` and `.golden` name no tracked file today, and are listed so the first snapshot is a fixture rather than data.
 */
export const FIXTURE_NAME_SUFFIXES: readonly string[] = [
  '.theirs',
  '.ours',
  '.base',
  '.resolved',
  '.status',
  '.out',
  '.stderr',
  '.dict',
  '.snap',
  '.golden',
];

/**
 The extensions whose files hold prose a person reads: a note, a page of the site's own text, a diff, a PDF. No test reads one,
 so a change that holds one beside a test is not a change to tests alone.

 `.txt` is not here. Of the 518 text files this repository tracks, some are licences and notes, but others are what a test
 compares against (`forbidden-strings/cli/data/builtin-rules.txt`, the `fixtures/*.js.txt` of the browserslist crate), and a
 `.txt` beside a test is read as material the test may read rather than as prose.
 */
export const PROSE_FILE_EXTENSIONS: ReadonlySet<string> = new Set([
  '.md',
  '.mdx',
  '.patch',
  '.pdf',
],);

/**
 The folders a path sits in, which are every segment before its own name.

 @param path - a path from the top of the checkout

 @returns the folder segments

 @example
 ```ts
 const folders = foldersOf('package/a/tests/shim.rs',);
 ```
 */
function foldersOf(path: string,): readonly string[] {
  /**
   Every segment of the path, of which the last is the file's own name.
   */
  const segments = path.split('/',);
  return segments.slice(
    0,
    -1,
  );
}

/**
 Says whether a path is one of the files a review leaves out beside a change: the tests that check it, which a reviewer cannot
 judge as the change itself, and the fixtures and stand-in programs they use.

 The test lists are asked first, so a `.rs` file under `tests/` is a test and not the code it looks like, and a `.json` under a
 `fixtures/` folder is material the tests beside it read rather than data a review leaves out.

 @param path - a path from the top of the checkout

 @returns whether the path is a test or a fixture

 @example
 ```ts
 const isATest = isATestOrAFixture({ path: 'src/gate_tests.rs', },);
 ```
 */
export function isATestOrAFixture({ path, }: { readonly path: string },): boolean {
  return TEST_FILE_NAME_SUFFIXES.some(function isATestName(suffix,) {
    return path.endsWith(suffix,);
  },)
    || FIXTURE_NAME_SUFFIXES.some(function isAFixtureName(suffix,) {
      return path.endsWith(suffix,);
    },)
    || FIXTURE_PATH_PARTS.some(function isAFixturePart(part,) {
      return path.includes(part,);
    },)
    || foldersOf(path,)
      .some(function isATestOrFixtureFolder(folder,) {
        return TEST_FOLDER_NAMES.has(folder,) || FIXTURE_FOLDER_NAMES.has(folder,);
      },);
}

/**
 What a review of a change to tests reads a file as: a test or a fixture by the measured lists, material a test may read although
 no list names its folder, or prose no test reads.
 */
export type WhatAChangeToTestsReads =
  | 'a-test-or-a-fixture'
    | 'material-a-test-may-read'
    | 'a-note';

/**
 Says what a review of a change to tests does with a file.

 The first answer is the measured lists. The second is every file beside a test that is neither a test nor a fixture nor prose: a
 recorded answer, a picture, a lockfile, a file of an extension no list names. The writer cannot read imports to know whether a
 test reads one, so it reads every such file: a file read that a test never opens costs a reviewer one file, while a recorded
 answer left out of the change it belongs to got that whole change answered as nothing to review on 2026-11-14, which is the
 defect this answer exists for.

 The third answer is prose a person reads, which no test reads, so a note beside a test leaves the change not a change to tests
 alone. `material-of-a-review.ts` refuses that range, with the words its own answer carries.

 @param path - a path from the top of the checkout

 @returns what a review of the change reads the file as

 @example
 ```ts
 const readAs = whatAChangeToTestsReads({ path: 'package/a/src/await-run.unit.test.ts', },);
 ```
 */
export function whatAChangeToTestsReads({ path, }: { readonly path: string },): WhatAChangeToTestsReads {
  if (isATestOrAFixture({ path, },)) {
    return 'a-test-or-a-fixture';
  }
  return PROSE_FILE_EXTENSIONS.has(extensionOfAPath({ path, },),)
    ? 'a-note'
    : 'material-a-test-may-read';
}
