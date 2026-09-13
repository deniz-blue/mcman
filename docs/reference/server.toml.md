# Server (`server.toml`)

Each `server.toml` file defines a differient server.

To generate one, you can use the [`mcman init`](../commands/init.md) command (see [Getting Started](../concepts/getting-started.md))

!!! note
    If you are in your server's sub-directories, mcman will be able to find the `server.toml` file recursively.

```toml
name = "My SMP"
mc_version = "1.20.1"

[jar]
type = "purpur"
```

## Fields

`name`: string

:   The name field defines the name of the server. It's recommended to be alphanumeric because of the other features using this field.

    For example, to [overwrite](../concepts/options.md#overriding-server-ports-in-networks) the `SERVER_PORT` variable, you can use the `PORT_name` environment variable where `name` is the server's name.

    The [`SERVER_NAME` variable](../concepts/variables.md#special-variables) can be used to access this field.

`mc_version`: string

:   The mc_version field is used in many ways, most notably:

    - Selecting the server jar
    - Filtering addon versions

    This field can also be accessed using the `SERVER_VERSION`, `mcversion` or `mcver` variables.

`jar`: [ServerType](./servertype/index.md)

:   In the `[jar]` section, you define what kind of server, such as Fabric or Purpur, you want to use.

    See [ServerType](./servertype/index.md) for a list of options you have.

`launcher`: [ServerLauncher](./server-launcher.md)

:   This section defines the [start scripts](./server-launcher.md) which you can configure or disable.

`plugins`: [Downloadable](./downloadable/index.md)[]

:   A list of plugins that mcman should download. Plugins are kept under `server/plugins/`

`mods`: [Downloadable](./downloadable/index.md)[]

:   A list of mods that mcman should download. Mods are kept under `server/mods/`

`variables`: Map<string, string>

:   Define server variables to use in `config/` here. For more information about how this works, read the [Bootstrapping](../concepts/variables.md) section.

`worlds`: Map<string, [World](./world.md)>

:   A table of [World](./world.md)s. [How can I use worlds?](../concepts/using-worlds.md)

`markdown`: [MarkdownOptions](./markdown-options.md)

:   Configure rendering markdown about your server using [Markdown Options](./markdown-options.md)
