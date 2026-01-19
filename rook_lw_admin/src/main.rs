use actix_files::{self as fs};
use actix_web::{App, HttpServer, middleware::{Logger, Compress}, web};

use tracing::info;

use rook_lw_admin::controllers;
use rook_lw_admin::app;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_log::LogTracer::init().expect("Failed to set logger");
    let _ = tracing_subscriber::fmt::try_init();

    // Ensure image directory exists
    let www_dir = "www";
    std::fs::create_dir_all(www_dir)?;
    info!("Serving static content from directory: {}", www_dir);

    // Ensure image directory exists
    let var_dir = "var";
    std::fs::create_dir_all(var_dir)?;
    info!("Serving var_data from directory: {}", var_dir);

    let protocol = "http";
    let host = "0.0.0.0";
    let port = 8080;
    info!("Listening on {protocol}://{host}:{port}");
    
    // Setup the app.
    let app_state = app::create_app()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to create app state: {}", e)))?;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(Logger::default())
            .wrap(Compress::default())
            .service(
                fs::Files::new("/var", var_dir)
                    .index_file("index.html")
                    .show_files_listing()
                    .files_listing_renderer(controllers::directory::sorted_listing),
            )
            .service(web::scope("")
                .configure(controllers::hello::register)
                .configure(controllers::home::register)
                .configure(controllers::image::register)
                .configure(controllers::process::register)
                .service(
                    fs::Files::new("/", www_dir)
                        .index_file("index.html")
                        .show_files_listing()
                        .files_listing_renderer(controllers::directory::sorted_listing),
                )
            )
    })
    .bind((host, port))?
    .run()
    .await
}
