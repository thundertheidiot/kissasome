use std::{fs::create_dir_all, path::PathBuf};

use crate::{Error, post::PostError::InvalidImage};
use actix_multipart::form::tempfile::TempFile;
use actix_web::web::Data;
use image::{EncodableLayout, ImageReader};
use tokio::fs;
use uuid::Uuid;
use webp::Encoder;

use crate::{AppState, post::PostError};

pub fn uuid_to_path(uuid: Uuid, state: &Data<AppState>) -> PathBuf {
    let mut path = state.image_path.clone();
    path.push::<String>(uuid.into());
    path.set_extension("webp");
    path
}

pub async fn image_to_webp(
    file: TempFile,
    state: &Data<AppState>,
) -> Result<Uuid, Error<PostError>> {
    async fn inner(file: TempFile, state: &Data<AppState>) -> anyhow::Result<Uuid> {
        let image = ImageReader::open(file.file)?
            .with_guessed_format()?
            .decode()?;

        // metadata isn't retained by the decode function

        // image doesn't support lossy webp
        let webp = Encoder::from_image(&image)
            // from_image only returns a "Unimplemented" &str
            .map_err(|_| InvalidImage)?
            .encode(80.0);

        create_dir_all(&state.image_path)?;

        let uuid = Uuid::new_v4();

        let path = uuid_to_path(uuid, state);
        fs::write(&path, webp.as_bytes()).await?;

        Ok(uuid)
    }

    inner(file, state).await.map_err(|err| {
        if let Ok(e) = err.downcast::<PostError>() {
            Error::Custom(e)
        } else {
            Error::Internal("internal server error")
        }
    })
}
