# mcman: Cardboard

Codenamed Cardboard is a mcman rewrite to fix a lot of flaws in the original design. The goal is to make it more modular enough to allow for a lot of use cases, providing future compatability.

**Terms:**

- **Target**: An "output" directory or file. For example, a folder where a server jar along with the plugins, config files, etc. OR something like a packwiz pack folder OR a mrpack/zip file.
- **Package**: A set of output files to be put into targets. A mod, a plugin, a server jar, etc.
  - **Preset**: Package building, downloading, etc, is handled by `mcman` itself.
  - **Custom**: Handled by the user using configuration.

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
│       └── <type>/<path...>/pkg.json
└── tmp
    └── <id>
```

## Store

### Objects

The store holds a flat directory of objects which each represent a single file. They are indexed by their hash, which is a 40 character hex string. The first two characters of the hash are used as a directory name to avoid having too many files in a single directory.

### Store Packages

The store also holds a directory of package metadata, which describe the contents and dependencies of each installed package in the `pkg.json` file. The path of the package metadata is determined by the package type and the package path.

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

## Manifest

`mcman.kdl` describes the packages to be installed. It is a KDL file that contains a list of packages.

```kdl
use "adoptium:jdk" version="21"

group "proxy" {
    use "papermc:velocity" version="latest"

    dir "plugins" {
        use "modrinth:luckperms" version="latest"
        use "modrinth:velocity-viasneak" version="latest"
    }

    target path="./run/proxy" type="server"
}

group "game-servers" {
    dir "plugins" {
        use "modrinth:luckperms" version="latest" // Resolves Bukkit/Spigot jar
        use "modrinth:spark" version="latest"
    }

    group "lobby" {
        target path="./run/lobby" type="server"

        use "papermc:paper" version="1.21.1"

        dir "plugins" {
            use "modrinth:fastasyncworldedit" version="latest"
        }

		package "customplugin" {
			git "https://...customplugin.git"
			build {
				execute "gradlew build" cd="."
			}
			link "build/libs/customplugin.jar" "plugins/customplugin.jar"
		}
    }

    group "smp" {
        target path="./run/smp" type="server"
        target path="./run/smp-packwiz" type="packwiz"
		
        use "fabric:fabric" version="1.21.1" loader="latest"

        dir "mods" {
            use "modrinth:create" version="latest"
        }
    }
}
```

### Tags

- **`group`**: Boundary
- **`use`**: Dependency
  - **Argument 0**: Preset identifier; `<provider>:<id...>`
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
    - **`link`**: Files to use from the buildtree
      - **Argument 0**: Source path relative to build directory
      - **Argument 1**: Destination path relative to target directory
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
    - **`package`**
	  - **Argument 0**: Label
    - **Children** of either `use` or `package`:
      - **`artifact`**
        - **Argument 0**: Path relative to target
        - **hash=** Hash of the file
        - **size=** Size of the file in bytes

```kdl
meta version=1 generated=2024-06-01T12:00:00Z

target "proxy" path="./run/proxy" {
	use "papermc:velocity" version="3.4.0" {
		artifact "velocity-3.4.0.jar" hash="abcdef" size=123456
	}
	package "modrinth:luckperms" {
		artifact "plugins/luckperms-fabric-10.4.1.jar" hash="abcdef" size=123456
	}
	package "modrinth:velocity-viasneak" {
		artifact "plugins/velocity-viasneak-1.0.0.jar" hash="abcdef" size=654321
	}
}
```

## Resolution Steps

1. Read the manifest and lockfile
2. Iterate through the manifest, build up a list of targets and their wanted packages
3. Resolve each target's wanted packages
  - If a preset, resolve the preset to an instructed package

**Package resolution & installation**:

1. Resolve the package to a specific version
2. Check if the package is already installed in the store
  - If it is, skip to step 4
3. If it is not, we will build the package
   - Check each source - `git` and `download`; they are also upserted into the store. If they are up to date, we can skip to step 5
   - If they are not up to date, we will download them and upsert them into the store
   - Run the build instructions, which will generate files in the buildtree
   - Collect the files via the `link` instructions, and upsert them into the store
   - Write the package metadata to the store
4. Symlink the package files from the store to the target directory

