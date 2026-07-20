BSPSuite
========

BSPSuite aims to be a modern implementation of a [Binary Space Partition](https://en.wikipedia.org/wiki/Binary_space_partitioning) map compiler for games with [Quake](https://en.wikipedia.org/wiki/Quake_(video_game)) heritage (ie. tech derived from the [Quake engine](https://en.wikipedia.org/wiki/Quake_engine)).

This project is primarily for me to implement my own, maintainable version of the ZHLT compiler that I use for [Nightfire Open](https://github.com/noodlecollie/nightfire-open), and also an excuse to learn Rust. I come back to this code from time to time, in between working on Nightfire Open as a primary project.


[License](license.txt)

## Setting Up Distrobox

These commands **must** be run from the root of the repo, so that the paths in the configuration are correct:

```bash
distrobox assemble create -R --file .distrobox/distrobox.ini
distrobox enter bspsuite_dev
```

Once entered, `sshd` will be started. You can then `exit` the shell, and reconnect from the IDE connect using `ssh -p 2222 <your username>@localhost`
