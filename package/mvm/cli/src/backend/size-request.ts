/**
 What a caller asked a machine to have, and the refusal a backend raises when
 nothing it may use fits.

 The two numbers are floors, not exact values: a provider sells sizes it chose,
 so a request for `--vcpus 2 --memory 4` gives a machine with at least two
 virtual CPUs and at least four gibibytes of memory, and the smallest size that
 satisfies both. Naming a size outright, through `--server-type` or the
 provider's own environment variable, is the other way to size a machine, and the
 two ways are never combined: they can disagree, and the refusal says so.

 @module
 */

//region Names

/**
 Smallest number of virtual CPUs a request may carry, because a machine with none
 runs no work.
 */
export const MINIMUM_VCPUS = 1;

/**
 Smallest memory a request may carry, in gibibytes.
 */
export const MINIMUM_MEMORY_GIB = 1;

/**
 How the request is written when nothing was asked.
 */
const NOTHING_ASKED = 'nothing';

/**
 Answer for a number that was not given, so absence travels as a distinct value
 rather than a nullish union.
 */
export const NO_NUMBER: unique symbol = Symbol(
  'returned for a number the caller did not give',
);

//endregion Names

//region The request

/**
 The size a caller asked a machine to have; either number may be absent.
 
 @example
 ```ts
 const request: SizeRequest = { vcpus: 2, memoryGiB: 4 };
 ```
 */
export type SizeRequest = {
  /**
   Fewest virtual CPUs the machine may have.
   */
  readonly vcpus?: number;
  /**
   Least memory the machine may have, in gibibytes.
   */
  readonly memoryGiB?: number;
};

/**
 Builds a request from two possibly absent numbers, so a caller never writes the
 same spread three times.
 
 @param vcpus - fewest virtual CPUs asked for, or {@link NO_NUMBER}
 
 @param memoryGiB - least memory asked for in gibibytes, or {@link NO_NUMBER}
 
 @returns the request, carrying only the numbers that were given
 
 @example
 ```ts
 sizeRequest({ memoryGiB: NO_NUMBER, vcpus: 2 }); // { vcpus: 2 }
 ```
 */
export function sizeRequest({
  memoryGiB,
  vcpus,
}: {
  readonly memoryGiB: number | typeof NO_NUMBER;
  readonly vcpus: number | typeof NO_NUMBER;
},): SizeRequest {
  return {
    ...((memoryGiB === NO_NUMBER) ? {} : { memoryGiB, }),
    ...((vcpus === NO_NUMBER) ? {} : { vcpus, }),
  };
}

/**
 Whether neither number was given, which is the case in which a backend does
 exactly what it did before the numbers existed.
 
 @param request - the request under test
 
 @returns whether the request carries no number
 
 @example
 ```ts
 isEmptySizeRequest({ request: {} }); // true
 ```
 */
export function isEmptySizeRequest({
  request,
}: {
  readonly request: SizeRequest;
},): boolean {
  return (request.vcpus === undefined) && (request.memoryGiB === undefined);
}

/**
 Describes a request as the floors it sets, for an error a person reads.
 
 @param request - the request to describe
 
 @returns one clause per number given, joined with `and`, or `nothing` when empty
 
 @example
 ```ts
 describeSizeRequest({ request: { vcpus: 2, memoryGiB: 4 } });
 // 'at least 2 vCPUs and at least 4 GiB of memory'
 ```
 */
export function describeSizeRequest({
  request,
}: {
  readonly request: SizeRequest;
},): string {
  /**
   One clause per number given, in the order a person reads them.
   */
  const clauses: string[] = [];
  if (request.vcpus !== undefined) {
    clauses.push(`at least ${String(request.vcpus,)} vCPUs`,);
  }
  if (request.memoryGiB !== undefined) {
    clauses.push(`at least ${String(request.memoryGiB,)} GiB of memory`,);
  }
  if (clauses.length === 0) {
    return NOTHING_ASKED;
  }
  return clauses.join(' and ',);
}

//endregion The request

//region Reading the numbers a caller gave

/**
 Reads one of the two numbers, refusing anything that is not a whole number of at
 least one.
 
 @param value - the value as it arrived, from a command line or an MCP call
 
 @param name - what the number is called in the refusal, such as `vcpus`
 
 @param minimum - the smallest value the number may carry
 
 @returns the number, in a list of one, or an empty list when the value was absent
 
 @throws Error when the value is absent from the type, not a number, not whole, or below the minimum
 
 @example
 ```ts
 positiveWholeNumber({ name: 'vcpus', value: 2, minimum: 1 }); // 2
 ```
 */
function positiveWholeNumber({
  minimum,
  name,
  value,
}: {
  readonly minimum: number;
  readonly name: string;
  readonly value: unknown;
},): readonly number[] {
  if (value === undefined) {
    return [];
  }
  if (((typeof value) !== 'number') || (!Number.isInteger(value,))
    || (value < minimum)) {
    throw new Error(
      `${name} must be a whole number of at least ${String(minimum,)}, and ${JSON.stringify(value,)} is not`,
    );
  }
  return [value,];
}

