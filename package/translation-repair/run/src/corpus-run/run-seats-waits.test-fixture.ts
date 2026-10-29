//region Waits of a seat reading
// TEST SUPPORT, NOT PACKAGE SOURCE. Reads the lines a seat reading wrote for the waits it took, for the cases that count
// how many cooldowns a reader waited out. Shared by the cooldown cases of the rig over the budgets and of the run client.

/**
 The readings of the translate lane that waited for a hold to end.

 @param lines - lines the seat reader wrote

 @returns Waiting lines

 @example
 ```ts
 expect(waitsIn({ lines, },).length,).toBe(1,);
 ```
 */
export function waitsIn({ lines, }: { readonly lines: readonly string[]; },): readonly string[] {
  return lines.filter(function isWait(line,): boolean {
    return line.startsWith('warn JUDGE SEATS phase=translate lane short of quorum',)
      && line.includes('waiting ',);
  },);
}

//endregion Waits of a seat reading
