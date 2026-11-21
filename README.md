# StarknetChat — Backend

The services and libraries that power [starknet-chat.tech](https://starknet-chat.tech).

This repository is a modified fork of Stoat's backend (AGPL-3.0). It is published here to comply with that license. Related repositories:

- Web client: https://github.com/EDGAR431/starknet-chat-web
- Desktop app: https://github.com/EDGAR431/starknet-chat-desktop
- Android app: https://github.com/EDGAR431/starknet-chat-android

| Crate              | Path                                               | Description                         |
| ------------------ | --------------------------------------------------- | ----------------------------------- |
| `core/config`      | [crates/core/config](crates/core/config)           | Core: Configuration                 |
| `core/database`    | [crates/core/database](crates/core/database)       | Core: Database Implementation       |
| `core/files`       | [crates/core/files](crates/core/files)             | Core: S3 and encryption subroutines |
| `core/models`      | [crates/core/models](crates/core/models)           | Core: API Models                    |
| `core/permissions` | [crates/core/permissions](crates/core/permissions) | Core: Permission Logic              |
| `core/presence`    | [crates/core/presence](crates/core/presence)       | Core: User Presence                 |
| `core/result`      | [crates/core/result](crates/core/result)           | Core: Result and Error types        |
| `core/coalesced`   | [crates/core/coalesced](crates/core/coalesced)     | Core: Coalescion service            |
| `delta`            | [crates/delta](crates/delta)                       | REST API server                     |
| `bonfire`          | [crates/bonfire](crates/bonfire)                   | WebSocket events server             |
| `services/january` | [crates/services/january](crates/services/january) | Proxy server                        |
| `services/gifbox`  | [crates/services/gifbox](crates/services/gifbox)   | Tenor proxy server                  |
| `services/autumn`  | [crates/services/autumn](crates/services/autumn)   | File server                         |
| `daemons/crond`    | [crates/daemons/crond](crates/daemons/crond)       | Timed data clean up daemon server   |
| `daemons/pushd`    | [crates/daemons/pushd](crates/daemons/pushd)       | Push notification daemon server     |

## Minimum Supported Rust Version

Rust 1.86.0 or higher.

## Development Guide

Before getting started, you'll want to install:

- mise
- Docker
- Git
- mold (optional, faster compilation)

> **Flake** is available for Nix users! Run `nix develop` to enter the dev shell.

As a heads-up, the development environment uses the following ports:

| Service                   |      Port      |
| ------------------------- | :------------: |
| MongoDB                   |     27017      |
| Redis                     |      6379      |
| MinIO                     |     14009      |
| Maildev                   | 14025<br>14080 |
| Web App                   |     14701      |
| RabbitMQ                  | 5672<br>15672  |
| Grafana LGTM (OTLP)       | 13000<br>4318  |
| `crates/delta`            |     14702      |
| `crates/bonfire`          |     14703      |
| `crates/services/autumn`  |     14704      |
| `crates/services/january` |     14705      |
| `crates/services/gifbox`  |     14706      |

Now you can clone and build the project:

```bash
git clone https://github.com/EDGAR431/starknet-chat-backend
cd starknet-chat-backend
mise install
mise build
```

> [!TIP]
> You can override `BUILDER` in your `.env` file to run cargo with mold if you installed it:
>
> ```bash
> # .env
> BUILDER = "mold --run cargo"
> ```

A default configuration `Revolt.toml` is present in this project that is suited for development.

If you'd like to change anything, create a `Revolt.overrides.toml` file and specify relevant variables.

> [!TIP]
> If you have port conflicts on common services, you can try the following:
>
> ```yaml
> # compose.override.yml
> services:
>   redis:
>     ports: !override
>       - "14079:6379"
>
>   database:
>     ports: !override
>       - "14017:27017"
>
>   rabbit:
>     ports: !override
>       - "14072:5672"
>       - "14672:15672"
>
>   otel-lgtm:
>     ports: !override
>       - "13000:3000"
>       - "14318:4318"
> ```
>
> With the corresponding configuration:
>
> ```toml
> #     Revolt.overrides.toml
> # and Revolt.test-overrides.toml
> [database]
> mongodb = "mongodb://127.0.0.1:14017/?directConnection=true"
> redis = "redis://127.0.0.1:14079/"
>
> [rabbit]
> port = 14072
> ```

Then continue:

```bash
cp livekit.example.yml livekit.yml

mise start
```

You can start the web client by cloning `starknet-chat-web` in another terminal — see its README for startup instructions.

To stop all services, hit (CTRL + c) in the terminal you ran `mise start` and run `mise docker:stop`.

## Environment Variables

This list is non-exhaustive right now.

| Variable                              | Default                 | Purpose                                                               |
| -------------------------------------- | ------------------------ | ----------------------------------------------------------------------- |
| `RUST_LOG`                            | `info`                  | Log filter, e.g. `info,revolt_autumn=debug`                           |
| `OTEL_EXPORTER_OTLP_LOGS_ENDPOINT`    |                         | OTLP/HTTP endpoint for logs                                           |
| `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT`  |                         | OTLP/HTTP endpoint for traces                                         |
| `OTEL_EXPORTER_OTLP_METRICS_ENDPOINT` |                         | OTLP/HTTP endpoint for metrics                                        |
| `OTEL_EXPORTER_OTLP_ENDPOINT`         | `http://localhost:4318` | Base URL for a single collector; appends `/v1/{logs,traces,metrics}`  |
| `OTEL_RESOURCE_ATTRIBUTES`            |                         | Extra resource attributes, e.g. `deployment.environment.name=staging` |
| `OTEL_TRACES_SAMPLER`                 | `parentbased_always_on` | Trace sampler, e.g. `parentbased_traceidratio`                       |
| `OTEL_TRACES_SAMPLER_ARG`             | `1.0`                   | Sampling ratio, e.g. `0.1`                                            |

## Testing

First, start the required services:

```sh
docker compose up -d
```

Now run tests for whichever database:

```sh
TEST_DB=REFERENCE cargo nextest run
TEST_DB=MONGODB cargo nextest run
```

## License

AGPL-3.0-or-later — see [LICENSE](./LICENSE). Individual crates may supply their own licenses.
