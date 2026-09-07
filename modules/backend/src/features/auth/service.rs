use crate::core::error::ApplicationError;
use crate::core::jwt::{JwtService, hash_password, verify_password};
use crate::features::auth::model::{User, UserId};
use crate::features::auth::repository::UserRepository;

#[derive(Clone)]
pub struct AuthService<R: UserRepository + 'static> {
  repo: R,
  jwt_service: JwtService,
}

impl<R> AuthService<R>
where
  R: UserRepository + 'static,
{
  pub fn new(repo: R, jwt_service: JwtService) -> Self {
    Self { repo, jwt_service }
  }

  pub async fn get(&self, id: UserId) -> Result<User, ApplicationError> {
    self
      .repo
      .find_by_id(id.clone())
      .await?
      .ok_or_else(|| ApplicationError::NotFound(format!("user {}", id)))
  }

  pub async fn get_by_email(&self, email: &str) -> Result<User, ApplicationError> {
    match self.repo.find_by_email(&email.to_lowercase()).await {
      Ok(Some(user)) => Ok(user),
      Ok(None) => Err(ApplicationError::NotFound(format!("user {}", email))),
      Err(err) => Err(err)?,
    }
  }

  pub async fn register(
    &self,
    email: String,
    password: String,
    username: String,
  ) -> Result<User, ApplicationError> {
    let hash = hash_password(&password)?;
    let user = User::new(email.to_lowercase(), hash, username);

    self.repo.create(user).await.map_err(ApplicationError::from)
  }

  pub async fn login(
    &self,
    email: &str,
    password: &str,
    mock_password_hash: &str,
  ) -> Result<(User, String), ApplicationError> {
    let user_res = self.repo.find_by_email(email).await?;

    let password_hash = user_res
      .as_ref()
      .map(|u| u.password_hash.as_str())
      .unwrap_or(mock_password_hash);

    // constant performance time for any user lookup result
    let password_ok = match verify_password(password, password_hash) {
      Ok(password_ok) => password_ok,
      Err(err) => return Err(ApplicationError::Internal(err.to_string())),
    };

    let user = match (user_res, password_ok) {
      (Some(user), true) => user,
      _ => return Err(ApplicationError::Unauthorized),
    };

    let token = self
      .jwt_service
      .generate_token(user.id.clone(), user.username.clone())
      .map_err(|err| ApplicationError::Internal(err.to_string()))?;

    Ok((user, token))
  }
}
