# Cipherlang Specification

# Introduction:

Cipherlang is a scripting language for applying transforms to text.

NOTE: This document is for Cipherlang v1, not the upcoming Cipherlang v2.

This document has not been finalized yet, as of 2024-11-18

No cipherlang implementation complies to this standard.

# Definitions & Conventions:

A cipherlang program is called a script. A compiled cipherlang script is called a method.
The dictionary is the global directory containing methods

In this document, if a word is written in CAPS, it represents an argument.

# Interpreter Details:
## I/O Behavior
The cipherlang interpreter needs to be able to take input from a file or from stdin.
It needs to be able to write output to a file or to stdout.

## Scripts:
The interpreter should be able to execute a script stored in the current directory.
It should be able to run a script stored in the global cipherlang directory:
    ~/.ciplang/methods/

The interpreter should be able to supply command-line arguments to a script

The interpreter should be able to write a compiled script into the global script
directory.

The specific flags, arguments, and so on are not defined by this specification. They are
up to the implementation.


# Language Details
## Statement types:


* Load
* Write
* Apply
* Var
* Const

### Load
A load statement loads a location with a specified source.\
Syntax: load VALUE from SOURCE.\

* VALUE must be a variable or writeable special variable
* SOURCE may be a variable, constant, readable special variable, or operation cluster.

### Write
A write statement writes a value to a specified location.\
Syntax: write SOURCE to VALUE.

* VALUE must be a variable or writeable special variable
* SOURCE may be a variable, constant, readable special variable, or operation cluster.

Note: The write statement is functionally identical to the load statement, and as such,
will likely be removed in cipherlang v2

### Apply
An apply statement applies a transform to the contents of a variable, then writes the
results back to that variable.\
Syntax: apply TRANSFORMNAME\<ARGUMENTS\> to VAR

* VAR must be a variable
* TRANSFORMNAME must be a valid transform. Can reference built-in transforms or external modules.
* ARGUMENTS must be a comma-delimited list of string arguments

### Var
A var statement declares a variable.\
Syntax: var VARNAME

* VARNAME must be a string. It cannot begin with an underscore or be in use

### Const
A const statement defines a constant.\
Syntax: const CONSTNAME CONSTVALUE

* CONSTNAME must be a string. It cannot begin with an underscore or be in use.
* CONSTVALUE must be a valid string constant.


## Operations:
An operation takes two values and produces a third. This value can be used in the same
ways as a constant can.

An operation's name is prefixed with $
Currently, operations use infix notation. Nested operations are executed right-to-left.
The execution order is constant. To apply transforms in another order, the sequence
must be split into multiple statements. This will be changed in Cipherlang v2.


Operations:
### $cat
concatenates the two arguments

### $eq
returns an empty string if the arguments are equal, returns a non-empty string otherwise.


### $repeat
Arg2 must be a nonnegative integer. Returns (Arg2) instances of Arg1, concatenated

## Transforms:
A transform is applied to a variable. It manipulates the data in some way, then
writes the result back to the variable in question.

Cipherlang contains several built-in transforms, which are listed here. Additionally,
methods found in the global dictionary may be called as transforms.

Built-in transforms are split into two categories: ciphers and utilities.
Utilities are described first.

In future versions, some ciphers will be external modules, rather than built-in functions


Built-in Transforms:
### upper: void
Transforms any lowercase ascii character into the corresponding uppercase character.
Fails for non-ascii input.

### lower: void
Transforms any uppercase ascii character into the corresponding lowercase character.
Fails for non-ascii input.

### trim\_special: void
Removes any special characters from the input

### trim\_numeric: void
Removes any numeric characters from the input

### trim\_whitespace: void
Removes any whitespace characters from the input

### trim\_alpha: void
Removes any (ascii) letters from the input

### prune: void
Removes any non-alphabetic characters from the input

### prune\_numeric: void
Removes any non-numeric characters from the input

### prune\_ascii: void
Removes any non-ascii characters from the input

### shift: int DISTANCE
Performs a caesarian shift on the input. Shifts input by DISTANCE mod 26.

### rc: uint NUM
Performs a rail cipher on the input, with NUM rails

### rc\_dec: uint NUM
Reverses a rail cipher on input, with NUM rails.



Other built-in transforms may be included, but their behavior is not standard.

## Special Variables
Cipherlang has several special variables: (A # denotes a number)
| Short Name | Verbose Name |Description | Status |
|------|-------------|--------|-------|
| _    | Last Transform | Contains the result of the previous transform | read-only |
| \_o  | Last Operation | Contains the result of the previous operation | read-only |
| \_stdout | Standard Output | Any data written here is sent to the script's output | write-only |
| \_stdin | Standard Input | Each read from this variable returns a line of stdin. |
| \_c  | Character | Contains a 1-character-long string with arbitrary contents | read-only |
| \_argc | Argument Count | Contains the number of arguments passed in |
| \_null | Null String | Can be written to a var to clear it | read-only |
| \_#  | #-th Argument | Gets argument number #. Can be 0x0 to 0x1f. If requested argument doesn't have a value, returns empty string | read-only |
