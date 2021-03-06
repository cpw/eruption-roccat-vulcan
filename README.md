# Table of Contents

- <a href="#eruption">Eruption</a>
- <a href="#important">Important</a>
- <a href="#overview">Overview</a>
- <a href="#features">Features</a>
- <a href="#installation">Installation</a>
- <a href="#config">Configuration and Usage</a>
- <a href="#profiles">Profiles</a>
- <a href="#scripts">Lua Scripts and Manifests</a>
- <a href="#gui">Browser based GUI</a>
- <a href="#audio">Support for Audio Playback and Capture </a>
- <a href="#macros">Support for Macros </a>
- <a href="#plugins">Available Plugins</a>
- <a href="#effects">Available Effects</a>
- <a href="#info">Info</a>
- <a href="#contributing">Contributing</a>

# Eruption <a name="eruption"></a>

A Linux user-mode driver for the ROCCAT Vulcan 100/12x series keyboards


# Important <a name="important"></a>

The project is still in an early stage of development, and thus may contain
some bugs.

If you ever need to forcefully disable the eruption daemon you may do so by adding
the following text snippet to the bootloader's (e.g. GRUB) kernel command line:

```sh
  systemd.mask=eruption.service
```

## Overview <a name="overview"></a>

Eruption is a Linux daemon written in Rust, consisting of a core, an integrated
Lua interpreter and additional plugin components. Its intended usage is to
execute Lua scripts that may react to certain events on the system like e.g.
"Key pressed" and subsequently control the AIMO LEDs on the keyboard. Plugins
may export additional functionality to the Lua scripting engine.
Multiple Lua scripts may be run in parallel. Each Lua scripts "submitted color
map" will be combined with all other scripts "submitted color maps" using an
alpha blending function, prior to sending it to the keyboard.


# Features <a name="features"></a>

* Integrated Lua interpreter
* AIMO LED Control via Lua scripts
* Multiple Lua scripts may be executed in parallel, with their outputs combined
* Allows for construction of complex "effect pipelines"
* Event-based architecture
* Daemon plugins may export functions to Lua
* May be run as a Linux user process or as a system daemon
* Profiles may be switched at runtime via a D-Bus method


# Installation <a name="installation"></a>

### Arch Linux and derivatives like Manjaro

```sh
$ yay -Sy aur/eruption-roccat-vulcan-git
```

### Fedora based

```sh
$ sudo dnf copr enable x3n0m0rph59/eruption-roccat-vulcan
$ sudo dnf install eruption-roccat-vulcan-git
```

To activate eruption now, you may either reboot your system or manually start
the daemon with the command:

```sh
$ sudo systemctl start eruption.service
```

Note: You don't have to enable the eruption service, since it is started by an
`udev rule` as soon as a compatible keyboard device is plugged into your system.

*Support for more distributions is coming soon!*

### From Source

```sh
$ git clone https://gitlab.com/X3n0m0rph59/eruption-roccat-vulcan.git
$ cd eruption-roccat-vulcan
$ cargo build --all --release
```


# Configuration and Usage <a name="config"></a>

## Eruption configuration file

