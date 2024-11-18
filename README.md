# Cipherlang
Cipherlang is a scripting language for applying transforms to text.

This is a rewrite of Cipherlang in rust.

## Usage
Cipherlang is called from the command line using the following syntax:\
cipherlang [-d] [-f]methodname -i[inputfile] -o[outputfile]\
-i and -o are optional. If omitted, stdin/stdout are used, respectively.\
If -f is specified, a method is searched for in the current directory. Otherwise,
cipherlang looks in ~/.ciplang/methods\
If -d is specified, the bytecode will be written to ~/.ciplang/methods

## Language Specification
A specification for the language, along with a coding guide, will be released eventually.

## Status
The rewrite is mostly feature-compatible with the original version, but not compliant
with the standard (which is not yet publicly available).

No Cipherlang V2 features have been implemented yet. Those features include for loops & 
switch statements, among others.

Target features:\
* Switch Statements
* For Loops
* implement all designed special variables
* Add a method for operation ordering (Parentheticals? Switch to prefix notation?)
* operations with different numbers of inputs (should work better with prefix notation)
* Enable calling global methods as transforms in script
* Enable using vars and const as arguments for transforms
* Enable using string constants directly in expressions

# Credits
Cipherlang was created by xavenna. This specific implementation is developed by xavenna.

This code is released under the MIT License. See LICENSE for details
