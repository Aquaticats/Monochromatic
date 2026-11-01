/**
 Tests for the settled rendering audit's driver: what it buys, in what
 order, what it prints before buying, and what one audited row carries.

 NO CASE REACHED THESE BEFORE. `capped` already carried one real cap defect
 (the args suite's header records it), `printPopulation` is the free reading
 a run prints before any roster is woken, and `auditOne` is where a subject's
 provenance and the roster's answers become the row every later reading
 interprets. Each is exported through the barrel for exactly this.

 THE CLIENT IS HANDED IN, which is the rendering-6 shape: `main` builds one
 per run and `auditOne` counts every subject into it, so a scripted client
 here sees every model the roster asks and answers each with silence. The
 roster is the production one, read from the run configuration, so the
 cases assert against what the client was asked rather than against a list
 copied here.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { CacheFileUnreadableError, } from '@monochromatic-dev/translation-repair-reference';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  auditOne,
  capped,
  eligibleSubjects,
  printPopulation,
  sameAuditedText,
  type SettledArtifactReading,
  type SettledAuditSubject,
  type SettledIdentity,
  type SettledVerification,
  withCitedReferences,
} from '../../dist/final/node/index.mjs';
import { directoryRefusalText, } from '@monochromatic-dev/translation-repair-test-fixture/ts/cache-file-refusal.test-fixture.ts';
import { infoLinesDuring, } from '@monochromatic-dev/translation-repair-typography/ts/console-warn-lines.test-fixture.ts';
import { refusalOrder, } from '@monochromatic-dev/translation-repair-test-fixture/ts/refusal-order.test-fixture.ts';
import { relayingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/rejection-of.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Original every subject carries.
 */
const SOURCE_TEXT = '三只猫住在书店的阁楼里。她们不吃罐头，每天傍晚只喝一碗温牛奶。';

/**
 Rendering every subject carries.
 */
const CANDIDATE_TEXT = 'Three cats live in the attic of the bookshop. They do not eat canned '
  + 'food, and every evening they drink one bowl of warm milk.';

/**
 Run set every fixture belongs to.
 */
const RUN_SET = 'naptime-20260825';

/**
 Built pipeline every fixture records.
 */
const DIGEST = 'sha256-tree-v1:cafef00d';

/**
 Characters in a SHA-1 object id.
 */
const OBJECT_ID_LENGTH = 40;

/**
 Corpus commit every fixture records.
 */
const CORPUS_SHA = 'b'.repeat(OBJECT_ID_LENGTH,);

/**
 Cap meaning every subject, as the args module spells it internally.
 */
const EVERY_SUBJECT = -1;

/**
 Cap that buys a prefix.
 */
const SMALL_BUY = 2;

/**
 Declared identity a producing run had, in the shape the subject carries.
 */
const IDENTITY_CONTEXT = 'Mittens is written Mao Mao on the Chinese side';

/**
 What the pages an original cites say, in the shape the reference reader returns.
 */
const REFERENCE_CONTEXT = 'Reference 1 (https://example.org/mittens): Mittens naps in the bakery window.';

/**
 Whole original page every subject was cut from unless a case says, linking
 the page the references read.
 */
const PAGE_TEXT = `${SOURCE_TEXT}\n\n[毛毛的博客](https://example.org/mittens)\n`;

/**
 A second entry's whole original, which links nowhere.
 */
const OTHER_PAGE_TEXT = '虎斑猫在屋顶上晒太阳。\n';

/**
 Builds one audit subject.

 @param entryId - corpus entry

 @param sliceIndex - slice index

 @param auditsArchiveText - whether it audits the archive's own English

 @param identity - what the producing run declared, none unless a case says

 @param pageSourceText - whole original the slice was cut from, the linking
 page unless a case says

 @returns Subject as the input module offers one

 @example
 ```ts
 const subject = subjectAt({ entryId: 'mittens', sliceIndex: 0, auditsArchiveText: false, },);
 ```
 */
