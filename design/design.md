# mcman: Cardboard

Codenamed Cardboard is a mcman rewrite to fix a lot of flaws in the original design. The goal is to make it more modular enough to allow for a lot of use cases, providing future compatability.

**Terms:**

- **Target**: An "output" directory or file. For example, a folder where a server jar along with the plugins, config files, etc. OR something like a packwiz pack folder OR a mrpack/zip file.
- **Package**: A set of output files to be put into targets. A mod, a plugin, a server jar, etc.
  - **Preset**: Package building, downloading, etc, is handled by `mcman` itself.
  - **Custom**: Handled by the user using configuration.
- **Runtime**: A dependency that configures the environment a target runs in rather than producing files inside it. A JDK, for example.
- **Platform**: What a target runs on — Paper, Velocity, Fabric. Providers resolve against it to pick between variants of the same package.

<FLAG>

## Components

- Global:
  - **Store**: Cached output of packages
  - **Buildtree**: Temporary directories for building packages
- User source directory
  - **Manifest**: Packages to be installed
  - **Lockfile**: Records the _exact_ versions of installed packages
  - **Target directory**: An output

Global folder definition:

```
.local/share/mcman
├── store
│   ├── objects
│   │   └── <2 letter hash>
│   │       └── <hash>
│   └── packages
│       └── <type>/<path...>/<identity>/pkg.json
└── tmp
    └── <id>
```

## Store

### Objects

The store holds a flat directory of objects which each represent a single file. They are indexed by their hash, which is a 40 character hex string. The first two characters of the hash are used as a directory name to avoid having too many files in a single directory.

Objects are written mode `444`, so a write through a hardlink fails rather than corrupting an object shared by every target.

### Store Packages

The store also holds a directory of package metadata, which describe the contents and dependencies of each installed package in the `pkg.json` file. The path of the package metadata is determined by the package type, the package path, and its **identity**.

Identity distinguishes two builds of the same package:

- **Preset**: the resolved version. `modrinth/luckperms/5.4.102/pkg.json`
- **Custom**: a hash of the resolved inputs — the git commit or download content hash, the build instructions, and the artifact list. This is also what determines whether a rebuild is needed.

### `pkg.json`

```ts
{
	"dependencies": ???,
	"files": {
		"mods/fabric-api-0.14.20+1.20.1.jar": {
			"hash": "<hash>",
			"size": 123456
		}
	}
}
```

## Buildtree

Buildtrees are temporary directories used to build packages. They are created in the store's `tmp` directory and are deleted after the package is built.

## Targets

### Materialization

A target directory contains **real files**. It is never a tree of symlinks into the store by default.

- On the same filesystem, artifacts are **hardlinked** from the store. Otherwise they are copied.
- Anything the server writes, such as config placed by `fs:copy`, is a real copy, never a hardlink.
- `fs:symlink` is an opt-in symlink, for local development.

### Build is offline

With a complete lockfile and a warm store, `mcman build` performs no network I/O. `--locked` enforces it: the build fails rather than resolving if the lockfile does not already cover the manifest.

```dockerfile
FROM mcman AS builder
WORKDIR /build
COPY mcman.kdl mcman.lock ./
RUN --mount=type=cache,target=/var/cache/mcman mcman build --locked smp

FROM eclipse-temurin:21-jre
COPY --from=builder /build/run/smp /server
```

The store lives in a BuildKit cache mount and never enters a layer.

## Manifest

`mcman.kdl` describes the packages to be installed. It is a KDL file that contains a list of packages.

```kdl
runtime "adoptium:jdk" version="21"

group "proxy" {
    platform "velocity"

    use "papermc:velocity" version="latest"

    dir "plugins" {
        use "modrinth:luckperms" version="latest"
        use "modrinth:velocity-viasneak" version="latest"
    }

    target "proxy" path="./run/proxy" type="server"
}

group "game-servers" {
    dir "plugins" {
        use "modrinth:luckperms" version="latest" // Resolves Bukkit/Spigot jar
        use "modrinth:spark" version="latest"
    }

    group "lobby" {
        target "lobby" path="./run/lobby" type="server"

        platform "paper" minecraft="1.21.1"
        use "papermc:paper"

        dir "plugins" {
            use "modrinth:fastasyncworldedit" version="latest"

            package "customplugin" {
                git "https://...customplugin.git"
                build {
                    execute "gradlew build" cd="."
                }
                artifact "build/libs/customplugin.jar"
            }
        }
    }

    group "smp" {
        platform "fabric" minecraft="1.21.1" loader="latest"

        dir "mods" {
            use "modrinth:create" version="latest"
        }

        target "smp-pack" path="./run/smp-packwiz" type="packwiz"

        group "smp-server" {
            target "smp-server" path="./run/smp" type="server"

            use "fabric:fabric"
        }
    }
}
```

