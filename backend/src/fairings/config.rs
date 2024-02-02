use crate::fairings::template::TemplateResolver;
use crate::guards::emailer::SmtpConfig;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use jsonwebtoken::{DecodingKey, EncodingKey};
use wildmatch::WildMatch;

#[derive(Default, Debug, Clone, serde::Deserialize)]
struct DebugConfig {
    pub additional_root_team: Vec<String>,
    pub bypass_email_validation: Vec<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct RetronomiconConfig {
    pub base_url: String,
    pub root_team: Vec<String>,
    pub root_team_id: i32,

    #[cfg(debug_assertions)]
    debug: Option<DebugConfig>,

    template_dir: String,

    pub smtp: SmtpConfig,
}

impl RetronomiconConfig {
    #[must_use]
    fn debug(&self) -> Option<&DebugConfig> {
        #[cfg(debug_assertions)]
        {
            self.debug.as_ref()
        }

        #[cfg(not(debug_assertions))]
        {
            None
        }
    }

    #[must_use]
    pub fn templates(&self) -> TemplateResolver {
        TemplateResolver::new(&self.template_dir)
    }

    pub(crate) fn bypass_email_validation(&self, email: &str) -> bool {
        if let Some(debug) = self.debug() {
            debug
                .bypass_email_validation
                .iter()
                .any(|e| WildMatch::new(e).matches(email))
        } else {
            false
        }
    }

    pub(crate) fn should_add_to_root(&self, email: &str) -> bool {
        if self
            .root_team
            .iter()
            .any(|e| WildMatch::new(e).matches(email))
        {
            true
        } else if let Some(debug) = self.debug() {
            debug
                .additional_root_team
                .iter()
                .any(|e| WildMatch::new(e.trim()).matches(email))
        } else {
            false
        }
    }
}

pub struct JwtKeys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

impl JwtKeys {
    pub fn from_base64(secret: &str) -> Self {
        let secret = STANDARD.decode(secret).expect("Invalid base64 JWT secret");
        let encoding = EncodingKey::from_secret(&secret);
        let decoding = DecodingKey::from_secret(&secret);
        Self { encoding, decoding }
    }
}

pub struct DbPepper(pub Vec<u8>);

impl DbPepper {
    pub fn from_base64(secret: &str) -> Self {
        let secret = STANDARD.decode(secret).expect("Invalid base64 pepper");
        Self(secret)
    }
}
