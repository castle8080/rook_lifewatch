use leptos::*;
use leptos::prelude::*;
use leptos::task::spawn_local;

use rook_lw_models::image::{ImageInfoSearchOptions, ImageInfo, Detection};

use crate::components::ErrorDisplay;
use crate::services::ImageInfoService;

#[component]
fn ImageInfoDetection(detection: Detection) -> impl IntoView {
    view! {
        <span>{ detection.class_name } ": " { detection.confidence }</span><br/>
    }
}

#[component]
pub fn ImageInfo(image_info: ImageInfo) -> impl IntoView {
    view! {
        <tr>
            <td>
                <a href={ format!("image_display/{}", image_info.image_id) } >
                //<a href={ format!("/var/images/{}", image_info.image_path) } target="rook_lw_image_view">
                    { image_info.capture_timestamp.to_string() }
                </a>
            </td>
            <td>{ image_info.motion_score.score }</td>
            <td>
                { match &image_info.detections {
                    None => view! { "No Detections" }.into_any(),
                    Some(detections) =>
                        detections
                            .iter()
                            .map(|d| view! { <ImageInfoDetection detection=d.clone() /> })
                            .collect_view()
                            .into_any()
                } }
            </td>
        </tr>
    }
}

#[component]
fn ImageInfos(image_infos: ReadSignal<Option<Vec<ImageInfo>>>) -> impl IntoView {
    view! {
        <table class="table is-striped is-hoverable is-fullwidth">
            <thead>
                <tr>
                    <th>"Image Taken"</th>
                    <th>"Motion Score"</th>
                    <th>"Detections"</th>
                </tr>
            </thead>
            <tbody>
                <For
                    each=move || image_infos.get().unwrap_or_else(Vec::new)
                    key=|image_info| image_info.image_id.clone()
                    let (image_info)
                >
                    <ImageInfo image_info=image_info/>
                </For>
            </tbody>
        </table>
    }
}

#[component]
pub fn ImageSearch() -> impl IntoView {
    let (error, set_error) = signal(None::<String>);
    let (loading, set_loading) = signal(false);
    let (image_infos, set_image_infos) = signal(None::<Vec<ImageInfo>>);

    let image_info_service = match use_context::<ImageInfoService>() {
        Some(s) => s,
        None => return view! {
            <div>Error</div>
        }.into_any()
    };

    Effect::new(move |_| {
        let set_error = set_error.clone();
        let set_loading = set_loading.clone();
        let image_info_service = image_info_service.clone();
        let set_image_infos = set_image_infos.clone();

        set_error.set(None);
        set_loading.set(true);

        spawn_local(async move {
            let search_options = ImageInfoSearchOptions::default();
            match image_info_service.search(&search_options).await {
                Err(e) => {
                    set_loading.set(false);
                    set_error.set(Some(format!("Error: {}", e)));
                }
                Ok(image_info) => {
                    set_loading.set(false);
                    set_image_infos.set(Some(image_info));
                }
            }
        });
    });

    view! {
        <div class="image-search-component card" style="max-width: 900px; margin: 2em auto;">
            <header class="card-header">
                <p class="card-header-title">
                    <span class="icon has-text-info" style="margin-right: 0.5em;"><i class="fas fa-search"></i></span>
                    Image Search
                </p>
            </header>
            <div class="card-content">
                <div style="margin-bottom: 1em;">
                    <ErrorDisplay error=error/>
                </div>
                { move ||
                    if loading.get() {
                        view! {
                            <div class="notification is-info is-light">Loading...</div>
                        }.into_any()
                    }
                    else {
                        view! {
                            <ImageInfos image_infos=image_infos/>
                        }.into_any()
                    }
                }
            </div>
        </div>
    }.into_any()
}