function subjectAt(
  {
    entryId,
    sliceIndex,
    auditsArchiveText,
    identity = { kind: 'none', },
    pageSourceText = PAGE_TEXT,
  }: {
    readonly entryId: string;
    readonly sliceIndex: number;
    readonly auditsArchiveText: boolean;
    readonly identity?: SettledIdentity;
    readonly pageSourceText?: string;
  },
): SettledAuditSubject {
  return {
    runSet: RUN_SET,
    entryId,
    artifactDigest: DIGEST,
    corpusSha: CORPUS_SHA,
    sliceIndex,
    deliveryKind: auditsArchiveText ? 'incumbent-retained' : 'replacement-shipped',
    auditsArchiveText,
    sourceText: SOURCE_TEXT,
    candidateText: CANDIDATE_TEXT,
    pageSourceText,
    pageRelation: { kind: 'survives', },
    identity,
  };
}

/**
 Builds one artifact reading.

 @param entryId - corpus entry

 @param subjects - slices it offers

 @param verification - provenance answer, verified unless a case says

 @returns Reading as the input module returns one

 @example
 ```ts
 const reading = readingOf({ entryId: 'mittens', subjects, },);
 ```
 */
function readingOf(
  {
    entryId,
    subjects,
    verification = { kind: 'verified', },
  }: {
    readonly entryId: string;
    readonly subjects: readonly SettledAuditSubject[];
    readonly verification?: SettledVerification;
  },
): SettledArtifactReading {
  return {
    runSet: RUN_SET,
    artifactFile: `${entryId}.json`,
    entryId,
    artifactDigest: DIGEST,
    verification,
    subjects,
  };
}

/**
 Two subjects of one entry, one archive and one fresh.
 */
const MITTENS = [
  subjectAt({
    entryId: 'mittens',
    sliceIndex: 0,
    auditsArchiveText: true,
  },),
  subjectAt({
    entryId: 'mittens',
    sliceIndex: 1,
    auditsArchiveText: false,
  },),
];

/**
 One subject of a second entry.
 */
const TABBY = [subjectAt({
  entryId: 'tabby',
  sliceIndex: 0,
  auditsArchiveText: false,
},),];

/**
 Client answering every auditor with silence and recording who was asked
 and what it was shown.

 @param asked - model ids, one per call, appended as calls arrive

 @param shown - every request's messages as JSON, appended as calls arrive

 @returns Client the driver calls

 @example
 ```ts
 const client = quietClient({ asked: [], shown: [], },);
 ```
 */