/**
 Reads the requested number of virtual CPUs.
 
 @param value - the value as it arrived, or `undefined` when it was not given
 
 @returns the number, in a list of one, or an empty list when the value was absent of virtual CPUs, or {@link NO_NUMBER}
 
 @throws Error when the value is not a whole number of at least {@link MINIMUM_VCPUS}
 
 @example
 ```ts
 vcpusArgument(2); // 2
 vcpusArgument(0); // throws
 ```
 */
export function vcpusArgument(value: unknown,): number | typeof NO_NUMBER {
  /**
   The number the value carried, or none.
   */
  const [vcpus,] = positiveWholeNumber({
    minimum: MINIMUM_VCPUS,
    name: 'vcpus',
    value,
  },);
  return vcpus ?? NO_NUMBER;
}

/**
 Reads the requested amount of memory, in gibibytes.
 
 @param value - the value as it arrived, or `undefined` when it was not given
 
 @returns the memory in gibibytes, or {@link NO_NUMBER}
 
 @throws Error when the value is not a whole number of at least {@link MINIMUM_MEMORY_GIB}
 
 @example
 ```ts
 memoryGiBArgument(4); // 4
 memoryGiBArgument(0); // throws
 ```
 */
export function memoryGiBArgument(value: unknown,): number | typeof NO_NUMBER {
  /**
   The number the value carried, or none.
   */
  const [memoryGiB,] = positiveWholeNumber({
    minimum: MINIMUM_MEMORY_GIB,
    name: 'memory',
    value,
  },);
  return memoryGiB ?? NO_NUMBER;
}

//endregion Reading the numbers a caller gave

//region Refusing a named size beside a number

/**
 Refuses a create that names a size outright and also sets a floor, because the
 two can disagree and neither is the other's correction.
 
 @param override - the size named by `--server-type` or its environment variable, `''` for none
 
 @param overrideOptions - how a caller names a size on this backend, for the refusal
 
 @param request - the floors the caller set
 
 @throws Error when both a named size and a floor were given
 
 @example
 ```ts
 refuseSizeAndOverride({ override: 'b3-8', overrideOptions: '--server-type', request: { vcpus: 2 } });
 ```
 */
export function refuseSizeAndOverride({
  override,
  overrideOptions,
  request,
}: {
  readonly override: string;
  readonly overrideOptions: string;
  readonly request: SizeRequest;
},): void {
  if ((override === '') || isEmptySizeRequest({ request, },)) {
    return;
  }
  throw new Error(
    `a size is named two ways at once: ${overrideOptions} names "${override}", and --vcpus or --memory asks for ${
      describeSizeRequest({ request, },)
    }; name a size with ${overrideOptions} alone, or set --vcpus and --memory and let the backend pick the smallest size that fits`,
  );
}

//endregion Refusing a named size beside a number

//region The refusal when nothing fits

/**
 Nothing the backend may use satisfies the request, so the machine is not made.
 
 @example
 ```ts
 try { await selectFlavor({ flavors, region, request }); }
 catch (error) { if (error instanceof NoSizeFitsError) report(error.message); }
 ```
 */
export class NoSizeFitsError extends Error {
  /**
   What the caller asked, as {@link describeSizeRequest} renders it.
   */
  readonly asked: string;

  /**
   How many sizes in stock the family rule left out, or zero when it left none.
   */
  readonly leftOut: number;

  /**
   The largest size the family rule left out, described, or `''` when it left none.
   */
  readonly largestLeftOut: string;

  /**
   @param asked - the floors the caller set, described back
 
   @param largest - the largest size the backend could have used, described, or `''` when it had none
 
   @param largestLeftOut - the largest such size, described, or `''` when none was left out
 
   @param leftOut - how many sizes in stock the family rule left out
 
   @param overrideOptions - how a caller names a size on this backend
 
   @param place - where the search ran, such as `region BHS5 on OVHcloud`
   */
  constructor({
    asked,
    largest,
    largestLeftOut = '',
    leftOut = 0,
    overrideOptions,
    place,
  }: {
    readonly asked: string;
    readonly largest: string;
    readonly largestLeftOut?: string;
    readonly leftOut?: number;
    readonly overrideOptions: string;
    readonly place: string;
  },) {
    super(
      `nothing ${place} can be made with ${asked}: ${
        (largest === '') ? 'no size it may use is in stock there' : `the largest size it could have used was ${largest}`
      }${
        (leftOut === 0)
          ? ''
          : `, and ${String(leftOut,)} size${(leftOut === 1) ? '' : 's'} in stock ${(leftOut === 1) ? 'was' : 'were'} left out as a family mvm does not pick without being told${
            (largestLeftOut === '') ? '' : `, the largest being ${largestLeftOut}`
          }`
      }; name a size outright with ${overrideOptions}`,
    );
    this.name = 'NoSizeFitsError';
    this.asked = asked;
    this.largestLeftOut = largestLeftOut;
    this.leftOut = leftOut;
  }
}

//endregion The refusal when nothing fits
