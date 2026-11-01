import { isWholeNumberText, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Stage roster
// Whether a fan-out stage ran with the roster it was configured with.
//
// Renamed from the refine-only version once the same failure appeared in the
// editor stage: a count that answers "could this stage speak" is not a property
// of the naturalness lane, it is a property of every stage that fans out.
//
// The stages that matter here write one finding per unit of work naming how
// many voices they heard out of how many they asked, for example
// `editor-candidates (1/2 heard, 1 repairing)`. That string is the ONLY record
// that a stage ran degraded: the outcome it produces is shaped exactly like the
// outcome of a healthy stage with less to do.
//
// Why this is worth counting, from live data rather than in principle. Between
// two consecutive passes on unchanged pipeline code, one model went from zero
// schema-mismatches to 61 across all four roles it holds. In the pass where it
// failed, one settled entry repaired all nine of its chunks with a single
// editor heard out of two, and nothing anywhere reported a fault. The editor
// ensemble exists precisely so no single model writes the shipped text, and it
// had stopped being an ensemble.

/**
 Text that closes the heard count in a stage finding.
 */
const HEARD_SUFFIX = ' heard';

/**
 Separator between voices heard and voices asked.
 */
const HEARD_SEPARATOR = '/';

/**
 How a fan-out stage fared across the artifacts read.

 @example
 ```ts
 const roster: StageRosterCoverage = { offered: 101, degraded: 12, silent: 6, };
 ```
 */
export type StageRosterCoverage = {
  /**
   Units of work the stage was asked to do.
   */
  readonly offered: number;

  /**
   Units where fewer voices answered than were configured.
   */
  readonly degraded: number;

  /**
   Units where NO voice answered, so the stage did not run at all there.
   */
  readonly silent: number;
};

/**
 Reads the heard and configured counts out of one stage finding.

 Scanned by index rather than matched by pattern. The two numbers sit between
 a known prefix and a known suffix, which an index scan states directly, and a
 finding this reader cannot parse must be skipped rather than throw: this
 count exists to notice a stage going quiet, so refusing to read drifted
 wording would silence it in exactly the case it was built for.

 @param finding - stage finding, already known to carry the prefix

 @param prefix - stage prefix the finding opens with

 @returns Voices heard and voices asked, or nothing when unreadable

 @example
 ```ts
 const voices = readVoices({ finding, prefix: 'editor-candidates (', },);
 ```
 */
function readVoices(
  {
    finding,
    prefix,
  }: {
    readonly finding: string;
    readonly prefix: string;
  },
): readonly {
  readonly heard: number;
  readonly asked: number
}[] {
  /**
   Where the heard count ends.
   */
  const suffixAt = finding.indexOf(
    HEARD_SUFFIX,
    prefix.length,
  );
  if (suffixAt === (-1))
    return [];

  /**
   The `heard/asked` pair, without its surroundings.
   */
  const pair = finding.slice(
    prefix.length,
    suffixAt,
  );

  /**
   Where the two counts divide.
   */
  const separatorAt = pair.indexOf(HEARD_SEPARATOR,);
  if (separatorAt === (-1))
    return [];

  /**
   Heard count as written.
   */
  const heard = pair.slice(
    0,
    separatorAt,
  );
  /**
   Asked count as written.
   */
  const asked = pair.slice(separatorAt + HEARD_SEPARATOR.length,);
  // DIGITS AS THE STAGE WRITES THEM, or the finding is skipped like any other
  // drifted wording: `Number` read an empty heard count as a stage that heard
  // nobody, and took a sign, a radix or an exponent (ledger B73).
  if ((!isWholeNumberText({ text: heard, },)) || (!isWholeNumberText({ text: asked, },)))
    return [];
  return [{
    heard: Number(heard,),
    asked: Number(asked,),
  },];
}

/**
 Counts how often one fan-out stage ran below its configured roster.

 @param entries - per-artifact findings, verbatim

 @param stage - stage prefix as the pipeline writes it, such as `editor`

 @returns Units offered, units run degraded, and units nobody answered

 @example
 ```ts
 const roster = summarizeStageRoster({ entries, stage: 'editor', },);
 ```
 */
export function summarizeStageRoster(
  {
    entries,
    stage,
  }: {
    readonly entries: readonly (readonly string[])[];
    readonly stage: string;
  },
): StageRosterCoverage {
  /**
   Prefix every finding of this stage opens with.
   */
  const prefix = `${stage}-candidates (`;

  /**
   Voice counts of every readable finding this stage wrote.
   */
  const voices = entries
    .flat()
    .filter(function isStageFinding(finding,) {
      return finding.startsWith(prefix,);
    },)
    .flatMap(function toVoices(finding,) {
      return readVoices({
        finding,
        prefix,
      },);
    },);

  return {
    offered: voices.length,
    degraded: voices
      .filter(function ranShort(count,) {
        return count.heard < count.asked;
      },)
      .length,
    silent: voices
      .filter(function ranSilent(count,) {
        return count.heard === 0;
      },)
      .length,
  };
}

//endregion Stage roster
