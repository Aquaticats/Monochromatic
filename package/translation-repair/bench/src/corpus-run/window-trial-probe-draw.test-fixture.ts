/**
 Test-only pages and corpus pin for the window trial's walk: entries whose
 pages the screen flags or leaves alone, and a reader of them that records
 what it was asked.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import type { CorpusPin, } from '@monochromatic-dev/translation-repair-corpus-source';

/**
 Hex digits in a git commit id.
 */
const COMMIT_HEX_LENGTH = 40;

/**
 Times the sentence repeats, enough that the section outweighs its translation
 by far.
 */
const SENTENCE_REPEATS = 5;

/**
 Pin the cases read at, which no reader of theirs ever opens.
 */
export const CAT_PIN: CorpusPin = {
  cloneDir: '/nowhere/cat-clone',
  commitSha: 'a'.repeat(COMMIT_HEX_LENGTH,),
};

/**
 One sentence the original repeats, long enough that a section of it leaves a
 translation of a few characters well under what it deserves.
 */
const LONG_SECTION = '猫猫在窗台上睡觉，尾巴垂在暖气片旁边，阳光落在它的背上，它偶尔睁开一只眼睛看看外面的麻雀。'.repeat(SENTENCE_REPEATS,);

/**
 Original of an entry whose middle section is long and whose archive English
 leaves it with its heading alone, which the screen flags as untranslated.
 */
export const FLAGGED_SOURCE: string = `## 第一节\n\n猫猫在窗台上睡觉。\n\n## 第二节\n\n${LONG_SECTION}\n\n## 第三节\n\n猫猫有自己的碗。\n`;

/**
 Archive English of the flagged entry.
 */
export const FLAGGED_TARGET = '## Section one\n\nThe cat sleeps on the sill.\n\n## Section two\n\n## Section three\n\nThe cat has a bowl.\n';

/**
 Identity block the flagged entry's headings alone give its preparation, which every entry with headings carries.
 */
export const FLAGGED_IDENTITY: string = 'NAMES AND LINKED TEXT THE ARCHIVE RENDERS ON THIS PAGE (render the same person or title the same way '
  + 'everywhere; a declared name inside a title takes its declared form):\n- 第一节 (heading): "Section one"\n'
  + '- 第二节 (heading): "Section two"\n- 第三节 (heading): "Section three"';

/**
 Line the death of 2025-02-26 adds to the head of an entry's identity block.
 */
export const DEATH_LINE: string = '- died: the date this person died, as the corpus records it: "2025-02-26".';

/**
 Original of an entry the screen leaves alone.
 */
export const CLEAN_SOURCE = '## 第一节\n\n猫猫在窗台上睡觉。\n\n## 第二节\n\n猫猫有自己的碗。\n';

/**
 Archive English of the clean entry.
 */
export const CLEAN_TARGET = '## Section one\n\nThe cat sleeps on the sill.\n\n## Section two\n\nThe cat has a bowl.\n';
