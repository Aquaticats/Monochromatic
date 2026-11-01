/**
 Tests for restoring the quote style an editor flattened.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { restoreTypography, } from '../dist/final/node/index.mjs';

/**
 Right single quotation mark, the apostrophe the corpus uses.
 */
const APOSTROPHE = '\u{2019}';

/**
 Left double quotation mark.
 */
const OPEN = '\u{201C}';

/**
 Right double quotation mark.
 */
const CLOSE = '\u{201D}';

/**
 Restores using the replaced region as its own convention.

 Every case in this file was written before the convention widened to the whole
 document, and each asserts REGION-scoped behaviour, so passing the replaced
 text as the convention keeps each assertion testing what it was written to
 test.

 @param replacement - text the editor wrote

 @param replaced - text it replaces, standing as its own convention

 @returns Restored replacement

 @example
 ```ts
 restoreFromRegion({ replacement: "didn't", replaced: "did not", },);
 ```
 */
function restoreFromRegion(
  {
    replacement,
    replaced,
  }: {
    readonly replacement: string;
    readonly replaced: string;
  },
): string {
  return restoreTypography({
    replacement,
    replaced,
    convention: replaced,
  },);
}

await describe({
  name: restoreTypography.name,
  children: [
    it({
      name: 'restores a curly apostrophe inside a word when the replaced text '
        + 'used one, which is the observed regression: an edit arrives with '
        + 'didn\'t where the paragraph around it reads didn\u{2019}t, and the '
        + 'document ends up carrying two conventions',
      fn: async () => {
        expect(restoreFromRegion({
          replacement: "The cat didn't nap.",
          replaced: `The cat didn${APOSTROPHE}t sleep.`,
        },),).toBe(`The cat didn${APOSTROPHE}t nap.`,);
      },
    },),

    it({
      name: 'leaves a straight apostrophe alone when the replaced text used '
        + 'straight ones, so a document written that way is never converted '
        + 'toward a convention it does not hold',
      fn: async () => {
        expect(restoreFromRegion({
          replacement: "The cat didn't nap.",
          replaced: "The cat didn't sleep.",
        },),).toBe("The cat didn't nap.",);
      },
    },),

    it({
      name: 'converts a BALANCED pair of straight doubles in open-close order, '
        + 'and refuses an odd count, since one straight double is doing '
        + 'something this rule cannot read',
      fn: async () => {
        expect(restoreFromRegion({
          replacement: 'They called her "Whiskers" often.',
          replaced: `They named her ${OPEN}Tabby${CLOSE} once.`,
        },),).toBe(`They called her ${OPEN}Whiskers${CLOSE} often.`,);
        expect(restoreFromRegion({
          replacement: 'The cat is 12" tall.',
          replaced: `A ${OPEN}cat${CLOSE} naps.`,
        },),).toBe('The cat is 12" tall.',);
      },
    },),

    it({
      name: 'never touches a quote inside a backtick span, because a straight '
        + 'quote there is part of code and converting it would change what the '
        + 'code says',
      fn: async () => {
        expect(restoreFromRegion({
          replacement: "Run `cat --name='Tabby'` first.",
          replaced: `Run the cat${APOSTROPHE}s tool.`,
        },),).toBe("Run `cat --name='Tabby'` first.",);
      },
    },),

    it({
      name: 'leaves a straight quote that is not inside a word alone, so a '
        + 'foot mark or a quote opening a phrase is never turned into an '
        + 'apostrophe',
      fn: async () => {
        expect(restoreFromRegion({
          replacement: "'Nap time, said the cat.",
          replaced: `The cat${APOSTROPHE}s season.`,
        },),).toBe("'Nap time, said the cat.",);
      },
    },),

    it({
      name: 'restores from the DOCUMENT when the replaced region carries no '
        + 'quote of its own, the case that made the region-scoped rule useless '
        + 'in practice. Regions run to a median of 75 characters, so most hold '
        + 'no quote to learn from, while English prose is full of apostrophes: '
        + 'measured over 56 settled entries, 40 of the 51 whose input carried '
        + 'curly quotes came out worse, 99 curly characters lost against 163 '
        + 'straight ones gained',
      fn: async () => {
        expect(restoreTypography({
          replacement: "The cat didn't nap.",
          replaced: 'The cat slept soundly.',
          convention: `Every other line here reads didn${APOSTROPHE}t.`,
        },),).toBe(`The cat didn${APOSTROPHE}t nap.`,);
      },
    },),

    it({
      name: 'still leaves a straight-quoted document alone, which is what keeps '
        + 'the widening from imposing a convention nowhere in evidence',
      fn: async () => {
        expect(restoreTypography({
          replacement: "The cat didn't nap.",
          replaced: 'The cat slept soundly.',
          convention: "Every other line here reads didn't.",
        },),).toBe("The cat didn't nap.",);
      },
    },),

    it({
      name: 'restores curly DOUBLES from the document too, and still refuses an '
        + 'odd count, so the pairing guard survives the widening rather than '
        + 'being bypassed by it',
      fn: async () => {
        expect(restoreTypography({
          replacement: 'They called her "Whiskers" often.',
          replaced: 'They called her Tabby once.',
          convention: `A neighbour said ${OPEN}hello${CLOSE}.`,
        },),).toBe(`They called her ${OPEN}Whiskers${CLOSE} often.`,);
        expect(restoreTypography({
          replacement: 'The cat is 12" tall.',
          replaced: 'The cat is small.',
          convention: `A neighbour said ${OPEN}hello${CLOSE}.`,
        },),).toBe('The cat is 12" tall.',);
      },
    },),
    it({
      name: 'LEAVES a JSX string literal inside a blockquoted component line straight on a curly '
        + 'page, since a literal in typographic quotes compiles nowhere (yulianNyanner, 2026-09-06)',
      fn: async () => {
        /**
         Component line as the archives write it.
         */
        const line = `> <PhotoScroll photos={["\${path}/photos/photo3.webp"]} />\n`;

        expect(restoreTypography({
          replacement: line,
          replaced: line,
          convention: `She said ${OPEN}hello${CLOSE}, didn${APOSTROPHE}t she.`,
        },),).toBe(line,);
      },
    },),

    it({
      name: 'LEAVES a double-quoted HTML attribute straight while curling the prose around it, '
        + 'and does not count the attribute quotes toward the pairing',
      fn: async () => {
        expect(restoreTypography({
          replacement: '<p style="text-align: end;">"Whiskers," she said.</p>',
          replaced: '<p style="text-align: end;">Tabby, she said.</p>',
          convention: `A neighbour said ${OPEN}hello${CLOSE}.`,
        },),).toBe(`<p style="text-align: end;">${OPEN}Whiskers,${CLOSE} she said.</p>`,);
      },
    },),

    it({
      name: 'ENDS A TAG AT A BRACKET WITH SPACES BEHIND IT, whether one space or more than three, since only '
        + 'a bracket at a line start is a blockquote marker, and curls the prose after it',
      fn: async () => {
        /**
         Replacements each closing a tag at a bracket behind spaces, then quoting.
         */
        const replacements = [
          '<p style="text-align: end;" >"Whiskers," she said.</p>',
          '<p style="text-align: end;"     >"Whiskers," she said.</p>',
        ];
        expect(replacements.map(function restore(replacement,): string {
          return restoreTypography({
            replacement,
            replaced: 'Tabby, she said.',
            convention: `A neighbour said ${OPEN}hello${CLOSE}.`,
          },);
        },),).toEqual([
          `<p style="text-align: end;" >${OPEN}Whiskers,${CLOSE} she said.</p>`,
          `<p style="text-align: end;"     >${OPEN}Whiskers,${CLOSE} she said.</p>`,
        ],);
      },
    },),

    it({
      name: 'LEAVES the attributes of a tag straight wherever the MDX compiler opens one (ledger B18): a name '
        + 'outside ASCII, a name opening on an underscore, and a name after a no-break space',
      fn: async () => {
        /**
         Replacements each holding one such tag.
         */
        const replacements = [
          'She drew <猫 title="nap" /> twice.',
          'She drew <_Paw title="nap" /> twice.',
          'She drew <\u{00A0}Paw title="nap" /> twice.',
        ];
        expect(replacements.map(function restore(replacement,): string {
          return restoreTypography({
            replacement,
            replaced: 'She drew.',
            convention: `A neighbour said ${OPEN}hello${CLOSE}.`,
          },);
        },),).toEqual(replacements,);
      },
    },),

    it({
      name: 'KEEPS a tag spanning blockquote lines as one tag, so the marker on each line does '
        + 'not end it early',
      fn: async () => {
        /**
         Component whose array runs over three quoted lines.
         */
        const block = `> <PhotoScroll photos={[\n>     "\${path}/a.webp",\n>     "\${path}/b.webp",\n> ]} />\n`;

        expect(restoreTypography({
          replacement: block,
          replaced: block,
          convention: `A neighbour said ${OPEN}hello${CLOSE}.`,
        },),).toBe(block,);
      },
    },),

    it({
      name: 'CURLS A PLURAL POSSESSIVE AFTER AN EMPHASIS SPAN when no word follows it and nothing in the '
        + 'replacement could be its opening quote',
      fn: async () => {
        expect(restoreTypography({
          replacement: 'The *cats*\' bowl is full.',
          replaced: 'The bowl is full.',
          convention: `Every other line here reads didn${APOSTROPHE}t.`,
        },),).toBe(`The *cats*${APOSTROPHE} bowl is full.`,);
      },
    },),

    it({
      name: 'RESTORES a trailing possessive apostrophe when nothing in the replacement could be '
        + 'its opening quote (yulianNyanner, 2026-09-06), and curls a quoted phrase as a whole pair, '
        + 'never half of it (class one hundred forty-seven, XingZ6013, 2026-09-26)',
      fn: async () => {
        expect(restoreTypography({
          replacement: "on the girls' side too.",
          replaced: 'on the other side too.',
          convention: `Every other line here reads didn${APOSTROPHE}t.`,
        },),).toBe(`on the girls${APOSTROPHE} side too.`,);
        expect(restoreTypography({
          replacement: "on the 'girls' side too.",
          replaced: 'on the other side too.',
          convention: `Every other line here reads didn${APOSTROPHE}t.`,
        },),).toBe('on the ‘girls’ side too.',);
      },
    },),
    it({
      name: 'RESTORES a possessive apostrophe after a link or an emphasis span, which the archive '
        + 'curls, and reads a mark between letters beside a span as the apostrophe it is (class seventy, mikaela_khara, '
        + '2026-09-19; n159)',
      fn: async () => {
        const convention = `Every other line here reads didn${APOSTROPHE}t.`;
        expect(restoreTypography({
          replacement: "Excerpt from [Whiskers](https://example.invalid/whiskers)'s diary, issue 3",
          replaced: 'Excerpt from the diary',
          convention,
        },),).toBe(`Excerpt from [Whiskers](https://example.invalid/whiskers)${APOSTROPHE}s diary, issue 3`,);
        expect(restoreTypography({
          replacement: "She hummed *Cat Chorus*'s opening.",
          replaced: 'She hummed the opening.',
          convention,
        },),).toBe(`She hummed *Cat Chorus*${APOSTROPHE}s opening.`,);
        // CHANGED IN n159 (X7): the emphasis delimiters show a reader nothing, so the first mark stands between two letters
        // (`Chorus'sad`) and is the apostrophe it is without the asterisks, and the last is a trailing apostrophe; the text
        // without the asterisks is restored the same way, which the next expectation shows.
        expect(restoreTypography({
          replacement: "She hummed *Cat Chorus*'sad tune' twice.",
          replaced: 'She hummed twice.',
          convention,
        },),).toBe(`She hummed *Cat Chorus*${APOSTROPHE}sad tune${APOSTROPHE} twice.`,);
        expect(restoreTypography({
          replacement: "She hummed Cat Chorus'sad tune' twice.",
          replaced: 'She hummed twice.',
          convention,
        },),).toBe(`She hummed Cat Chorus${APOSTROPHE}sad tune${APOSTROPHE} twice.`,);
      },
    },),
    it({
      name: 'RESTORES a possessive apostrophe after a letter outside the Basic Multilingual Plane, a '
        + 'mathematical-script handle (class ninety-six, mikaela_khara, 2026-09-23: the archive curls '
        + '\u{1D4E0}\u{1D4FE}\u{1D4EA}\u{1D4F7}\u2019s and the bench wrote it straight), and a quoted '
        + 'handle curled as a pair (class one hundred forty-seven)',
      fn: async () => {
        /**
         A handle set in mathematical script, every letter a surrogate pair.
         */
        const handle = '\u{1D4DC}\u{1D4F2}\u{1D4EA}\u{1D4F8}';
        const convention = `Every other line here reads didn${APOSTROPHE}t.`;
        expect(restoreTypography({
          replacement: `Whiskers (${handle}'s classmate in school) and I got off the train.`,
          replaced: 'Whiskers and I got off the train.',
          convention,
        },),).toBe(`Whiskers (${handle}${APOSTROPHE}s classmate in school) and I got off the train.`,);
        expect(restoreTypography({
          replacement: `It was ${handle}'${handle} all along.`,
          replaced: 'It was the pair all along.',
          convention,
        },),).toBe(`It was ${handle}${APOSTROPHE}${handle} all along.`,);
        expect(restoreTypography({
          replacement: `She said '${handle}' and left.`,
          replaced: 'She said the name and left.',
          convention,
        },),).toBe(`She said \u{2018}${handle}${APOSTROPHE} and left.`,);
      },
    },),
    it({
      name: 'RESTORES a possessive apostrophe after a letter written with a combining accent as after its '
        + 'composed spelling, and reads the trailing one after it (ledger B18: the mark stood beside the '
        + 'quote, was no letter, and read as an opening quote); a standalone accent, a mark on a space, is '
        + 'still no space a quotation opens after',
      fn: async () => {
        const convention = `Every other line here reads didn${APOSTROPHE}t.`;
        expect(restoreTypography({
          replacement: "The cafe\u{0301}'s cats sat on the girls' side.",
          replaced: 'The cats sat.',
          convention,
        },),).toBe(`The cafe\u{0301}${APOSTROPHE}s cats sat on the girls${APOSTROPHE} side.`,);
        expect(restoreTypography({
          replacement: "The caf\u{00E9}'s cats sat on the girls' side.",
          replaced: 'The cats sat.',
          convention,
        },),).toBe(`The caf\u{00E9}${APOSTROPHE}s cats sat on the girls${APOSTROPHE} side.`,);
        expect(restoreTypography({
          replacement: "She said 'Noe\u{0301}l' twice.",
          replaced: 'She said it.',
          convention,
        },),).toBe(`She said \u{2018}Noe\u{0301}l${APOSTROPHE} twice.`,);
        expect(restoreTypography({
          replacement: "She said \u{0301}'nap' twice.",
          replaced: 'She said it.',
          convention,
        },),).toBe("She said \u{0301}'nap' twice.",);
      },
    },),
    it({
      name: 'RESTORES three dots where the document writes three dots, collapsing the doubled '
        + 'U+2026 Chinese carries, and leaves a tag alone (yulianNyanner, 2026-09-07)',
      fn: async () => {
        expect(restoreTypography({
          replacement: 'No chance anymore\u{2026}\u{2026} <a href="x…y">so\u{2026}</a>',
          replaced: 'No chance now.',
          convention: 'She waited... and waited...',
        },),).toBe('No chance anymore... <a href="x…y">so...</a>',);
      },
    },),

    it({
      name: 'RESTORES U+2026 where the document writes it, from a run of exactly three dots and '
        + 'not from a longer one',
      fn: async () => {
        expect(restoreTypography({
          replacement: 'Well... no.... never',
          replaced: 'Well, no.',
          convention: 'She waited\u{2026} and waited\u{2026}',
        },),).toBe('Well\u{2026} no.... never',);
      },
    },),

    it({
      name: 'CURLS A LEADING ELISION TO THE CLOSING MARK, the apostrophe, never to an opening quotation mark, WHEN NO LATER STRAIGHT '
        + 'MARK CLOSES IT: `\'tis`, `\'twas`, `\'til`, `\'em`, `\'cause`, `\'round`, a decade like `\'90s`, and the `\'n\'` of '
        + '`rock \'n\' roll`, which never opens a quotation, even with a trailing apostrophe after it',
      fn: async () => {
        const convention = `Every other line here reads didn${APOSTROPHE}t.`;
        /**
         The replacement as the restoration writes it on a curly page.
         */
        function curled(replacement: string,): string {
          return restoreTypography({ replacement, replaced: '', convention, },);
        }
        expect([
          curled("It was 'tis fine.",),
          curled("'Twas the night of the cat dinner.",),
          curled("Wait 'til dinner.",),
          curled("Give 'em all a bowl.",),
          curled("Do it 'cause I said so.",),
          curled("Walk 'round the bowl.",),
          curled("Rock 'n' roll for the cats' bowl.",),
          curled("Back in the '90s there was a cat.",),
          curled("Rock 'n' roll.",),
          curled("The '60s and the '80s.",),
          curled("'Tis 'Em 'Round 'cause",),
        ],).toEqual([
          'It was ’tis fine.',
          '’Twas the night of the cat dinner.',
          'Wait ’til dinner.',
          'Give ’em all a bowl.',
          'Do it ’cause I said so.',
          'Walk ’round the bowl.',
          'Rock ’n’ roll for the cats’ bowl.',
          'Back in the ’90s there was a cat.',
          'Rock ’n’ roll.',
          'The ’60s and the ’80s.',
          '’Tis ’Em ’Round ’cause',
        ],);
      },
    },),

    it({
      name: 'CURLS A QUOTATION THAT BEGINS WITH A LISTED ELISION WORD AS A QUOTATION, the opening mark to the opening single mark and '
        + 'the closing mark to the closing one, since a later straight mark that closes the leading mark makes it the opening of '
        + 'a pair: `\'Tis the season,\' she said`, an ordinary quotation like `\'Round things are best\'`, a decade like '
        + '`\'90s kids\'`, and a possessive apostrophe after it, which closes it as it closes any opening mark',
      fn: async () => {
        const convention = `Every other line here reads didn${APOSTROPHE}t.`;
        /**
         The replacement as the restoration writes it on a curly page.
         */
        function curled(replacement: string,): string {
          return restoreTypography({ replacement, replaced: '', convention, },);
        }
        expect([
          curled("'Tis the season,' she said",),
          curled("She said 'Tis the season,' and left.",),
          curled("'Em and me,' they said",),
          curled("'Round and round,' went the song.",),
          curled("'90s kids,' he said",),
          curled("He said '90s kids' twice.",),
          curled("'Til the end,' she said",),
          curled("'Twas the night,' he read.",),
          curled("'Cause I said so,' she said.",),
          curled("'Em and me' they said",),
          curled("She said 'Round things are best' and left.",),
          curled("'Tis the season, 'Em and me, and 'Round we go.'",),
          curled("It was 'tis the cats' bowl.",),
          curled("In the '90s the cats' bowl was full.",),
        ],).toEqual([
          '‘Tis the season,’ she said',
          'She said ‘Tis the season,’ and left.',
          '‘Em and me,’ they said',
          '‘Round and round,’ went the song.',
          '‘90s kids,’ he said',
          'He said ‘90s kids’ twice.',
          '‘Til the end,’ she said',
          '‘Twas the night,’ he read.',
          '‘Cause I said so,’ she said.',
          '‘Em and me’ they said',
          'She said ‘Round things are best’ and left.',
          '‘Tis the season, ’Em and me, and ’Round we go.’',
          'It was ‘tis the cats’ bowl.',
          'In the ‘90s the cats’ bowl was full.',
        ],);
      },
    },),

    it({
      name: 'NEVER LETS A LISTED ELISION TAKE THE PLACE OF THE OPENING MARK THAT IS WAITING FOR ITS CLOSING MARK, so an elision inside '
        + 'or at the end of a quotation (`\'give \'em hell\'`, `\'sit \'round\'`, `\'wait \'til\'`, `\'the \'90s were fun\'`) curls to '
        + 'the apostrophe, the quotation keeps its own opening mark, a quoted number (`\'90s\'`) pairs its own marks, and a '
        + 'straight page is left alone',
      fn: async () => {
        const convention = `Every other line here reads didn${APOSTROPHE}t.`;
        /**
         The replacement as the restoration writes it on a curly page.
         */
        function curled(replacement: string,): string {
          return restoreTypography({ replacement, replaced: '', convention, },);
        }
        /**
         The replacement as the restoration writes it on a straight page.
         */
        function straight(replacement: string,): string {
          return restoreTypography({ replacement, replaced: '', convention: "Every other line here reads didn't.", },);
        }
        expect([
          curled("He said 'give 'em hell' to us.",),
          curled("She said 'sit 'round' and 'wait 'til' dawn.",),
          curled("He said 'give them 'em' and left.",),
          curled("He said 'it was great 'cause cats' and left.",),
          curled("He said 'the '90s' and left.",),
          curled("'Give 'em hell,' she said.",),
          curled("“She said 'give 'em hell' to us.”",),
          curled("He said 'it was 'twas' and left.",),
          curled("They said 'the '90s were fun' and left.",),
          straight("He said 'give 'em hell' to us.",),
          straight("She said 'sit 'round' and 'wait 'til' dawn.",),
        ],).toEqual([
          'He said ‘give ’em hell’ to us.',
          'She said ‘sit ’round’ and ‘wait ’til’ dawn.',
          'He said ‘give them ’em’ and left.',
          'He said ‘it was great ’cause cats’ and left.',
          // `'90s'` is a quoted number (n156, W7), but the readings that pair its marks or the `'the` before it with the last mark
          // leave the same number of marks unpaired, and a tie keeps the elision (n159): the last mark closes the quotation.
          'He said ‘the ’90s’ and left.',
          '‘Give ’em hell,’ she said.',
          '“She said ‘give ’em hell’ to us.”',
          'He said ‘it was ’twas’ and left.',
          'They said ‘the ’90s were fun’ and left.',
          "He said 'give 'em hell' to us.",
          "She said 'sit 'round' and 'wait 'til' dawn.",
        ],);
      },
    },),

    it({
      name: 'READS THE `\'n\'` OF `rock \'n\' roll` AS ONE UNIT WHOSE MARKS NEVER OPEN OR CLOSE A QUOTATION, so a quotation around it '
        + 'pairs its own marks and both marks of the unit curl to the apostrophe',
      fn: async () => {
        const convention = `Every other line here reads didn${APOSTROPHE}t.`;
        /**
         The replacement as the restoration writes it on a curly page.
         */
        function curled(replacement: string,): string {
          return restoreTypography({ replacement, replaced: '', convention, },);
        }
        expect([
          curled("'Rock 'n' roll,' she said.",),
          curled("She said 'rock 'n' roll rules' and left.",),
          curled("She said 'fish 'n' chips are good' to us.",),
          curled("She said 'rock 'n' roll' and left.",),
          curled("Rock 'n' roll and fish 'n' chips.",),
          curled("'Rock 'N' roll,' she said.",),
          curled("'Rock 'n' roll,' she said. Then 'hi",),
          curled("She said 'rock 'n' roll rules' and left. Then 'hi",),
        ],).toEqual([
          '‘Rock ’n’ roll,’ she said.',
          'She said ‘rock ’n’ roll rules’ and left.',
          'She said ‘fish ’n’ chips are good’ to us.',
          'She said ‘rock ’n’ roll’ and left.',
          'Rock ’n’ roll and fish ’n’ chips.',
          '‘Rock ’N’ roll,’ she said.',
          '‘Rock ’n’ roll,’ she said. Then \'hi',
          'She said ‘rock ’n’ roll rules’ and left. Then \'hi',
        ],);
      },
    },),

    it({
      name: 'CURLS A STRAIGHT SINGLE MARK LEFT UNPAIRED IN CLOSING SHAPE TO THE APOSTROPHE, after a letter, a digit or punctuation, the '
        + 'one glyph of a closing mark and an apostrophe, where no unpaired opening mark could be its partner, and leaves a '
        + 'straight page alone',
      fn: async () => {
        const convention = `Every other line here reads didn${APOSTROPHE}t.`;
        /**
         The replacement as the restoration writes it on a curly page.
         */
        function curled(replacement: string,): string {
          return restoreTypography({ replacement, replaced: '', convention, },);
        }
        expect([
          curled("She said 'tis the dogs' and the cats' bowls,' and left.",),
          curled("The cats' bowls,' she said.",),
          curled("Both cats' bowls?' she asked.",),
          curled("In the 90s' bowls.",),
          curled("She said 'hi' and the cats' bowls.",),
          restoreTypography({
            replacement: "She said 'tis the dogs' and the cats' bowls,' and left.",
            replaced: '',
            convention: "Every other line here reads didn't.",
          },),
        ],).toEqual([
          'She said ‘tis the dogs’ and the cats’ bowls,’ and left.',
          'The cats’ bowls,’ she said.',
          'Both cats’ bowls?’ she asked.',
          'In the 90s’ bowls.',
          'She said ‘hi’ and the cats’ bowls.',
          "She said 'tis the dogs' and the cats' bowls,' and left.",
        ],);
      },
    },),

    it({
      name: 'STILL CURLS ANY OTHER LEADING MARK THAT PAIRS WITH A CLOSING ONE TO AN OPENING QUOTATION MARK, a word that only begins '
        + 'like an elision (`\'tisane\'`, `\'emperor\'`, `\'rounders\'`, `\'1990s\'`) included, and leaves a straight page alone',
      fn: async () => {
        const convention = `Every other line here reads didn${APOSTROPHE}t.`;
        /**
         The replacement as the restoration writes it on a curly page.
         */
        function curled(replacement: string,): string {
          return restoreTypography({ replacement, replaced: '', convention, },);
        }
        expect([
          curled("She said 'Mio' now.",),
          curled("She drank 'tisane' now.",),
          curled("She said 'emperor' now.",),
          curled("She said 'rounders' now.",),
          curled("She said '1990s' now.",),
          curled("She said 'Mio' and the cats' bowl was full.",),
          restoreTypography({ replacement: "It was 'tis fine.", replaced: '', convention: "Every other line here reads didn't.", },),
        ],).toEqual([
          'She said ‘Mio’ now.',
          'She drank ‘tisane’ now.',
          'She said ‘emperor’ now.',
          'She said ‘rounders’ now.',
          'She said ‘1990s’ now.',
          'She said ‘Mio’ and the cats’ bowl was full.',
          "It was 'tis fine.",
        ],);
      },
    },),

    it({
      name: 'LEAVES the ellipsis alone where the document shows both forms or neither',
      fn: async () => {
        expect(restoreTypography({
          replacement: 'Well... no\u{2026}',
          replaced: 'Well, no.',
          convention: 'She waited\u{2026} and waited...',
        },),).toBe('Well... no\u{2026}',);
        expect(restoreTypography({
          replacement: 'Well... no\u{2026}',
          replaced: 'Well, no.',
          convention: 'She waited and waited.',
        },),).toBe('Well... no\u{2026}',);
      },
    },),
  ],
},);
