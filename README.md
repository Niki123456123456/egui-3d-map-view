# egui-3d-map-view

Experimental Rust crate for embedding `three-d` rendering inside `egui`/`eframe`, with map-oriented helpers for orbit controls, Google Photorealistic 3D Tiles, search, and GPX route display.

![3D map view](3d_map_view.gif)

## What It Does

- Renders `three-d` scenes inside egui panels and windows.
- Provides orbit-style camera controls.
- Loads and renders Google 3D map tiles.
- Supports place search through Nominatim.
- Supports GPX route loading in the richer map example.
- Includes native and WebAssembly examples.


## Status

This is experimental code. It currently depends on forked Git versions of `three-d` and `draco-gltf-rs`.
