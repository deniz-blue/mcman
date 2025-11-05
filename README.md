# mcman

![mcman](./docs/mcman-3.png)

[![GitHub release](https://img.shields.io/github/release/deniz-blue/mcman.svg)](https://github.com/deniz-blue/mcman/releases/latest)
[![builds](https://img.shields.io/github/actions/workflow/status/deniz-blue/mcman/build.yml?logo=github)](https://github.com/deniz-blue/mcman/actions/workflows/build.yml)
[![docker publish](https://img.shields.io/github/actions/workflow/status/deniz-blue/mcman/publish.yml?logo=github&label=docker%20publish)](https://github.com/deniz-blue/mcman/actions/workflows/publish.yml)
![GitHub Repo stars](https://img.shields.io/github/stars/deniz-blue/mcman?logo=github)
[![quiltmc featured](https://img.shields.io/badge/quiltmc-featured-8A2BE2)](https://tech.lgbt/@quiltmc/110690787441497920)
![downloads](https://img.shields.io/github/downloads/deniz-blue/mcman/total?logo=github)
[![discord server](https://img.shields.io/discord/1197520507617153064?logo=discord)](https://discord.com/invite/QrBzpDxUyB)

Powerful Minecraft Server Manager CLI. Manage your servers using git - easily install jars (server, plugins & mods); manage config files, worlds, entire networks, and more.

> [!NOTE]
> A rewrite which fixes a ton of flaws is underway, join discussion on the discord to be updated of the progress. It can be found under the `v2` branch

## Getting Started

![mcman init](https://cdn.discordapp.com/attachments/1109215116060266567/1134187743300296815/render1690481729604.gif)

| 🚀 [Installation](https://deniz-blue.github.io/mcman/installation/) | ✨ [Getting Started](https://deniz-blue.github.io/mcman/tutorials/getting-started/) | 📜 [Documentation](https://deniz-blue.github.io/mcman/) |
| ------------------------------------------------------------------ | ---------------------------------------------------------------------------------- | ------------------------------------------------------ |

- Join the [discord](https://discord.gg/YFuxNmKvSr) for support!
- 📋 Some examples can be found under [examples/](./examples/)
- 🚀 Here's [BlanketCon '23](https://github.com/ParadigmMC/mcman-bc23) imported to mcman - you can even see its [test workflow](https://github.com/ParadigmMC/mcman-bc23/actions/workflows/bc23test.yml)

Submit a PR or open an issue if you have a mcman-server repository that we can add here!

## Features

- 📜 Everything in one simple `server.toml` file!
- 📥 Downloads everything automatically for you
- 🔁 Always keep up to date with new builds and releases
- 🔒 Does size & hash checks even for cached files for integrity
- :octocat: Fully `git`-compatible
- 📦 **Import** from or **export** to `mrpack`s or `packwiz` packs
- 📚 Supports way too many sources, some are:
  - Modrinth, CurseRinth, Spigot, Hangar, Github Releases, Jenkins, Maven
  - Not here? You can use custom urls.
- 🏷️ Easily render a list of plugins/mods or the server info to **markdown** files
- ✨ Managing a network? Use `network.toml` to manage servers' ports, have shared variables, config files, plugins and mods.
- 🌐 Keep worlds as `worlds/*.zip` to version control them, or set it to be downloaded from somewhere!
- ☔ Develop your servers with hot reloading using `mcman dev`
- 🔁 Test your servers using CI (`mcman run --test`)
- 🔗 mclo.gs integration
- 🐳 Supports Docker, out of the box
- ✔️ No third-party hosts (metadata/mirrors)
- ⚙️ Better configuration files with `config/`!
  - Allows you to use **config variables** inside your config files
  - Use **environment variables** for secrets

> [!IMPORTANT]
> While `mcman` can manage your server, its not designed to run it. You should use something else, for example, docker or pterodactyl to run your servers.

## Reviews

> "faster than gradle builds"
- kuylar

> "makes even oracle linux usable"
- PureComedi

> "it's ok"
- null

> "I'm technically a contributor"
- Trash Panda

## Changelog

Changelogs can be see in detail [here](CHANGELOG.md)

