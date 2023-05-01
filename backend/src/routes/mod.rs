use std::{collections::HashMap, sync::Arc};

use actix_multipart::{form::MultipartForm, Multipart, MultipartError};
use actix_web::{
    http::StatusCode,
    post,
    web::{self, ServiceConfig},
    HttpResponse, Responder, ResponseError,
};
use bytes::BytesMut;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::{
    game::{BasicConfig, GameConfig, GameTiming, GetImageMessage, Image, Question},
    games::{GameToken, Games, GetGameMessage, PrepareGameMessage},
};

/// Configuration function for configuring
/// all the routes
pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(create_quiz);
    cfg.service(quiz_image);
}

#[derive(Debug, MultipartForm)]
pub struct QuizForm {}

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
async fn create_quiz(mut payload: Multipart) -> Result<impl Responder, CreateError> {
    // Configuration data
    let mut config: Option<GameConfigUpload> = None;
    // Map of stored uploaded images
    let mut images = HashMap::new();

    while let Some(mut field) = payload.try_next().await? {
        /// Cap the image max size to 100mb
        const MAX_BUFFER_SIZE_BYTES: usize = 1024 * 100;

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

    let games = Games::get();

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

#[post("/api/quiz/{token}/{image}")]
async fn quiz_image(path: web::Path<(String, Uuid)>) -> Result<impl Responder, ImageError> {
    let (token, uuid) = path.into_inner();
    let token: GameToken = token.parse().unwrap();
    let games = Games::get();

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
