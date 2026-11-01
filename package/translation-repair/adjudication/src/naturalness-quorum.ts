import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';

/**
 Tests whether a recorded wider bench can contain its independently named seats.

 @param quorumOver - effective bench size used for exact-half quorum

 @param seatCount - requested or recorded distinct seats

 @returns Whether runtime and artifact can share this quorum basis

 @example
 ```ts
 validNaturalnessQuorum({ quorumOver: 9, seatCount: 6 });
 ```
 */
export function validNaturalnessQuorum(
  {
    quorumOver,
    seatCount,
  }: {
    readonly quorumOver: number;
    readonly seatCount: number;
  },
): boolean {
  return Number.isSafeInteger(quorumOver,) && (quorumOver >= seatCount);
}

/**
 Invalid quorum configuration, rejected before any reviewer is asked.
 */
export class NaturalnessQuorumError extends Error {
  /**
   Only numeric configuration is exposed by this diagnostic.
   */
  readonly messageNamesOnly: true = true;

  /**
   Names only bench counts, never candidate wording.

   @param quorumOver - caller-supplied bench size

   @param seatCount - independently requested seats

   @example
   ```ts
   throw new NaturalnessQuorumError({ quorumOver: 2, seatCount: 3 });
   ```
   */
  constructor(
    {
      quorumOver,
      seatCount,
    }: {
      readonly quorumOver: number;
      readonly seatCount: number;
    },
  ) {
    super(
      `Naturalness quorum basis ${String(quorumOver,)} must be a safe integer covering ${String(seatCount,)} `
        + `requested ${
          wordForCount({
            count: seatCount,
            one: 'seat',
            many: 'seats',
          },)
        }.`,
    );
    this.name = 'NaturalnessQuorumError';
  }
}
