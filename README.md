# Local games plugin for Playtron

This plugin allows sideloading games to your Playtron device that are not 
available on supported stores (development builds, DRM free games, shareware, launchers, ...)

## Installation

If your GameOS version doesn't provide the local plugin already, you can add it manually.

SSH into your device and create the folder for the plugin and the installed games.

```
// You can find the device's IP in the Wi-Fi settings of your device
export DEVICE_IP=192.168.x.x
ssh playtron@$DEVICE_IP
mkdir -p ~/.local/share/playtron/plugins/local
mkdir -p ~/.local/share/playtron/apps/local
```

Build the plugin with `cargo build` then copy the plugin to your device:
`scp target/debug/plugin-local playtron@$DEVICE_IP:~/.local/share/playtron/plugins/local/`

Create a file named `pluginmanifest.json`, this file should contain the following content: 

```
{
  "id": "local",
  "startup_command": "/var/home/playtron/.local/share/playtron/plugins/local/plugin-local"
}
```

and also copy it to the same location:
`scp pluginmanifest.json playtron@$DEVICE_IP:~/.local/share/playtron/plugins/local/`

## Setting up rsync on Windows

Rsync will allow you to copy a game folder to your device and make incremental updates.
To get Rsync on Windows, download the Cygwin installer from their website: https://cygwin.com/
Once in the Cygwin installer, make sure to select rsync and OpenSSH in the package selection.

You can now launch a bash console which will give you access to rsync.

## Loading games

On the machine you want to load the game from, locate your game folder 
and create a file named `gameinfo.yaml`. Populate this file with the following information:

```
name: The game's name
executable: game_executable.exe
```

Example:

```
name: "Street Fighter X Tekken"
executable: "SFTK.exe"
```

In the case of a Linux game, also add `os: linux` and prefix the executable with `./`

Example:

```
name: Minecraft
executable: ./minecraft-launcher
os: linux
```

Copy the game folder to your Playtron device:

```
# Run the command from the parent folder relative to your game
cd Games
# Make sure there is no trailing slash after `my-game` 
rsync -avz my-game playtron@DEVICE_IP:~/.local/share/playtron/apps/local/
```

Restart your device to see your newly added game.

## Updating games

To update a game that has already been loaded on the device, simply run the rsync command again.
It might be necessary to add the `--delete` flag to rsync if some game files have been removed.