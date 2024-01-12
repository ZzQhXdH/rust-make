use serde::Serialize;
use sqlx::{Executor, Row, SqliteConnection};

use crate::{error::SqlxErr, utils::{current_timestamp, Array}};

use super::{get_pool, coin, bill};

const CREATE_SQL: &'static str = r#"
    CREATE TABLE IF NOT EXISTS tb_device {
        id INTEGER PRIMARY KEY AUTOINCREMENT, 
        name TEXT NOT NULL, 
        create_timestamp INTEGER NOT NULL, 
        mac_addr TEXT NOT NULL,
        mcu_version TEXT NOT NULL, 
        app_version TEXT NOT NULL,
        UNIQUE(mac_addr)
    }
"#;

pub async fn init() -> Result<(), SqlxErr> {

    get_pool().execute(CREATE_SQL).await?;

    Ok(())
}

#[derive(Debug, Serialize)]
pub struct TableDevice {
    pub id: i64,
    pub name: String,
    pub mac_addr: String,
    pub create_timestamp: i64,
    pub mcu_version: String,
    pub app_version: String,
}

pub async fn create(conn: &mut SqliteConnection, mac_addr: &str, name: &str) -> Result<i64, SqlxErr> {

    let ret = sqlx::query(r#"
        INSERT INTB tb_device 
        (name, mac_addr, create_timestamp, mcu_version, app_version) 
        VALUES(?, ?, ?, ?, ?)
    "#)
    .bind(name)
    .bind(mac_addr)
    .bind( current_timestamp() )
    .bind("未知")
    .bind("未知")
    .execute( &mut *conn )
    .await?;

    let id = ret.last_insert_rowid();

    coin::create(&mut *conn, id).await?;
    bill::create(&mut *conn, id).await?;

    Ok( id )
}


pub async fn create_if_not_exists(mac_addr: &str) -> Result<i64, SqlxErr> {
    
    let mut tx = get_pool().begin().await?;

    let ret = sqlx::query("SELECT id FROM tb_device WHERE mac_addr = ? LIMIT 1")
        .bind( mac_addr )
        .fetch_one( &mut *tx )
        .await;

    match ret {

        Ok(ret) => { 
            let id = ret.get(0);
            tx.commit().await?;
            return Ok(id) 
        },
        Err(sqlx::Error::RowNotFound) => {},
        Err(e) => { return Err(e); }
    };

    let id = create(&mut *tx, mac_addr, "未命名设备").await?;

    tx.commit().await?;

    Ok( id )
}

pub async fn get(id: i64) -> Result<TableDevice, SqlxErr> {

    let row = sqlx::query(r#"
        SELECT 
        id, name, create_timestamp, mac_addr, mcu_version, app_version 
        FROM tb_device WHERE id = ?
        "#)
        .bind(id)
        .fetch_one( get_pool() )
        .await?;

    let device = TableDevice {
        id: row.get(0),
        name: row.get(1),
        create_timestamp: row.get(2),
        mac_addr: row.get(3),
        mcu_version: row.get(4),
        app_version: row.get(5)
    };

    Ok(device)
}

pub async fn select() -> Result<Array<TableDevice>, SqlxErr> {

    let rows = sqlx::query(r#"
        SELECT 
        id, name, create_timestamp, mac_addr, mcu_version, app_version
        FROM tb_device
        "#)
        .fetch_all( get_pool() )
        .await?;

    let vec: Vec<TableDevice> = rows.iter().map(|row| TableDevice {
        id: row.get(0),
        name: row.get(1),
        create_timestamp: row.get(2),
        mac_addr: row.get(3),
        mcu_version: row.get(4),
        app_version: row.get(5)
    }).collect();

    Ok(vec.into_boxed_slice())
}

pub async fn delete(id: i64) -> Result<(), SqlxErr> {

    sqlx::query("DELETE FROM tb_device WHERE id = ?")
        .bind(id)
        .execute( get_pool() )
        .await?;

    Ok(())
}


pub async fn update(id: i64, mac_addr: Option<&str>, name: Option<&str>) -> Result<(), SqlxErr> {

    if let Some(mac) = mac_addr {
        if let Some(n) = name {

            sqlx::query("UPDATE tb_device SET mac_addr = ?, name = ? WHERE id = ?")
            .bind(mac)
            .bind(n)
            .bind(id)
            .execute( get_pool() )
            .await?;
        } else {
            sqlx::query("UPDATE tb_device SET mac_addr = ? WHERE id = ?")
                .bind(mac)
                .bind(id)
                .execute(get_pool())
                .await?;
        }
    } else {
        if let Some(n) = name {
            sqlx::query("UPDATE tb_device SET name = ? WHERE id = ?")
                .bind(n)
                .bind(id)
                .execute( get_pool() )
                .await?;
        }
    }

    Ok(())
}


pub async fn set_version(mcu_version: &str, app_version: &str) -> Result<(), SqlxErr> {

    sqlx::query(r#"UPDATE SET tb_device mcu_version = ?, app_version = ?"#)
        .bind(mcu_version)
        .bind(app_version)
        .execute(get_pool())
        .await?;
    Ok(())
}

