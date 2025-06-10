# Local Games Plugin for Playtron GameOS

Table of Contents:
- [Introduction](#introduction)
- [Build](#build)
- [Installation](#installation)
- [Setting up rsync on Windows](#setting-up-rsync-on-windows)
- [Loading Games](#loading-games)
- [Updating Games](#updating-games)

## Introduction

This plugin allows sideloading games to your [Playtron GameOS](https://github.com/playtron-os/gameos) device that are not 
available on supported stores (development builds, DRM-free games, shareware, launchers, etc.)

As of Playtron GameOS Beta 1, the local plugin is installed by default. Older versions of the operating system do not fully support plugins.

## Build

Building is optional and recommended for plugin developers only.

A basic build can be done with `cargo build`. Copy the plugin to your device:
`scp target/debug/playtron-plugin-local playtron@$DEVICE_IP:~/.local/share/playtron/plugins/local/`

Fully automated binary tarball and RPM builds are supported with the use of a container. Both x86_64 and aarch64 CPU architectures are supported. These commands will output packages to the `dist` directory.

```shell
make in-docker TARGET='dist'
```
```shell
make in-docker TARGET='dist' ARCH="aarch64"
```

## Installation

SSH into your device and create the folder for the installed games.

```shell
# You can find the device's IP in the Wi-Fi settings of your device
export DEVICE_IP=192.168.x.x
ssh playtron@$DEVICE_IP
```
```shell
mkdir -p ~/.local/share/playtron/apps/local
```


## Setting up rsync on Windows

Rsync will allow you to copy a game folder to your device and make incremental updates.
To get Rsync on Windows, download the Cygwin installer from their website: https://cygwin.com/
Once in the Cygwin installer, make sure to select `rsync` and `OpenSSH` in the package selection.

You can now launch a bash console which will give you access to rsync.

## Loading Games

On the machine you want to load the game from, locate your game folder 
and create a file named `gameinfo.yaml`. Populate this file with the following information:

```yaml
name: The game's name
executable: game_executable.exe
```

Example:

```yaml
name: "Street Fighter X Tekken"
executable: "SFTK.exe"
```

In the case of a Linux game, also add `os: linux` and prefix the executable with `./`

Example:
```yaml
name: Minecraft (Official)
executable: ./minecraft-launcher
os: linux
```

Flatpaks are also supported by using a simple launch script. Use `chmod +x flatpak-launcher.sh` to make the script executable.

Example:
```shell
#!/bin/bash

/usr/bin/flatpak run org.prismlauncher.PrismLauncher
```
```yaml
name: Minecraft (Prism Launcher)
executable: ./flatpak-launcher.sh
os: linux
```

Optionally, add an image to your game for the Playtron GameOS library. The image should be as close as possible to a 16:9 ratio and be in the highest quality possible (1080p recommended, maximum 4k)

```yaml
image: https://url/of/the/game/artwork.jpg
```

Copy the game folder to your Playtron GameOS device:
```shell
# Run the command from the parent folder relative to your game
cd Games
# Make sure there is no trailing slash after `my-game` 
rsync -avz my-game playtron@DEVICE_IP:~/.local/share/playtron/apps/local/
```

Restart the `playserve` service or reboot to see your newly added game.

```shell
systemctl --user restart playserve
```

## Updating Games

To update a game that has already been loaded on the device, simply run the rsync command again.
It might be necessary to add the `--delete` flag to rsync if some game files have been removed.
