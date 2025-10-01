river-swww
==========
[![Workflow Status](https://github.com/l1na-forever/river-swww/actions/workflows/autobuild.yml/badge.svg)](https://github.com/l1na-forever/river-swww/actions) [![Crates.io Version](https://img.shields.io/crates/v/river-swww)](https://crates.io/crates/river-swww)

Sets [swww](https://github.com/LGFae/swww)'s wallpaper based on [river](https://codeberg.org/river/river)'s focused tags.

![Demo GIF showing wallpaper transitions](https://github.com/l1na-forever/river-swww/raw/main/demo.gif)

## Usage

Install [`river-bedload`](https://git.sr.ht/~novakane/river-bedload). `river-bedload` is used to stream river's events (tag changes) to `river-sww`.

Next, install `river-swww`:

```sh
cargo install river-swww
```

Typically, `river-swww` is added as a service that starts with `river` (for example, by adding a runit service entry that invokes `river-swww`). `river-swww` could also be added to `~/.config/river/init`, however, the service won't be automatically restarted upon termination.

## Configuration

Upon first run, a configuration file with default values is generated at `~/.config/river-swww/config.json`. After customizing the configuration, restart `river-swww`.

The following keys will configure river-swww:

* `swww_args`: Command-line arguments to pass to `swww`. This is useful for configuring transition settings.
* `default`: A path to a default wallpaper, used as a fallback when a tag is applied that doesn't have a matching entry in `tags`.
* `tags`: A map keyed by tag name to the associated wallpaper's path.

Example configuration:

```json
{
  "swww_args": "--transition-type simple --transition-fps 30 --transition-step 12",
  "default": "/home/lina/Pictures/Wallpapers/default.jpg",
  "tags": {
    "1": "/home/lina/Pictures/Wallpapers/1.jpg",
    "2": "/home/lina/Pictures/Wallpapers/2.jpg",
    "3": "/home/lina/Pictures/Wallpapers/3.jpg",
    "4": "/home/lina/Pictures/Wallpapers/4.jpg",
    "5": "/home/lina/Pictures/Wallpapers/5.jpg"
  }
}
```

## Troubleshooting

Run `river-swww` interactively and monitor its output as river's tag focus changes.

## Licence

Copyright © 2025 Lina

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDER
