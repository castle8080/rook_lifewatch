use leptos::*;
use leptos::prelude::*;

use gloo_net::http::Request;
use serde_qs;

use rook_lw_models::image::{ImageInfoSearchOptions, ImageInfo};
use crate::RookLWAppResult;

async fn fetch_image_infos() -> RookLWAppResult<Vec<ImageInfo>> {
    let query = ImageInfoSearchOptions {
        start_date: None,
        end_date: None,
    };
    let url = format!("/api/image_info?{}", serde_qs::to_string(&query)?);

    let mut images = Request::get(url.as_str())
        .send()
        .await?
        .json::<Vec<ImageInfo>>()
        .await?;

    images.sort_by(|a, b| b.capture_timestamp.cmp(&a.capture_timestamp));
    images.truncate(100);

    Ok(images)
}

const IMAGE_INFO_STYLE: &str = r#"
    .image-search-component table, .image-search-component th, .image-search-component td {
        border: 1px solid black;
        border-collapse: collapse;
    }
    .image-search-component th, .image-search-component td {
        padding: 5px;
        text-align: left;
    }
"#;

#[component]
pub fn ImageInfo(image_info: ImageInfo) -> impl IntoView {
    view! {
        <tr>
            <td>
                <a href={ format!("/var/images/{}", image_info.image_path) } target="_blank">
                    { image_info.capture_timestamp.to_string() }
                </a>
            </td>
            <td>{ image_info.motion_score.score }</td>
            <td>
            { match &image_info.detections {
                None => view! { "No Detections" }.into_any(),
                Some(detections) => view! {
                    { detections.iter().map(|d| {
                        view! {
                            <span>{ d.class_name.as_str() } ": " { d.confidence }</span><br/>
                        }
                    }).collect_view() }
                }.into_any(),
            } }
            </td>
        </tr>
    }
}

#[component]
pub fn ImageSearch() -> impl IntoView {
    let image_info_data = LocalResource::new(move || fetch_image_infos());

    view! {
        <style>{ IMAGE_INFO_STYLE }</style>
        <div class="image-search-component">
            <h1>"Images"</h1>
            { move || {
                match image_info_data.get() {
                    None => view! {
                        <div>"Loading..."</div>
                    }.into_any(),
                    Some(Err(e)) => view! {
                        <div>"Error loading images: " { e.to_string() }</div>
                    }.into_any(),
                    Some(Ok(image_infos)) => view! {
                        <table>
                        <thead>
                            <tr>
                                <th>"Image Taken"</th>
                                <th>"Motion Score"</th>
                                <th>"Detections"</th>
                            </tr>

                        </thead>
                        <tbody>
                            { image_infos.into_iter().map(|image_info| {
                                view! { <ImageInfo image_info=image_info /> }
                            }).collect_view() }
                        </tbody>
                        </table>
                    }.into_any(),
                }
            }}
        </div>
    }
}