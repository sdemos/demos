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
