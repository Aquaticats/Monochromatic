import type {
  ArtifactContestVerdict,
  LaneChoice,
} from '@monochromatic-dev/translation-repair-lane-contest/ts';

//region Consolidate standing text
// BASELINE A CONSOLIDATION MUST BEAT at one contested slice. Lane winner is
// standing text. When contest chose neither, archive is comparison baseline so
// third rendering can still be judged; final-selection guard separately refuses
// archive itself unless contest endorsed it.

/**
 Names the rendering a contested slice would ship without this stage.

 A DECLINED CONTEST LEAVES NO LANE STANDING. Archive remains comparison
 baseline rather than invented lane choice, allowing third rendering to be
 judged against current page. Keeping baseline is not approval:
 `finalSelectionFindings` records it as evidence unless contest endorsed archive.

 @param choice - what the lane contest settled

 @param repairText - what the repair lane would ship

 @param translateText - what the translate lane would ship

 @param incumbentText - archive wording available as decline baseline

 @returns Lane winner or archive baseline

 @example
 ```ts
 const standing = standingTextFor({ choice: 'repair', repairText, translateText, incumbentText, },);
 ```
 */
export function standingTextFor(
  {
    choice,
    repairText,
    translateText,
    incumbentText,
  }: {
    readonly choice: LaneChoice;
    readonly repairText: string;
    readonly translateText: string;
    readonly incumbentText: string;
  },
): string {
  if (choice === 'repair')
    return repairText;
  if (choice === 'translate')
    return translateText;
  return incumbentText;
}

/**
 Reports whether standing baseline may become final output unchanged.

 @param choice - lane selected or neither

 @param verdict - full contest result carrying archive endorsement

 @param standingValid - whether baseline passes syntax publication invariants

 @returns Whether prior contest approved publishable standing baseline

 @example
 ```ts
 const mayShip = contestStandingMayShip({ choice, verdict, });
 ```
 */
export function contestStandingMayShip(
  {
    choice,
    verdict,
    standingValid = true,
  }: {
    readonly choice: LaneChoice;
    readonly verdict: ArtifactContestVerdict;
    readonly standingValid?: boolean;
  },
): boolean {
  if (!standingValid)
    return false;
  if (choice !== 'neither')
    return true;
  if (verdict.kind !== 'settled-neither')
    return false;
  return verdict.archive === 'endorsed';
}

//endregion Consolidate standing text
