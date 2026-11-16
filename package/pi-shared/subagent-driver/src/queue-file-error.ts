/**
 The error a queue raises when a folder, a file or a name of it cannot be read or written as a queue.

 It lives in a module of its own so that the names a queue writes and the folder a queue is can both raise it without one importing the
 other: a name is escaped before a folder is read, and the folder's reading needs the names.

 @module
 */

/**
 A queue folder, or one entry in it, that cannot be read as a queue.
 */
export class QueueFileError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name: string = 'QueueFileError';
}
