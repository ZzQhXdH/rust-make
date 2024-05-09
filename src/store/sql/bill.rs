use serde::Serialize;
use sqlx::{Executor, Row, SqliteConnection};

use crate::error::SqlxErr;

use super::get_pool;

const COIN_CREATE_SQL: &'static str = r#"
    CREATE TABLE IF NOT EXISTS tb_bill (
        id INTEGER PRIMARY KEY AUTOINCREMENT, 
        device_id INTEGER NOT NULL, 
        type_mask INTEGER NOT NULL, 
        serial_number TEXT NOT NULL, 
        model TEXT NOT NULL,
        version TEXT NOT NULL,
        UNIQUE(device_id)
    )
"#;

#[derive(Debug, Serialize)]
pub struct TableBill {
    pub id: i64,
    pub device_id: i64,
    pub type_mask: u32,
    pub serial_number: String,
    pub model: String,
    pub version: String,
}

pub async fn create(conn: &mut SqliteConnection, device_id: i64) -> Result<(), SqlxErr> {
    sqlx::query(
        r#"
        INSERT INTO tb_bill 
        (device_id, type_mask, serial_number, model, version) 
        VALUES (?, ?, ?, ?, ?)
    "#,
    )
    .bind(device_id)
    .bind(0)
    .bind("未知")
    .bind("未知")
    .bind("未知")
    .execute(conn)
    .await?;


    Ok(())
}

pub async fn update(device_id: i64, model: &str, version: &str, serial_number: &str) -> Result<(), SqlxErr> {
    sqlx::query(
        r#"
        UPDATE tb_bill SET model = ?, version = ?, serial_number = ? WHERE device_id = ?
    "#,
    )
    .bind(model)
    .bind(version)
    .bind(serial_number)
    .bind(device_id)
    .execute(get_pool())
    .await?;
    Ok(())
}

pub async fn set_type_mask(device_id: i64, type_mask: u32) -> Result<(), SqlxErr> {
    sqlx::query(
        r#"
        UPDATE tb_bill SET type_mask = ? WHERE device_id = ?
    "#,
    )
    .bind(type_mask)
    .bind(device_id)
    .execute(get_pool())
    .await?;
    Ok(())
}

pub async fn get(device_id: i64) -> Result<TableBill, SqlxErr> {
    let row = sqlx::query(
        r#"
        SELECT id, device_id, type_mask, serial_number, model, version 
        FROM tb_bill WHERE device_id = ?
    "#,
    )
    .bind(device_id)
    .fetch_one(get_pool())
    .await?;

    let coin = TableBill {
        id: row.get(0),
        device_id: row.get(1),
        type_mask: row.get(2),
        serial_number: row.get(3),
        model: row.get(4),
        version: row.get(5),
    };

    Ok(coin)
}




pub async fn init() {
    get_pool().execute(COIN_CREATE_SQL).await.unwrap();
}






