//region Refusal order
// TWO WAYS TO REFUSE THAT END IN AN ORDER A CASE CHOOSES.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. A case that wants to know which of two
// concurrent members a site reports must make one member refuse first and the
// other refuse after that, by construction, so the machine's load cannot
// reorder them. The member that refuses at once opens a gate as it refuses;
// the one that refuses after that is raised only once the gate has opened, a
// turn of the microtask queue later, which no timer, child or disk decides.

/**
 Refusals of one case, tied together by the gate the first opens.

 @example
 ```ts
 const { refuseAtOnce, refuseAfterThat, }: RefusalOrder = refusalOrder();
 ```
 */
type RefusalOrder = {
  /**
   Rejects with the given error at once, opening the gate on the way.
   */
  readonly refuseAtOnce: (error: Error,) => Promise<never>;

  /**
   Rejects with the given error once a refusal at once has happened.
   */
  readonly refuseAfterThat: (error: Error,) => Promise<never>;
};

/**
 Makes the two refusals of one case.

 @returns A refusal that ends at once and one that ends after it

 @example
 ```ts
 const { refuseAtOnce, refuseAfterThat, } = refusalOrder();
 await refuseAtOnce(new Error('the cat basket is empty',),);
 ```
 */
export function refusalOrder(): RefusalOrder {
  /**
   Opened by the first refusal, which the later one waits for.
   */
  const opened = Promise.withResolvers<undefined>();
  return {
    refuseAtOnce: function refuseAtOnce(error: Error,): Promise<never> {
      opened.resolve(undefined,);
      return Promise.reject(error,);
    },
    refuseAfterThat: async function refuseAfterThat(error: Error,): Promise<never> {
      await opened.promise;
      throw error;
    },
  };
}

//endregion Refusal order
