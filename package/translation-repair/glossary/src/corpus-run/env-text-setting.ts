//region Environment text setting
// A text setting read from the environment, with an exported-but-empty
// variable folded into absence (ledger D15). `artifact-pool.ts` states the
// reason for its own: an empty export is an ordinary shell accident, not a
// value anyone chose, and a seed or a sheet name of nothing draws a sample or
// reads a file nobody asked for.

/**
 Reads a text setting, taking the fallback when the variable is unset or empty.

 @param env - environment to read

 @param name - variable naming the setting

 @param fallback - value when the variable says nothing

 @returns The variable's text, or the fallback

 @example
 ```ts
 const seed = textSettingOf({ env: process.env, name: 'DAMAGE_SAMPLE_SEED', fallback: 'damage-round-one', },);
 ```
 */
export function textSettingOf(
  {
    env,
    name,
    fallback,
  }: {
    readonly env: Readonly<NodeJS.ProcessEnv>;
    readonly name: string;
    readonly fallback: string;
  },
): string {
  /**
   The variable as written, empty when unset.
   */
  const written = env[name] ?? '';
  return (written === '') ? fallback : written;
}

//endregion Environment text setting
