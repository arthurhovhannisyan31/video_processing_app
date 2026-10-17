use std::sync::Arc;

use async_trait::async_trait;
use lettre::message::Mailbox;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use parking_lot::Mutex;

use crate::core::app_config::SmtpConfig;
use crate::core::error::{ServerError, StartupError};

#[async_trait]
pub trait EmailSender: Send + Sync {
  async fn send(&self, to: &str, subject: &str, body: String) -> Result<(), ServerError>;
}

#[derive(Clone)]
pub struct SmtpEmailSender {
  transport: AsyncSmtpTransport<Tokio1Executor>,
  from: Mailbox,
}

impl SmtpEmailSender {
  pub fn new(config: &SmtpConfig) -> Result<Self, ServerError> {
    let from = config
      .from
      .parse::<Mailbox>()
      .map_err(|e| StartupError::VarError(format!("Invalid BACKEND_SMTP_FROM: {e}")))?;

    let mut builder = if config.tls {
      AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host)?
    } else {
      AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.host)
    }
    .port(config.port);

    if let (Some(username), Some(password)) = (&config.username, &config.password) {
      builder = builder.credentials(Credentials::new(username.clone(), password.clone()));
    }

    Ok(Self {
      transport: builder.build(),
      from,
    })
  }
}

#[async_trait]
impl EmailSender for SmtpEmailSender {
  async fn send(&self, to: &str, subject: &str, body: String) -> Result<(), ServerError> {
    let message = Message::builder()
      .from(self.from.clone())
      .to(to.parse::<Mailbox>()?)
      .subject(subject)
      .header(ContentType::TEXT_PLAIN)
      .body(body)?;

    self.transport.send(message).await?;

    Ok(())
  }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SentEmail {
  pub to: String,
  pub subject: String,
  pub body: String,
}

/// Captures emails in memory instead of sending them, used in tests
#[allow(dead_code)]
#[derive(Clone, Default)]
pub struct MockEmailSender {
  pub sent: Arc<Mutex<Vec<SentEmail>>>,
}

#[async_trait]
impl EmailSender for MockEmailSender {
  async fn send(&self, to: &str, subject: &str, body: String) -> Result<(), ServerError> {
    self.sent.lock().push(SentEmail {
      to: to.to_string(),
      subject: subject.to_string(),
      body,
    });

    Ok(())
  }
}
