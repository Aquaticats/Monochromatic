 -  Made `multiple()` share its state with the earlier states it was extended
    from instead of rebuilding the whole array for each item it accepts,
    so a
    successful parse of many repeated items is no longer spent copying the
    items it already holds.
