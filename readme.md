BSPSuite
========

BSPSuite aims to be a modern implementation of a [Binary Space Partition](https://en.wikipedia.org/wiki/Binary_space_partitioning) map compiler for games with [Quake](https://en.wikipedia.org/wiki/Quake_(video_game)) heritage (ie. tech derived from the [Quake engine](https://en.wikipedia.org/wiki/Quake_engine)).

This project is primarily for me to implement my own, maintainable version of the ZHLT compiler that I use for [Nightfire Open](https://github.com/noodlecollie/nightfire-open), and also an excuse to learn Rust. I come back to this code from time to time, in between working on Nightfire Open as a primary project.

[License](license.txt)

## Setting Up Distrobox

Because I do a lot of editing on Bazzire and Bluefin systems which come packaged with [Distrobox](https://distrobox.it/), there is a `distrobox.ini` file in the repo which will set up a container with the relevant dependencies for development.

From the root of the repo, run:

```bash
distrobox assemble create -R && distrobox enter bspsuite_dev
```

Once entered, `sshd` will be started. You can then `exit` the shell, and reconnect from the IDE connect using `ssh -p 2222 <your username>@localhost`.

Visual Studio Code supports Devcontainers using an extension, but unfortunately Zed does not at time of writing, so should use the `ssh` method above. You can add a convenience shortcut into your settings file using:

```
"ssh_connections": [
	{
		"host": "localhost",
		"username": "<your username here>",
		"port": 2222,
		"args": [],
		"projects": [
			{
				"paths": [
					"/workspace"
				]
			}
		]
	}
],
```

You can connect to this folder from the remote projects panel, shown using `Ctrl+Alt+Shift+O`.