> You may want to try the
	[Eruption Profile Switcher](https://extensions.gnome.org/extension/2621/eruption-profile-switcher/)
	GNOME Shell extension, for easy switching of profiles on the fly.

The eruption configuration file `/etc/eruption/eruption.conf`:

```toml
# Eruption - Linux user-mode driver for the ROCCAT Vulcan 100/12x series keyboards
# Main configuration file

[global]
profile_dir = "/var/lib/eruption/profiles/"
profile = "default"

script_dir = "/usr/share/eruption/scripts/"
script_files = ["batique.lua"]

# select your keyboard variant
# keyboard_variant = "ANSI"
keyboard_variant = "ISO"

[frontend]
enabled = true
# theme = "eruption"
```

### Section [global]

*profile* = The profile to load, when the daemon is started, and no previous saved state is found

*keyboard_variant* = Switch between sub-variants of your device. (Only partially supported)

*script_files* = A list of Lua script files to execute, *when no profile is specified*

### Section [frontend]

Please note that the "frontend" (a browser-based GUI) is not currently shipped
with the pre-built packages, since it is considered not ready yet.

## Profiles <a name="profiles"></a>

The file `default.profile` from the directory `/var/lib/eruption/profiles`

```toml
id = '5dc62fa6-e965-45cb-a0da-e87d29713095'
name = 'Default'
description = 'The default profile'
active_scripts = [
	'batique.lua',
	'shockwave.lua',
	'impact.lua',
]
```

The file `preset-red-yellow.profile` from the directory `/var/lib/eruption/profiles`

```toml
id = '5dc62fa6-e965-45cb-a0da-e87d29713099'
name = 'Preset: Red and Yellow'
description = '''Presets for a 'red and yellow' color scheme'''
active_scripts = [
	'batique.lua',
	'shockwave.lua'
]

# ....

[[config.Batique]]
type = 'float'
name = 'color_divisor'
value = 8.0
```

This will run the `batique.lua` script to "paint the background", and on top of
that, display the shockwave effect from `shockwave.lua` when a key has been
pressed. Configuration values may be overriden on a per-profile basis. If a
configuration value is not listed in the `.profile` file, the default value
will be taken from the script's `.manifest` file.

### Switching profiles at runtime

> You may want to install the GNOME Shell extension
	[Eruption Profile Switcher](https://extensions.gnome.org/extension/2621/eruption-profile-switcher/)
	or visit the [Github page](https://github.com/X3n0m0rph59/eruption-profile-switcher)

You may switch to the `fx1.profile` with the following command:

```sh
$ dbus-send --print-reply --system --dest=org.eruption /org/eruption/profile org.eruption.Profile.SwitchProfile string:"fx1.profile"
```

## Lua Scripts and Manifests <a name="scripts"></a>

All script files and their corresponding manifests reside in the directory
`/usr/share/eruption/scripts`. You may use the provided scripts as a starting
point to write your own effects.

## Browser-based GUI <a name="gui"></a>

If you built eruption from source, and did enable support for the browser-based
GUI, you may reach it with the link below. This will open the eruption GUI in
your browser: [http://localhost:8059/](http://localhost:8059/)

> Please note that the browser-based GUI is currently considered *not ready*!


## Support for Audio Playback and Capture <a name="audio"></a>

If you want eruption to be able to play back sound effects, or use one of the
audio visualizer Lua scripts, then you have to perform a few additional steps.
The following steps will allow the eruption daemon to access the PulseAudio
server of the current user, for playback and for capturing of audio signals.

Create the PulseAudio config directory and edit the server configuration file
for your user account:

```sh
$ mkdir -p ~/.config/pulse/
$ cp /etc/pulse/default.pa ~/.config/pulse/default.pa
$ nano ~/.config/pulse/default.pa
```

then add the following line at the end of the file:

```
load-module module-native-protocol-unix auth-group=root socket=/tmp/pulse-server
```

Create the PulseAudio configuration directory and edit the client configuration
file in `/root/.config/pulse/client.conf` for the user that eruption runs as
(default: root)

```sh
$ sudo mkdir -p /root/.config/pulse/
$ EDITOR=nano sudoedit /root/.config/pulse/client.conf
```

and then add the following lines:

```
autospawn = no
default-server = unix:/tmp/pulse-server
enable-memfd = yes
```

Finally, restart PulseAudio and eruption for the changes to take effect:

```sh
$ systemctl --user restart pulseaudio.service
$ sudo systemctl restart eruption.service
```


## Support for Macros <a name="macros"></a>

Eruption 0.1.1 added the infrastructure to support injection of keystrokes
("macros"), and via this infrastructure is able to utilize the media keys
(F9-F12) without native kernel support being in place.
This is achieved by adding a "virtual keyboard" to the system that injects
keystroke sequences as needed. The "real hardware" keyboard will be grabbed
exclusively on startup of the daemon, so keystrokes won't be reported twice.

> Please note that supporting the media keys this way is a crude hack until
  native kernel support is in place! The `FN` Key can not be supported this way,
  so please be aware that you have to press the `Right Control` key (RCTRL) as
  the modifier instead of the `FN` key!


# Available Plugins <a name="plugins"></a>

* Keyboard: Process keyboard events, like e.g. "Key pressed"
* System: Basic system information and status, like e.g. running processes
* Sensors: Query system sensor values, like e.g. CPU package temperature
* Audio: Audio related tasks, like playing sounds
* Introspection: Provides internal status information of the Eruption daemon
* Profiles: Switch profiles based on system state


# Available Effects <a name="effects"></a>

Eruption currently ships with the following Lua scripts:

| Name      | Class      | File             | Status | Description                                                                                                   |
| --------- | ---------- |----------------- | ------ | ------------------------------------------------------------------------------------------------------------- |
| Afterglow | Effect     |`afterglow.lua`  | Ready  | Hit keys are lit for a certain amount of time, then they are faded out                                         |
| Afterhue  | Effect     |`afterhue.lua`   | Ready  | Hit keys cycle through the HSL color-space, using a linearly decreasing hue angle                              |
| Batique   | Background | `batique.lua`    | Ready  | Batique effect, based on the Open Simplex Noise function that serves as input to get a HSL color              |
| Billow    | Background |  `billow.lua`     | Ready  | Effect based on the Billow noise function that serves as input to produce a HSL color                        |
| Fractal Brownian Motion | Background |  `fbm.lua` | Ready | Effect based on the Fractal Brownian Motion noise function that serves as input to produce a HSL color |
| Perlin Noise | Background |  `perlin.lua` | Ready | Effect based on the Perlin Noise function that serves as input to produce a HSL color                          |
| Ridged Multifractal Noise | Background |  `rmf.lua` | Ready | Effect based on the Ridged Multifractal noise function that serves as input to produce a HSL color   |
| Voronoi | Background |  `voronoi.lua` | Ready | Effect based on the Voronoi noise function that serves as input to produce a HSL color                             |
| Heartbeat | Effect     |`heartbeat.lua`  | Ready  | Heartbeat effect. The more the system is loaded the faster the heartbeat effect                                |
| Impact    | Effect     |`impact.lua`     | Ready  | Hit keys and keys in their immediate vicinity stay lit for a certain amount of time, then they are faded out   |
| Raindrops | Effect     |`raindrops.lua`  | Ready  | Rain effect, randomly light up keys and fade them out again                                                    |
| Solid     | Background |  `solid.lua`      | Ready  | Display a solid color                                                                                        |
| Stripes   | Background |  `stripes.lua`    | Ready  | Display horizontal stripes of multiple colors                                                                |

The following scripts are unfinished/still in development, and some of them have known bugs:

| Name        | Class      | File              | Progress         | Description                                                                                         |
| ----------- | ---------- | ----------------- | ---------------- | --------------------------------------------------------------------------------------------------- |
| Fire        | Background | `fire.lua`        | Approx. 65% done | Shows a bonfire effect on the keyboard                                                              |
| Fireworks   | Background |`fireworks.lua`   | Approx. 45% done | Shows a fireworks effect on the keyboard                                                             |
| Water       | Effect     |`water.lua`       | Approx. 95% done | Shows a water effect on the keyboard                                                                 |
| Gaming      | Effect     |`gaming.lua`      | Approx. 85% done | Highlight a fixed set of keys, like e.g. 'WASD'                                                      |
| Gradient    | Background | `gradient.lua`    | Approx. 95% done | Display a color gradient                                                                            |
| Multi Gradient | Background | `multigradient.lua` | Approx. 65% done | Display a color gradient, supports multiple gradient stops                                     |
| Rainbow     | Background | `rainbow.lua`     | Approx. 65% done | Display a rainbow color gradient                                                                    |
| Shockwave   | Effect     |`shockwave.lua`   | Approx. 75% done | Like Impact, but shows propagating waves when a key has been pressed                                 |
| Sysmon      | Background | `sysmon.lua`      | Approx. 10% done | System monitor, keyboard reflects system state                                                      |
| Temperature | Background | `temperature.lua` | Approx. 85% done | Temperature monitor. The keyboard reflects the CPU temperature, from 'green = cold' to 'red = hot'  |
| Audio Visualizer 1 | Background | `audioviz1.lua` | Approx 95% done | Shows the current loudness of the configured audio source as a color gradient                   |
| Audio Visualizer 2 | Background | `audioviz2.lua` | Approx 85% done | Shows the current loudness of the configured audio source as HSL colors progressively           |
| Audio Visualizer 3 | Background | `audioviz3.lua` | Approx 95% done | Shows a "spectrum analyzer" visualization of the configured audio source                        |
| Audio Visualizer 4 | Background | `audioviz4.lua` | Approx 85% done | VU-meter like heartbeat effect                                                                  |
| Audio Visualizer 5 | Background | `audioviz5.lua` | Approx 75% done | Like Batique, but with additional audio feedback                                                |

You may combine multiple scripts to so called "effect pipelines" using a profile.
E.g.: You may use one or more backgrounds, and then stack multiple
effects scripts on top of that.


# Further Information <a name="info"></a>

For a documentation of the supported Lua functions and libraries, please
refer to the developer documentation [LIBRARY.md](./LIBRARY.md)


# Contributing <a name="contributing"></a>

Contributions are welcome!
Please see `src/scripts/examples/*.lua` directory for Lua scripting examples.

