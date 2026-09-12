# Server Launcher

The `[launcher]` table in [`server.toml`](./server.toml.md) lets mcman create launch scripts for you.

The scripts are named `start.sh` and `start.bat` and are created inside the output directory, `server/`.

The order of these arguments are:

`java [jvm_args] [memory] [preset] [eula] [properties] <startup> [nogui] [game_args]`

Where `startup` is either `-jar *.jar` or some library shenanigans (NeoForge/Forge require this).

??? example "Example ServerLauncher"

    ``` toml
    [launcher]
    disable = false # (1)

    # (2)
    jvm_args = "-exampleidk"
    game_args = "--world abc"

    preset_flags = "aikars" # (3)

    eula_args = true # (4)

    nogui = true # (5)

    memory = "2048M" # (6)

    # (7)
    [launcher.properties]
    hello="thing"
    ```


    1. Disables generating launch scripts completely. `false` by default
    2. If needed, you can add custom arguments here. The format is `java [jvm_args] -jar server.jar [game_args]`
    3. Select a preset: `aikars` uses Aikar's flags for server optimizations, while `proxy` is intended for BungeeCord, Waterfall, and Velocity.
    4. Adds `-Dcom.mojang.eula.agree=true` - this flag exists in spigot/paper to ignore `eula.txt`. Writes to `eula.txt` when on fabric or quilt
    5. Adds `--nogui` to game args, disable if its a proxy server as they dont support it
    6. Specify `-Xmx`/`-Xms` (memory) for the server.
    7. A table of properties. This is the same as using
       ``` toml
       jvm_args = "-Dhello=thing"
       ```

## Fields

`disable`: bool

:   If set to true, mcman will not generate start scripts

`memory`: string

:   Amount of memory to give, in jvm byte units. These are set using the `-Xmx`/`-Xms` arguments.

    For example, `2048M`

    You can also override this using the `MC_MEMORY` environment variable while building.

`java_version`: string

:   This field does not add any arguments to the startup command but rather helps mcman decide which java binary to use.

    ```toml
    [launcher]
    java_version = "17"
    ```

    See [this section](../tutorials/options.md#setting-the-java-binary) for more information.

`nogui`: bool

:   Adds `--nogui` to the end as a game argument. Set this to false for proxy servers since they dont support it.

`preset_flags`: PresetFlags

:   Select a preset. Available presets are:

    - `none` (default)
    - `aikars`: Use [Aikar's Flags](https://mcflags.emc.gs), there's also [a post by PaperMC](https://docs.papermc.io/paper/aikars-flags) about it
    - `proxy`: Preset for proxy servers

`eula_args`: bool

:   Bukkit/Spigot forks such as Paper and Purpur all support the `-Dcom.mojang.eula.agree=true` system property flag which allows the agreement of eula without `eula.txt`. If this is set to **`true`**, mcman will add this flag to the arguments. If the server software does not support this argument (such as fabric), `eula=true` will be written to `eula.txt`. 

`jvm_args`: string

:   Add JVM arguments here. JVM arguments are generally in the format of `-X...` and are entered before `-jar server.jar`

`game_args`: string

:   Game arguments are entered *after* `-jar server.jar` and are picked up by the server process.

`properties`: Map<string, string>

:   Enter a table of system property arguments. Properties are in the format of `-Dkey=value`.

    For example, to write `-Dterminal.jline=false -Dterminal.ansi=true`:
    ```toml
    [launcher.properties]
    terminal.jline=false
    terminal.ansi=true
    ```
