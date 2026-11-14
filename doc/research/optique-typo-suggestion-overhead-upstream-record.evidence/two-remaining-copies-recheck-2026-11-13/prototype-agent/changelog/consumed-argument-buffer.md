 -  Dropped the tokens `argument()` consumed from the input buffer without
    copying the tokens after them,
    so a parse of many positional arguments no
    longer copies the rest of the command line for each token it consumes.
