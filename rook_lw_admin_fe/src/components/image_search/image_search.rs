use leptos::prelude::*;

use rook_lw_models::image::{ImageInfoSearchOptions, ImageInfo};
use gloo_net::http::Request;
use serde::Deserialize;

#[component]
pub fn ImageSearch() -> impl IntoView {

    //et image_infos = use_state(|| None);

    /*
    {
        let image_infos = image_infos.clone();
        let query = ImageInfoSearchOptions {
            start_date: None,
            end_date: None,
        };
        let url = format!("/api/image_info?{}", serde_qs::to_string(&query).unwrap());
        let resp: Vec<ImageInfo> = Request::get(&url)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        image_infos.set(Some(resp));
    }
    */

    view! {
        <h1>Image Search</h1>
        }
}
