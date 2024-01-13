use crate::{config::SQLITE_PATH, error::SqlxErr};
use sqlx::SqlitePool;
use std::mem::MaybeUninit;

static mut POOL: MaybeUninit<SqlitePool> = MaybeUninit::uninit();

pub mod bill;
pub mod coin;
pub mod device;

pub async fn sql_init() -> Result<(), SqlxErr> {
    let pool = SqlitePool::connect(SQLITE_PATH).await?;

    unsafe {
        POOL.write(pool);
    }

    device::init().await;
    coin::init().await;
    bill::init().await;

    Ok(())
}

pub fn get_pool() -> &'static SqlitePool {
    unsafe { POOL.assume_init_ref() }
}
