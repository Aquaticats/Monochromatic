/**
 What the scan names: the container programs it looks for, the verbs that start one, and the start it reports.

 A start and the argument text it reaches are read in two modules (`container-starts-in-source-text.ts` and
 `argument-text-of-a-container-start.ts`), and both speak of the same programs, verbs and invocation shape.
 */

//region Programs and verbs

/**
 Container programs whose starts the scan finds.
 */
export type ContainerProgram = 'docker' | 'podman';

/**
 Verbs after a program name that start a container or build an image.
 */
export type ContainerStartVerb = 'build' | 'create' | 'run';

/**
 Container programs the scan looks for.
 */
export const CONTAINER_PROGRAMS: readonly ContainerProgram[] = [
  'docker',
  'podman',
];

/**
 Verbs that make a program mention a start.
 */
export const CONTAINER_START_VERBS: readonly ContainerStartVerb[] = [
  'build',
  'create',
  'run',
];

//endregion Programs and verbs

//region The shape a start takes

/**
 One place a file starts a container or builds an image.
 */
export type ContainerStartInvocation = {
  /**
   Program the start runs.
   */
  readonly program: ContainerProgram;
  /**
   Verb the start passes to that program, absent when the array literal spreads a call that builds the arguments.
   */
  readonly verb?: ContainerStartVerb;
  /**
   Line of the program name, counted from one in the text the scan read.
   */
  readonly line: number;
  /**
   Argument text that reaches the program: the elements after a leading program name, or the array literal or command
   line that follows the name.
   */
  readonly argvText: string;
};

//endregion The shape a start takes
