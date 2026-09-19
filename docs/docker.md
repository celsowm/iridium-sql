# Docker and Testcontainers

The official Iridium SQL image uses the native Rust TDS server on TCP port
`1433`. It is intended for local development and isolated integration tests;
it is **not** a complete Microsoft SQL Server implementation. Check the
[compatibility matrix](compatibility-matrix.md) for supported T-SQL and TDS
behavior.

## Run an isolated database

After the [Docker image workflow](../.github/workflows/docker-image.yml) has
successfully published the image, start an ephemeral instance:

```sh
docker run --rm --name iridium-sql -p 127.0.0.1:1433:1433 \
  ghcr.io/celsowm/iridium-sql:latest --memory
```

To build and run from this checkout instead of pulling a published image:

```sh
docker build -t iridium-sql:local .
docker run --rm -p 127.0.0.1:1433:1433 iridium-sql:local --memory
```

The container listens on `0.0.0.0:1433` internally. Bind the published port
to `127.0.0.1` on your host to avoid exposing this test server on the network.

**Security:** The image starts without TLS or authentication for simple,
isolated test use. The server accepts any login while authentication is
disabled; do not expose it to an untrusted network. A client using
`Microsoft.Data.SqlClient` should set `Encrypt=False` explicitly. You may
supply server flags such as `--user sa --password ...` and
`--tls-cert ... --tls-key ... --tls` if your environment needs them; do not
place production passwords in container command-line arguments.

## Persist data

The image defaults to native persistent storage in `/var/lib/iridium`.
Mount a named volume when data must survive container removal:

```sh
docker volume create iridium-data
docker run --rm --name iridium-sql -p 127.0.0.1:1433:1433 \
  -v iridium-data:/var/lib/iridium \
  ghcr.io/celsowm/iridium-sql:latest
```

For disposable tests, pass `--memory`: no database files are created. The
container runs as the non-root user/group `10001:10001`. If you bind-mount
a host directory, make it writable by that UID.

The image contains a Docker health check that verifies TCP port 1433 is
accepting connections. This is a readiness signal, not proof of full SQL
Server protocol or query compatibility.

## Testcontainers for .NET

Use the generic `ContainerBuilder` rather than `MsSqlBuilder`: the latter
targets Microsoft's SQL Server image and may assume Microsoft-specific
environment variables and startup behavior. The following example uses
`Testcontainers` and `Microsoft.Data.SqlClient` NuGet packages:

```csharp
using DotNet.Testcontainers.Builders;
using Microsoft.Data.SqlClient;

await using var container = new ContainerBuilder()
    .WithImage("ghcr.io/celsowm/iridium-sql:latest")
    .WithPortBinding(1433, true)
    .WithCommand("--memory")
    .WithWaitStrategy(Wait.ForUnixContainer().UntilPortIsAvailable(1433))
    .Build();

await container.StartAsync();

var connectionString =
    $"Server={container.Hostname},{container.GetMappedPublicPort(1433)};" +
    "Database=master;User Id=sa;Password=test-only;" +
    "Encrypt=False;TrustServerCertificate=True;Connection Timeout=10;";

await using var connection = new SqlConnection(connectionString);
await connection.OpenAsync();
await using var command = new SqlCommand("SELECT 1", connection);
var value = await command.ExecuteScalarAsync();
if (Convert.ToInt32(value) != 1)
    throw new Exception("Iridium SQL smoke query failed");
```

The password above is illustrative: the default image does not validate it.
For application test suites, substitute this connection string for the
SQL Server endpoint and validate the actual SQL features your tests need.
Port readiness alone does not guarantee a query will work.

## Image tags and checks

The workflow builds and tests the image on pull requests and publishes
`ghcr.io/celsowm/iridium-sql:latest` and a `sha-<12-character-sha>` tag
for successful builds on `main`. Git version tags are published with their
same tag name. The workflow exercises a real `Microsoft.Data.SqlClient`
`SELECT 1` against both persistent and `--memory` container modes before
publishing. Image builds currently target Linux/amd64.
