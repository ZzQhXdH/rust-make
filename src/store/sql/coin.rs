use serde::{Serialize, Deserialize};
use sqlx::{Executor, SqliteConnection, Row};

use crate::{error::SqlxErr, utils::Array};

use super::get_pool;


const COIN_CREATE_SQL: &'static str = r#"
    CREATE TABLE IF NOT EXISTS tb_coin (
        id INTEGER PRIMARY KEY AUTOINCREMENT, 
        device_id INTEGER NOT NULL, 
        type_mask INTEGER NOT NULL, 
        serial_number TEXT NOT NULL, 
        model TEXT NOT NULL,
        version TEXT NOT NULL,
        UNIQUE(device_id)
    )
"#;

const COIN_INFO_CREATE_SQL: &'static str = r#"
    CREATE TABLE IF NOT EXISTS tb_coin_info (
        id INTEGER PRIMARY KEY AUTOINCREMENT, 
        device_id INTEGER NOT NULL, 
        coin_type INTEGER NOT NULL, 
        coin_value INTEGER NOT NULL, 
        coin_count INTEGER NOT NULL, 
        UNIQUE(device_id, coin_type)
    )
"#;

#[derive(Debug, Serialize)]
pub struct TableCoin {
    pub id: i64,
    pub device_id: i64,
    pub type_mask: u32,
    pub serial_number: String,
    pub model: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TableCoinInfo {
    pub coin_type: u8,
    pub coin_value: u16,
    pub coin_count: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TableCoinInfos {
    pub device_id: i64,
    pub infos: Array<TableCoinInfo>,
}

async fn update_coin_info(conn: &mut SqliteConnection, device_id: i64, info: &TableCoinInfo) -> Result<(), SqlxErr> {

    sqlx::query(r#"
        INSERT OR REPLACE INTO tb_coin_info 
        (device_id, coin_type, coin_value, coin_count) 
        VALUES (?, ?, ?, ?)
    "#)
    .bind(device_id)
    .bind(info.coin_type)
    .bind(info.coin_value)
    .bind(info.coin_count)
    .execute( conn)
    .await?;

    Ok(())
}

async fn all_type(conn: &mut SqliteConnection, device_id: i64) -> Result<Array<u8>, SqlxErr> {

    let rows = sqlx::query(r#"
        SELECT coin_type FROM tb_coin_info WHERE device_id = ?
    "#)
    .bind(device_id)
    .fetch_all(&mut *conn)
    .await?;

    let vec: Vec<u8> = rows.iter().map(|row| {
        let v: u8 = row.get(0);
        v
    }).collect();

    Ok(vec.into_boxed_slice())
}

async fn delete_with_type(conn: &mut SqliteConnection, device_id: i64, coin_type: u8) -> Result<(), SqlxErr> {

    sqlx::query(r#"
        DELETE FROM tb_coin_info WHERE device_id = ? AND coin_type = ?
    "#).bind(device_id).bind(coin_type)
    .execute(conn)
    .await?;

    Ok(())
}

fn contain_tpye(infos: &[TableCoinInfo], t: u8) -> bool {
    for v in infos.iter() {
        if v.coin_type == t {
            return true;
        }
    }
    false
}

pub async fn update_info(device_id: i64, infos: &[TableCoinInfo]) -> Result<(), SqlxErr> {

    let mut tx = get_pool().begin().await?;

    let all = all_type(&mut *tx, device_id).await?;

    for info in infos {
        update_coin_info(&mut *tx, device_id, info).await?;
    }

    for t in all.into_iter() {
        let v = *t;
        if !contain_tpye(infos, v) {
            delete_with_type(&mut *tx, device_id, v).await?;
        }
    }

    tx.commit().await?;
    Ok(())
}

pub async fn set_type_mask(device_id: i64, type_mask: u32) -> Result<(), SqlxErr> {
    sqlx::query(r#"
        UPDATE tb_coin SET type_mask = ? WHERE device_id = ?
    "#)
    .bind(type_mask)
    .bind(device_id)
    .execute( get_pool() )
    .await?;
    Ok(())
}

pub async fn update(device_id: i64, model: &str, version: &str) -> Result<(), SqlxErr> {
    sqlx::query(r#"
        UPDATE tb_coin SET model = ?, version = ? WHERE device_id = ?
    "#)
    .bind(model)
    .bind(version)
    .bind(device_id)
    .execute( get_pool() )
    .await?;
    Ok(())
}

pub async fn create(conn: &mut SqliteConnection, device_id: i64) -> Result<(), SqlxErr> {

    sqlx::query(r#"
        INSERT INTO tb_coin 
        (device_id, type_mask, serial_number, model, version) 
        VALUES (?, ?, ?, ?, ?)
    "#)
    .bind(device_id)
    .bind(0)
    .bind("未知")
    .bind("未知")
    .bind("未知")
    .execute( conn )
    .await?;

    Ok(())
}

pub async fn get(device_id: i64) -> Result<TableCoin, SqlxErr> {

    let row = sqlx::query(r#"
        SELECT id, device_id, type_mask, serial_number, model, version 
        FROM tb_coin WHERE device_id = ?
    "#)
    .bind(device_id)
    .fetch_one( get_pool() )
    .await?;

    let coin = TableCoin {
        id: row.get(0),
        device_id: row.get(1),
        type_mask: row.get(2),
        serial_number: row.get(3),
        model: row.get(4),
        version: row.get(5)
    };

    Ok(coin)
}

pub async fn get_info(device_id: i64) -> Result<TableCoinInfos, SqlxErr> {

    let rows = sqlx::query(r#"
        SELECT coin_type, coin_value, coin_count 
        FROM tb_coin_info WHERE device_id = ?
    "#)
    .bind(device_id)
    .fetch_all( get_pool() )
    .await?;

    let vec: Vec<TableCoinInfo> = rows.iter().map(|row| TableCoinInfo {
        coin_type: row.get(0),
        coin_value: row.get(1),
        coin_count: row.get(2)
    }).collect();

    let info = TableCoinInfos {
        device_id,
        infos: vec.into_boxed_slice()
    };

    Ok(info)
}

pub async fn init() -> Result<(), SqlxErr> {

    get_pool().execute(COIN_CREATE_SQL).await?;
    get_pool().execute(COIN_INFO_CREATE_SQL).await?;

    Ok(())
}

