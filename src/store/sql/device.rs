use serde::Serialize;
use sqlx::{Executor, Row, SqliteConnection};

use crate::{
    error::SqlxErr,
    utils::{current_timestamp, Array},
};

use super::{bill, coin, get_pool};

const CREATE_SQL: &'static str = r#"
    CREATE TABLE IF NOT EXISTS tb_device (
        id INTEGER PRIMARY KEY AUTOINCREMENT, 
        name TEXT NOT NULL, 
        create_timestamp INTEGER NOT NULL, 
        mac_addr TEXT NOT NULL,
        mcu_version TEXT NOT NULL, 
        app_version TEXT NOT NULL,
        address TEXT NOT NULL, 
        UNIQUE(mac_addr)
    )
"#;

pub async fn init() {
    get_pool().execute(CREATE_SQL).await.unwrap();
}

#[derive(Debug, Serialize)]
pub struct TableDevice {
    pub id: i64,
    pub name: String,
    pub mac_addr: String,
    pub create_timestamp: i64,
    pub mcu_version: String,
    pub app_version: String,
    pub address: String,
}

async fn create(
    conn: &mut SqliteConnection,
    mac_addr: &str,
    name: &str,
    address: &str,
) -> Result<i64, SqlxErr> {
    let ret = sqlx::query(
        r#"
        INSERT INTO tb_device 
        (name, mac_addr, create_timestamp, mcu_version, app_version, address) 
        VALUES(?, ?, ?, ?, ?, ?)
    "#,
    )
    .bind(name)
    .bind(mac_addr)
    .bind(current_timestamp())
    .bind("未知")
    .bind("未知")
    .bind(address)
    .execute(&mut *conn)
    .await?;

    let id = ret.last_insert_rowid();

    coin::create(&mut *conn, id).await?;
    bill::create(&mut *conn, id).await?;

    Ok(id)
}

pub async fn create_by(mac_addr: &str, name: &str, address: &str) -> Result<i64, SqlxErr> {
    let mut tx = get_pool().begin().await?;
    let id = create(&mut tx, mac_addr, name, address).await?;
    tx.commit().await?;
    Ok(id)
}

pub async fn create_if_not_exists(mac_addr: &str) -> Result<i64, SqlxErr> {
    let mut tx = get_pool().begin().await?;

    let ret = sqlx::query("SELECT id FROM tb_device WHERE mac_addr = ? LIMIT 1")
        .bind(mac_addr)
        .fetch_one(&mut *tx)
        .await;

    match ret {
        Ok(ret) => {
            let id = ret.get(0);
            tx.commit().await?;
            return Ok(id);
        }
        Err(sqlx::Error::RowNotFound) => {}
        Err(e) => {
            return Err(e);
        }
    };

    let id = create(&mut *tx, mac_addr, "未命名设备", "未知地址").await?;

    tx.commit().await?;

    Ok(id)
}

pub async fn get(id: i64) -> Result<TableDevice, SqlxErr> {
    let row = sqlx::query(
        r#"
        SELECT 
        id, name, create_timestamp, mac_addr, mcu_version, app_version, address
        FROM tb_device WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_one(get_pool())
    .await?;

    let device = TableDevice {
        id: row.get(0),
        name: row.get(1),
        create_timestamp: row.get(2),
        mac_addr: row.get(3),
        mcu_version: row.get(4),
        app_version: row.get(5),
        address: row.get(6),
    };

    Ok(device)
}

pub async fn select() -> Result<Array<TableDevice>, SqlxErr> {
    let rows = sqlx::query(
        r#"
        SELECT 
        id, name, create_timestamp, mac_addr, mcu_version, app_version, address
        FROM tb_device
        "#,
    )
    .fetch_all(get_pool())
    .await?;

    let vec: Vec<TableDevice> = rows
        .iter()
        .map(|row| TableDevice {
            id: row.get(0),
            name: row.get(1),
            create_timestamp: row.get(2),
            mac_addr: row.get(3),
            mcu_version: row.get(4),
            app_version: row.get(5),
            address: row.get(6),
        })
        .collect();

    Ok(vec.into_boxed_slice())
}

pub async fn delete(id: i64) -> Result<(), SqlxErr> {
    sqlx::query("DELETE FROM tb_device WHERE id = ?")
        .bind(id)
        .execute(get_pool())
        .await?;

    Ok(())
}

pub async fn set_mac_addr(id: i64, mac_addr: Option<&str>) -> Result<(), SqlxErr> {
    if let Some(val) = mac_addr {
        sqlx::query("UPDATE ta_device SET mac_addr = ? WHERE id = ?")
            .bind(val)
            .bind(id)
            .execute(get_pool())
            .await?;
    }
    Ok(())
}

pub async fn set_name(id: i64, name: Option<&str>) -> Result<(), SqlxErr> {
    if let Some(val) = name {
        sqlx::query("UPDATE ta_device SET name = ? WHERE id = ?")
            .bind(val)
            .bind(id)
            .execute(get_pool())
            .await?;
    }
    Ok(())
}

pub async fn set_address(id: i64, address: Option<&str>) -> Result<(), SqlxErr> {
    if let Some(val) = address {
        sqlx::query("UPDATE ta_device SET address = ? WHERE id = ?")
            .bind(val)
            .bind(id)
            .execute(get_pool())
            .await?;
    }
    Ok(())
}

pub async fn set_muc_version(id: i64, mcu_version: &str) -> Result<(), SqlxErr> {
    sqlx::query(r#"UPDATE SET tb_device mcu_version = ? WHERE id = ?"#)
        .bind(mcu_version)
        .bind(id)
        .execute(get_pool())
        .await?;
    Ok(())
}

pub async fn set_app_version(id: i64, app_version: &str) -> Result<(), SqlxErr> {
    sqlx::query("UPDATE SET tb_device app_version = ? WHERE id = ?")
        .bind(app_version)
        .bind(id)
        .execute(get_pool())
        .await?;
    Ok(())
}
