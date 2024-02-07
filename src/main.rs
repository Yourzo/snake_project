mod database;
mod models;
mod api;

use actix_web::{HttpServer, App, };
use actix_web::web::{Data};
use actix_web::middleware::Logger;
use crate::api::api::{
    get_all_allowed, get_all_allowed_files_info,
    info_to_upload, login_user, set_user, upload
};
use crate::database::Database;

#[actix_web::main]
async fn  main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();
    let db = Database::init()
        .await
        .expect("ERROR CONNECTING TO DATABASE");
    let db_data = Data::new(db);

    let res = db_data
        .put_master_key("-1".to_string())
        .await;

    if res == None {
        println!("master key might be already in database")
    }

    HttpServer::new(move || {
        let logger = Logger::default();
        App::new()
            .wrap(logger)
            .service(login_user)
            .service(upload)
            .service(get_all_allowed_files_info)
            .service(get_all_allowed)
            .service(set_user)
            .service(info_to_upload)
            .app_data(db_data.clone())
    })
        .workers(2)
        .bind(("127.0.0.1",8080))?
        .run()
        .await
}
