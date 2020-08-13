# Don't use this in production!

- It uses lots of `unsafe` which is documented but has only been reviewed by me
- it needs nightly (`#![feature(untagged_unions)]` and `#![feature(proc_macro_diagnostic)]`)
- The resulting structs are always `repr(C)`.

# What does it do?

It allows you to annotate fields of a struct as private and
create variants of the struct that do / don't have those fields.
You can then take references to each one.

It also allows you to have attributes on only one one of the structs or both of them and to consume the container to turn it into the private variant.

You may not use `cfg` on only one of the variants since that would break internal layout guarantees.

TL/DR:
There are [examples / tests](https://github.com/soruh/sanitizeable/blob/master/example/).

# Why did you create this?

I wanted automatic compile time guarantees that I don't accidentaly expose private data.

# Contributing

If you think this is cool and want to make it useable, feel free to create a PR an Issue or to message me.
