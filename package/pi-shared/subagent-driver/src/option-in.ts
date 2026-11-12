/**
 Reading one `--name=value` option out of a command line that a caller passes on to another command, so the commands that share
 those options read them the same way.

 @module
 */

/**
 Reads one option's value from a command line.

 @param commandLine - the arguments, as the process was given them

 @param option - the option, with its equals sign

 @returns the value, or empty text when the line does not carry that option

 @example
 ```ts
 const name = optionIn({ commandLine: ['--name=a',], option: '--name=', },);
 ```
 */
export function optionIn({
  commandLine,
  option,
}: {
  readonly commandLine: readonly string[];
  readonly option: string;
}): string {
  return commandLine.find(function startsWithOption(argument,) {
    return argument.startsWith(option,);
  },)
    ?.slice(option.length,)
    ?? '';
}
