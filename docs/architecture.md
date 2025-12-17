# Principles

BSPSuite aims to be a customisable BSP-based map compilation pipeline that can support multiple different games. This is achieved with the following architectural principles:

* **Modular pipeline:** compiler support for a game may be added by creating a config for the game, to set up runtime parameters, and a shared library, to provide the required compiler routines.
* **Programmer support:** as well as being able to launch a BSP compiler executable, the compile process can just as easily be run with a function call, allowing it to integrate into game engines or other applications.
* **Multi-platform:** the compiler libraries and executables should support at least Windows 10 onwards, and at least Ubuntu 22.04 onwards.

# Package Structure

An installed version of the BSPSuite compilers on Windows will look something like:

```
bspsuite\
  bspc.exe
  bspcore.dll
  games\
    my-game
      my-game.cfg
      my-game.dll
  extensions\
    quakeext.dll
    goldsrcext.dll
    sourceext.dll
    imageext.dll
```

The structure on Linux will be identical, only with the file naming conventions modified appropriately.

The important features of this structure are:

* The compiler executable is in the root directory. Rather than having an executable for each stage, the main executable takes command line arguments to specify which stage(s) to run, eg. `bspc.exe rad`.
* `bspcore.dll` is where the main compiler logic lives. `bspc.exe` simply takes in arguments and translates them to function calls.
* `bspcore.dll` looks for supported games in the `games` directory. Here, each supported game has its own subdirectory, and a config file within. The config file can set parameters relevant to the game (eg. the max allowed number of brushes), and can specify attributes such as the compiler library to load to support the game.
* Libraries in the `extensions` directory provide common routines that may be useful to more than one game. This can include, for example, map parsers or file loaders.

# SIMD Support

I didn't know much about the specifics of SIMD instructions, but after [a little research](https://www.techspot.com/article/2166-mmx-sse-avx-explained/), it seems that:

* SSE is an older standard with versions from 1999 to 2008, with 128-bit registers, which pretty much all modern processors should support.
* AVX is a newer standard from 2008, with 256-bit registers, and AVX2 is from 2013.
* AVX-512 uses 512-bit registers, but is only supported by Intel; others like AMD have decided that that much data bandwidth is better placed on the GPU. AVX-512 seems to be more useful for supercomputers.

It would certainly be useful to leverage vector instructions for the compiler, particularly the lighting stage. Questions:

* This would certainly require building different versions of the compiler, depending on the instruction set to be used. Would this be worth it?
  * Probably, if we can separate out the instructions into a library as below.
* Would it be possible to build one shared library which supports AVX, and another that supports SSE, and dynamically load one or the other depending on what the processor supports?
  * Stack Overflow seems to think so: https://stackoverflow.com/a/38317801
  * We should also be able to detect support at runtime (https://gist.github.com/hi2p-perim/7855506), and load the appropriate library.
* Would it be worth making vector classes in the public interface SIMD-capable?
  * Difficult to see how this would be made to work well - these vectors need to be general-purpose, and we don't know the architecture of the user's machine in advance.
  * Might be better to leverage SIMD instructions for the critical algorithms like BSP/VIS/RAD, and use a naive general implementation for the arbitrary maths. Could provide functions to perform these algorithms, which modules could call if they need.
  * This approach seems to be validated by [the design principles described here](https://github.com/Cincinesh/tue): _"SIMD vectors aren't very efficient when used as traditional 3D vectors. ... SIMD instructions should be used to perform the same logic on multiple vectors in parallel."_
