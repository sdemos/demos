# DemOS

DemOS is a kernel and (maybe, eventually) userspace written in rust. It is
primarily a toy, and is not intended for actual use.

## Goals

The goal of this OS, besides the obvious one of creating an OS named after
myself, is to learn more about operating systems development and have fun doing
it. 

My strategy for this is to write as much of it as possible in rust, and at all
opportunities, use the expressive type system to my advantage to force invalid
behavior to be a compile time error.

It also should generally adhere to the principles of microkernel architecture.

## Inspirations

This operating system stands on the shoulders of giants. Here are a few of them.

* a [bunch of blog posts](https://os.phil-opp.com) by Philipp Oppermann
* hawkw's [SOS kernel](https://github.com/hawkw/sos-kernel)
* the entire [OSDev wiki](wiki.osdev.org/Main_Page)
* and, of course, projects like [redox](https://github.com/redox-os/redox)

and a bunch of other wonderful resources on the internet I am sure I'm
forgetting.

## Next Steps

I have several ideas for things to do next. Here they are, in rough order, with
no guarantees of recency.

* Investigate higher-half kernels
* Better memory allocator
* Implement processes
* have a userspace at all
* UEFI booting to eliminate/drastically reduce assembly

## Target Files

Unfortunately, the JSON spec doesn't allow comments, and there isn't a way to
define the target spec in rust unless you are the compiler, so I'm going to
reproduce my target files here and describe (to the best of my ability) why each
thing is set the way it is.

First of all, all the target file fields are defined in the [librustc_target
crate][1] in the compiler. There is a [set of required fields][2] and a [set of
optional fields][3]. The required fields have [slightly different names][4] than
their corresponding fields, while all the optional fields just replace the
underscores with hyphens. That crate also has the target definitions for all the
supported platforms, which are useful to peruse for examples, although they are
defined directly in rust instead of using the json syntax.

This project has two target files. One is for the UEFI bootloader, which was
mostly just stolen from the [uefi-rs project][5]. That one is neat, but I didn't
write it. The other is specifically for my kernel.

[1]: https://github.com/rust-lang/rust/blob/7e031b0907e90fc083e4f1f7c6a7f62e98325a9a/src/librustc_target/spec/mod.rs
[2]: https://github.com/rust-lang/rust/blob/7e031b0907e90fc083e4f1f7c6a7f62e98325a9a/src/librustc_target/spec/mod.rs#L388
[3]: https://github.com/rust-lang/rust/blob/7e031b0907e90fc083e4f1f7c6a7f62e98325a9a/src/librustc_target/spec/mod.rs#L429
[4]: https://github.com/rust-lang/rust/blob/7e031b0907e90fc083e4f1f7c6a7f62e98325a9a/src/librustc_target/spec/mod.rs#L783
[5]: https://github.com/GabrielMajeri/uefi-rs/blob/0892f6674cd622ed30970a66c3f56b734cc49c8f/uefi-test-runner/x86_64-uefi.json