### Tags

- **`group`**: Boundary
  - **Argument 0**: Label
- **`use`**: Dependency
  - **Argument 0**: Preset identifier; `<provider>:<id...>`
  - **version=** The version of this package — a Paper build, a Modrinth file. Never a Minecraft version; that comes from `platform`
- **`runtime`**: Environment dependency that produces no files in the target
  - **Argument 0**: Preset identifier; `<provider>:<id...>`
  - **version=**
- **`platform`**: What the target runs on, and the compatibility context providers resolve against
  - **Argument 0**: Platform name; `paper`, `velocity`, `fabric`, ...
  - **Remaining properties** are defined by the platform, not by mcman
- **`package`**: Package to install
  - **Argument 0**: Label
  - **Children**:
    - **`git`**: Clone a git repository
      - **Argument 0**: Repository URL
      - **path=**
    - **`download`**: Download a file
      - **Argument 0**: URL
      - **path=**
    - **`build`**: Build instructions
      - **Children**:
        - **`execute`**: Execute a command
          - **Argument 0**: Command
          - **cd=** Working dir
    - **`artifact`**: A file the package produces
      - **Argument 0**: Source path relative to the build directory
      - **Argument 1**: Destination path relative to the directory the `package` is declared in. Defaults to the file name of argument 0
- **`target`**: Define a target to output something to
  - **Argument 0**: Label
  - **path=** default "."
  - **type=** one of `none`, `client`, `server`, `packwiz`, `mrpack`, `unsup`
- **`dir`**: Specify a directory, appends to target path
- **`fs:copy`**: Copy a file from the source directory to the target directory
  - **Argument 0**: Source path relative to source directory
  - **Argument 1**: Destination path relative to the declaring `dir`
  - **overwrite=** Whether to overwrite existing files (default: false)
- **`fs:symlink`**: Create a readonly symlink from the source directory to the target directory
  - **Argument 0**: Source path relative to source directory
  - **Argument 1**: Destination path relative to the declaring `dir`

## Scoping

Scoping is lexical. A target's contents are a function of its position in the tree and nothing else.

### Inheritance

A target receives every package declared in **its own group and in every ancestor group**, walking from the root down to the group where the target is declared.

```kdl
group "servers" {
    dir "plugins" { use "modrinth:luckperms" }

    group "lobby" { target "lobby" path="./run/lobby" }
    group "smp"   { target "smp"   path="./run/smp" }
}
```

```
run/lobby/plugins/luckperms.jar
run/smp/plugins/luckperms.jar
```

The root group is therefore a global scope: a `use` or `runtime` at the top level applies to every target in the file.

### Nothing flows upward

A target never sees anything declared in a sibling or descendant group.

```kdl
group "network" {
    target "modpack" path="./out/pack" type="mrpack"

    group "lobby" {
        target "lobby" path="./run/lobby" type="server"
        dir "mods" { use "modrinth:sodium" }
    }
}
```

`modpack` does **not** contain sodium. Adding a subgroup can never change what a target above it contains.

### Merging

Directories with the same path merge. `dir "."` and a bare `use` both address the target root, so they merge too.

A package may be declared **once** on the path from the root to a target. Redeclaring one nearer the target is an error, not an override:

```kdl
group "servers" {
    dir "plugins" {
        use "modrinth:spark" version="1.10"
    }

    group "lobby" {
        target "lobby" path="./run/lobby"

        dir "plugins" {
            use "modrinth:spark" version="1.11"   // error
        }
    }
}
```

