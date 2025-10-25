# mcman-rewrite2

This README file will act as a project guide/todo list until things get into a usable state

## Architecture

- `crates/`
  - `mcman-core` context, caching, downloading, source registry logic etc
  - `mcman-sources` external source providers
  - `mcman-types` abstract types
  - `mcman-models` config types (server.toml, pack.toml etc)
  - `mcman-compile` building modpack zips etc
  - `mcman-launch` child process logic
  - `mcman` brings everything above together
  - `mcman-cli` cli commands that use `mcman`

## TODO

- Now
  - [ ] implement core
    - [ ] Context
    - [ ] http client
    - [ ] caching
    - [ ] SourceProvider
- Important
  - [ ] implement sources
    - [ ] modrinth
    - [ ] curseforge
    - [ ] fabric
    - [ ] quilt
    - [ ] forge
    - [ ] neoforge
    - [ ] spigot
    - [ ] papermc
    - [ ] purpur
- Later
- Future
- Very very future
  - [ ] add nix-flake back
