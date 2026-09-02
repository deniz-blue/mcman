# mcman: Cardboard

Codenamed Cardboard is a mcman rewrite to fix a lot of flaws in the original design. The goal is to make it more modular enough to allow for a lot of use cases, providing future compatability.

**Terms:**

- **Target**: An "output" directory or file. For example, a folder where a server jar along with the plugins, config files, etc. OR something like a packwiz pack folder OR a mrpack/zip file.
- **Package**: A set of output files to be put into targets. A mod, a plugin, a server jar, etc.
  - **Preset**: Package building, downloading, etc, is handled by `mcman` itself.
  - **Custom**: Handled by the user using configuration.
- **Runtime**: A dependency that configures the environment a target runs in rather than producing files inside it. A JDK, for example.

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

Objects are written mode `444`. Targets hardlink to them, so a process that writes through a hardlink must fail loudly rather than silently corrupt an object shared by every other target.

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
- A hardlink is indistinguishable from a copy to Docker, rsync, hosting panels, `zip` and packwiz; it needs no privileges on Windows; and it does not dangle when the store is absent.
- Anything the server is expected to write — config placed by `copy` — is a real copy, never a hardlink.
- Group-level `link` is an explicit opt-in to symlinking, for local development where the store should stay the single source of truth.

### Build is offline

With a complete lockfile and a warm store, `mcman build` performs no network I/O. This is the invariant that makes mcman usable as a build step:

```dockerfile
FROM mcman AS builder
WORKDIR /build
COPY mcman.kdl mcman.lock ./
RUN --mount=type=cache,target=/var/cache/mcman mcman build --target smp

FROM eclipse-temurin:21-jre
COPY --from=builder /build/run/smp /server
```

The store lives in a BuildKit cache mount, so it never enters a layer, and the runtime stage receives plain files.

## Manifest

`mcman.kdl` describes the packages to be installed. It is a KDL file that contains a list of packages.

```kdl
runtime "adoptium:jdk" version="21"

group "proxy" {
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

        use "papermc:paper" version="1.21.1"

        dir "plugins" {
            use "modrinth:fastasyncworldedit" version="latest"
        }

        package "customplugin" {
            git "https://...customplugin.git"
            build {
                execute "gradlew build" cd="."
            }
            artifact "build/libs/customplugin.jar" "plugins/customplugin.jar"
        }
    }

    group "smp" {
        target "smp-server" path="./run/smp" type="server"
        target "smp-pack" path="./run/smp-packwiz" type="packwiz"

        use "fabric:fabric" version="1.21.1" loader="latest"

        dir "mods" {
            use "modrinth:create" version="latest"
        }
    }
}
```

### Tags

- **`group`**: Boundary
  - **Argument 0**: Label
- **`use`**: Dependency
  - **Argument 0**: Preset identifier; `<provider>:<id...>`
  - **version=**
- **`runtime`**: Environment dependency that produces no files in the target
  - **Argument 0**: Preset identifier; `<provider>:<id...>`
  - **version=**
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
    - **`artifact`**: Files the package produces
      - **Argument 0**: Source path relative to the build directory
      - **Argument 1**: Destination path relative to the target directory
- **`target`**: Define a target to output something to
  - **Argument 0**: Label
  - **path=** default "."
  - **type=** one of `none`, `client`, `server`, `packwiz`, `mrpack`, `unsup`
- **`dir`**: Specify a directory, appends to target path
- **`copy`**: Copy a file from the source directory to the target directory
  - **Argument 0**: Source path relative to source directory
  - **Argument 1**: Destination path relative to target directory
  - **overwrite=** Whether to overwrite existing files (default: false)
- **`link`**: Create a readonly symlink from the source directory to the target directory
  - **Argument 0**: Source path relative to source directory
  - **Argument 1**: Destination path relative to target directory

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

Directories with the same path merge. Where the same preset identifier appears more than once along the path, the declaration closest to the target wins.

```kdl
group "servers" {
    dir "plugins" {
        use "modrinth:luckperms"
        use "modrinth:spark" version="1.10"
    }

    group "lobby" {
        target "lobby" path="./run/lobby"
        dir "plugins" {
            use "modrinth:fastasyncworldedit"
            use "modrinth:spark" version="1.11"
        }
    }
}
```

```
run/lobby/plugins/luckperms.jar
run/lobby/plugins/spark-1.11.jar
run/lobby/plugins/fastasyncworldedit.jar
```

**There is no way to remove an inherited package.** The set a target receives is a monotone union along its root path, so answering "does this target have X?" never requires proving an absence — no removal three levels up can invalidate what a group reads like locally. Version override preserves this; removal would not.

If a package is wanted by some descendants and not others, it was never shared by the ancestor. Express that in the structure:

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

Should cross-cutting exceptions turn out to need more than nesting can express, the answer is an additive one — a named block referenced from several groups — not a subtractive one.

### Placement

Placement is determined solely by `dir` nesting. Providers never choose a path. A `use` outside any `dir` places its files at the target root.

`type=` does not participate. It selects what the target *outputs* — a server directory, a client directory, a packwiz pack, an mrpack archive — and nothing else.

Which artifact a provider resolves to is a separate axis again: `modrinth:luckperms` is a Bukkit jar under Paper and a Fabric jar under Fabric, and both of those are `type="server"`. That is decided by the **platform** in the target's resolved set, from `use "papermc:paper"` or `use "fabric:fabric"`. The `smp` group above shows the two axes are independent — one Fabric loader feeding a `server` target and a `packwiz` target.

**Open:** how the platform is established when no loader is in scope, as in a client modpack that lists only mods.

`runtime` has no placement at all. It is recorded in the lockfile and consumed by the launcher; it never emits a file into a target.

### Dead groups

A group containing no `target`, and no descendant group containing one, has no effect on any output. mcman warns.

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
2. Walk the manifest tree. For each target, accumulate the packages, runtimes and directories declared along the path from the root to the target's group, merging same-path directories and letting the declaration closest to the target win
   - Warn on any group whose subtree contains no target
3. Resolve each target's wanted packages
  - If a preset, resolve the preset to an instructed package

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
