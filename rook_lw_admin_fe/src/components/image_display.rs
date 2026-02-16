use leptos::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;
use rook_lw_models::image::ImageInfo;

use crate::components::ErrorDisplay;
use crate::services::ImageInfoService;
use crate::services::get_signed_url;

#[component]
pub fn ImageDetections(image_info: ImageInfo) -> impl IntoView {
    view! {
        <table class="image-display-detections">
            <tr>
                <th>"Class"</th>
                <th>"Confidence"</th>
            </tr>
            { match image_info.detection {
                None => view! {}.into_any(),
                Some(ref detection) => {
                    detection.detections.iter().map (|d| {
                        view! {
                            <tr>
                                <td>{ d.class_name.clone() }</td>
                                <td>{ d.confidence }</td>
                            </tr>
                        }
                    }).collect_view().into_any()
                }
            } }
        </table>
    }
}

#[component]
pub fn ImageDisplay() -> impl IntoView {
    let params = use_params_map();
    let (error, set_error) = signal(None::<String>);
    let (image_info, set_image_info) = signal(None::<ImageInfo>);
    let (image_url, set_image_url) = signal(String::new());

    let base_service = match use_context::<ImageInfoService>() {
        Some(s) => s,
        None => return view! {
            <div>Error</div>
        }.into_any()
    };
    
    let user_service_signal = expect_context::<RwSignal<crate::services::UserService>>();

    let image_id = {
        move || {
            params.with(|p| p.get("image_id").unwrap_or_default().clone())
        }
    };

    // Looks up image Info
    Effect::new(move |_| {
        let base_service = base_service.clone();
        let set_image_info = set_image_info.clone();
        let set_error = set_error.clone();
        let set_image_url = set_image_url.clone();

        set_error.set(None);
        set_image_info.set(None);
        set_image_url.set(String::new());

        let image_id = image_id();

        spawn_local(async move {
            let user_service = user_service_signal.get_untracked();
            let image_info_service = base_service.with_user_service(user_service.clone());
            
            match image_info_service.get(&image_id).await {
                Err(e) => {
                    set_error.set(Some(format!("Couldn't retrieve image info: {}", e)));
                }
                Ok(image_info_opt) => {
                    if let Some(ref info) = image_info_opt {
                        let url = format!("/api/image/{}", &info.image_path);
                        match get_signed_url(Some(&user_service), &url) {
                            Ok(signed_url) => {
                                set_image_url.set(signed_url);
                            }
                            Err(e) => {
                                set_error.set(Some(format!("Failed to sign URL: {}", e)));
                            }
                        }
                    }
                    set_image_info.set(image_info_opt);
                }
            }
        });
    });

    view! {
        <ErrorDisplay error=error/>
        <div class="image_display">
            { move ||
                match image_info.get() {
                    Some(image_info) => {
                        let url = image_url.get();
                        if url.is_empty() {
                            view! {
                                <span>"Loading image..."</span>
                            }.into_any()
                        } else {
                            view! { 
                                <img src={ url }/>
                                <br/>
                                <ImageDetections image_info=image_info/>
                            }.into_any()
                        }
                    },
                    None => {
                        view! {
                            <span>"Image info loading..."</span>
                        }.into_any()
                    }
                }
            }
        </div>
    }.into_any()
}