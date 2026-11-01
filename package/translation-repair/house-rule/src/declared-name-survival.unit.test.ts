/**
 Tests for the guard that keeps a declared name from being edited away.

 WHY IT IS A GUARD. Measured against the repair lane's own judge sheet and
 roster, six of six judges preferred the candidate that dropped a declared
 alias, and adding a carve-out to the criterion did not flip the vote. The
 check therefore cannot live in a prompt.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  declaredNameForms,
  declaredNameRefusalReport,
  findDroppedDeclaredNames,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: declaredNameForms.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SPLITS a comma-joined alias field into handles, longest first',
          fn: async () => {
            expect(declaredNameForms({
              identity: {
                name: 'Mittens',
                alias: 'Blossom, Patch',
              },
            },),).toEqual([ 'Mittens', 'Blossom', 'Patch', ],);
          },
        },),
        it({
          name: 'IGNORES the declared location, which is a place and not a name',
          fn: async () => {
            // A translation may render or omit a place, and demanding its survival
            // would refuse candidates that did nothing wrong.
            expect(declaredNameForms({
              identity: {
                name: 'Mittens',
                location: 'Windowsill, Kyoto',
              },
            },),).toEqual([ 'Mittens', ],);
          },
        },),
        it({
          name: 'DROPS a handle too short for its survival to mean anything',
          fn: async () => {
            expect(declaredNameForms({ identity: { name: 'Mo', }, },),).toEqual([],);
          },
        },),
        it({
          name: 'KEEPS a handle written in letters beyond the first plane, a Han name with an ideograph '
            + 'from Extension B or one in mathematical script, which a scan by UTF-16 unit read as two '
            + 'characters or none and dropped as too short (ledger B21)',
          fn: async () => {
            expect(declaredNameForms({
              identity: {
                name: '\u{20BB7}\u{7965}\u{732B}',
                alias: '\u{1D4DC}\u{1D4F2}\u{1D4F5}\u{1D4F8}',
              },
            },),).toEqual([ '\u{1D4DC}\u{1D4F2}\u{1D4F5}\u{1D4F8}', '\u{20BB7}\u{7965}\u{732B}', ],);
          },
        },),
        it({
          name: 'NAMES one form once when two fields declare it',
          fn: async () => {
            expect(declaredNameForms({
              identity: {
                name: 'Mittens',
                alias: 'Mittens',
              },
            },),).toEqual([ 'Mittens', ],);
          },
        },),
        it({
          name: 'REPORTS nothing when neither field is declared',
          fn: async () => {
            expect(declaredNameForms({ identity: {}, },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: findDroppedDeclaredNames.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES a declared handle the base carried and the candidate lost',
          fn: async () => {
            expect(findDroppedDeclaredNames({
              forms: [ 'Mittens', 'Blossom', ],
              baseText: 'Mittens (Blossom) is a calico cat.',
              candidateText: 'Mittens is a calico cat.',
            },),).toEqual([ 'Blossom', ],);
          },
        },),
        it({
          name: 'REQUIRES nothing the base text never carried',
          fn: async () => {
            // THE CONSERVATIVE PROPERTY THIS RESTS ON. Only what was already there
            // has to stay, so this can refuse a real loss and can never demand that
            // a translator insert a name the passage never had.
            expect(findDroppedDeclaredNames({
              forms: [ 'Mittens', 'Blossom', ],
              baseText: 'She is a calico cat.',
              candidateText: 'She is a calico cat who naps.',
            },),).toEqual([],);
          },
        },),
        it({
          name: 'ACCEPTS a respelled handle, since spelling is not loss',
          fn: async () => {
            expect(findDroppedDeclaredNames({
              forms: [ 'Blossom', ],
              baseText: 'Blossom naps.',
              candidateText: 'blossom naps.',
            },),).toEqual([],);
          },
        },),
        it({
          name: 'REPORTS a lost long form ONCE, not once per fragment inside it',
          fn: async () => {
            // Losing `Mittens Blossom` should not read as two separate losses when
            // the shorter form only ever appeared inside the longer one.
            expect(findDroppedDeclaredNames({
              forms: [ 'Mittens Blossom', 'Blossom', ],
              baseText: 'Mittens Blossom naps on the sill.',
              candidateText: 'The cat naps on the sill.',
            },),).toEqual([ 'Mittens Blossom', ],);
          },
        },),
        it({
          name: 'STAYS quiet when every declared handle survives',
          fn: async () => {
            expect(findDroppedDeclaredNames({
              forms: [ 'Mittens', 'Blossom', ],
              baseText: 'Mittens (Blossom) naps.',
              candidateText: 'Blossom, also called Mittens, naps in the sun.',
            },),).toEqual([],);
          },
        },),
        it({
          name: 'SEES a handle the archive escaped for Markdown, which a raw substring comparison '
            + 'cannot: the underscores are written back-slashed and the declaration is not, so the '
            + 'guard would have called the name absent and skipped it entirely',
          fn: async () => {
            expect(findDroppedDeclaredNames({
              forms: [ 'Mittens_the_Cat', ],
              baseText: String.raw`Mittens\_the\_Cat naps on the sill.`,
              candidateText: 'The cat naps on the sill.',
            },),).toEqual([ 'Mittens_the_Cat', ],);
          },
        },),
        it({
          name: 'SEES a handle the two sides separate differently, one writing a space where the other '
            + 'writes an underscore, since a handle is one name however it is punctuated',
          fn: async () => {
            expect(findDroppedDeclaredNames({
              forms: [ 'Mittens Cat', ],
              baseText: 'Everyone called her Mittens_Cat back then.',
              candidateText: 'Everyone called her that back then.',
            },),).toEqual([ 'Mittens Cat', ],);
          },
        },),
        it({
          name: 'SEES a handle the archive spaces and the declaration runs together, which is the same '
            + 'disagreement in the other direction',
          fn: async () => {
            expect(findDroppedDeclaredNames({
              forms: [ 'MittensCat', ],
              baseText: 'Mittens Cat slept through it.',
              candidateText: 'She slept through it.',
            },),).toEqual([ 'MittensCat', ],);
          },
        },),
        it({
          name: 'ACCEPTS a handle whose diacritic is spelled the other way, since a combining mark is '
            + 'neither letter nor digit and would otherwise vanish from one side only, reporting a '
            + 'name lost that is sitting right there',
          fn: async () => {
            expect(findDroppedDeclaredNames({
              forms: [ 'Mitt\u00EBns', ],
              baseText: 'Mitte\u0308ns naps.',
              candidateText: 'Mitte\u0308ns naps in the sun.',
            },),).toEqual([],);
          },
        },),
        it({
          name: 'REPORTS that same handle as lost when it really goes, so the "ACCEPTS a handle whose diacritic '
            + 'is spelled the other way" case is the '
            + 'comparison working rather than the comparison never firing',
          fn: async () => {
            expect(findDroppedDeclaredNames({
              forms: [ 'Mitt\u00EBns', ],
              baseText: 'Mitte\u0308ns naps.',
              candidateText: 'The cat naps in the sun.',
            },),).toEqual([ 'Mitt\u00EBns', ],);
          },
        },),
        it({
          name: 'REPORTS a handle in mathematical script as lost when it goes, which a scan by UTF-16 unit '
            + 'projected to nothing and so found on every side (ledger B21)',
          fn: async () => {
            expect(findDroppedDeclaredNames({
              forms: [ '\u{1D4DC}\u{1D4F2}\u{1D4F5}\u{1D4F8}', ],
              baseText: '\u{1D4DC}\u{1D4F2}\u{1D4F5}\u{1D4F8} naps.',
              candidateText: 'The cat naps.',
            },),).toEqual([ '\u{1D4DC}\u{1D4F2}\u{1D4F5}\u{1D4F8}', ],);
          },
        },),
        it({
          name: 'NAMES A NAME LOST WHERE THE CANDIDATE HOLDS ITS LETTERS ONLY ACROSS A WORD EDGE (ledger B23): '
            + '"Ann" inside "cannot" is no Ann',
          fn: async () => {
            expect(findDroppedDeclaredNames({
              forms: [ 'Ann', ],
              baseText: 'Ann naps on the sill.',
              candidateText: 'The cat cannot nap on the sill.',
            },),).toEqual([ 'Ann', ],);
          },
        },),
        it({
          name: 'REQUIRES NOTHING OF A NAME THE BASE HELD ONLY ACROSS A WORD EDGE',
          fn: async () => {
            expect(findDroppedDeclaredNames({
              forms: [ 'Ann', ],
              baseText: 'The cat cannot nap.',
              candidateText: 'The cat naps.',
            },),).toEqual([],);
          },
        },),
        it({
          name: 'READS A SMALL LETTER RUNNING ON AS NO EDGE, and a case change or a digit inside a handle as one',
          fn: async () => {
            expect(findDroppedDeclaredNames({
              forms: [ 'Blossom', ],
              baseText: 'Blossom naps.',
              candidateText: 'The cherryblossom naps.',
            },),).toEqual([ 'Blossom', ],);
            for (const candidateText of [
              'MittensBlossom naps.',
              'Blossom2024 naps.',
              '2024Blossom naps.',
            ]) {
              expect(findDroppedDeclaredNames({
                forms: [ 'Blossom', ],
                baseText: 'Blossom naps.',
                candidateText,
              },),).toEqual([],);
            }
          },
        },),
        it({
          name: 'NEEDS NO EDGE BESIDE A HAN NAME, since Chinese writes no spaces',
          fn: async () => {
            expect(findDroppedDeclaredNames({
              forms: [ '\u{732B}\u{5C0F}\u{59D0}', ],
              baseText: 'Mittens\u{732B}\u{5C0F}\u{59D0}naps.',
              candidateText: 'cat\u{732B}\u{5C0F}\u{59D0}naps.',
            },),).toEqual([],);
          },
        },),
        it({
          name: 'REPORTS A LOST SHORTER FORM ON ITS OWN when it stood apart and only runs on inside the longer one',
          fn: async () => {
            expect(findDroppedDeclaredNames({
              forms: [ 'Annabel', 'Ann', ],
              baseText: 'Annabel and Ann nap.',
              candidateText: 'The cats nap.',
            },),).toEqual([ 'Annabel', 'Ann', ],);
          },
        },),
        it({
          name: 'ACCEPTS a handle whose emoji the candidate leaves out, since an emoji is neither letter nor '
            + 'digit and drops out of both sides alike',
          fn: async () => {
            expect(findDroppedDeclaredNames({
              forms: [ 'Mittens\u{1F63A}', ],
              baseText: 'Mittens\u{1F63A} naps.',
              candidateText: 'Mittens naps in the sun.',
            },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: `${findDroppedDeclaredNames.name} with a subject identity`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS a swap between two declared forms of the subject, since the person is still named '
            + 'where the base named them',
          fn: async () => {
            expect(findDroppedDeclaredNames({
              forms: [ 'Mittens', 'Blossom', ],
              subjectForms: [ 'Mittens', 'Blossom', ],
              baseText: 'Mittens naps on the sill.',
              candidateText: 'Blossom naps on the sill.',
            },),).toEqual([],);
          },
        },),
        it({
          name: 'ACCEPTS THE SHAPE OF XingZ60 SLICE 80: the archive names the subject by the display name, '
            + 'which holds the alias `Z60` across a word edge, and the replacement names her by another '
            + 'alias, so one lost name is met by one new name and the nested alias is not a second loss',
          fn: async () => {
            expect(findDroppedDeclaredNames({
              forms: [ 'MittensZ60', 'Blossom', 'Z60', ],
              subjectForms: [ 'MittensZ60', 'Blossom', 'Z60', ],
              baseText: 'From this we learn that MittensZ60 is actually quite indifferent to life and death.',
              candidateText: 'From this I learned that Blossom had actually grown quite at peace with life and death.',
            },),).toEqual([],);
          },
        },),
        it({
          name: 'KEEPS REFUSING A PLAIN LOSS, the measured case the survival guard exists for: '
            + '`Mittens (Blossom)` becoming `Mittens` loses `Blossom` with nothing in its place',
          fn: async () => {
            expect(findDroppedDeclaredNames({
              forms: [ 'Mittens', 'Blossom', ],
              subjectForms: [ 'Mittens', 'Blossom', ],
              baseText: 'Mittens (Blossom) is a calico cat.',
              candidateText: 'Mittens is a calico cat.',
            },),).toEqual([ 'Blossom', ],);
          },
        },),
        it({
          name: 'KEEPS REFUSING WHEN NO SUBJECT IDENTITY IS SUPPLIED, so a caller that cannot say which forms '
            + 'name one person gets the plain rule',
          fn: async () => {
            expect(findDroppedDeclaredNames({
              forms: [ 'Mittens', 'Blossom', ],
              baseText: 'Mittens naps on the sill.',
              candidateText: 'Blossom naps on the sill.',
            },),).toEqual([ 'Mittens', ],);
          },
        },),
        it({
          name: 'COUNTS FOR COUNT: two declared forms lost against one new form leaves one person '
            + 'named where the base named them twice over, which is refused with both forms named',
          fn: async () => {
            expect(findDroppedDeclaredNames({
              forms: [ 'Mittens', 'Blossom', 'Patch', ],
              subjectForms: [ 'Mittens', 'Blossom', 'Patch', ],
              baseText: 'Mittens (Blossom) naps on the sill.',
              candidateText: 'Patch naps on the sill.',
            },),).toEqual([ 'Mittens', 'Blossom', ],);
          },
        },),
        it({
          name: 'DOES NOT COUNT A FORM THE BASE CARRIED AS A SUBSTITUTE: `Mittens (Blossom)` becoming '
            + '`Blossom` loses `Mittens`, and `Blossom` was already there',
          fn: async () => {
            expect(findDroppedDeclaredNames({
              forms: [ 'Mittens', 'Blossom', ],
              subjectForms: [ 'Mittens', 'Blossom', ],
              baseText: 'Mittens (Blossom) naps on the sill.',
              candidateText: 'Blossom naps on the sill.',
            },),).toEqual([ 'Mittens', ],);
          },
        },),
        it({
          name: 'COUNTS A NEW FORM AND THE SHORTER FORM RUNNING ON INSIDE IT ONCE, so one new name '
            + 'cannot stand in for two lost ones',
          fn: async () => {
            expect(findDroppedDeclaredNames({
              forms: [ 'Mittens', 'Gizmo', 'Blossom Patch', 'Patch', ],
              subjectForms: [ 'Mittens', 'Gizmo', 'Blossom Patch', 'Patch', ],
              baseText: 'Mittens and Gizmo nap on the sill.',
              candidateText: 'Blossom Patch naps on the sill.',
            },),).toEqual([ 'Mittens', 'Gizmo', ],);
          },
        },),
        it({
          name: 'REFUSES A CONTRIBUTOR NAME REPLACED BY A SUBJECT NAME, because the contributor is another '
            + 'person and goes unnamed',
          fn: async () => {
            expect(findDroppedDeclaredNames({
              forms: [ 'Mittens', 'Blossom', 'Patch', ],
              subjectForms: [ 'Mittens', 'Blossom', ],
              baseText: 'Patch watched Mittens nap on the sill.',
              candidateText: 'Blossom watched Mittens nap on the sill.',
            },),).toEqual([ 'Patch', ],);
          },
        },),
        it({
          name: 'MEETS THE SUBJECT\'S LOSS AND STILL NAMES THE CONTRIBUTOR: one new subject form pays for '
            + 'the lost subject form only, so the contributor lost beside it is reported on its own',
          fn: async () => {
            expect(findDroppedDeclaredNames({
              forms: [ 'Mittens', 'Blossom', 'Patch', ],
              subjectForms: [ 'Mittens', 'Blossom', ],
              baseText: 'Patch watched Mittens nap on the sill.',
              candidateText: 'Blossom napped on the sill.',
            },),).toEqual([ 'Patch', ],);
          },
        },),
        it({
          name: 'REFUSES A SUBJECT NAME REPLACED BY A CONTRIBUTOR NAME, because a contributor is not '
            + 'another form of the subject',
          fn: async () => {
            expect(findDroppedDeclaredNames({
              forms: [ 'Mittens', 'Blossom', 'Patch', ],
              subjectForms: [ 'Mittens', 'Blossom', ],
              baseText: 'Mittens naps on the sill.',
              candidateText: 'Patch naps on the sill.',
            },),).toEqual([ 'Mittens', ],);
          },
        },),
        it({
          name: 'REFUSES A NEW FORM THAT THE CANDIDATE HOLDS ONLY ACROSS A WORD EDGE, since `Ann` inside '
            + '`cannot` names nobody',
          fn: async () => {
            expect(findDroppedDeclaredNames({
              forms: [ 'Mittens', 'Ann', ],
              subjectForms: [ 'Mittens', 'Ann', ],
              baseText: 'Mittens naps on the sill.',
              candidateText: 'The cat cannot nap on the sill.',
            },),).toEqual([ 'Mittens', ],);
          },
        },),
      ],
    },),

    describe({
      name: `${findDroppedDeclaredNames.name} with a subject identity, where a new form must stand as a whole name`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES A NEW FORM RUN ON FROM OR INTO OTHER LETTERS, a camel edge, a digit, a hyphen or an '
            + 'underscore, since `WoolyBlossom` is a name nobody declared and `Blossom` inside it names '
            + 'no one (XingZ60: `XingLili`, `Xing-Lili`)',
          fn: async () => {
            for (const candidateText of [
              'WoolyBlossom naps on the sill.',
              'Wooly-Blossom naps on the sill.',
              'Wooly_Blossom naps on the sill.',
              'BlossomWooly naps on the sill.',
              'Blossom-Wooly naps on the sill.',
              'Blossom2024 naps on the sill.',
            ]) {
              expect(findDroppedDeclaredNames({
                forms: [ 'Mittens', 'Blossom', ],
                subjectForms: [ 'Mittens', 'Blossom', ],
                baseText: 'Mittens naps on the sill.',
                candidateText,
              },),).toEqual([ 'Mittens', ],);
            }
          },
        },),
        it({
          name: 'REFUSES A NEW FORM FUSED WITH A CAPITALISED WORD BESIDE IT that is no declared form, either side, '
            + 'since `Wooly Blossom` is a renamed person (XingZ60: `Xing Lili`)',
          fn: async () => {
            for (const candidateText of [
              'Wooly Blossom naps on the sill.',
              'Blossom Wooly naps on the sill.',
              'The cat Wooly Blossom naps on the sill.',
              'When Wooly Blossom napped, the sill was warm.',
            ]) {
              expect(findDroppedDeclaredNames({
                forms: [ 'Mittens', 'Blossom', ],
                subjectForms: [ 'Mittens', 'Blossom', ],
                baseText: 'Mittens naps on the sill.',
                candidateText,
              },),).toEqual([ 'Mittens', ],);
            }
          },
        },),
        it({
          name: 'REFUSES A NEW FORM SPELLED SPLIT, since the projection folds `Mi Mi` into the key of `Mimi` '
            + '(the plain rule sees the name there) but the spelling is not the declared one (XingZ60: `Xing Li Li`)',
          fn: async () => {
            expect(findDroppedDeclaredNames({
              forms: [ 'Mittens', 'Mimi', ],
              subjectForms: [ 'Mittens', 'Mimi', ],
              baseText: 'Mittens naps on the sill.',
              candidateText: 'Mi Mi naps on the sill.',
            },),).toEqual([ 'Mittens', ],);
            expect(findDroppedDeclaredNames({
              forms: [ 'Mittens', 'Mimi', ],
              subjectForms: [ 'Mittens', 'Mimi', ],
              baseText: 'Mittens naps on the sill.',
              candidateText: 'Wooly Mi Mi naps on the sill.',
            },),).toEqual([ 'Mittens', ],);
          },
        },),
        it({
          name: 'STILL READS A SPLIT SPELLING AS THE NAME WHERE THE NAME IS CARRIED, which is the carriage rule '
            + 'this change leaves alone: only an arrival is held to the declared spelling',
          fn: async () => {
            expect(findDroppedDeclaredNames({
              forms: [ 'Mimi', ],
              baseText: 'Mimi naps on the sill.',
              candidateText: 'Mi Mi naps on the sill.',
            },),).toEqual([],);
          },
        },),
        it({
          name: 'ACCEPTS A NEW FORM BESIDE AN ORDINARY CAPITALISED WORD, a sentence opener, a title or a kin word, '
            + 'so `When Blossom napped` and `Dear Blossom,` are real swaps',
          fn: async () => {
            for (const candidateText of [
              'When Blossom napped, the sill was warm.',
              'Dear Blossom, the sill is warm.',
              'Perhaps Blossom naps on the sill.',
              'Since Blossom napped, the sill is warm.',
              'The old sill was warm for Blossom.',
              'Sister Blossom naps on the sill.',
              'In June Blossom naps on the sill.',
            ]) {
              expect(findDroppedDeclaredNames({
                forms: [ 'Mittens', 'Blossom', ],
                subjectForms: [ 'Mittens', 'Blossom', ],
                baseText: 'Mittens naps on the sill.',
                candidateText,
              },),).toEqual([],);
            }
          },
        },),
        it({
          name: 'ACCEPTS A NEW FORM BESIDE A WORD THE BASE ALREADY HELD BESIDE THE LOST FORM, and beside a word '
            + 'that is part of a declared form of anyone, since neither is a fusion the candidate made',
          fn: async () => {
            expect(findDroppedDeclaredNames({
              forms: [ 'Mittens', 'Blossom', ],
              subjectForms: [ 'Mittens', 'Blossom', ],
              baseText: 'Aoba Mittens naps on the sill.',
              candidateText: 'Aoba Blossom naps on the sill.',
            },),).toEqual([],);
            expect(findDroppedDeclaredNames({
              forms: [ 'Mittens', 'Blossom', ],
              subjectForms: [ 'Mittens', 'Blossom', ],
              baseText: 'Mittens Aoba naps on the sill.',
              candidateText: 'Blossom Aoba naps on the sill.',
            },),).toEqual([],);
            expect(findDroppedDeclaredNames({
              forms: [ 'Mittens', 'Blossom', 'Patch Quill', ],
              subjectForms: [ 'Mittens', 'Blossom', ],
              baseText: 'Mittens naps on the sill.',
              candidateText: 'Patch Quill and Blossom nap on the sill.',
            },),).toEqual([],);
            expect(findDroppedDeclaredNames({
              forms: [ 'Mittens', 'Blossom', 'Patch', ],
              subjectForms: [ 'Mittens', 'Blossom', ],
              baseText: 'Mittens naps on the sill.',
              candidateText: 'Patch Blossom naps on the sill.',
            },),).toEqual([],);
          },
        },),
        it({
          name: 'ACCEPTS A NEW FORM THAT IS WRITTEN AS DECLARED, a whole multi-word form, a Markdown-escaped '
            + 'handle, or a form beside punctuation, so only a fusion is held back',
          fn: async () => {
            expect(findDroppedDeclaredNames({
              forms: [ 'Mittens', 'Wooly Blossom', 'Blossom', ],
              subjectForms: [ 'Mittens', 'Wooly Blossom', 'Blossom', ],
              baseText: 'Mittens naps on the sill.',
              candidateText: 'Wooly Blossom naps on the sill.',
            },),).toEqual([],);
            expect(findDroppedDeclaredNames({
              forms: [ 'Mittens', 'Wooly_Cat', ],
              subjectForms: [ 'Mittens', 'Wooly_Cat', ],
              baseText: 'Mittens naps on the sill.',
              candidateText: String.raw`Wooly\_Cat naps on the sill.`,
            },),).toEqual([],);
            expect(findDroppedDeclaredNames({
              forms: [ 'Mittens', 'Blossom', ],
              subjectForms: [ 'Mittens', 'Blossom', ],
              baseText: 'Mittens naps on the sill.',
              candidateText: 'The cat (Blossom), who naps, is warm; "Blossom" naps.',
            },),).toEqual([],);
            // A no-break space is a space, and a bracket at the end of a declared form is not part of its spelling.
            expect(findDroppedDeclaredNames({
              forms: [ 'Mittens', 'Wooly Blossom', ],
              subjectForms: [ 'Mittens', 'Wooly Blossom', ],
              baseText: 'Mittens naps on the sill.',
              candidateText: 'Wooly\u00A0Blossom naps on the sill.',
            },),).toEqual([],);
            expect(findDroppedDeclaredNames({
              forms: [ 'Mittens', 'Blossom (Peanut)', ],
              subjectForms: [ 'Mittens', 'Blossom (Peanut)', ],
              baseText: 'Mittens naps on the sill.',
              candidateText: 'Blossom (Peanut) naps on the sill.',
            },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: declaredNameRefusalReport.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE FIELD WITH AN EMPTY LIST when nothing was refused, so a reader of a settled '
            + 'slice never has to tell dropping nothing from nobody writing the field',
          fn: async () => {
            /**
             Report for a slice where every declared handle survived.
             */
            const quiet = declaredNameRefusalReport({
              sliceIndex: 3,
              dropped: [],
            },);

            expect(quiet.record.droppedDeclaredNames,).toEqual([],);
            expect(quiet.findings,).toEqual([],);
          },
        },),
        it({
          name: 'CARRIES the refused forms into both the record a reader queries and the finding a '
            + 'scorecard counts, since a refusal missing from either is one nobody can audit',
          fn: async () => {
            /**
             Report for a slice whose replacement lost two declared handles.
             */
            const refused = declaredNameRefusalReport({
              sliceIndex: 3,
              dropped: [
                'Mittens',
                'Whiskers',
              ],
            },);

            expect(refused.record.droppedDeclaredNames,).toEqual([
              'Mittens',
              'Whiskers',
            ],);
            expect(refused.findings.length,).toBe(1,);
            expect(refused.findings[0],).toBe(
              'translate-refused-declared-name (slice 3: archive text carries "Mittens", "Whiskers" '
                + 'and the replacement does not; keeping the archive text)',
            );
          },
        },),
      ],
    },),
  ],
},);
