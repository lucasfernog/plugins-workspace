![plugin-sql](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/sql/banner.png)

Interface with SQL databases through [sqlx](https://github.com/launchbadge/sqlx). It supports the `sqlite`, `mysql` and `postgres` drivers, enabled by a Cargo feature.

| Platform | Supported |
| -------- | --------- |
| Linux    | ✓         |
| Windows  | ✓         |
| macOS    | ✓         |
| Android  | ✓         |
| iOS      | ✓         |

## Install

_This plugin requires a Rust version of at least **1.77.2**_

There are three general methods of installation that we can recommend.

1. Use crates.io and npm (easiest, and requires you to trust that our publishing pipeline worked)
2. Pull sources directly from Github using git tags / revision hashes (most secure)
3. Git submodule install this repo in your tauri project and then use file protocol to ingest the source (most secure, but inconvenient to use)

Install the Core plugin by adding the following to your `Cargo.toml` file:

`src-tauri/Cargo.toml`

```toml
[dependencies]
# enable one or more of the "sqlite", "mysql" and "postgres" features
tauri-plugin-sql = { version = "2", features = ["sqlite"] }
# alternatively with Git:
tauri-plugin-sql = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v2", features = ["sqlite"] }
```

No database driver is enabled by default, so at least one feature is required. Several drivers can be enabled at the same time.

You can install the JavaScript Guest bindings using your preferred JavaScript package manager:

```sh
pnpm add @tauri-apps/plugin-sql
# or
npm add @tauri-apps/plugin-sql
# or
yarn add @tauri-apps/plugin-sql
```

## Usage

First you need to register the core plugin with Tauri:

`src-tauri/src/lib.rs`

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::new().build())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Afterwards all the plugin's APIs are available through the JavaScript guest bindings:

```javascript
import Database from '@tauri-apps/plugin-sql'

// sqlite. The path is relative to `tauri::path::BaseDirectory::AppConfig`.
const db = await Database.load('sqlite:test.db')
// mysql
const db = await Database.load('mysql://user:pass@host/database')
// postgres
const db = await Database.load('postgres://postgres:password@localhost/test')

await db.execute('INSERT INTO ...')
```

## Syntax

We use sqlx as our underlying library, adopting their query syntax:

- sqlite and postgres use the "$#" syntax when substituting query data
- mysql uses "?" when substituting query data

```javascript
// INSERT and UPDATE examples for sqlite and postgres
const result = await db.execute(
  'INSERT into todos (id, title, status) VALUES ($1, $2, $3)',
  [todos.id, todos.title, todos.status]
)

const result = await db.execute(
  'UPDATE todos SET title = $1, status = $2 WHERE id = $3',
  [todos.title, todos.status, todos.id]
)

// INSERT and UPDATE examples for mysql
const result = await db.execute(
  'INSERT into todos (id, title, status) VALUES (?, ?, ?)',
  [todos.id, todos.title, todos.status]
)

const result = await db.execute(
  'UPDATE todos SET title = ?, status = ? WHERE id = ?',
  [todos.title, todos.status, todos.id]
)
```

## Limitations

### Connection pools and transactions

