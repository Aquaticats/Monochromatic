/**
 Tests for whose front matter the page carries.

 THE OWNER'S RULE OF 2026-09-08, after one entry's ninth pass renamed the
 person in slice zero while the body kept the archive's name, and a census
 found nine of the last ten read pages rewriting `desc` or `alias`: the
 archive's front matter is published as it stands, and the lanes render it
 only where the archive never translated it. Measured over the pinned corpus
 on the day of the decision: 23 of 92 archives name their directory, 8 of
 them because the source does too, and the other 15 all stand by the
 exemptions of 2026-09-07, so every pinned archive stands and the rendering
 path is kept for the directory-id shape alone.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  archiveFrontMatterStands,
  frontMatterAuthorityOf,
} from '../../dist/final/node/index.mjs';

//region Fixtures

/**
 Source metadata of an entry whose archive renders a name of its own.
 */
const MOONCAT_SOURCE = '---\nname: 月见小猫\ninfo:\n    alias: 小月, Tsukimi, Mooncat\n---\n\n正文。\n';

/**
 The archive's translated metadata, whose display name is editorial.
 */
const MOONCAT_ARCHIVE = '---\nname: Tsukimi\ninfo:\n    alias: Tsukimi Koneko, Mooncat\n---\n\nBody.\n';

/**
 A source naming the person by a name of her own.
 */
const NAMED_SOURCE = '---\nname: 猫猫\ninfo:\n  alias: 猫咪\n---\n\n正文。\n';

/**
 An archive still showing the directory id with nothing Latin beside it:
 the directory-id shape, never translated.
 */
const FOLDER_ARCHIVE = '---\nname: EntryId\ninfo:\n  alias: 猫咪\n---\n\nBody.\n';

/**
 An archive showing the directory id beside an English rendering.
 */
const FOLDER_WITH_ALIAS_ARCHIVE = '---\nname: EntryId\ninfo:\n  alias: Maomao\n---\n\nBody.\n';

//endregion Fixtures

await describe({
  name: archiveFrontMatterStands.name,
  children: [
    it({
      name: 'STANDS where the archive rendered a name of its own, one entry\'s ninth pass: the display '
        + 'name is the memorial\'s choice, whatever the judges would render 月见小猫 as',
      fn: async () => {
        expect(archiveFrontMatterStands({
          entryId: 'mooncat',
          sourceText: MOONCAT_SOURCE,
          archiveText: MOONCAT_ARCHIVE,
        },),).toBe(true,);
        expect(frontMatterAuthorityOf({
          entryId: 'mooncat',
          sourceText: MOONCAT_SOURCE,
          archiveText: MOONCAT_ARCHIVE,
        },),).toBe('archive',);
      },
    },),
    it({
      name: 'DOES NOT STAND where the archive shows the directory id with nothing Latin beside it while the '
        + 'source names the person, the directory-id shape, which is the one shape the lanes still render',
      fn: async () => {
        expect(archiveFrontMatterStands({
          entryId: 'EntryId',
          sourceText: NAMED_SOURCE,
          archiveText: FOLDER_ARCHIVE,
        },),).toBe(false,);
        expect(frontMatterAuthorityOf({
          entryId: 'EntryId',
          sourceText: NAMED_SOURCE,
          archiveText: FOLDER_ARCHIVE,
        },),).toBe('rendered',);
      },
    },),
    it({
      name: 'STANDS by each exemption of 2026-09-07: the source names the person by the handle too, the id '
        + 'is the pinyin of the source\'s name, or an English rendering sits beside the id in the alias',
      fn: async () => {
        expect(archiveFrontMatterStands({
          entryId: 'EntryId',
          sourceText: '---\nname: EntryId\ninfo:\n  alias: EntryId\n---\n\n正文。\n',
          archiveText: '---\nname: EntryId\ninfo:\n  alias: EntryId\n---\n\nBody.\n',
        },),).toBe(true,);
        expect(archiveFrontMatterStands({
          entryId: 'maotong',
          sourceText: '---\nname: 猫童\ninfo:\n  alias: 小童\n---\n\n正文。\n',
          archiveText: '---\nname: maotong\ninfo:\n  alias: 小童\n---\n\nBody.\n',
        },),).toBe(true,);
        expect(archiveFrontMatterStands({
          entryId: 'EntryId',
          sourceText: NAMED_SOURCE,
          archiveText: FOLDER_WITH_ALIAS_ARCHIVE,
        },),).toBe(true,);
      },
    },),
    it({
      name: 'DOES NOT STAND where the archive declares no metadata, since a source-only front matter is '
        + 'inserted by the lanes as it always was, and STANDS where the source declares none',
      fn: async () => {
        expect(archiveFrontMatterStands({
          entryId: 'EntryId',
          sourceText: NAMED_SOURCE,
          archiveText: 'Body.\n',
        },),).toBe(false,);
        expect(archiveFrontMatterStands({
          entryId: 'EntryId',
          sourceText: '正文。\n',
          archiveText: MOONCAT_ARCHIVE,
        },),).toBe(true,);
      },
    },),
  ],
},);
