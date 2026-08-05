use crate::features::auth::routes::{__path_login, __path_register};
use crate::features::system::routes::{__path_health, __path_openapi};
use crate::features::video::routes::{
  __path_inspect_video, __path_process_video,
};

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
  info(title = "Video processing API specification", version = "1.0.0"),
  paths(login, register, health, openapi, inspect_video, process_video)
)]
pub struct OpenApiSpec;
