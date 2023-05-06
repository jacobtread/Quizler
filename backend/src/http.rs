use actix_multipart::{Multipart, MultipartError};
use actix_web::{
    get,
    http::StatusCode,
    post,
    web::{self, ServiceConfig},
    HttpRequest, HttpResponse, Responder, ResponseError,
};
use actix_web_actors::ws::{self};
use bytes::BytesMut;
use futures_util::TryStreamExt;
use log::debug;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::Display,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};
use uuid::Uuid;

use crate::{
    game::{GameConfig, GameTiming, GetImageMessage},
    games::{Games, GetGameMessage, PrepareGameMessage},
    session::Session,
    types::{Image, Question},
};

/// Configuration function for configuring
/// all the routes
pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(create_quiz);
    cfg.service(quiz_image);
    cfg.service(quiz_socket);
}

#[derive(Deserialize)]
pub struct GameConfigUpload {
    pub name: String,
    pub text: String,
    pub timing: GameTiming,
    pub questions: Vec<Arc<Question>>,
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
#[post("/api/quiz")]
async fn create_quiz(mut payload: Multipart) -> Result<impl Responder, CreateError> {
    // Configuration data
    let mut config: Option<GameConfigUpload> = None;
    // Map of stored uploaded images
    let mut images = HashMap::new();

    while let Some(mut field) = payload.try_next().await? {
        /// Cap the image max size to 100mb
        const MAX_BUFFER_SIZE_BYTES: usize = 1024 * 1024 * 1024;

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

        let name = field.name();

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
            .ok_or_else(|| CreateError::MissingImageType(uuid))?
            .clone();

        debug!(
            "Recieved uploaded file (UUID: {}, Mime: {}, Size: {})",
            uuid,
            mime,
            buffer.len()
        );

        images.insert(
            uuid,
            Image {
                mime,
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
        timing: config.timing,
        questions: config.questions,
        images,
    };

    let uuid = Games::get()
        .send(PrepareGameMessage { config })
        .await
        .expect("Games service is not running");

    debug!("Created new prepared game {}", uuid);

    Ok(HttpResponse::Created().json(QuizCreated { uuid }))
}

#[derive(Debug)]
pub enum ImageError {
    UnknownGame,
    UnknownImage,
}

#[get("/api/quiz/{token}/{image}")]
async fn quiz_image(path: web::Path<(String, Uuid)>) -> Result<impl Responder, ImageError> {
    let (token, uuid) = path.into_inner();

    let game = Games::get()
        .send(GetGameMessage { token })
        .await
        .expect("Games service is not running")
        .map_err(|_| ImageError::UnknownGame)?;

    let image = game
        .send(GetImageMessage { uuid })
        .await
        .map_err(|_| ImageError::UnknownGame)?
        .ok_or(ImageError::UnknownImage)?;

    Ok(HttpResponse::Ok().content_type(image.mime).body(image.data))
}

static SESSION_ID: AtomicU32 = AtomicU32::new(0);

#[get("/api/quiz/socket")]
async fn quiz_socket(
    req: HttpRequest,
    stream: web::Payload,
) -> Result<impl Responder, actix_web::Error> {
    let session_id = SESSION_ID.fetch_add(1, Ordering::AcqRel);
    debug!("Starting new socket {}", session_id);
    ws::start(
        Session {
            id: session_id,
            game: None,
        },
        &req,
        stream,
    )
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
            CreateError::TooLarge => f.write_str("Uploaded content was over 100mb"),
            CreateError::MissingQuestions => f.write_str("Quiz must have atleast 1 question"),
        }
    }
}

impl ResponseError for CreateError {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
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

impl ResponseError for ImageError {
    fn status_code(&self) -> StatusCode {
        match self {
            ImageError::UnknownGame | ImageError::UnknownImage => StatusCode::BAD_REQUEST,
        }
    }
}
