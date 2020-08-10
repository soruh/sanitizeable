# You probably do not want to use this

You don't!

# Why should you not use this

- I wrote this in an hour at 3AM.
- It uses lots of `unsafe` none of which has Safety comments
- it needs nightly (`#![feature(untagged_unions)]` and `#![feature(drain_filter)]`)
- (it uses `untagged_unions`)
- The resulting structs are always `repr(C)`.

# What does it do?

This abomination allows you to annotate fields of a struct as private,
creates variants of the struct that do / don't have those fields and allows you to take references to each one.
It also allows you to have attributes only one one of the struct, or both of them and to consume the container and turn it into the private variant.

# Why did you create this?

I wanted automatic compile time guarantees that I don't accidentaly expose private data.

# How do I use this

## You shouldn't

If you really want to though, there is an [example](https://github.com/soruh/sanitizeable/blob/master/example/src/main.rs) in the example folder, which you can run with `cargo run`.

# Contributing

If you think this is cool and want to make it useable, feel free to create a PR an Issue and to message me.
