//region Command line types
// What a runner declares it reads from its command line, and what the one
// command-line reader (`command-line.ts`, ledger B75) hands its body back.
// Kept apart from the reader so the reader stays within the line budget, and
// so a module that only names these types imports nothing that runs.

/**
 What a flag carried, or that nobody wrote it.

 NAMED RATHER THAN LEFT NULLISH because the two answers lead to opposite
 behaviour one line later: an unwritten flag takes a default, and a written
 one is read and may be refused. Each answer carries the flag as the person
 types it, so a reader refusing the value names the flag without being told
 it a second time.

 @example
 ```ts
 const asked: FlagValue = { kind: 'written', flag: '--cap', value: '4', };
 ```
 */
export type FlagValue = {
  readonly kind: 'written';

  /**
   Flag as typed, dashes included.
   */
  readonly flag: string;

  /**
   What was written after it, never empty: an empty value is refused.
   */
  readonly value: string;
} | {
  readonly kind: 'unwritten';

  /**
   Flag as typed, dashes included.
   */
  readonly flag: string;
};

/**
 What a command reads after its flags, by position.

 @example
 ```ts
 const logs: PositionalSpec = { names: ['log file',], least: 1, rest: true, };
 ```
 */
export type PositionalSpec = {
  /**
   What each position holds, in order, as the usage line names it.
   */
  readonly names: readonly string[];

  /**
   How many positions must be written.
   */
  readonly least: number;

  /**
   Whether the last position may be written any number of times.
   */
  readonly rest: boolean;
};

/**
 Everything a command reads from its command line.

 @example
 ```ts
 const spec: CommandLineSpec = {
   valued: { only: 'entry ids', },
   repeatable: {},
   switches: { plan: true, },
   positionals: { names: [], least: 0, rest: false, },
 };
 ```
 */
export type CommandLineSpec<
  Valued extends string = string,
  Listed extends string = string,
  Switch extends string = string,
  > = {
  /**
   Flags written once with a value, each mapped to what the value is, as the
   usage line names it.
   */
    readonly valued: Readonly<Record<Valued, string>>;

  /**
   Flags that may be written any number of times, each with a value.
   */
    readonly repeatable: Readonly<Record<Listed, string>>;

  /**
   Flags written once with no value, each mapped to `true`, so a table
   naming the switches a command reads must name every one of them.
   */
    readonly switches: Readonly<Record<Switch, true>>;

  /**
   What the command reads after its flags.
   */
    readonly positionals: PositionalSpec;
  };

/**
 A command line read against what its command declares, offering only the
 flags declared: asking for any other is a type error rather than a flag that
 always reads as unwritten.

 @example
 ```ts
 const asked = line.flag('only',);
 ```
 */
export type CommandLineFor<Spec extends CommandLineSpec> = {
  /**
   Path of the script the runtime ran, which names the runner's built file.
   */
  readonly script: string;

  /**
   What a flag written once with a value carried.
   */
  readonly flag: (name: keyof Spec['valued'] & string) => FlagValue;

  /**
   Every value a repeatable flag carried, in the order written.
   */
  readonly list: (name: keyof Spec['repeatable'] & string) => readonly string[];

  /**
   Whether a switch was written.
   */
  readonly switched: (name: keyof Spec['switches'] & string) => boolean;

  /**
   What was written after the flags, by position.
   */
  readonly positionals: readonly string[];
};

/**
 The part of a command line a reader of one valued flag needs, so a reader
 shared by several commands accepts the line of any command declaring it.

 @example
 ```ts
 function readCap({ line, }: { readonly line: ReadsFlag<'cap'>; },): FlagValue { return line.flag('cap',); }
 ```
 */
export type ReadsFlag<Name extends string> = {
  readonly flag: (name: Name) => FlagValue;
};

/**
 The part of a command line a reader of one switch needs.

 @example
 ```ts
 function readPlan({ line, }: { readonly line: ReadsSwitch<'plan'>; },): boolean { return line.switched('plan',); }
 ```
 */
export type ReadsSwitch<Name extends string> = {
  readonly switched: (name: Name) => boolean;
};

//endregion Command line types
