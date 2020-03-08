# `bytie` - convinient byte stream manipulation

[![crates.io](https://img.shields.io/crates/v/bytie)](https://crates.io/crates/bytie)
[![License](https://img.shields.io/badge/License-BSD%203--Clause-blue.svg)](https://opensource.org/licenses/BSD-3-Clause)

## Previews

**TODO** Pictues here

## Instruction

`bytie` allows to add, delete, replace and cut bytes of an input byte
stream or from a file. Surely, one is able to do the same thing with `dd`,
however I find its command line interface a bit cumbersome at times.

## Installation

### Via Cargo

Install `bytie` from crates.io.

```sh
> cargo install bytie
```

### Building from source

Clone the repository

```sh
> git clone https://github.com/awidegreen/bytie.git
> cd bytie
```
Build and install via `cargo`. Note that you need a fairly recent rust version.

```sh
> cargo install --path .
```

## Usage

### Examples

Add a string at a certain position.
```sh
> echo "foobar" | bytie add -v WORLD 3
fooWORLDbar
```

Replace a string at a certain position, where replacement data comes from `STDIN`.
```sh
# create test file
> echo "foobar" > test

> echo -n "FOO" | bytie test replace 0
FOObar
```

Cut/extract bytes from input.
```sh
# note positional argument (range)
> echo "foobar" | bytie cut 1:4
ooba
```

Delete bytes from input.
```sh
# note positional argument (length)
> echo "foobar" | bytie delete 1+3
fr
```
### General `bytie` options

`bytie` has several general command line options which are valid and usable for all
subcommands:
* `-b|--blocksize`: By default `bytie` reads from the input with a blocksize of
  `1024` bytes, this can be changed using this option.
* `-o|--out`: Use this option if the result should be written to a file
  instead of `STDOUT`.
* `-i|--in-place`: Write byte manipulation output to the provided input
  `<file>`. This only works if `<file>` has been specified.
* `<file>` (optional): The input file which will act as a data source for the
  subcommand operation.

All position markers, like the subcommands `begin` or positional parameter (e.g.
for ranges), accept use human-readable byte format like `1Mb` or `1kib`.

**NOTE:** Based on the specification of the `<file>` parameter, `bytie` will
decide where the input data originates from. Meaning, if `<file>` is omitted,
`STDIN` will be used as input stream for the respective subcommand action. In
such cases, subcommands like `replace` and `add` will *not* be able take input
data for the replacement/insertion from `STDIN`.

For more information consult `bytie`s help (`-h|--help`).

### Subcommands

`bytie` provides the following subcommand to fulfill different use cases:

**NOTE:**: `bytie` positional indicators (start, end) start from index `0`, so
the `'b'` in `foobar` is at index `4`.

#### `cut` - Extract data from input
*alias: `extract`*

Can be used to cut/extract certain bytes from an input stream by providing the
start and end position (or a length, see position parameter description below).

Cut in this context means that only the specified range will remain in the
output. In contrast to `delete` which will remove bytes from the input.

#### `delete` - Remove data from input
*alias: `remove`*

Deletes a range of bytes from the provided input stream. In contrast to `cut`
where the specified range will be the output. As for `cut` a positional
parameter need to be specified (see below).

#### `add` - Insert data to input
*alias: `insert`*

Inserts provided data to the input data at the specified start position
(`begin`). The data to be inserted/added can originate from `STDIN` or the
subcommands `--value` parameter (`STDIN` is only possible if source data is
*not* provided via `STDIN`).

#### `replace` - Replace data from input
*alias: `substitute`*

Replaces provided data at the input data at the specified start position
(`begin`). The data to be replaced can originate from `STDIN` or the
subcommands `--value` parameter (`STDIN` is only possible if source data
is *not* provided via `STDIN`).

`bytie` will always write the complete replacement data, meaning that the output
data might be longer than the input.


#### Positional parameter

The `cut` and `delete` subcommands require a `position` as an argument. This has
the following format:

```
<begin>         Begin to the end of input
<begin>:<end>   Begin to end (exclusive), requires <end> > <begin>
                Example: 'foobar', 0:2 == 'fo' or 3:5 == 'ba'
<begin>:=<end>  Begin to end (inclusive), requires <end> > <begin>
                Example: 'foobar', 0:=2 == 'foo' or 3:=5 == 'bar'
<begin>+<count> Begin plus <count> (exclusive), requires <count> > 0.
                The length includes the begin position: 0+10 is 10 bytes, from 0..9 (same as 0:9)
```

## Possible feature extensions

* Implement line instead of byte mode. All subcommand should behave the same
but instead of working and manipulating bytes, lines should be used. This might
be useful if one just want to see a specific line of a file - which otherwise
could be achieved with `sed` if one remembers the syntax.
* allow multiple operations in one executions

## License

Copyright (c) 2020 - Armin Widegreen

`bytie` is licensed under the 3-Clause BSD License ([3-Clause BSD](LICENSE-BSD3) or
https://opensource.org/licenses/BSD-3-Clause)

