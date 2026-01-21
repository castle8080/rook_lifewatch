use actix_files::{self as fs};
use actix_web::{App, HttpServer, middleware::{Logger, Compress}, web};
use std::fs::File;
use std::io::BufReader;
use rustls::{Certificate, PrivateKey};
use rustls::ServerConfig;
use rustls_pemfile::{certs, pkcs8_private_keys};

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

    let host = "0.0.0.0";
    let http_port = 8080;

    // Check for https certificates.
    let https_port = 8443;
    let cert_path = "certs/cert.pem";
    let key_path = "certs/key.pem";
    let use_https = std::path::Path::new(cert_path).exists() && std::path::Path::new(key_path).exists();
    
    if use_https {
        info!("HTTPS certificates found. Serving HTTPS on https://{host}:{https_port}");
    } else {
        info!("No HTTPS certificates found. Serving HTTP on http://{host}:{http_port}");
    }
    
    // Setup the app.
    let app_state = app::create_app()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to create app state: {}", e)))?;

    let server = HttpServer::new(move || {
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
    });

    if use_https {
        // Load cert and key
        let cert_file = &mut BufReader::new(File::open(cert_path)?);
        let key_file = &mut BufReader::new(File::open(key_path)?);
        let cert_chain = certs(cert_file)?.into_iter().map(Certificate).collect();
        let mut keys = pkcs8_private_keys(key_file)?;
        
        if keys.is_empty() {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "No private keys found in key.pem"));
        }
        
        let config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(cert_chain, PrivateKey(keys.remove(0)))
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to create TLS config: {}", e)))?;
        
        server.bind_rustls((host, https_port), config)?.run().await
    }
    else {
        server.bind((host, http_port))?.run().await
    }
}
