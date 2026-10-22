//! `asset-cache` URI scheme: `asset-cache://localhost/<hash>?src=<url-encoded original>`.
//! The first request downloads the image to the disk cache; later ones are served from disk.

use services::assets::AssetStore;
use tauri::http::{Response, StatusCode, header};
use tauri::{Builder, Manager, Runtime, UriSchemeContext, UriSchemeResponder};

/// Scheme name; keep in sync with the CSP in `tauri.conf.json`.
pub const SCHEME: &str = "asset-cache";
const CACHE_CONTROL: &str = "public, max-age=31536000, immutable";

/// Register the scheme on the builder.
pub fn register<R: Runtime>(builder: Builder<R>) -> Builder<R> {
    builder.register_asynchronous_uri_scheme_protocol(SCHEME, handle)
}

fn handle<R: Runtime>(
    context: UriSchemeContext<'_, R>,
    request: tauri::http::Request<Vec<u8>>,
    responder: UriSchemeResponder,
) {
    let Some(store) = context.app_handle().try_state::<AssetStore>() else {
        responder.respond(status_only(StatusCode::SERVICE_UNAVAILABLE));
        return;
    };
    let store = store.inner().clone();
    let uri = request.uri().to_string();
    tauri::async_runtime::spawn(async move {
        responder.respond(serve(&store, &uri).await);
    });
}

async fn serve(store: &AssetStore, uri: &str) -> Response<Vec<u8>> {
    match store.load_image(uri).await {
        Ok(image) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, image.content_type)
            .header(header::CACHE_CONTROL, CACHE_CONTROL)
            // The UI reads pixels of these images (ASCII backdrop); without this the canvas
            // would be tainted because the scheme is a different origin than the page.
            .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
            .body(image.bytes)
            .unwrap_or_else(|_| status_only(StatusCode::INTERNAL_SERVER_ERROR)),
        Err(error) => {
            tracing::debug!(%error, "asset_image_failed");
            status_only(StatusCode::NOT_FOUND)
        }
    }
}

fn status_only(status: StatusCode) -> Response<Vec<u8>> {
    let mut response = Response::new(Vec::new());
    *response.status_mut() = status;
    response
}