```
× `modrinth:spark` is declared again in group `lobby`
help: A package may be declared once on the path from the root to a target. To
      give some targets a different version, move the package into a group that
      only those targets are under.
```

The same rule covers `runtime` and `platform`, and covers `fs:copy` and `fs:symlink` by destination.

**An inherited package cannot be overridden or removed.** What a target receives is the union along its path, and every member of that union is declared exactly once. A package wanted by some descendants and not others is expressed in the structure:

```kdl
group "servers" {
    dir "plugins" { use "modrinth:luckperms" }

    group "game" {
        dir "plugins" { use "modrinth:spark" }
        group "smp"      { target "smp" path="./run/smp" }
        group "creative" { target "creative" path="./run/creative" }
    }

    group "lobby" { target "lobby" path="./run/lobby" }
}
```

### Placement

Placement is determined solely by `dir` nesting. Providers never choose a path. Everything that writes a file into a target does so under the `dir` it is declared in: `use`, `package`, `fs:copy` and `fs:symlink` alike. Declared outside any `dir`, they write at the target root.

An `artifact` names one file in the build directory and where it goes. The second argument is a path under the declaring `dir`, and defaults to the file name of the first. Both of these are declared inside `dir "plugins"`:

```kdl
artifact "build/libs/customplugin.jar"           // plugins/customplugin.jar
artifact "build/libs/customplugin.jar" "x/o.jar" // plugins/x/o.jar
```

`type=` does not participate. It selects what the target *outputs* — a server directory, a client directory, a packwiz pack, an mrpack archive — and nothing else.

Which variant a provider resolves to is a third axis, decided by the target's `platform`: `modrinth:luckperms` is a Bukkit jar under Paper and a Fabric jar under Fabric, and both are `type="server"`.

`runtime` has no placement. It is recorded in the lockfile and consumed by the launcher; it never emits a file into a target.

### Platform

A platform is the compatibility context providers resolve against. Each platform defines what its own context contains; there is no universal Minecraft-version field.

```kdl
platform "paper" minecraft="1.21.1"
platform "velocity"
platform "fabric" minecraft="1.21.1" loader="0.16.5"
```

The argument names the platform. Every property past it is defined by that platform, not by mcman.

`platform` is scoped and inherited like `use` and `runtime`, under the same declare-once rule. A target may have **at most one**; a second is a plan-time error, before any network I/O.

**A platform is never inferred.** `use "papermc:paper"` does not establish one. Presets read the platform instead: `platform "paper" minecraft="1.21.1"` with `use "papermc:paper"` is complete, because the provider takes the game version from the context.

`version=` is therefore always the version of that package — a Paper build, a Velocity release, a Modrinth file — and never a Minecraft version.

Each provider declares which platform properties it consumes: `papermc:paper` reads `minecraft`, `fabric:fabric` reads `minecraft` and `loader`, `modrinth` reads `minecraft` and the platform name. A provider whose resolved artifact carries its own compatibility metadata checks it against the context and fails on disagreement.

Declaring a platform does not download it. `platform "fabric" minecraft="1.21.1"` establishes the context; `use "fabric:fabric"` asks for the jar. A packwiz pack records the versions in `pack.toml` and contains no loader jar, so it takes the platform and not the package — which is why `smp` above puts its server in a subgroup.

A missing platform is an error only on demand, raised by the provider that needs one. Modrinth cannot filter versions without it; `download` never asks. A target that only performs `fs:copy` needs no platform.

### Dead groups

A group containing no `target`, and no descendant group containing one, has no effect on any output. mcman warns.

## CLI

Two halves. The first needs an `mcman.kdl`; the second exposes the store and the providers on their own.

### Manifest commands

| Command | |
| --- | --- |
| `mcman build [targets...]` | Resolve, fetch, materialize. No targets means all of them. |
| `mcman lock [targets...]` | Resolve and write `mcman.lock`. Nothing is materialized. |
| `mcman update [packages...]` | Re-resolve past the existing pins. |
| `mcman explain [targets...]` | What each target resolves to, annotated with the group each entry was declared in. |
| `mcman init` | Scaffold an `mcman.kdl`. |

`build` resolves and updates the lockfile on its own when the manifest has moved.

