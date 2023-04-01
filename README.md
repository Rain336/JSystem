# Nintendo JSystem

A set of crates for parsing Nintendo Wii JSystem files.
I'm kinda in the midst of updating some of the libraries.

## Library Overview

`lib/bcsv`
A crate for reading and writing Nintendo BCSV / JMap files.
BCSV is a binary CSV table with typed columns and hashed column names.
A `Table` type as well as two hash functions are supplied by the crate.

`lib/rarc`
A crate for reading Nintendo Revolution Archive (RARC) files.
The library isn't finished yet, is is currently being rewritten from the old `jsystem` library.

`jsystem`
A crate for reading Nintendo Revolution Archive (RARC) and BCSV files.
This library is currently being rewritten and split up into `lib/bcsv` and `lib/rarc`.

`wii`
A crate for reading a lot of different wii formats:
- U8 Archives
- IMD5
- IMET
- Disc Header, Partition Table & Ticket
- Binary Revolution Layout Files (BRLYT)
In the future I'll most likely split them all into their own crates under `lib/`

## Tools

`bin/bcsv-cli`
A conversion tool between BCSV and regular CSV files.
It is built against the new `lib/bcsv` library.
It also allows trying to crack the name of a column based on a list of possible names.

`rarc`
Allows editing Nintendo Revolution Archives (RARC) with a tar-like interface.
It is still based on the old `jsystem` library and will be updated when `lib/rarc` is usable.

`u8`
Allows editing Nintendo U8 Archives with a tar-like interface.
It is still based on the `wii` library and might be updated when the library is split into smaller crates.