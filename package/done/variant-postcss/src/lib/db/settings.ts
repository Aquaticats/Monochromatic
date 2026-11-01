/**
 Settings key-value store access layer.
 
 The `settings` table holds instance configuration as simple key-value pairs.
 Keys are plain strings; values are stored as TEXT (callers JSON.stringify complex values).
 */
import db from '../db.ts';

/**
 Raw database row shape for the settings table.
 */
type SettingRow = {
  readonly key: string;
  readonly value: string;
};

/**
 Sentinel returned by {@link getSetting} when no row matches the key.
 
 A unique `Symbol` keeps "missing" out of a nullish union (banned by
 `no-nullish-union`); callers narrow with `=== SETTING_ABSENT`.
 */
export const SETTING_ABSENT: unique symbol = Symbol('settings table row absent for lookup key',);

/**
 Retrieves a single setting by key.
 
 @param key - Setting identifier
 
 @returns Stored value, or {@link SETTING_ABSENT} when the key does not exist
 
 @example
 ```ts
 const apiKey = await getSetting('ai_api_key');
 ```
 */
export async function getSetting(key: string,): Promise<string | typeof SETTING_ABSENT> {
  /**
   Single-row result from the lookup; nullish when the key is missing.
   */
  const row: unknown = await (await db.prepare('SELECT value FROM settings WHERE key = ?',))
    .get(key,);
  if ((row === undefined) || (row === null))
    return SETTING_ABSENT;
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- database query returns the SettingRow shape
  return (row as Pick<SettingRow, 'value'>).value;
}

/**
 Upserts a setting: inserts the key if absent, replaces the value if present.
 
 @param key - Setting identifier
 
 @param value - Text payload to store
 
 @example
 ```ts
 await setSetting({ key: 'ai_api_key', value: 'sk-...' });
 ```
 */
export async function setSetting(
  {
    key,
    value,
  }: {
    readonly key: string;
    readonly value: string;
  },
): Promise<void> {
  await (await db
    .prepare(
    'INSERT INTO settings (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value',
    ))
    .run(
    key,
    value,
    );
}

/**
 Deletes a setting by key.
 
 @param key - Setting identifier
 
 @returns `true` when the key existed and was removed
 
 @example
 ```ts
 const removed = await deleteSetting('ai_api_key');
 ```
 */
export async function deleteSetting(key: string,): Promise<boolean> {
  /**
   Run result whose `.changes` distinguishes a successful delete from a no-op.
   */
  const result = await (await db.prepare('DELETE FROM settings WHERE key = ?',))
    .run(key,);
  return result.changes
    > 0;
}

/**
 Returns all settings as a key-value record.
 
 @returns All settings as key-value pairs
 
 @example
 ```ts
 const settings = await getAllSettings();
 ```
 */
export async function getAllSettings(): Promise<Record<string, string>> {
  /* oxlint-disable typescript/no-unsafe-type-assertion -- database query returns SettingRow shape */
  /**
   All settings rows in alphabetical key order, converted to a plain object below.
   */
  const rows = (await (await db
    .prepare('SELECT key, value FROM settings ORDER BY key ASC',))
    .all()) as SettingRow[];
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  return Object.fromEntries(rows.map(function toEntry(row,) {
    return [
      row.key,
      row.value,
    ];
  },),);
}
