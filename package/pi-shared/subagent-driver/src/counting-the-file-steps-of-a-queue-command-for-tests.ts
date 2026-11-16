/**
 Counts the file-system steps one queue command takes and kills the program at the one a control names, so a control can walk every
 instruction a command takes and ask what the next command finds.

 The counting is a patch of `node:fs/promises`, because a control cannot otherwise stop a program between two of its file-system calls. The
 patch is a table of methods of one object: a method is not a function declaration with positional parameters and not a function expression
 bound to a variable, so it satisfies the rules that shape functions here, and `Object.defineProperty` installs each one without a cast.

 @internal

 @module
 */

import fs, { type PathLike, } from 'node:fs';
import { syncBuiltinESMExports, } from 'node:module';

/**
 What the counting keeps: how many steps have been taken, and the one the program is killed at.
 */
const counting = {
  steps: 0,
  killAt: 0,
};

/**
 The functions this module counts, kept before they are replaced.
 */
const original = {
  writeFile: fs.promises
    .writeFile,
  link: fs.promises
    .link,
  rename: fs.promises
    .rename,
  rm: fs.promises
    .rm,
  appendFile: fs.promises
    .appendFile,
  mkdir: fs.promises
    .mkdir,
};

/**
 Counts one changing step and kills the program when it is the one asked for.
 */
function aStep(): void {
  counting.steps += 1;
  if (counting.steps === counting.killAt) {
    // The program kills itself: the only signal this probe sends, to the only process it signals.
    process.kill(
      process.pid,
      'SIGKILL',
    );
  }
}

/**
 The counted steps, each one a method that takes what `node:fs` passes it and reports nothing.
 */
const counted = {
  /**
   Writes a file.

   @param path - where the file goes

   @param data - what it holds
   */
  async writeFile(
    path: Parameters<typeof fs.promises.writeFile>[0],
    data: Parameters<typeof fs.promises.writeFile>[1],
  ): Promise<void> {
    aStep();
    await original.writeFile(
      path,
      data,
    );
  },

  /**
   Links a file into a name.

   @param existing - the file made already

   @param created - the name it is linked to
   */
  async link(
    existing: PathLike,
    created: PathLike,
  ): Promise<void> {
    aStep();
    await original.link(
      existing,
      created,
    );
  },

  /**
   Renames a file.

   @param from - the name it has

   @param to - the name it takes
   */
  async rename(
    from: PathLike,
    to: PathLike,
  ): Promise<void> {
    aStep();
    await original.rename(
      from,
      to,
    );
  },

  /**
   Removes a file.

   @param path - the file's path
   */
  async rm(path: PathLike,): Promise<void> {
    aStep();
    await original.rm(
      path,
      { force: true, },
    );
  },

  /**
   Appends to a file, which is how the changeover records a name it has written.

   @param path - the file's path

   @param data - what it adds
   */
  async appendFile(
    path: Parameters<typeof fs.promises.appendFile>[0],
    data: Parameters<typeof fs.promises.appendFile>[1],
  ): Promise<void> {
    aStep();
    await original.appendFile(
      path,
      data,
    );
  },

  /**
   Makes a folder, which is how the queue and its claims folder come to be.

   @param path - the folder's path
   */
  async mkdir(path: PathLike,): Promise<string> {
    aStep();
    return (await original.mkdir(
      path,
      { recursive: true, },
    )) ?? '';
  },
};

/**
 Counts the steps one command takes and kills the program at the one named.

 @param killAt - the instruction to kill at, which is past every instruction when it is larger than the command has

 @example
 ```ts
 countTheSteps({ killAt: 3 });
 ```
 */
export function countTheSteps({ killAt, }: { readonly killAt: number },): void {
  counting.killAt = killAt;
  Object.defineProperty(
    fs.promises,
    'writeFile',
    {
      value: counted.writeFile
        .bind(counted,),
      configurable: true,
      writable: true,
    },
  );
  Object.defineProperty(
    fs.promises,
    'link',
    {
      value: counted.link
        .bind(counted,),
      configurable: true,
      writable: true,
    },
  );
  Object.defineProperty(
    fs.promises,
    'rename',
    {
      value: counted.rename
        .bind(counted,),
      configurable: true,
      writable: true,
    },
  );
  Object.defineProperty(
    fs.promises,
    'rm',
    {
      value: counted.rm
        .bind(counted,),
      configurable: true,
      writable: true,
    },
  );
  syncBuiltinESMExports();
}

/**
 Reads how many steps the program has taken, which it prints when it was not killed.

 @returns the count

 @example
 ```ts
 const steps = stepsTaken();
 ```
 */
export function stepsTaken(): number {
  return counting.steps;
}
