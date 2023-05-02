use actix::Addr;
use actix_multipart::{Multipart, MultipartError};
use actix_web::{
    get,
    http::StatusCode,
    post,
    web::{self, Data, ServiceConfig},
    HttpRequest, HttpResponse, Responder, ResponseError,
};
use actix_web_actors::ws::{self};
use bytes::BytesMut;
use futures::TryStreamExt;
use log::debug;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};
use thiserror::Error;
use uuid::Uuid;

use crate::{
    game::{BasicConfig, GameConfig, GameTiming, GetImageMessage, Image, Question},
    games::{GameToken, Games, GetGameMessage, PrepareGameMessage},
    session::Session,
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
    pub basic: BasicConfig,
    pub timing: GameTiming,
    pub questions: Vec<Arc<Question>>,
}

#[derive(Debug, Error)]
pub enum CreateError {
    #[error("Missing config data")]
    MissingConfig,

    #[error(transparent)]
    InvalidConfig(#[from] serde_json::Error),

    #[error(transparent)]
    InvalidImageUuid(#[from] uuid::Error),

    #[error("Missing image mime type for {0}")]
    MissingImageType(Uuid),

    #[error(transparent)]
    Multipart(#[from] MultipartError),

    #[error("Uploaded content was over 100mb")]
    TooLarge,
}

impl ResponseError for CreateError {
    fn status_code(&self) -> StatusCode {
        match self {
            CreateError::MissingConfig
            | CreateError::InvalidConfig(_)
            | CreateError::InvalidImageUuid(_)
            | CreateError::MissingImageType(_)
            | CreateError::Multipart(_)
            | CreateError::TooLarge => StatusCode::BAD_REQUEST,
        }
    }
}

#[derive(Serialize)]
struct QuizCreated {
    uuid: Uuid,
}

/// Endpoint for creating a new quiz
#[post("/api/quiz")]
async fn create_quiz(
    mut payload: Multipart,
    games: Data<Addr<Games>>,
) -> Result<impl Responder, CreateError> {
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
            debug!("Loading buffered data: {}", buffer.len());
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
            let value: GameConfigUpload = serde_json::from_slice(&buffer)?;
            config = Some(value);
            continue;
        }

        let uuid: Uuid = name.parse()?;
        let mime = field
            .content_type()
            .ok_or_else(|| CreateError::MissingImageType(uuid))?
            .clone();

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
    let config = GameConfig {
        basic: config.basic,
        timing: config.timing,
        questions: config.questions,
        images,
    };

    let uuid = games
        .send(PrepareGameMessage { config })
        .await
        .expect("Games service is not running");

    Ok(HttpResponse::Created().json(QuizCreated { uuid }))
}

#[derive(Debug, Error)]
pub enum ImageError {
    #[error("The target game could not be found")]
    UnknownGame,
    #[error("The target image could not be found")]
    UnknownImage,
}

impl ResponseError for ImageError {
    fn status_code(&self) -> StatusCode {
        match self {
            ImageError::UnknownGame | ImageError::UnknownImage => StatusCode::BAD_REQUEST,
        }
    }
}

#[get("/api/quiz/{token}/{image}")]
async fn quiz_image(
    path: web::Path<(String, Uuid)>,
    games: Data<Addr<Games>>,
) -> Result<impl Responder, ImageError> {
    let (token, uuid) = path.into_inner();
    let token: GameToken = token.parse().unwrap();

    let game = games
        .send(GetGameMessage { token })
        .await
        .expect("Games service is not running")
        .ok_or(ImageError::UnknownGame)?;

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
    games: Data<Addr<Games>>,
) -> Result<impl Responder, actix_web::Error> {
    let session_id = SESSION_ID.fetch_add(1, Ordering::AcqRel);
    ws::start(
        Session {
            id: session_id,
            game: None,
            // Take refernece to the games addr
            games: games.get_ref().clone(),
        },
        &req,
        stream,
    )
}
