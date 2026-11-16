/**
 The order of the entries of a queue, kept as a key inside each entry rather than as a place in a list: the queue is a folder of one file
 per entry, so a put at a position writes one file and renumbers nothing.

 A key is a string of base-36 digits read as a fraction after the point, so two keys compare in the order their fractions do and a string
 compare of the keys is the order of the queue. Any two keys can be parted: the digits between theirs are found digit by digit, and when
 they are adjacent the key is extended by one digit, which is how a key grows by one character per insert that lands between two keys
 with nothing between them. A queue written whole gets keys of one length, evenly spaced, so ordinary puts stay the same length.

 @module
 */

/**
 The digits an order key is written in, in the order they count: a plain string compare of two keys of these digits is their order.
 */
const DIGITS = '0123456789abcdefghijklmnopqrstuvwxyz';

/**
 How many digits one key may take before a put refuses to find a place between two keys, which only a queue that has been parted at one
 place more times than a queue of runs will ever be.
 */
const LONGEST_ORDER = 256;

/**
 The key the middle of a whole queue is placed at: the digit that leaves as much room before it as after it.
 */
const MIDDLE_DIGIT_PLACE = Math.floor(DIGITS.length / 2);

/**
 What a key that is no key of this module is, as a symbol so no key can be mistaken for one.
 */
export const NOT_AN_ORDER: unique symbol = Symbol('this string is no order key',);

/**
 Reads the place of one digit of a key.

 @param digit - the digit

 @returns the digit's place, counted from zero, or minus one when the character is no digit

 @example
 ```ts
 const place = placeOfADigit('a');
 ```
 */
function placeOfADigit(digit: string,): number {
  return DIGITS.indexOf(digit,);
}

/**
 Reads the digit at one place.

 @param place - the place, counted from zero

 @returns the digit

 @example
 ```ts
 const digit = digitAt(10);
 ```
 */
function digitAt(place: number,): string {
  return DIGITS.slice(
    place,
    place + 1,
  );
}

/**
 Reads whether a value is one order key.

 @param value - the value, of any shape

 @returns whether it is a key this module wrote

 @example
 ```ts
 const fine = isAnOrder('i');
 ```
 */
export function isAnOrder(value: unknown,): value is string {
  return ((typeof value) === 'string')
    && (value.length > 0)
    && (value.length <= LONGEST_ORDER)
    && [...value,].every(function isADigit(digit,) {
      return placeOfADigit(digit,) !== (-1);
    },);
}

/**
 Reads the key of an entry, or that the entry carries none this module wrote.

 @param order - what the entry says its order is

 @returns the key, or that it is none

 @example
 ```ts
 const key = anOrderIn({ order: 'i' });
 ```
 */
export function anOrderIn({ order, }: { readonly order: unknown },): string | typeof NOT_AN_ORDER {
  return isAnOrder(order,) ? order : NOT_AN_ORDER;
}

/**
 Reads a key that stands between two others, which is what a put at a position writes.

 @param lower - the key the new one must come after, or nothing when it goes at the head

 @param upper - the key the new one must come before, or nothing when it goes at the tail

 @returns a key strictly between them

 @throws Error when no key is found, which only a place parted more times than a key may grow allows

 @example
 ```ts
 const key = anOrderBetween({ lower: 'i', upper: 'j' });
 ```
 */
export function anOrderBetween({
  lower,
  upper,
}: {
  readonly lower?: string;
  readonly upper?: string;
},): string {
  /**
   The digits of the key so far, which are the lower bound's digits where there was no room between the two bounds.
   */
  let before = '';
  for (let place = 0; ; place += 1) {
    /**
     The lower bound's digit at this place, which a key that has ended reads as zero.
     */
    const low = (place < (lower ?? '').length)
      ? placeOfADigit((lower ?? '').slice(
        place,
        place + 1,
      ),)
      : 0;
    /**
     The upper bound's digit at this place, which a bound that has ended reads as past every digit.
     */
    const high = (upper === undefined)
      ? DIGITS.length
      : ((place < upper.length)
        ? placeOfADigit(upper.slice(
          place,
          place + 1,
        ),)
        : DIGITS.length);
    if ((high - low) > 1) {
      return `${before}${digitAt(Math.floor((low + high) / 2,),)}`;
    }
    if (before.length >= LONGEST_ORDER) {
      throw new Error(
        `no order key fits between ${String(lower,)} and ${String(upper,)}: this place has been parted more times than a key may grow`,
      );
    }
    before = `${before}${digitAt(low,)}`;
  }
  /* oxlint-enable no-constant-condition */
}

/**
 Reads the keys of a queue of a given length written whole, one key for each place, in order and evenly spaced.

 @param count - how many entries the queue holds

 @returns one key per place, in order

 @example
 ```ts
 const orders = ordersOfAPlace({ count: 3 });
 ```
 */
export function ordersOfAPlace({ count, }: { readonly count: number },): readonly string[] {
  /**
   The longest key a queue of this length needs to leave a digit of room between every two of its places.
   */
  const digits = Math.max(
    1,
    Math.ceil(Math.log(count + 1,) / Math.log(DIGITS.length,),),
  );
  /**
   The places between one key and the next: every place gets its own stretch, so a put between two of them finds a digit at once.
   */
  const step = Math.floor((DIGITS.length ** digits) / (count + 1),);
  return Array.from(
    { length: count, },
    function orderAt(
      _,
      place,
    ) {
      return (step * (place + 1)).toString(DIGITS.length,)
        .padStart(
        digits,
        '0',
        );
    },
  );
}

/**
 Reads the key a queue's head is written at, which the first entry of a whole queue takes when its place is the head.

 @returns the key

 @example
 ```ts
 const head = theOrderOfAHead();
 ```
 */
export function theOrderOfAHead(): string {
  return digitAt(MIDDLE_DIGIT_PLACE,);
}
