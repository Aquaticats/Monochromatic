import type { SeatRole, } from './seat-role.ts';
import type { RosterModelId, } from './roster-id.ts';

//region Seat role standing
// THE STANDING OF A MODEL IN THE ROLE IT STANDS IN FOR (ledger B1144). The owner's answer of 2026-10-31, "Order by role
// standing (Recommended)": the stand-ins of an editor or a refiner whose reasoning ran away are asked best first by the
// standing the editor calibration recorded for that role, and the models never measured in it follow, in the order of
// their share of runaways. Every other role has no standing recorded and keeps the lowest share of runaways first
// ("Lowest runaway share (Recommended)", 2026-10-30).
//
// ONE RECORDED SOURCE, NOT A HAND-ORDERED LIST: the table `ROLE_STANDINGS` holds the shares as the calibration printed them, and the
// order is derived from them by `standingOrderOf`. A model of the calibration that has since left the roster
// (`deepseek-v4-pro-0813`, `deepseek-v4-flash-0731`, `hf:openai/gpt-oss-120b`) is not in the table, since no roster model
// can be asked by it.

/**
 One model's share of the disinterested ballots in a role, as the calibration recorded it.

 @example
 ```ts
 const row: RoleStanding = { modelId: 'hf:Qwen/Qwen3.8-27B', share: 17.3, };
 ```
 */
export type RoleStanding = {
  /**
   Model measured in the role.
   */
  readonly modelId: RosterModelId;

  /**
   Availability-adjusted share of disinterested ballots, in percent, against the pooled null of the calibration.
   */
  readonly share: number;
};

/**
 The standings recorded for the roles that have one.

 THE EDITOR CALIBRATION OF 2026-09-01 (`editor-calibrate` over 40 bench slices at build `48799e6d1`; the figures and the
 method are on `RUN_MODELS` in `run-config.ts` and in `doc/decision/translation-repair-roster-seating-2026-09-01.md`):
 111 judged editor rounds, pooled null 15.8 percent, and 25 judged refiner rounds, pooled null 14.0 percent. The editor
 shares are GLM-5.3-Flash 32.2, `glm-5.3` 21.4, Qwen3.8-27B 17.3, Kimi-K3 14.2, minimax-m3 8.4 and gemma-4-26b 7.0; the
 refiner shares are GLM-5.3-Flash 32.5, `glm-5.3` 12.1, minimax-m3 9.7, Kimi-K3 7.8, Qwen3.8-27B 7.7 and gemma-4-26b 3.4.
 `google.gemma-4-e2b` and `inception/mercury-2.5` were not measured in either role.

 @example
 ```ts
 const editors = ROLE_STANDINGS.editor;
 ```
 */
export const ROLE_STANDINGS: Readonly<Record<'editor' | 'refiner', readonly RoleStanding[]>> = {
  editor: [
    {
      modelId: 'hf:zai-org/GLM-5.3-Flash',
      share: 32.2,
    },
    {
      modelId: 'glm-5.3',
      share: 21.4,
    },
    {
      modelId: 'hf:Qwen/Qwen3.8-27B',
      share: 17.3,
    },
    {
      modelId: 'hf:moonshotai/Kimi-K3',
      share: 14.2,
    },
    {
      modelId: 'minimax-m3',
      share: 8.4,
    },
    {
      modelId: 'gemma-4-26b-a4b-it',
      share: 7,
    },
  ],
  refiner: [
    {
      modelId: 'hf:zai-org/GLM-5.3-Flash',
      share: 32.5,
    },
    {
      modelId: 'glm-5.3',
      share: 12.1,
    },
    {
      modelId: 'minimax-m3',
      share: 9.7,
    },
    {
      modelId: 'hf:moonshotai/Kimi-K3',
      share: 7.8,
    },
    {
      modelId: 'hf:Qwen/Qwen3.8-27B',
      share: 7.7,
    },
    {
      modelId: 'gemma-4-26b-a4b-it',
      share: 3.4,
    },
  ],
};

/**
 The models measured in a role, best standing first.

 @param role - role the stand-ins are asked for

 @param modelIds - models to place

 @returns The models of the list that the role has a standing for, by standing, best first; none for a role with no
 standing recorded, so the caller keeps its other order for it

 @example
 ```ts
 const measured = standingOrderOf({ role: 'editor', modelIds, },);
 ```
 */
export function standingOrderOf(
  {
    role,
    modelIds,
  }: {
    readonly role: SeatRole;
    readonly modelIds: readonly RosterModelId[];
  },
): readonly RosterModelId[] {
  return ((role === 'editor') || (role === 'refiner') ? ROLE_STANDINGS[role] : [])
    .filter(function isAmong(row,): boolean {
      return modelIds.includes(row.modelId,);
    },)
    .toSorted(function bestFirst(
      left,
      right,
    ): number {
      return right.share - left.share;
    },)
    .map(function nameOf(row,): RosterModelId {
      return row.modelId;
    },);
}

//endregion Seat role standing
