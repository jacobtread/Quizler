use crate::{
    game::GameConfig,
    games::Games,
    session::Session,
    types::{GameToken, ImStr, Image, NameFiltering, Question},
};
use axum::{
    body::Full,
    extract::{multipart::MultipartError, Multipart, Path, WebSocketUpgrade},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use bytes::BytesMut;
use embeddy::Embedded;
use futures_util::TryStreamExt;
use hyper::{header::CONTENT_TYPE, http::HeaderValue, Request, StatusCode};
use log::debug;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    convert::Infallible,
    future::{ready, Ready},
    sync::Arc,
    task::{Context, Poll},
};
use thiserror::Error;
use tower::Service;
use uuid::Uuid;

/// Configuration function for configuring
/// all the routes
pub fn router() -> Router {
    Router::new()
        .route("/api/quiz", post(create_quiz))
        .route("/api/quiz/:token/:image", get(quiz_image))
        .route("/api/quiz/socket", get(quiz_socket))
        .fallback_service(Assets)
}

/// Intermediate structure for GameConfigs parsed from
/// quiz upload form data
#[derive(Deserialize)]
struct GameConfigUpload {
    /// The quiz name
    name: ImStr,
    /// The quiz description
    text: ImStr,
    /// The max number of quiz players
    max_players: usize,
    /// The quiz name filter
    filtering: NameFiltering,
    /// The quiz questions
    questions: Box<[Arc<Question>]>,
}

/// Errors that can occur when creating a quiz
#[derive(Debug, Error)]
enum CreateError {
    /// Quiz was missing its config
    #[error("Missing config data")]
    MissingConfig,
    /// Quiz config was invalid
    #[error(transparent)]
    InvalidConfig(serde_json::Error),
    /// Quiz failed server validation
    #[error("Validation failure incorrect values")]
    ValidationFailed,
    /// Uploaded image had an invalid ID
    #[error(transparent)]
    InvalidImageUuid(uuid::Error),
    /// Image was missing its mime type
    #[error("Missing image mime type for {0}")]
    MissingImageType(Uuid),
    /// Multipart read error
    #[error(transparent)]
    Multipart(#[from] MultipartError),
    /// Content was too large
    #[error("Uploaded content was too large")]
    TooLarge,
}

#[derive(Serialize)]
struct QuizCreated {
    uuid: Uuid,
}

/// # POST /api/quiz
///
/// Endpoint for uploading and creating a new Quiz.
async fn create_quiz(mut payload: Multipart) -> Result<Response, CreateError> {
    // Configuration data
    let mut config: Option<GameConfigUpload> = None;
    // Map of stored uploaded images
    let mut images = HashMap::new();

    while let Some(mut field) = payload.next_field().await? {
        /// Cap the upload max size to 15mb
        const MAX_BUFFER_SIZE_BYTES: usize = 1024 * 1024 * 15;

        // Read the field content until the max buffer size
        let mut buffer = BytesMut::new();
        while let Some(chunk) = field.try_next().await? {
            buffer.extend_from_slice(&chunk);

            if buffer.len() >= MAX_BUFFER_SIZE_BYTES {
                return Err(CreateError::TooLarge);
            }
        }

        let name = match field.name() {
            Some(value) => value,
            // Skip un-named fields
            None => continue,
        };

        // Handle the config
        if name == "config" {
            let value: GameConfigUpload =
                serde_json::from_slice(&buffer).map_err(CreateError::InvalidConfig)?;
            config = Some(value);
            continue;
        }

        let uuid: Uuid = name.parse().map_err(CreateError::InvalidImageUuid)?;
        let mime = field
            .content_type()
            .ok_or(CreateError::MissingImageType(uuid))?;

        debug!(
            "Recieved uploaded file (UUID: {}, Mime: {}, Size: {})",
            uuid,
            mime,
            buffer.len()
        );

        images.insert(
            uuid,
            Image {
                mime: mime.into(),
                data: buffer.freeze(),
            },
        );
    }

    // Create the full configuration
    let config = config.ok_or(CreateError::MissingConfig)?;

    let config = GameConfig {
        name: config.name,
        text: config.text,
        max_players: config.max_players,
        filtering: config.filtering,
        questions: config.questions,
        images,
    };

    // Validate the config is acceptable
    if !config.validate() {
        return Err(CreateError::ValidationFailed);
    }

    let uuid = Games::prepare(config).await;

    debug!("Created new prepared game {}", uuid);

    Ok((StatusCode::CREATED, Json(QuizCreated { uuid })).into_response())
}

#[derive(Debug, Error)]
enum ImageError {
    #[error("The target game could not be found")]
    UnknownGame,
    #[error("The target image could not be found")]
    UnknownImage,
    #[error("Image mime type was invalid")]
    InvalidImageMime,
}

/// # GET /api/quiz/:token/:uuid
///
/// Endpoint for getting the contents of an image from
/// a quiz
async fn quiz_image(Path((token, uuid)): Path<(String, Uuid)>) -> Result<Response, ImageError> {
    let token: GameToken = token.parse().map_err(|_| ImageError::UnknownGame)?;
    let game = Games::get_game(&token)
        .await
        .ok_or(ImageError::UnknownGame)?;

    let image = game
        .read()
        .await
        .get_image(uuid)
        .ok_or(ImageError::UnknownImage)?;

    let mut res = Full::from(image.data).into_response();
    let content_type =
        HeaderValue::from_str(&image.mime).map_err(|_| ImageError::InvalidImageMime)?;
    res.headers_mut().insert(CONTENT_TYPE, content_type);

    Ok(res)
}

/// # GET /api/quiz/socket
///
/// Endpoint for creating a new websocket session
async fn quiz_socket(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(Session::start)
}

/// Embedded assets for serving the frontend of the application
#[derive(Embedded, Clone)]
#[folder = "public"]
struct Assets;

/// Fallback service implementation for using the assets from within
/// the embedded data
impl<T> Service<Request<T>> for Assets {
    type Response = Response;
    type Error = Infallible;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<T>) -> Self::Future {
        let path = req.uri().path();
        // Strip the leading slash in order to match paths correctly
        let path = match path.strip_prefix('/') {
            Some(value) => value,
            None => path,
        };

        let std_path = std::path::Path::new(path);

        let (file, content_type) = if let Some(file) = Assets::get(path) {
            // Find a matching content type or default to text/plain
            let content_type = std_path
                .extension()
                .and_then(|ext| {
                    if ext == "js" {
                        Some("application/javascript")
                    } else if ext == "css" {
                        Some("text/css")
                    } else if ext == "html" {
                        Some("text/html")
                    } else {
                        None
                    }
                })
                .unwrap_or("text/plain");

            (file, content_type)
        } else {
            // Fallback to the index.html file for all unknown pages
            let index = Assets::get("index.html").expect("Missing index.html from build");
            (index, "text/html")
        };

        let mut res = Full::from(file).into_response();
        res.headers_mut()
            .insert(CONTENT_TYPE, HeaderValue::from_static(content_type));

        ready(Ok(res))
    }
}

impl IntoResponse for CreateError {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, self.to_string()).into_response()
    }
}

impl IntoResponse for ImageError {
    fn into_response(self) -> Response {
        let status_code = match self {
            Self::UnknownGame | Self::UnknownImage => StatusCode::BAD_REQUEST,
            Self::InvalidImageMime => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status_code, self.to_string()).into_response()
    }
}
