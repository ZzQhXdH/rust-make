use tokio::fs;

use crate::error::IoErr;



pub const WEB_ADDR: &'static str = "0.0.0.0:3656";

pub const SQLITE_PATH: &'static str = "sqlite://./data/data.db?mode=rwc";
pub const HTML_PATH: &'static str = "./data/html";

pub async fn init() -> Result<(), IoErr> {
    fs::create_dir_all(HTML_PATH).await?;
    Ok(())
}

