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

A plugin-based system in Rust needs careful consideration regarding the exchange of data over the boundary between the plugin host (here, the compiler) and the plugin libraries (here, the extensions). After some experimentation, the general approach for extension calls across a library boundary is as follows.

## FfiTable

The `FfiTable` is basically a struct containing a context pointer, and one or more `unsafe extern "C"` function pointers.

```
// In pseudo-Rust:
FfiTable:
	// Context passed to each function
  context_ptr,

  // Function 1, taking no other arguments, and returning nothing
  extern "C" fn fptr_func1(context_ptr),

  // Function 2, taking an additional i32 argument, and returning an i32
  extern "C" fn fptr_func2(context_ptr, i32) -> i32,
```

The `FfiTable` for an API represents the set of functions implemented by `bspcore.dll` to support the API.

Each of the function pointers stored in the `FfiTable` takes a context pointer as its first argument, similarly to `self` in normal Rust. This essentially allows the implementation of the function to act like a member function of a struct, where the struct data is stored opaquely in the context pointer.

The `FfiTable` for an API is defined within an `internal` module for that API, to make it obvious that extensions should not use it directly.

## ffi_impl

Within `bspcore.dll`, the functions referenced in the `FfiTable` are implemented in a relevant submodule for the API, called `ffi_impl`. This keeps the boilerplate functions separate from the rest of the code.

## Client Struct

The extension code itself interacts with a given API using a "client struct" which presents the API. Internally, the client struct keeps a reference to the `FfiTable` for the API, and manages converting arguments and return values to and from FFI-safe types.

The client struct is named after the API. It does not use a prefix on its type name to identify it, since as far as the extension is concerned, this struct is the main way with which to interact with the API.

## Host Struct

When `bspcore.dll` calls into an extension, it provides the extension with a reference to the relevant client struct for the API. When creating the `FfiTable` for the client struct to use, the context pointer is set to point to a "host struct". When an `ffi_impl` function is called from the `FfiTable`, the context pointer is translated back into a host struct pointer, and the relevant function on the host struct is called.

The host struct is named after the API that it implements, except with a suffix of `Impl`. This easily distinguishes its type name from the type name of the client-facing struct type.

## Call Sequence

This is an example of calling a `dummy_action()` functon on an extension, and providing a `DummyApi` client struct for the extension to call into.

1. Host constructs `DummyApiHost` struct.
2. Host constructs `internal::FfiTable`, binding function pointers and setting context pointer to `DummyApiHost`.
3. Host constructs `DummyApi` struct, providing a reference to `FfiTable`.
4. Host calls `dummy_action()` on extension, passing a reference to `DummyApi` struct.
5. Within `dummy_action()`, extension call `store_number(3)` on `DummyApi` struct.
6. `DummyApi` struct calls `self.ffi_table.fptr_store_number(self.ffi_table.context_ptr, num_to_store)`.
7. The `FFIImpl` function `store_number()` calls `(*context_ptr as DummyApiHost).store_number(num_to_store)`.
8. After execution of `dummy_action()` has finished, the number `3` is stored in the `DummyApiHost` struct.
