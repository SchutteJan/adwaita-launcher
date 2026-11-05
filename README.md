<div align="center">
  <h1>Adwaita Launcher</h1>
  <img src="resources/screenshot-fire.webp" alt="Adwaita Launcher showing search results for &quot;fire&quot; in light and dark styles">
</div>

An application launcher for Linux, built with libadwaita, a frontend for [pop-launcher](https://github.com/pop-os/launcher). Similar to `cosmic-launcher` for the Cosmic Desktop environment.

## Building

You need the GTK 4 and libadwaita development packages to build it, and `pop-launcher` on your `PATH` at runtime.

```sh
cargo install --git https://github.com/SchutteJan/adwaita-launcher --locked
```

Bind `adwaita-launcher` binary to a keyboard shortcut, (full path `~/.cargo/bin/adwaita-launcher`).


## Blur support

For the background blur to work, your Wayland compositor must support the [`ext-background-effect-v1`](https://wayland.app/protocols/ext-background-effect-v1) protocol.
