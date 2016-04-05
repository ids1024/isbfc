isbfc
======

Description
-----------
Isbfc is an optimized brainfuck compiler targeting x86_64 Linux. It is still a work in progress. It should not be used in a production environment, but then if you are using any kind of brainfuck in a production environment...

Building
--------
To build isbfc, you need rustc and cargo. Then, just run `cargo build`, or `cargo build --release` for more optimized binaries. The binary will then be in `target/debug/isbfc` or `target/release/isbfc`, respectively.

Licencing
---------
Isbfc is released under the MIT license.
