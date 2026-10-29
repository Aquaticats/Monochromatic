import * as v from 'valibot';

/**
 Default port number for the RSS server when no environment variable is set.
 
 @see {@link PORT} for the actual port used by the server
 */
const DEFAULT_PORT = 4_112;

/**
 Port number on which the RSS server will listen for incoming requests.
 Can be overridden by setting the RSS_PORT environment variable.
 
 @see {@link DEFAULT_PORT} for the fallback port value
 */
export const PORT: number = v.parse(
  v.pipe(
    v.unknown(),
    v.transform(Number,),
    v.number(),
  ),
  process.env
    .RSS_PORT
    ?? DEFAULT_PORT,
);
