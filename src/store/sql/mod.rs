use std::mem::MaybeUninit;
use sqlx::SqlitePool;
use crate::{error::SqlxErr, config::SQLITE_PATH};

static mut POOL: MaybeUninit<SqlitePool> = MaybeUninit::uninit();

mod device;
mod coin;
mod bill;

pub async fn sql_init() -> Result<(), SqlxErr> {

    let pool = SqlitePool::connect(SQLITE_PATH).await?;

    unsafe {
        POOL.write(pool);
    }

    device::init().await?;
    coin::init().await?;
    bill::init().await?;

    Ok(())
}

pub fn get_pool() -> &'static SqlitePool {
    unsafe {
        POOL.assume_init_ref()
    }
}