- **`--locked`** fails instead, if the lockfile would have to change.
- **`--offline`** never touches the network and fails if the store is missing something.
- **`--force`** rebuilds custom packages whose identity is unchanged.

The manifest is found by searching upward from the working directory, or named with `-f`.

### Standalone commands

| Command | |
| --- | --- |
| `mcman store put <file>...` | Prints the hash of each. |
| `mcman store get <hash> [-o <path>]` | Materializes it, or prints its store path. |
| `mcman store has <hash>` | Exit code only. |
| `mcman store list` / `path` | |
| `mcman store prune [--older-than 30d]` / `clear` / `verify` | |
| `mcman resolve <id>...` | The concrete version, URL and hash. Network, no writes. |
| `mcman fetch <id\|url>... [-o <dest>]` | Resolve, put in the store, materialize if `-o` is given. |

`fetch` accepts a bare URL as well as a preset. The platform is given by flag, and is not inferred from the identifier:

```bash
mcman fetch papermc:paper --platform paper --minecraft 1.21.1 -o server.jar
```

`mcman fetch papermc:velocity` needs neither flag; Velocity's context is empty.

`store prune` removes objects by age. Liveness cannot be computed — the store is global and mcman cannot know every manifest on the machine.

### Output

`--json` on everything that emits data — `explain`, `resolve`, `fetch`, `store list`. Its shape is a stability commitment.

`MCMAN_STORE` sets the store path alongside `--store`.

## Lockfile

Lockfile also uses KDL format.

**Tags**:

- **`meta`**
  - **version=** 1
  - **`generated`**: Timestamp
- **`target`**
  - **Argument 0**: Label
  - **path=** Path
  - **Children**:
    - **`platform`**: The resolved context; a change to it invalidates the lock
      - **Argument 0**: Platform name
      - Platform-defined properties
    - **`use`**
      - **Argument 0**: `<provider>:<id...>`
      - **version=** Locked version
    - **`runtime`**
      - **Argument 0**: `<provider>:<id...>`
      - **version=** Locked version
    - **`package`**
	  - **Argument 0**: Label
	  - **identity=** Hash of the resolved inputs
    - **Children** of `use` or `package`:
      - **`artifact`**
        - **Argument 0**: Path relative to target
        - **hash=** Hash of the file
        - **size=** Size of the file in bytes

`runtime` has no `artifact` children.

```kdl
meta version=1 generated=2024-06-01T12:00:00Z

target "proxy" path="./run/proxy" {
	platform "velocity"
	runtime "adoptium:jdk" version="21.0.4+7"

	use "papermc:velocity" version="3.4.0" {
		artifact "velocity-3.4.0.jar" hash="abcdef" size=123456
	}
	use "modrinth:luckperms" version="5.4.102" {
		artifact "plugins/luckperms-velocity-5.4.102.jar" hash="abcdef" size=123456
	}
	package "customplugin" identity="abcdef" {
		artifact "plugins/customplugin.jar" hash="abcdef" size=654321
	}
}
```

## Resolution Steps

1. Read the manifest and lockfile
2. Walk the manifest tree. For each target, accumulate the packages, runtimes, directories and platform declared along the path from the root to the target's group, merging same-path directories. A second declaration of anything already on the path is an error, and more than one `platform` is an error
   - Warn on any group whose subtree contains no target
3. Resolve each target's wanted packages against its platform
  - If a preset, resolve the preset to an instructed package
  - A provider that needs a platform and finds none declared fails here, not during the walk
  - A provider reads whatever platform properties it needs; `version=` is only ever the package's own version
  - A provider that resolves something carrying its own compatibility metadata checks it against the declared platform and fails on disagreement

**Package resolution & installation**:

1. Resolve the package to a specific version, or compute its identity from its inputs
2. Check if the package is already installed in the store
  - If it is, skip to step 4
3. If it is not, we will build the package
   - Check each source - `git` and `download`; they are also upserted into the store. If they are up to date, we can skip to step 5
   - If they are not up to date, we will download them and upsert them into the store
   - Run the build instructions, which will generate files in the buildtree
   - Collect the files via the `artifact` instructions, and upsert them into the store
   - Write the package metadata to the store
4. Materialize the package files from the store into the target directory — hardlink where the filesystem allows, otherwise copy
