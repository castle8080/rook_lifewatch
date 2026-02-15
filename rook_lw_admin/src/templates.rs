pub mod home;
pub mod directory;

use actix_web::{HttpResponse, Responder, http::header::ContentType, HttpRequest};
use maud::Markup;

pub struct HtmlTemplate(pub Markup);

impl Responder for HtmlTemplate {
	type Body = actix_web::body::BoxBody;
	fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
		HttpResponse::Ok()
			.content_type(ContentType::html())
			.insert_header(("Cache-Control", "no-store, no-cache, must-revalidate, max-age=0"))
			.insert_header(("Pragma", "no-cache"))
			.insert_header(("Expires", "0"))
			.body(self.0.into_string())
	}
}
