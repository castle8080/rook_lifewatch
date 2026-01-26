use actix_web::{Responder, HttpResponse, web};
use actix_web::web::ServiceConfig;
use rook_lw_models::Status;
use std::process::Command;
use tokio::task::spawn_blocking;

use crate::{RookLWAdminError, RookLWAdminResult};

async fn run_shutdown_command() -> RookLWAdminResult<String> {

    // The shutdown command needs to be added to sudeo NOPASSD to work:
    // e.g. for user 'rook':
    // rook ALL=(ALL) NOPASSWD: /usr/bin/systemctl poweroff
    #[cfg(unix)]
    let command: Vec<String> = vec![
        "sudo".into(),
        "-n".into(),
        "/usr/bin/systemctl".into(),
        "poweroff".into(),
    ];

    #[cfg(windows)]
    let command: Vec<String> = vec![
        "shutdown".into(),
        "/s".into(),
        "/t".into(),
        "0".into(),
    ];
    
    let mut cmd = Command::new(&command[0]);
    cmd.args(&command[1..]);

    let output = spawn_blocking(move || cmd.output()).await??;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let result = "Ran shutdown command:\n".to_string() + &stdout;
        Ok(result)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(RookLWAdminError::Other(format!("Shutdown command failed: {}", stderr)))
    }
}

async fn server_shutdown() -> Result<impl Responder, RookLWAdminError> {
    //let process_list: Vec<ProcessInfo> = spawn_blocking(move || DaemonService::list_all()).await??;
    let result = run_shutdown_command().await?;
    return Ok(HttpResponse::Ok().json(Status { message: result, ..Default::default() }));
}

pub fn register(sc: &mut ServiceConfig) {
    sc.route("/api/server/shutdown", web::post().to(server_shutdown));
}