function quietClient(
  {
    asked,
    shown,
  }: {
    readonly asked: string[];
    readonly shown: string[];
  },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by the audit',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      asked.push(request.modelId,);
      shown.push(JSON.stringify(request.messages,),);

      /**
       An auditor that found nothing.
       */
      const quiet: unknown = {
        verdict: 'no-defect-found',
        findings: [],
      };
      if (!request.validate(quiet,))
        throw new Error('scripted payload failed the guard',);

      return {
        kind: 'ok',
        value: quiet,
        rawText: JSON.stringify(quiet,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused by the audit',);
    },
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: capped.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'BUYS EVERY SUBJECT under the every-subject sentinel, which is what a run with no '
            + '--cap asks for',
          fn: async () => {
            expect(capped({
              eligible: MITTENS,
              cap: EVERY_SUBJECT,
            },),).toEqual(MITTENS,);
          },
        },),

        it({
          name: 'BUYS NOTHING at zero, which is the wiring check that reads the archive and asks '
            + 'nobody',
          fn: async () => {
            expect(capped({
              eligible: MITTENS,
              cap: 0,
            },),).toEqual([],);
          },
        },),

        it({
          name: 'BUYS THE PREFIX a positive cap allows, in the order the subjects arrived',
          fn: async () => {
            expect(capped({
              eligible: [
                ...MITTENS,
                ...TABBY,
              ],
              cap: SMALL_BUY,
            },),).toEqual(MITTENS,);
          },
        },),
      ],
    },),

    describe({
      name: eligibleSubjects.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'OFFERS every subject of every reading, in archive order, when no entry was named',
          fn: async () => {
            expect(eligibleSubjects({
              readings: [
                readingOf({
                  entryId: 'mittens',
                  subjects: MITTENS,
                },),
                readingOf({
                  entryId: 'tabby',
                  subjects: TABBY,
                },),
              ],
              onlyIds: [],
            },),).toEqual([
              ...MITTENS,
              ...TABBY,
            ],);
          },
        },),

        it({
          name: 'KEEPS only the named entries, so a capped buy is a fraction of what --only left '
            + 'rather than of the whole archive',
          fn: async () => {
            expect(eligibleSubjects({
              readings: [
                readingOf({
                  entryId: 'mittens',
                  subjects: MITTENS,
                },),
                readingOf({
                  entryId: 'tabby',
                  subjects: TABBY,
                },),
              ],
              onlyIds: ['tabby',],
            },),).toEqual(TABBY,);
          },
        },),

        it({
          name: 'REFUSES an entry the archive does not hold, beside one it does, where the audit once bought the '
            + 'rest and named the stray nowhere (ledger B76)',
          fn: async () => {
            expect(function asksForAStray(): void {
              eligibleSubjects({
                readings: [
                  readingOf({
                    entryId: 'mittens',
                    subjects: MITTENS,
                  },),
                  readingOf({
                    entryId: 'tabby',
                    subjects: TABBY,
                  },),
                ],
                onlyIds: [
                  'tabby',
                  'tabbby',
                ],
              },);
            },).toThrow('--only asks for "tabbby", which the settled archive does not hold',);
          },
        },),
      ],
    },),

    describe({
      name: printPopulation.name,
      children: [
        it({
          name: 'PRINTS one line per artifact with every count beside its denominator: subjects, '
            + 'retained, replaced, displaced, undecided and the verification',
          fn: async (ctx) => {
            using printed = relayingConsoleLog({ sinon: ctx.sinon, },);

            printPopulation({
              readings: [readingOf({
                entryId: 'mittens',
                subjects: MITTENS,
              },),],
            },);

            expect(printed.lines,).toEqual([
              `${RUN_SET}/mittens.json  subjects=2 retained=1 replaced=1 displaced=0 undecided=0 `
                + 'verification=verified',
            ],);
          },
        },),

        it({
          name: 'SAYS WHAT A REFUSED VERIFICATION OBJECTED TO on its own line, since a slicing that '
            + 'moved under a settled artifact is the loudest thing this free reading can find',
          fn: async (ctx) => {
            using printed = relayingConsoleLog({ sinon: ctx.sinon, },);

            printPopulation({
              readings: [readingOf({
                entryId: 'mittens',
                subjects: MITTENS,
                verification: {
                  kind: 'refused',
                  detail: 'the slicing moved',
                },
              },),],
            },);

            expect(printed.lines[0]?.endsWith('verification=refused',),).toBe(true,);
            expect(printed.lines[1],).toBe('   REFUSED: the slicing moved',);
          },
        },),

        it({
          name: 'NAMES THE RECIPE HALVES an unverifiable artifact lacks, so a reader can decide '
            + 'whether the gap explains the disagreement',
          fn: async (ctx) => {
            using printed = relayingConsoleLog({ sinon: ctx.sinon, },);

            printPopulation({
              readings: [readingOf({
                entryId: 'mittens',
                subjects: MITTENS,
                verification: {
                  kind: 'unverifiable',
                  unrecorded: [
                    'sectionPairing',
                    'blockPairing',
                  ],
                  detail: 'the rebuild guessed both halves',
                },
              },),],
            },);

            expect(printed.lines[0]?.endsWith('verification=unverifiable',),).toBe(true,);
            expect(printed.lines[1],)
              .toBe('   UNVERIFIABLE (records no sectionPairing, blockPairing): the rebuild guessed both halves',);
          },
        },),
      ],
      concurrency: 1,
    },),

    describe({
      name: auditOne.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'BUILDS ONE ROW from the subject\'s provenance and the roster\'s answers, one voice row '
            + 'per model the client was asked, with the texts digested rather than kept',
          fn: async () => {
            /**
             Who the driver asked, and what it showed them.
             */
            const asked: string[] = [];
            const shown: string[] = [];

            /**
             The row one quiet roster produces.
             */
            const row = await auditOne({
              subject: subjectAt({
                entryId: 'mittens',
                sliceIndex: 1,
                auditsArchiveText: false,
              },),
              references: { kind: 'none', },
              client: quietClient({
                asked,
                shown,
              },),
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            expect(row.runSet,).toBe(RUN_SET,);
            expect(row.entryId,).toBe('mittens',);
            expect(row.sliceIndex,).toBe(1,);
            expect(row.deliveryKind,).toBe('replacement-shipped',);
            expect(row.auditsArchiveText,).toBe(false,);
            expect(row.artifactDigest,).toBe(DIGEST,);
            expect(row.corpusSha,).toBe(CORPUS_SHA,);
            expect(row.identityKind,).toBe('none',);
            expect(row.textIdentity.kind,).toBe('digested',);
            expect(asked.length,).toBeGreaterThan(0,);
            expect(row.report.rows.length,).toBe(new Set(asked,).size,);
            expect(row.report.rows.every(function answered(voice,): boolean {
              return asked.includes(voice.modelId,);
            },),).toBe(true,);
          },
        },),

        it({
          name: 'SHOWS THE ROSTER A DECLARED IDENTITY and withholds an absent one, since an auditor '
            + 'without the names the producing judges had calls a declared name a fabrication',
          fn: async () => {
            /**
             What the roster was shown with the identity declared.
             */
            const shownDeclared: string[] = [];
            await auditOne({
              subject: subjectAt({
                entryId: 'mittens',
                sliceIndex: 0,
                auditsArchiveText: true,
                identity: {
                  kind: 'declared',
                  context: IDENTITY_CONTEXT,
                },
              },),
              references: { kind: 'none', },
              client: quietClient({
                asked: [],
                shown: shownDeclared,
              },),
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            /**
             What it was shown with none.
             */
            const shownNone: string[] = [];
            await auditOne({
              subject: subjectAt({
                entryId: 'mittens',
                sliceIndex: 0,
                auditsArchiveText: true,
              },),
              references: { kind: 'none', },
              client: quietClient({
                asked: [],
                shown: shownNone,
              },),
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            expect(shownDeclared.join('\n',)
              .includes(IDENTITY_CONTEXT,),).toBe(true,);
            expect(shownNone.join('\n',)
              .includes(IDENTITY_CONTEXT,),).toBe(false,);
          },
        },),
        // LEDGER B29, the rendering audit's open gap: the producing judges had
        // the pages the original cites; an auditor without them calls a detail
        // one states an addition.
        it({
          name: 'SHOWS THE ROSTER THE CITED REFERENCES, records that it did, and keys the row on them, and shows and '
            + 'records none where the original cites nothing',
          fn: async () => {
            /**
             What the roster was shown with references.
             */
            const shownCited: string[] = [];
            /**
             Row audited with references.
             */
            const cited = await auditOne({
              subject: subjectAt({ entryId: 'mittens', sliceIndex: 0, auditsArchiveText: true, },),
              references: { kind: 'cited', context: REFERENCE_CONTEXT, },
              client: quietClient({ asked: [], shown: shownCited, },),
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            /**
             What it was shown with none.
             */
            const shownNone: string[] = [];
            /**
             Row audited with none.
             */
            const none = await auditOne({
              subject: subjectAt({ entryId: 'mittens', sliceIndex: 0, auditsArchiveText: true, },),
              references: { kind: 'none', },
              client: quietClient({ asked: [], shown: shownNone, },),
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            /**
             What it was shown where the page links pages nobody could read.
             */
            const shownUnread: string[] = [];
            /**
             Row audited without the references its page links.
             */
            const unread = await auditOne({
              subject: subjectAt({ entryId: 'mittens', sliceIndex: 0, auditsArchiveText: true, },),
              references: { kind: 'unread', links: 1, },
              client: quietClient({ asked: [], shown: shownUnread, },),
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect(shownCited.join('\n',),).toContain(REFERENCE_CONTEXT,);
            expect(shownUnread.join('\n',),).not
              .toContain('CITED REFERENCES',);
            expect(unread.referencesKind,).toBe('unread',);
            // SHOWN THE SAME TEXTS as the row shown none, so a repeat; only the
            // kind says the page linked pages the audit never read.
            expect(sameAuditedText({ left: unread, right: none, },),).toBe(true,);
            expect(shownNone.join('\n',),).not
              .toContain('CITED REFERENCES',);
            expect(cited.referencesKind,).toBe('cited',);
            expect(none.referencesKind,).toBe('none',);
            expect(sameAuditedText({ left: cited, right: none, },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: withCitedReferences.name,
      // ONE AT A TIME: two cases divert the process's console.info across an
      // await to read what the reads logged.
      concurrency: 1,
      children: [
        it({
          name: 'READS EACH PAGE ONCE and pairs every bought subject with its own page\'s references, in buying order',
          fn: async () => {
            /**
             Pages the reader was asked for, in order.
             */
            const read: string[] = [];
            /**
             Subjects bought over two pages, the linking one twice.
             */
            const buying = [
              subjectAt({ entryId: 'mittens', sliceIndex: 0, auditsArchiveText: true, },),
              subjectAt({ entryId: 'tabby', sliceIndex: 0, auditsArchiveText: false, pageSourceText: OTHER_PAGE_TEXT, },),
              subjectAt({ entryId: 'mittens', sliceIndex: 1, auditsArchiveText: false, },),
            ];
            /**
             Each subject beside what its page cites.
             */
            const paired = await withCitedReferences({
              subjects: buying,
              reader: async function recorded({ sourceText, },) {
                read.push(sourceText,);
                return (sourceText === PAGE_TEXT) ? REFERENCE_CONTEXT : '';
              },
            },);
            expect(read,).toStrictEqual([PAGE_TEXT, OTHER_PAGE_TEXT,],);
            expect(paired.map(function subjectOf(pair,) {
              return pair.subject;
            },),).toStrictEqual(buying,);
            expect(paired.map(function referencesOf(pair,) {
              return pair.references;
            },),).toStrictEqual([
              { kind: 'cited', context: REFERENCE_CONTEXT, },
              { kind: 'none', },
              { kind: 'cited', context: REFERENCE_CONTEXT, },
            ],);
          },
        },),
        it({
          name: 'LOGS WHAT EACH PAGE CITES IN BUYING ORDER when the later page\'s read ends first, so two runs over '
            + 'one archive log the same lines',
          fn: async () => {
            /**
             Opened by the later page's read as it ends, which the earlier
             page's read waits for.
             */
            const laterRead = Promise.withResolvers<undefined>();
            /**
             What the reads logged at the info level.
             */
            const { logged, } = await infoLinesDuring({
              run: async function boughtOverTwoPages() {
                return await withCitedReferences({
                  subjects: [
                    subjectAt({ entryId: 'mittens', sliceIndex: 0, auditsArchiveText: true, },),
                    subjectAt({ entryId: 'tabby', sliceIndex: 0, auditsArchiveText: false, pageSourceText: OTHER_PAGE_TEXT, },),
                  ],
                  reader: async function readsTabbyFirst({ sourceText, },): Promise<string> {
                    if (sourceText === OTHER_PAGE_TEXT) {
                      laterRead.resolve(undefined,);
                      return 'Reference 1 (https://example.org/tabby): Tabby suns on the roof.';
                    }
                    await laterRead.promise;
                    return REFERENCE_CONTEXT;
                  },
                },);
              },
            },);
            expect(logged.filter(function isCited(line,): boolean {
              return line.includes('CITED REFERENCES',);
            },),).toEqual([
              '[rendering-audit-settled:withCitedReferences] CITED REFERENCES entry=mittens cited, characters=77',
              '[rendering-audit-settled:withCitedReferences] CITED REFERENCES entry=tabby cited, characters=64',
            ],);
          },
        },),
        it({
          name: 'COUNTS WHAT A PAGE CITES IN CHARACTERS, a character outside the basic plane once, as the line names '
            + 'its unit',
          fn: async () => {
            /**
             What the read logged at the info level.
             */
            const { logged, } = await infoLinesDuring({
              run: async function boughtOverOnePage() {
                return await withCitedReferences({
                  subjects: [subjectAt({ entryId: 'mittens', sliceIndex: 0, auditsArchiveText: true, },),],
                  reader: async function readsACat(): Promise<string> {
                    return '\u{1F408} naps.';
                  },
                },);
              },
            },);
            expect(logged.filter(function isCited(line,): boolean {
              return line.includes('CITED REFERENCES',);
            },),).toEqual([
              '[rendering-audit-settled:withCitedReferences] CITED REFERENCES entry=mittens cited, characters=7',
            ],);
          },
        },),
        it({
          name: 'RECORDS A PAGE THAT LINKS PAGES AND READ AS NOTHING AS UNREAD, never as none, since a missing key '
            + 'reads every linked page as nothing',
          fn: async () => {
            /**
             Each subject beside what its page cites, every read empty.
             */
            const paired = await withCitedReferences({
              subjects: [
                subjectAt({ entryId: 'mittens', sliceIndex: 0, auditsArchiveText: true, },),
                subjectAt({ entryId: 'tabby', sliceIndex: 0, auditsArchiveText: false, pageSourceText: OTHER_PAGE_TEXT, },),
              ],
              reader: async function unkeyed() {
                return '';
              },
            },);
            expect(paired.map(function referencesOf(pair,) {
              return pair.references;
            },),).toStrictEqual([
              { kind: 'unread', links: 1, },
              { kind: 'none', },
            ],);
          },
        },),
        it({
          name: 'READS NOTHING when nothing is bought, so the wiring check that buys nothing spends nothing',
          fn: async () => {
            /**
             Pages the reader was asked for.
             */
            const read: string[] = [];
            /**
             Pairs for an empty buy.
             */
            const paired = await withCitedReferences({
              subjects: [],
              reader: async function recorded({ sourceText, },) {
                read.push(sourceText,);
                return REFERENCE_CONTEXT;
              },
            },);
            expect(read,).toStrictEqual([],);
            expect(paired,).toStrictEqual([],);
          },
        },),
        it({
          name: 'REFUSES with the first bought page\'s cache refusal when two pages\' caches cannot be read and the '
            + 'second page\'s refusal ends first',
          fn: async () => {
            /**
             The two refusals the page reads end in.
             */
            const { refuseAtOnce, refuseAfterThat, } = refusalOrder();
            /**
             What the buy refused with.
             */
            const refusal = await rejectionOf({
              promise: withCitedReferences({
                subjects: [
                  subjectAt({ entryId: 'mittens', sliceIndex: 0, auditsArchiveText: true, },),
                  subjectAt({ entryId: 'tabby', sliceIndex: 0, auditsArchiveText: false, pageSourceText: OTHER_PAGE_TEXT, },),
                ],
                reader: async function refusesSecondPageFirst({ sourceText, },): Promise<string> {
                  /**
                   Cache file the page's references would be read from.
                   */
                  const path = (sourceText === PAGE_TEXT) ? '/cache/mittens' : '/cache/tabby';
                  /**
                   What the lookup cache says of a path holding a directory.
                   */
                  const unreadable = new CacheFileUnreadableError({
                    path,
                    failure: 'EISDIR',
                    cause: new Error('the cat basket is a directory',),
                  },);
                  return await ((sourceText === PAGE_TEXT) ? refuseAfterThat(unreadable,) : refuseAtOnce(unreadable,));
                },
              },),
            },);
            expect(String(refusal,),).toBe(directoryRefusalText({ path: '/cache/mittens', },),);
          },
        },),
      ],
    },),
  ],
},);
