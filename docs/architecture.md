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
    	config.toml
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

# Extension FFI

A plugin-based system in Rust needs careful consideration regarding the exchange of data over the boundary between the plugin host (here, the compiler) and the plugin libraries (here, the extensions). This project makes use of [thin_trait_object](https://docs.rs/thin_trait_object/latest/thin_trait_object/), which allows for creation of traits which can be called across shared library boundaries. These traits should be sure to make use of FFI-safe data types, some of which can be found in the `bspsuite-ffi` crate.

Note that as of version `1.1.2` of `thin_trait_object`, there is a compilation issue which prohibits trait functions with more than one argument following `self`. For this reason, the repo has been added manually to this project, so that we can build off the `main` branch.

# Extension APIs and Components

There are several different pieces that need to come together in order to represent a usable API that an extension may make use of. For the purposes of example, we'll consider the map format API, which allows an extension to parse a loaded map file and construct geometry from it.

1. The **API interface** is represented by an FFI-safe [thin trait object](https://docs.rs/thin_trait_object/latest/thin_trait_object/). This interface is defined in the `bspsuite-extinterface` crate, and represents the set of functions that an extension can call to interact with the interface. For the map format API, the trait would be named `MapFormatApi`, and the `thin_trait_object` interface struct would by default be named `MapFormatApiProvider`.
2. The **implementation** of the API interface is the struct that implements the functions on the FFI-safe trait. The implementation struct belongs to the `bspsuite-core` crate, and is implemented in a module under `crate::extensions::api_impl`. The implementation struct manages any state that is required when an extension is interacting with the API interface, but it lives only as long as the API interface is being used by the extension. For the map format API, the implementation struct would be named `MapFormatApiImpl`.
3. The **API endpoint** is the per-extension part of the API that keeps persistent state. It is part of the extension as loaded by the compiler library, and stores anything that is computed while the extension uses the API interface. For the map format API, the endpoint would store the name of each registered map format, along with the file extensions it is associated with, and the extension callback function that should be called to parse a map of this format. The map format API's endpoint struct would be named `MapFormatApiEndpoint`.
4. The **API collector** is a struct that allows querying all endpoints of an API across all loaded extensions. For the map format API, the collector would provide an easy way to find the particular callback that should be called to load a map of a given format. The map format API's collector would be named `MapFormatApiCollector`.
