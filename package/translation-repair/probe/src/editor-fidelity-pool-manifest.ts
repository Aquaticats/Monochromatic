import { DRAWN_EDITOR_SPECS, } from './editor-fidelity-pool-drawn.ts';
import { REVIEWED_FIDELITY_REFERENCES, } from './fidelity-reference-manifest.ts';
import type { FidelityReferenceSpec, } from './fidelity-reference-model.ts';

//region Editor fidelity pool
// THE FROZEN POOL every arm and every later run of the editor fidelity probe reads, so every card sees the same trials.
//
// FOUR TRIALS CANNOT SEPARATE EFFORT LEVELS: every card that fixes all four reads the same. The pool is the three
// source-reviewed references, first and unchanged, and the heading sections drawn beside them from the same corpus pin
// by the rule named THE RULE in this header, which together stand for every trial the pinned clone offers of the two
// defects that have a deterministic fix: a hundred and fifty were asked for in the sixth round and the clone offers fewer.
//
// NOT SOURCE-REVIEWED. A drawn section was chosen by a rule and replayed, never read for meaning by a person. That is
// enough here because the score compares the patched text with the text before the defect was planted, so what the
// reference says is not what is measured; `reviewedOn` on a drawn specification dates its freeze and says nothing
// of review.
//
// THE DRAW IS CODE, NOT A SCRIPT KEPT ELSEWHERE. `editor-fidelity-pool-draw` (`corpus-run/editor-fidelity-pool-draw.ts`)
// writes `editor-fidelity-pool-drawn*.ts` from the pinned clone, and `--check` compares the committed files with a fresh
// draw and replays every committed reference against the clone. The files here are what `--seed round-4 --want 150` gives.
//
// THE RULE, every condition (the constants are in `editor-fidelity-pool-rule.ts`).
//
//   1. Entries are the people of the pinned clone whose id is plain: nonempty, letters, digits, underscore, dot and hyphen,
//      not beginning with a dot. The entries the reviewed references hold are left out.
//   2. The entries are visited in the order of the sha256 of the domain, the seed and the id, joined by a NUL: a seeded
//      sample of all the eligible entries, not an alphabetical prefix. The seed is `round-4`.
//   3. An entry takes part when its Chinese and English pages split into the same number of heading sections with
//      matching leading kinds, which is the production aligner's own fast path.
//   4. A section is eligible when its Chinese is nonempty and at most 2200 characters and its folded English is 400 to
//      2200 characters.
//   5. The insertion trial is the first eligible section, in document order, whose borrowed-sentence twin builds and
//      passes the scoreability check. The donor is the furthest other section that gives a usable sentence, and between
//      two equally far the later section is offered first.
//   6. The alteration trial is the first eligible section other than that one whose altered-number twin builds (a number
//      both pages state, the English stating it once) and passes the check, else that one.
//   7. The draw takes entries in the seeded order until the pool, the reviewed trials counted, holds the trials wanted,
//      or the eligible entries are all taken. Each entry gives at most two trials, so an entry is a cluster of at most
//      two, and the leading trials of the pool are the same for every arm and every redraw of the same seed.
//
// THE SCOREABILITY CHECK, per trial: the trial builds with one envelope and a nonempty quoted region; scoring the
// damaged text reads not fixed; deleting the quoted region does not read as a clean fix (the region of an insertion is
// widened by the neighbouring sentence on each side, so that the deletion removes a sentence that was there); the clean
// defect written alone does not read as a clean fix; and a scripted editor that applies the known fix through the real
// stage path (its sheet, its parse and its apply gate) is read as fixed with nothing else touched. A trial that fails the
// check is passed over and the reason is counted in the draw's log.
//
// THE POPULATION AND WHAT THE DRAW TOOK, from the run that wrote these files: 93 people listed in the clone, 50 of them
// with at least one trial that passes, all 50 taken in the seeded order, 84 drawn trials (47 insertions, 37 alterations)
// with the 4 reviewed trials making 88. The clone offers no more: no entry gives a third trial, and the rule is not
// loosened to reach the 150 asked for. Entries were left out for unequal shape (2) and unreadable pages (1); sections for no
// donor sentence (25), no donor span (16), no shared number (45), and a failed check (6: five where the scripted ideal
// editor read as unusable, one where it scored as collateral).
//
// ONLY IDS, OFFSETS AND HASHES ARE COMMITTED. The corpus is unlicensed. Each run reads the pinned clone and checks every
// range and every damaged twin against these hashes, as it does for the reviewed references.

/**
 The references and heading sections the editor fidelity probe draws its trials from, the reviewed references first.

 @example
 ```ts
 const specs: readonly FidelityReferenceSpec[] = EDITOR_FIDELITY_POOL;
 ```
 */
export const EDITOR_FIDELITY_POOL: readonly FidelityReferenceSpec[] = [
  ...REVIEWED_FIDELITY_REFERENCES,
  ...DRAWN_EDITOR_SPECS,
];

//endregion Editor fidelity pool
