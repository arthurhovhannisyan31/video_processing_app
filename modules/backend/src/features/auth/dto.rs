use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

/// CreateUserRequest requirements
///
/// Username:
/// - Length 8-255 characters
/// - Only alfa-numeric characters
///
/// Email:
/// - length at most 255 characters
/// - should match `email` regex
///
/// Password:
/// - length 8-100 characters
/// - Include 1 uppercase letter: A-Z
/// - Include 1 lowercase letter: a-z
/// - Include 1 numeric character: 0-9
/// - Include a special character: !,$,#,%, etc

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateUserRequest {
  #[validate(
    length(min = 8, max = 255),
    custom(function = "validate_alfa_numeric_username")
  )]
  pub username: String,
  #[validate(length(max = 255), email)]
  pub email: String,
  #[validate(
    length(min = 8, max = 100),
    custom(function = "validate_password_strength")
  )]
  pub password: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct AuthenticatedUser {
  pub email: String,
  pub user_id: String,
  pub username: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AuthRequest {
  pub email: String,
  pub password: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct AuthResponse {
  pub token: String,
  pub user: AuthenticatedUser,
}

fn validate_alfa_numeric_username(
  username: &str,
) -> Result<(), ValidationError> {
  // String slice length in not validated since it's done in joined test validate(length(min, max))

  if !(username.chars().all(char::is_alphanumeric)) {
    return Err(ValidationError::new(
      "Username should include only alphanumeric values",
    ));
  }

  Ok(())
}

fn validate_password_strength(password: &str) -> Result<(), ValidationError> {
  let mut has_upper = false;
  let mut has_lower = false;
  let mut has_numeric = false;
  let mut has_special = false;

  for c in password.chars() {
    if c.is_uppercase() {
      has_upper = true;
    } else if c.is_lowercase() {
      has_lower = true;
    } else if c.is_numeric() {
      has_numeric = true;
    } else if c.is_ascii_punctuation() {
      has_special = true;
    }
  }

  if !has_upper {
    return Err(ValidationError::new(
      "Password should include at least 1 uppercase character",
    ));
  }
  if !has_lower {
    return Err(ValidationError::new(
      "Password should include at least 1 lowercase character",
    ));
  }
  if !has_numeric {
    return Err(ValidationError::new(
      "Password should include at least 1 numeric character",
    ));
  }
  if !has_special {
    return Err(ValidationError::new(
      "Password should include at least 1 special character",
    ));
  }

  Ok(())
}

#[cfg(test)]
mod test_create_user_request {
  use super::*;
  use proptest::prelude::*;
  use proptest::string::string_regex;
  use validator::Validate;

  const INVALID_REGEX_ERROR: &str = "Invalid regex pattern structure";

  fn valid_username() -> impl Strategy<Value = String> {
    string_regex("[a-zA-Z0-9]{8,255}").expect(INVALID_REGEX_ERROR)
  }

  fn valid_email() -> impl Strategy<Value = String> {
    string_regex("[a-z0-9]{3,10}@[a-z0-9]{3,10}\\.[a-z]{3}")
      .expect(INVALID_REGEX_ERROR)
  }

  fn valid_password() -> impl Strategy<Value = String> {
    string_regex("[a-zA-Z0-9*.!@#$%^&(){}\\[\\]:;<>,?~_+-\\/|=]{4,96}")
      .expect(INVALID_REGEX_ERROR)
      .prop_map(|base| format!("A1_a{}", base))
  }

  fn password_missing_uppercase() -> impl Strategy<Value = String> {
    string_regex("[a-z0-9*.!@#$%^&(){}\\[\\]:;<>,?~_+-\\/|=]{8,100}")
      .expect(INVALID_REGEX_ERROR)
  }

  fn password_missing_lowercase() -> impl Strategy<Value = String> {
    string_regex("[A-Z0-9*.!@#$%^&(){}\\[\\]:;<>,?~_+-\\/|=]{8,100}")
      .expect(INVALID_REGEX_ERROR)
  }

  fn password_missing_numeric() -> impl Strategy<Value = String> {
    string_regex("[a-zA-Z*.!@#$%^&(){}\\[\\]:;<>,?~_+-\\/|=]{8,100}")
      .expect(INVALID_REGEX_ERROR)
  }

  fn password_missing_special() -> impl Strategy<Value = String> {
    string_regex("[a-zA-Z0-9]{8,100}").expect(INVALID_REGEX_ERROR)
  }

  fn password_exceeds_range_bound() -> impl Strategy<Value = String> {
    prop_oneof![
      // Generates password with length less than 8 characters
      string_regex("[a-zA-Z0-9]{0,3}")
        .expect(INVALID_REGEX_ERROR)
        .prop_map(|base| { format!("A1_a{}", base) }),
      // Generates password with length more than 100 characters
      string_regex("[a-zA-Z0-9]{97}")
        .expect(INVALID_REGEX_ERROR)
        .prop_map(|base| { format!("A1_a{}", base) })
    ]
  }

  fn username_exceeds_range_bound() -> impl Strategy<Value = String> {
    prop_oneof![
      // Generates username with length less than 8 characters
      string_regex("[a-zA-Z0-9]{0,7}").expect(INVALID_REGEX_ERROR),
      // Generates username with length more than 255 characters
      string_regex("[a-zA-Z0-9]{256}").expect(INVALID_REGEX_ERROR)
    ]
  }

  fn username_is_not_alphanumeric() -> impl Strategy<Value = String> {
    string_regex("[a-zA-Z0-9*.!@#$%^&(){}\\[\\]:;<>,?~_+-\\/|=]{8,255}")
      .expect(INVALID_REGEX_ERROR)
  }

  fn email_exceeds_upper_bound() -> impl Strategy<Value = String> {
    // Generate an email with exceeding length 256(246 + 10 characters) (max 255)
    // No lower bound for email length is considered
    string_regex("[a-z0-9]{246}")
      .expect(INVALID_REGEX_ERROR)
      .prop_map(|prefix| format!("{}@test.test", prefix))
  }

  fn email_has_special_characters() -> impl Strategy<Value = String> {
    string_regex(
      "[a-z0-9]{3,10}[.]{0,1}[@\\s:;()<>]{1}@[a-z0-9]{3,10}\\.[a-z]{3}",
    )
    .expect(INVALID_REGEX_ERROR)
  }

  // Positive tests
  proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
      fn test_entire_valid_dto(
          username in valid_username(),
          email in valid_email(),
          password in valid_password(),
      ) {
          let request = CreateUserRequest { username, email, password };
          prop_assert!(request.validate().is_ok());
      }
  }

  // Username negative tests
  proptest! {
    // Just a few cases for length validation
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_username_exceeds_range_bound(
          password in valid_password(),
          username in username_exceeds_range_bound(),
          email in valid_email(),
    ){
      let request = CreateUserRequest {username, email, password};
      let result = request.validate();
      prop_assert!(result.is_err());

      let error_keys = result
        .unwrap_err()
        .field_errors()
        .keys()
        .map(|val| val.to_string())
        .collect::<Vec<String>>();
      prop_assert!(error_keys.eq(&vec!["username".to_string()]));
    }
  }

  // Username negative tests
  proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_username_is_not_alphanumeric(
          password in valid_password(),
          username in username_is_not_alphanumeric(),
          email in valid_email(),
    ){
      let request = CreateUserRequest {username, email, password};
      let result = request.validate();
      prop_assert!(result.is_err());

      let error_keys = result
        .unwrap_err()
        .field_errors()
        .keys()
        .map(|val| val.to_string())
        .collect::<Vec<String>>();
      prop_assert!(error_keys.eq(&vec!["username".to_string()]));
    }
  }

  // Email negative tests
  proptest! {
    // Just a few cases for length validation
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_email_exceeds_upper_bound(
          password in valid_password(),
          username in valid_username(),
          email in email_exceeds_upper_bound(),
    ){
      let request = CreateUserRequest {username, email, password};
      let result = request.validate();
      prop_assert!(result.is_err());

      let error_keys = result
        .unwrap_err()
        .field_errors()
        .keys()
        .map(|val| val.to_string())
        .collect::<Vec<String>>();
      prop_assert!(error_keys.eq(&vec!["email".to_string()]));
    }
  }

  // Email negative tests
  proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_email_has_special_characters(
          password in valid_password(),
          username in valid_username(),
          email in email_has_special_characters(),
    ){
      let request = CreateUserRequest {username, email, password};
      let result = request.validate();
      prop_assert!(result.is_err());

      let error_keys = result
        .unwrap_err()
        .field_errors()
        .keys()
        .map(|val| val.to_string())
        .collect::<Vec<String>>();
      prop_assert!(error_keys.eq(&vec!["email".to_string()]));
    }
  }

  // Password negative tests
  proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_password_exceeds_range_bound(
        password in password_exceeds_range_bound(),
        username in valid_username(),
        email in valid_email(),
    ) {
        let request = CreateUserRequest { username, email, password };
        let result = request.validate();
        prop_assert!(result.is_err());

        let error_keys = result
          .unwrap_err()
          .field_errors()
          .keys()
          .map(|val| val.to_string())
          .collect::<Vec<String>>();
        prop_assert!(error_keys.eq(&vec!["password".to_string()]));
    }
  }

  // Password negative tests
  proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_password_missing_uppercase(
        password in password_missing_uppercase(),
        username in valid_username(),
        email in valid_email(),
    ) {
        let request = CreateUserRequest { username, email, password };
        let result = request.validate();
        prop_assert!(result.is_err());

        let error_keys = result
          .unwrap_err()
          .field_errors()
          .keys()
          .map(|val| val.to_string())
          .collect::<Vec<String>>();
        prop_assert!(error_keys.eq(&vec!["password".to_string()]));
    }

    #[test]
    fn test_password_missing_lowercase(
        password in password_missing_lowercase(),
        username in valid_username(),
        email in valid_email(),
    ) {
        let request = CreateUserRequest { username, email, password };
        let result = request.validate();
        prop_assert!(result.is_err());

        let error_keys = result
          .unwrap_err()
          .field_errors()
          .keys()
          .map(|val| val.to_string())
          .collect::<Vec<String>>();
        prop_assert!(error_keys.eq(&vec!["password".to_string()]));
    }

    #[test]
    fn test_password_missing_numeric(
        password in password_missing_numeric(),
        username in valid_username(),
        email in valid_email(),
    ) {
        let request = CreateUserRequest { username, email, password };
        let result = request.validate();

        prop_assert!(result.is_err());

        let error_keys = result
          .unwrap_err()
          .field_errors()
          .keys()
          .map(|val| val.to_string())
          .collect::<Vec<String>>();
        prop_assert!(error_keys.eq(&vec!["password".to_string()]));
    }

    #[test]
    fn test_password_missing_special(
        password in password_missing_special(),
        username in valid_username(),
        email in valid_email(),
    ) {
        let request = CreateUserRequest { username, email, password };
        let result = request.validate();
        prop_assert!(result.is_err());

        let error_keys = result
          .unwrap_err()
          .field_errors()
          .keys()
          .map(|val| val.to_string())
          .collect::<Vec<String>>();
        prop_assert!(error_keys.eq(&vec!["password".to_string()]));
    }
  }
}
