use gloo_net::http::Response;

use crate::RookLWAppResult;

pub async fn response_ok_or_not_found(response: Response) -> RookLWAppResult<Response> {
    _response_ok(response, |r| r.ok() || r.status() == 404).await
}

pub async fn response_ok(response: Response) -> RookLWAppResult<Response> {
    _response_ok(response, |r| r.ok()).await
}

async fn _response_ok<F: Fn(&Response) -> bool>(response: Response, is_ok: F) -> RookLWAppResult<Response> {
    if is_ok(&response) {
        Ok(response)
    }
    else if response.status() >= 400 && response.status() <= 499 {
        Err(crate::RookLWAppError::Request(format!(
            "HTTP {}:{} - {}",
            response.status(),
            response.status_text(),
            response.text().await.unwrap_or_default()
        )))
    }
    else if response.status() >= 500 && response.status() < 599 {
        Err(crate::RookLWAppError::Server(format!(
            "HTTP {}:{} - {}",
            response.status(),
            response.status_text(),
            response.text().await.unwrap_or_default()
        )))
    }
    else {
        Err(crate::RookLWAppError::Io(format!(
            "Unexpected HTTP {}:{} - {}",
            response.status(),
            response.status_text(),
            response.text().await.unwrap_or_default()
        )))
    }
}


