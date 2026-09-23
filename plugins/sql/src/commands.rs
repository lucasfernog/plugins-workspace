// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use indexmap::IndexMap;
use serde_json::Value as JsonValue;
use sqlx::migrate::Migrator;
use tauri::{command, AppHandle, Runtime, State};

use crate::{DbInstances, DbPool, Error, LastInsertId, Migrations};

#[command]
pub(crate) async fn load<R: Runtime>(
    app: AppHandle<R>,
    db_instances: State<'_, DbInstances>,
    migrations: State<'_, Migrations>,
    db: String,
) -> Result<String, crate::Error> {
    // reuse the pool of a database that is already loaded instead of opening
    // a second one and dropping the first while it may still be in use
    if is_open(&db_instances, &db).await {
        return Ok(db);
    }

    let pool = DbPool::connect(&db, &app).await?;

    if let Some(migrations) = migrations.0.lock().await.remove(&db) {
        let migrator = Migrator::new(migrations).await?;
        pool.migrate(&migrator).await?;
    }

    let mut instances = db_instances.0.write().await;
    if instances.get(&db).is_some_and(|pool| !pool.is_closed()) {
        // loaded concurrently while this pool was connecting
        drop(instances);
        pool.close().await;
    } else {
        instances.insert(db.clone(), pool);
    }

    Ok(db)
}

async fn is_open(db_instances: &DbInstances, db: &str) -> bool {
    db_instances
        .0
        .read()
        .await
        .get(db)
        .is_some_and(|pool| !pool.is_closed())
}

/// Allows the database connection(s) to be closed; if no database
/// name is passed in then _all_ database connection pools will be
/// shut down.
#[command]
pub(crate) async fn close(
    db_instances: State<'_, DbInstances>,
    db: Option<String>,
) -> Result<bool, crate::Error> {
    // clone the pools so the lock is not held while waiting for in-flight
    // queries to finish, which would block `load` in the meantime
    let pools: Vec<DbPool> = {
        let instances = db_instances.0.read().await;
        if let Some(db) = db {
            vec![instances
                .get(&db)
                .cloned()
                .ok_or(Error::DatabaseNotLoaded(db))?]
        } else {
            instances.values().cloned().collect()
        }
    };

    for pool in pools {
        pool.close().await;
    }

    Ok(true)
}

/// Execute a command against the database
#[command]
pub(crate) async fn execute(
    db_instances: State<'_, DbInstances>,
    db: String,
    query: String,
    values: Vec<JsonValue>,
) -> Result<(u64, LastInsertId), crate::Error> {
    let instances = db_instances.0.read().await;

    let db = instances.get(&db).ok_or(Error::DatabaseNotLoaded(db))?;
    db.execute(query, values).await
}

#[command]
pub(crate) async fn select(
    db_instances: State<'_, DbInstances>,
    db: String,
    query: String,
    values: Vec<JsonValue>,
) -> Result<Vec<IndexMap<String, JsonValue>>, crate::Error> {
    let instances = db_instances.0.read().await;

    let db = instances.get(&db).ok_or(Error::DatabaseNotLoaded(db))?;
    db.select(query, values).await
}
