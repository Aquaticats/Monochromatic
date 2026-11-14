// Parses the separator shape repeatedly under `node --cpu-prof`, so the V8
// sampling profiler records that parse in the arm's own Optique.

import { pathToFileURL, } from 'node:url';

//region Entry point

/**
 * Parses the separator shape `--repeat` times and reports the validation.
 */
async function main(): Promise<void> {
  /** Parsed `--key=value` pairs. */
  const flags = new Map<string, string>(
    process.argv
      .slice(2,)
      .map(function asPair(argument,) {
        /** Position of the key/value separator. */
        const separatorIndex = argument.indexOf('=',);

        return [
          argument.slice(
            2,
            separatorIndex,
          ),
          argument.slice(separatorIndex + 1,),
        ] as [string, string];
      },),
  );
  /** Arm label. */
  const arm = flags.get('arm',);
  /** Repeat count. */
  const repeatText = flags.get('repeat',);

  if (arm === undefined || repeatText === undefined) {
    throw new Error('required: --arm=<version> --repeat=<count>',);
  }

  /** Iterations to parse. */
  const repeat = Number.parseInt(
    repeatText,
    10,
  );
  /** Paths after the separator. */
  const tokenCount = Number.parseInt(
    flags.get('tokenCount',) ?? '1000',
    10,
  );
  /** Arm-local separator module. */
  const module = await import(
    pathToFileURL(`/bench/arms/${arm}/bench/separator-parser.ts`,).href,
  ) as {
    createSeparatorRunner: (tokenCountIn: number,) => () => {
      readonly success: boolean;
      readonly tokens: number;
    };
  };
  /** Runner for this arm. */
  const runner = module.createSeparatorRunner(tokenCount,);

  for (let index = 0; index < repeat; index += 1) {
    runner();
  }

  /** Final validation. */
  const outcome = runner();

  process.stderr.write(
    `[profile-separator] arm=${arm} repeat=${repeat} tokens=${outcome.tokens} ok=${outcome.success}\n`,
  );
}

await main();

//endregion Entry point
