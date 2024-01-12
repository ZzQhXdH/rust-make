use ntex::web::{ServiceConfig, self};

mod device;

pub fn register(cfg: &mut ServiceConfig) {
    let scope = web::scope("/api")
        .configure(device::register)
        ;

    cfg.service(scope);
}

