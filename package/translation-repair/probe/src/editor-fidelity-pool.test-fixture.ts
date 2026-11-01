import {
  CORPUS_COMMIT_SHA,
  type CorpusPin,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';

//region Invented pages for the editor fidelity pool draw
// An entry of three headed sections, English and Chinese, written for the tests: no corpus content. Each English section
// is longer than the draw's floor and holds sentences of its own, so a sentence can be borrowed from another section,
// and the second section states a year the Chinese states too, so a number can be altered in it.

/**
 Pin the draw tests run under; no clone is read.
 */
export const DRAW_PIN: CorpusPin = {
  cloneDir: '/nonexistent/clone',
  commitSha: CORPUS_COMMIT_SHA,
};

/**
 First English section: no number in it.
 */
const ENGLISH_ONE = '## One\nThe grey cat slept beside the window of the old bookshop every single afternoon. '
  + 'A kettle of green tea cooled slowly on the shelf above the counter near the door. '
  + 'Visitors stepped carefully around the stacks of dusty atlases piled across the wooden floor. '
  + 'Nobody in the street remembered who had painted the faded sign over the narrow entrance. '
  + 'The shopkeeper polished the brass scales each morning before he opened the heavy shutters. '
  + 'Rain tapped gently against the glass while the cat turned over and purred in her sleep.';

/**
 Second English section: states the year 1987 once.
 */
const ENGLISH_TWO = '## Two\nIn 1987 the shopkeeper hung a small brass bell above the narrow green door. '
  + 'Children from the neighbouring lane came to listen to its thin silvery note after school. '
  + 'The cat followed every visitor to the poetry shelf and watched them choose a slim volume. '
  + 'Autumn leaves drifted across the threshold and collected beneath the ancient oak reading table. '
  + 'A retired sailor traded charts of distant harbours for a pot of jasmine tea every Friday. '
  + 'Evening lamps were lit one by one until the whole room glowed like a quiet lantern.';

/**
 Third English section: no number in it.
 */
const ENGLISH_THREE = '## Three\nThe mapmaker upstairs coloured her rivers with ink she ground from blue mussel shells. '
  + 'On market days the lane filled with the smell of roasted chestnuts and warm cinnamon bread. '
  + 'A travelling puppeteer lodged in the attic and rehearsed his stories for the winter fair. '
  + 'Snow covered the roofs so gently that the pigeons hardly noticed it had begun to fall. '
  + 'The baker next door left a loaf on the sill for the cat whenever the oven ran too hot. '
  + 'By the new year every shelf of the shop held a lantern waiting for someone to borrow it.';

/**
 Chinese of the sections, aligned by position; only the second states the year.
 */
const CHINESE = '## 一\n灰猫每天下午在旧书店的窗边睡觉。\n\n## 二\n1987年，店主在绿色的窄门上挂了一只小铜铃。\n\n## 三\n楼上的制图师用蓝贻贝壳磨成的墨水给河流上色。\n';

/**
 The English page of the invented entry.
 */
export const DRAW_ENGLISH: string = `${ENGLISH_ONE}\n\n${ENGLISH_TWO}\n\n${ENGLISH_THREE}\n`;

/**
 The Chinese page of the invented entry.
 */
export const DRAW_CHINESE: string = CHINESE;

/**
 What reading an invented entry gives: its two pages, or that it cannot be read.
 */
type InventedEntry =
  | {
    /**
     Chinese page.
     */
    readonly zh: string;

    /**
     English page.
     */
    readonly en: string;
  }
    | {
    /**
     The entry cannot be read.
     */
      readonly unreadable: true;
    };

/**
 Reads an invented entry: every id has the same pages, except one whose English has an extra section and so is not of
 equal shape.

 @param entryId - id read

 @returns The two pages, or that the entry cannot be read

 @example
 ```ts
 const pages = readInventedEntry({ entryId: 'tabby', },);
 ```
 */
export function readInventedEntry(
  { entryId, }: { readonly entryId: string; },
): InventedEntry {
  if (entryId === 'ghost')
    return { unreadable: true, };
  if (entryId === 'lopsided') {
    return {
      zh: DRAW_CHINESE,
      en: `${DRAW_ENGLISH}\n## Four\nOne more section the Chinese does not have.\n`,
    };
  }
  return {
    zh: DRAW_CHINESE,
    en: DRAW_ENGLISH,
  };
}

//endregion Invented pages for the editor fidelity pool draw
