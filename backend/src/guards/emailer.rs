use crate::fairings::config::RetronomiconConfig;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use rocket::error;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::serde::json::json;

fn _default_smtp_port() -> u16 {
    587
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SmtpConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<String>,

    #[serde(default = "_default_smtp_port")]
    pub port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,

    pub from: String,
}

pub struct EmailGuard {
    config: SmtpConfig,
    template: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for EmailGuard {
    type Error = String;

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let config = match request.rocket().state::<RetronomiconConfig>() {
            Some(c) => c,
            None => return Outcome::Error((Status::InternalServerError, "No config".to_string())),
        };
        let smtp_config = config.smtp.clone();

        let template = config.templates().email_verification();

        Outcome::Success(Self {
            config: smtp_config,
            template,
        })
    }
}

impl EmailGuard {
    pub fn send_email_verification(&self, email: &str, url: &str) -> Result<(), (Status, String)> {
        let server_url = match self.config.server.as_ref() {
            Some(api_key) => api_key,
            None => {
                rocket::warn!("No SMTP server set, not sending email");
                rocket::warn!("Url to validate email: {}", url);
                return Ok(());
            }
        };

        let hbar = handlebars::Handlebars::new();
        let text = hbar
            .render_template(
                &self.template,
                &json!({
                    "email": email,
                    "url": url,
                }),
            )
            .map_err(|e| (Status::InternalServerError, e.to_string()))?;

        let from = self.config.from.parse().map_err(|e| {
            error!("Failed to parse from ({}) address: {}", self.config.from, e);
            (
                Status::InternalServerError,
                Status::InternalServerError.to_string(),
            )
        })?;
        let to = email.parse().map_err(|e| {
            (
                Status::InternalServerError,
                format!("Failed to parse to ({}) address: {}", email, e),
            )
        })?;
        let email = Message::builder()
            .from(from)
            .to(to)
            .subject("Retronomicon Email Verification")
            .header(ContentType::TEXT_PLAIN)
            .body(text)
            .unwrap();

        // Open a remote connection to gmail
        let mut mailer = SmtpTransport::relay(server_url).unwrap();

        if let (Some(username), Some(password)) =
            (self.config.username.as_ref(), self.config.password.as_ref())
        {
            mailer =
                mailer.credentials(Credentials::new(username.to_string(), password.to_string()));
        }
        let mailer = mailer.build();

        // Send the email
        match mailer.send(&email) {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Could not send email: {e:?}");
                Err((Status::InternalServerError, e.to_string()))
            }
        }
    }
}
