/**
 Names of the units this package installs and the state folder the alert entry uses.

 Kept in one module so the alert text, the argument defaults and the install task
 cannot disagree about a unit name.

 @module
 */

/**
 Unit that runs the bisync passes; its runs are what a sync alert describes.
 */
export const SYNC_UNIT = 'rclone-bisync-garage.service';

/**
 Unit whose only step starts a sync run after the run that may be going now ends.
 */
export const TRIGGER_UNIT = 'rclone-bisync-garage-after-current.service';

/**
 Unit that runs directory-watcher and turns a file change into a trigger run.
 */
export const WATCH_UNIT = 'rclone-bisync-garage-watch.service';

/**
 Folder under the state home that holds one alert-streak file per unit.
 */
export const ALERT_STATE_DIRECTORY_NAME = 'rclone-bisync-garage-alert';