Every database is backed by a connection pool, and each `execute` and `select` call can run on a different connection of that pool. Running `BEGIN`, the statements and `COMMIT` as separate calls is therefore not supported: the statements can end up on different connections, and a connection can go back to the pool with a transaction still open (on SQLite this keeps the database locked, and later writes fail with `database is locked`). For writes that must be atomic, write a Tauri command that runs a sqlx transaction on the pool (see [Using the pools from Rust](#using-the-pools-from-rust)).

### Bound values

- JavaScript numbers are bound as 64-bit floating point values. Where the database requires an integer, cast the parameter, for example `LIMIT $1::bigint` on PostgreSQL.
- `null` is bound as a JSON null, and booleans, arrays and objects are bound as JSON. On PostgreSQL they are sent as `jsonb`, so cast the parameter when the column has another type (`$1::int`, `$1::boolean`); on SQLite `true` is stored as the text `'true'`, so bind `1`/`0` for boolean columns.
- Strings are bound as text. On PostgreSQL cast them for `uuid`, `date`, `timestamptz` or `numeric` columns (`$1::uuid`).

### Returned values

- Rows are returned as objects keyed by column name.
- Integers are returned as JavaScript numbers, which cannot represent values above `Number.MAX_SAFE_INTEGER` (2^53 - 1) exactly. Cast such columns to text in the query if you need them exactly.
- SQLite returns the value as stored: `BOOLEAN` columns come back as `0`/`1`, `DATETIME` columns as the stored text or number.
- Date and time columns of MySQL and PostgreSQL are returned as strings, `BLOB`/`BYTEA` columns as arrays of bytes, `JSON`/`JSONB` columns as parsed JSON and PostgreSQL `NUMERIC` as a number (or a string when it cannot be represented as one).
- A value that cannot be decoded is returned as `null`, and `select` rejects with `unsupported datatype` for column types it does not know. Cast such columns in the query, for example to text.
- `lastInsertId` is `null` on PostgreSQL. Use `select` with a `RETURNING` clause (`INSERT INTO todos (title) VALUES ($1) RETURNING id`) instead.

## Using the pools from Rust

The connection pools are managed as Tauri state (`tauri_plugin_sql::DbInstances`), keyed by the connection string they were loaded with, so Rust code can run its own queries on a database the frontend loaded or that is preloaded:

```rust
use tauri_plugin_sql::{DbInstances, DbPool};

#[tauri::command]
async fn add_todos(
    instances: tauri::State<'_, DbInstances>,
    titles: Vec<String>,
) -> Result<(), String> {
    let instances = instances.0.read().await;
    let Some(DbPool::Sqlite(pool)) = instances.get("sqlite:mydatabase.db") else {
        return Err("database not loaded".into());
    };
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    for title in titles {
        sqlx::query("INSERT INTO todos (title) VALUES ($1)")
            .bind(title)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }
    tx.commit().await.map_err(|e| e.to_string())
}
```

This requires `sqlx` as a direct dependency of your app, with the same major version as the plugin.

## Permissions

The `sql:default` permission set allows `load`, `select` and `close`. Statements run with `execute` (`INSERT`, `UPDATE`, `CREATE TABLE`, ...) need the `sql:allow-execute` permission as well:

`src-tauri/capabilities/default.json`

```json
{
  "permissions": ["sql:default", "sql:allow-execute"]
}
```

Leaving out `sql:allow-execute` does not make the database read-only: `select` runs any statement it is given, including ones that modify data. `load` accepts any connection string, so a frontend that can call it can open any SQLite file the app can access, or connect to any MySQL/PostgreSQL server it can reach.

## Migrations

This plugin supports database migrations, allowing you to manage database schema evolution over time.

### Defining Migrations

Migrations are defined in Rust using the `Migration` struct. Each migration should include a unique version number, a description, the SQL to be executed, and the type of migration (Up or Down). Only `MigrationKind::Up` migrations are executed; `Down` migrations are currently ignored.

Example of a migration:

```rust
use tauri_plugin_sql::{Migration, MigrationKind};

let migration = Migration {
    version: 1,
    description: "create_initial_tables",
    sql: "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT);",
    kind: MigrationKind::Up,
};
```

### Adding Migrations to the Plugin Builder

Migrations are registered with the `Builder` struct provided by the plugin. Use the `add_migrations` method to add your migrations to the plugin for a specific database connection.

Example of adding migrations:

```rust
use tauri_plugin_sql::{Builder, Migration, MigrationKind};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let migrations = vec![
        // Define your migrations here
        Migration {
            version: 1,
            description: "create_initial_tables",
            sql: "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT);",
            kind: MigrationKind::Up,
        }
    ];

    tauri::Builder::default()
        .plugin(
            Builder::new()
                .add_migrations("sqlite:mydatabase.db", migrations)
                .build(),
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

The connection string passed to `add_migrations` must be exactly the one used in `preload` or `Database.load()` (for example `sqlite:mydatabase.db`), otherwise the migrations are not run.

### Applying Migrations

To apply the migrations when the plugin is initialized, add the connection string to the `tauri.conf.json` file:

```json
{
  "plugins": {
    "sql": {
      "preload": ["sqlite:mydatabase.db"]
    }
  }
}
```

Alternatively, the client side `load()` also runs the migrations for a given connection string:

```ts
import Database from '@tauri-apps/plugin-sql'
const db = await Database.load('sqlite:mydatabase.db')
```

### Migration Management

- **Applied once**: sqlx records every applied migration in the `_sqlx_migrations` table of the database, so each version runs at most once per database. Migrations therefore do not need to be safe to re-run.
- **Version Control**: Each migration must have a unique version number. Register the migrations in ascending version order.
- **Never change a shipped migration**: sqlx stores a checksum of every applied migration. Editing the SQL of a migration that already ran fails every later load with a `VersionMismatch` error, and removing one from the list fails with a `VersionMissing` error. Add a new migration with a higher version instead.
- **Transactions**: Each migration runs in its own transaction. If one fails, only that migration is rolled back; the ones before it stay applied. MySQL does not support transactional DDL, so a failed MySQL migration can leave partial changes behind.
- **Testing**: Thoroughly test migrations to ensure they work as expected and do not compromise the integrity of your database.

## Contributing

PRs accepted. Please make sure to read the Contributing Guide before making a pull request.

## Partners

<table>
  <tbody>
    <tr>
      <td align="center" valign="middle">
        <a href="https://crabnebula.dev" target="_blank">
          <img src="https://github.com/tauri-apps/plugins-workspace/raw/v2/.github/sponsors/crabnebula.svg" alt="CrabNebula" width="283">
        </a>
      </td>
    </tr>
  </tbody>
</table>

For the complete list of sponsors please visit our [website](https://tauri.app#sponsors) and [Open Collective](https://opencollective.com/tauri).

## License

Code: (c) 2015 - Present - The Tauri Programme within The Commons Conservancy.

MIT or MIT/Apache 2.0 where applicable.
