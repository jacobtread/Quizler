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
    fmt::Display,
    future::{ready, Ready},
    sync::{atomic::AtomicU32, Arc},
    task::{Context, Poll},
};
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

/// Embedded assets for serving the frontend of the application
#[derive(Embedded, Clone)]
#[folder = "public"]
struct Assets;

impl<T> Service<Request<T>> for Assets {
    type Response = Response;
    type Error = Infallible;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<T>) -> Self::Future {
        let path = req.uri().path();
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

#[derive(Deserialize)]
pub struct GameConfigUpload {
    pub name: ImStr,
    pub text: ImStr,
    pub max_players: usize,
    pub filtering: NameFiltering,
    pub questions: Box<[Arc<Question>]>,
}

#[derive(Debug)]
pub enum CreateError {
    MissingConfig,
    InvalidConfig(serde_json::Error),
    InvalidImageUuid(uuid::Error),
    MissingImageType(Uuid),
    Multipart(MultipartError),
    TooLarge,
    MissingQuestions,
}

#[derive(Serialize)]
struct QuizCreated {
    uuid: Uuid,
}

/// Endpoint for creating a new quiz
async fn create_quiz(mut payload: Multipart) -> Result<Response, CreateError> {
    // Configuration data
    let mut config: Option<GameConfigUpload> = None;
    // Map of stored uploaded images
    let mut images = HashMap::new();

    while let Some(mut field) = payload.next_field().await? {
        /// Cap the upload max size to 15mb
        const MAX_BUFFER_SIZE_BYTES: usize = 1024 * 1024 * 15;

        // Read all the buffered content for the config message
        let mut buffer = BytesMut::new();
        loop {
            if buffer.len() >= MAX_BUFFER_SIZE_BYTES {
                return Err(CreateError::TooLarge);
            }

            let chunk = match field.try_next().await? {
                Some(value) => value,
                None => break,
            };
            buffer.extend_from_slice(&chunk);
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

    // Validate the config is correct
    if config.questions.is_empty() {
        return Err(CreateError::MissingQuestions);
    }

    let config = GameConfig {
        name: config.name,
        text: config.text,
        max_players: config.max_players,
        filtering: config.filtering,
        questions: config.questions,
        images,
    };

    let uuid = Games::prepare(config).await;

    debug!("Created new prepared game {}", uuid);

    Ok((StatusCode::CREATED, Json(QuizCreated { uuid })).into_response())
}

#[derive(Debug)]
pub enum ImageError {
    UnknownGame,
    UnknownImage,
}

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
    res.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_str(&image.mime).expect("Failed to create mime header"),
    );

    Ok(res)
}

static SESSION_ID: AtomicU32 = AtomicU32::new(0);

async fn quiz_socket(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(Session::start)
}

impl From<MultipartError> for CreateError {
    fn from(value: MultipartError) -> Self {
        CreateError::Multipart(value)
    }
}

impl Display for CreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CreateError::MissingConfig => f.write_str("Missing config data"),
            CreateError::InvalidConfig(err) => err.fmt(f),
            CreateError::InvalidImageUuid(err) => err.fmt(f),
            CreateError::MissingImageType(uuid) => {
                write!(f, "Missing image mime type for {}", uuid)
            }
            CreateError::Multipart(err) => err.fmt(f),
            CreateError::TooLarge => f.write_str("Uploaded content was too large"),
            CreateError::MissingQuestions => f.write_str("Quiz must have atleast 1 question"),
        }
    }
}

impl IntoResponse for CreateError {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, self.to_string()).into_response()
    }
}

impl Display for ImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageError::UnknownGame => f.write_str("The target game could not be found"),
            ImageError::UnknownImage => f.write_str("The target image could not be found"),
        }
    }
}

impl IntoResponse for ImageError {
    fn into_response(self) -> Response {
        match self {
            ImageError::UnknownGame | ImageError::UnknownImage => {
                (StatusCode::BAD_REQUEST).into_response()
            }
        }
    }
}
