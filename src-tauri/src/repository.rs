use std::str::FromStr;

use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    SqlitePool, Row
};

use crate::Sticker;

type DbResult<T> = Result<T, Box<dyn std::error::Error>>;

pub(crate) async fn create_sqlite_pool(database_url: &str) -> DbResult<SqlitePool> {
    let connection_options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal);

    let sqlite_pool = SqlitePoolOptions::new()
        .connect_with(connection_options)
        .await?;

    Ok(sqlite_pool)
}

pub(crate) async fn migrate_database(pool: &SqlitePool) -> DbResult<()> {
    sqlx::migrate!("./db").run(pool).await?;
    Ok(())
}

pub(crate) async fn insert_sticker(pool: &SqlitePool, sticker: Sticker) -> DbResult<()> {
    let mut tx = pool.begin().await?;

    sqlx::query("INSERT INTO sticker (uuid, markdown, color, pos_x, pos_y, height, width) VALUES (?, ?, ?, ?, ?, ?, ?)")
        .bind(sticker.uuid)
        .bind(sticker.markdown)
        .bind(sticker.color)
        .bind(sticker.pos_x)
        .bind(sticker.pos_y)
        .bind(sticker.height)
        .bind(sticker.width)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(())
}

pub(crate) async fn update_sticker_markdown(pool: &SqlitePool, uuid: &str, markdown: &str) -> DbResult<()> {
    let mut tx = pool.begin().await?;

    sqlx::query("UPDATE sticker SET markdown=? WHERE uuid=?")
        .bind(markdown)
        .bind(uuid)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(())
}

pub(crate) async fn update_sticker_color(pool: &SqlitePool, uuid: &str, color: &str) -> DbResult<()> {
    let mut tx = pool.begin().await?;

    sqlx::query("UPDATE sticker SET color=? WHERE uuid=?")
        .bind(color)
        .bind(uuid)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(())
}

pub(crate) async fn update_sticker_position(pool: &SqlitePool, uuid: &str, pos_x: i32, pos_y: i32) -> DbResult<()> {
    let mut tx = pool.begin().await?;

    sqlx::query("UPDATE sticker SET pos_x=?, pos_y=? WHERE uuid=?")
        .bind(pos_x)
        .bind(pos_y)
        .bind(uuid)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(())
}

pub(crate) async fn update_sticker_size(pool: &SqlitePool, uuid: &str, width: u32, height: u32) -> DbResult<()> {
    let mut tx = pool.begin().await?;

    sqlx::query("UPDATE sticker SET height=?, width=? WHERE uuid=?")
        .bind(height)
        .bind(width)
        .bind(uuid)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(())
}

pub(crate) async fn toggle_sticker_pinned(pool: &SqlitePool, uuid: &str) -> DbResult<bool> {
    let rows = sqlx::query("UPDATE sticker SET pinned=(CASE WHEN pinned=1 THEN 0 ELSE 1 END) WHERE uuid=? RETURNING pinned")
        .bind(uuid)
        .fetch_all(pool)
        .await?;

    for row in rows {
        let pinned: i8 = row.try_get("pinned")?;
        return Ok(pinned != 0)
    }

    Ok(false)
}

pub(crate) async fn remove_sticker(pool: &SqlitePool, uuid: &str) -> DbResult<()> {
    let mut tx = pool.begin().await?;

    sqlx::query("UPDATE sticker SET archived=? WHERE uuid=?")
        .bind(1)
        .bind(uuid)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(())
}

pub(crate) async fn delete_stickers(pool: &SqlitePool, stickers: &Vec<Sticker>) -> DbResult<()> {
    let mut tx = pool.begin().await?;

    for sticker in stickers {
        sqlx::query("DELETE FROM sticker WHERE uuid=?")
            .bind(&sticker.uuid)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;

    Ok(())
}

pub(crate) async fn recover_stickers(pool: &SqlitePool, stickers: &Vec<Sticker>) -> DbResult<()> {
    let mut tx = pool.begin().await?;

    for sticker in stickers {
        sqlx::query("UPDATE sticker SET archived=? WHERE uuid=?")
            .bind(0)
            .bind(&sticker.uuid)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;

    Ok(())
}

pub(crate) async fn list_stickers(pool: &SqlitePool) -> DbResult<Vec<Sticker>> {
    let stickers = sqlx::query_as::<_, Sticker>("SELECT * FROM sticker WHERE archived = 0 ORDER BY uuid").fetch_all(pool).await?;

    Ok(stickers)
}

pub(crate) async fn list_archived_stickers(pool: &SqlitePool) -> DbResult<Vec<Sticker>> {
    let stickers = sqlx::query_as::<_, Sticker>("SELECT * FROM sticker WHERE archived = 1 ORDER BY uuid").fetch_all(pool).await?;

    Ok(stickers)
}

pub(crate) async fn get_sticker(pool: &SqlitePool, uuid: &str) -> DbResult<Sticker> {
    let sticker = sqlx::query_as::<_, Sticker>("SELECT * FROM sticker WHERE uuid = ?").bind(uuid).fetch_one(pool).await?;

    Ok(sticker)
}
