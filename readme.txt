If you want to use this code, you'll need to compile it, first.

First, install the Rust programming language - https://www.rust-lang.org
(You'll need the STD library and Cargo)

Next, CD into the directory that contains Cargo.toml and run
"cargo build --release". If you run just "cargo build" then it'll be slow
when running. Note that Cargo will handle downloading any required
dependencies (there are only two - tritium and byteorder).

When it's run (and hasn't errored), go into the new target folder, go to
release, and you'll see a static library (.a, .lib, etc.). If you do not
want to compile a static library, edit Cargo.toml and change crate-type to
"dylib" instead of "staticlib" and recompile.