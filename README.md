# Yazelix Screen

Standalone terminal screen animations from Yazelix

The user-facing command is `yzs`

```bash
nix run github:luccahuguet/yazelix#yzs
nix run github:luccahuguet/yazelix#yzs -- mandelbrot
nix run github:luccahuguet/yazelix#yzs -- game_of_life_bloom --cell-style dotted
```

## Status

This repository is a placeholder for the standalone Yazelix Screen project

The source currently lives in the Yazelix monorepo:

https://github.com/luccahuguet/yazelix/tree/main/rust_core/yazelix_screen

The standalone package is available through the Yazelix flake as `.#yzs`

## Surfaces

- Product/repository: `yazelix-screen`
- Command: `yzs`
- Rust crate: `yazelix_screen`
- Integrated Yazelix command: `yzx screen`

## Why

Some terminal users may want the animated terminal screens without installing or launching full Yazelix

This repo gives that standalone surface a stable home before the code is extracted from the monorepo
