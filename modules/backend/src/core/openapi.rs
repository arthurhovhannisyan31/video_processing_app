use crate::features::auth::routes::{__path_login, __path_register};
use crate::features::protected::routes::__path_protected;
use crate::features::system::routes::{__path_health, __path_openapi};

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(login, register, protected, health, openapi))]
pub struct OpenApiSpec;
