use std::str::FromStr;

use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    SqlitePool
};

use crate::Sticker;

/// このモジュール内の関数の戻り値型
type DbResult<T> = Result<T, Box<dyn std::error::Error>>;

/// SQLiteのコネクションプールを作成して返す
pub(crate) async fn create_sqlite_pool(database_url: &str) -> DbResult<SqlitePool> {
    // コネクションの設定
    let connection_options = SqliteConnectOptions::from_str(database_url)?
        // DBが存在しないなら作成する
        .create_if_missing(true)
        // トランザクション使用時の性能向上のため、WALを使用する
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal);

    // 上の設定を使ってコネクションプールを作成する
    let sqlite_pool = SqlitePoolOptions::new()
        .connect_with(connection_options)
        .await?;

    Ok(sqlite_pool)
}

/// マイグレーションを行う
pub(crate) async fn migrate_database(pool: &SqlitePool) -> DbResult<()> {
    sqlx::migrate!("./db").run(pool).await?;
    Ok(())
}

pub(crate) async fn insert_sticker(pool: &SqlitePool, sticker: Sticker) -> DbResult<()> {
    let mut tx = pool.begin().await?;

    sqlx::query("INSERT INTO sticker (uuid, markdown, pos_x, pos_y, height, width) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(sticker.uuid)
        .bind(sticker.markdown)
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

pub(crate) async fn list_stickers(pool: &SqlitePool) -> DbResult<Vec<Sticker>> {
    let stickers = sqlx::query_as::<_, Sticker>("SELECT * FROM sticker WHERE archived = 0 ORDER BY uuid").fetch_all(pool).await?;

    Ok(stickers)
}
