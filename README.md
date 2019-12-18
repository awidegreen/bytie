# `bytie` - easy to use byte stream manipulator

## Previews

Pictues here

## Instruction

`bytie` allows to add, delete, replace and cut bytes of an input byte stream or
from a file. Surely, one is able to do the same thing with `dd`, however I find
its command line interface a bit cumbersome at times.

## Installation

### Via Cargo
cargo install instructions here

```sh
> cargo install bytie
```

## Examples usage

### General `bytie` options

`bytie` has some general command line options which are valid and usable for all
subcommands:
* `-b|--blocksize`: By default `bytie` reads from the input with a blocksize of
  `1024` bytes, this can be changed using this option.
* `-|--output`: Use this option if the result should be written to a file
  instead of STDOUT.
* `-i|--in-place`: `bytie` outputs the byte manipulation result to STDOUT (or
  to a file, see `--output` option). Instead, if a `<file>` is provided,
  `bytie` is able to change the input file in-place using this option.



## License

Copyright (c) 2020 - Armin Widegreen

`bytie` is licensed under the 3-Clause BSD License ([3-Clause BSD](LICENSE-BSD3) or
https://opensource.org/licenses/BSD-3-Clause)

