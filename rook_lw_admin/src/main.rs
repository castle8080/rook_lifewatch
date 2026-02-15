use actix_files::{self as fs};
use actix_web::{App, HttpServer, middleware::{Logger, Compress, from_fn}, web};
use std::fs::File;
use std::io::BufReader;
use rustls::{Certificate, PrivateKey};
use rustls::ServerConfig;
use rustls_pemfile::{certs, pkcs8_private_keys};

use clap::Parser;
use tracing::info;

use rook_lw_admin::RookLWAdminResult;
use rook_lw_admin::controllers;
use rook_lw_admin::app;
use rook_lw_admin::middleware::{signature_validation, SignatureValidationConfig};

/// Command line options
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Protocol to use: http or https
    #[arg(long, default_value = "http")]
    protocol: String,

    /// Port to listen on
    #[arg(long, default_value_t = 8080)]
    port: u16,

    /// Directory to serve static content from
    #[arg(long, default_value = "www")]
    www_dir: String,

    /// Directory to serve var data from
    #[arg(long, default_value = "var")]
    var_dir: String,

    /// Directory the daemon home is
    #[arg(long, default_value = ".")]
    app_dir: String,
}

async fn run() -> RookLWAdminResult<()> {
    tracing_log::LogTracer::init().expect("Failed to set logger");
    let _ = tracing_subscriber::fmt::try_init();

    // Parse command line arguments using clap
    let cli = Cli::parse();
    let protocol = cli.protocol.to_lowercase();
    let port = cli.port;
    if protocol != "http" && protocol != "https" {
        eprintln!("Invalid value for --protocol: {}. Use 'http' or 'https'", protocol);
        std::process::exit(1);
    }
    info!("Protocol: {}, Port: {}", protocol, port);

    // Ensure image directory exists
    let www_dir = cli.www_dir.clone();
    std::fs::create_dir_all(&www_dir)?;
    info!("Serving static content from directory: {}", &www_dir);

    // Ensure image directory exists
    let var_dir = cli.var_dir.clone();
    std::fs::create_dir_all(&var_dir)?;
    info!("Serving var_data from directory: {}", &var_dir);

    // List on all interfaces.
    let host = "0.0.0.0";

    // Check for https certificates.
    let cert_path = "certs/cert.pem";
    let key_path = "certs/key.pem";

    // Setup the app.
    let app_state = app::create_app(
        &var_dir, 
        format!("{}/admin", &www_dir).as_str(),
        &cli.app_dir,
    )?;

    // Configure signature validation for /api/* paths
    let sig_config = SignatureValidationConfig::new(vec!["/api/*".to_string()]);

    let server = HttpServer::new(move || {
        let cfg = sig_config.clone();
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(Logger::default())
            .wrap(Compress::default())
            .wrap(from_fn(move |req, next| {
                signature_validation(cfg.clone(), req, next)
            }))
            .service(
                fs::Files::new("/var", &var_dir)
                    .index_file("index.html")
                    .show_files_listing()
                    .files_listing_renderer(controllers::directory::sorted_listing),
            )
            .service(web::scope("")
                .configure(controllers::admin::register)
                .configure(controllers::daemon::register)
                .configure(controllers::home::register)
                .configure(controllers::image::register)
                .configure(controllers::process::register)
                .configure(controllers::server::register)
                .service(
                    fs::Files::new("/", &www_dir)
                        .index_file("index.html")
                        .show_files_listing()
                        .files_listing_renderer(controllers::directory::sorted_listing),
                )
            )
    });

    if protocol == "https" {
        // Load cert and key
        let cert_file = &mut BufReader::new(File::open(cert_path)?);
        let key_file = &mut BufReader::new(File::open(key_path)?);
        let cert_chain = certs(cert_file)?.into_iter().map(Certificate).collect();
        let mut keys = pkcs8_private_keys(key_file)?;
        
        if keys.is_empty() {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "No private keys found in key.pem"))?;
        }
        
        let config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(cert_chain, PrivateKey(keys.remove(0)))
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to create TLS config: {}", e)))?;
        
        server.bind_rustls((host, port), config)?.run().await?;
    }
    else {
        server.bind((host, port))?.run().await?;
    }

    Ok(())
}

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    run().await.map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, format!("Application error: {}", e))
    })?;
    Ok(())
}
