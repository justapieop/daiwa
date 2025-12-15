use std::error::Error;

use jsonwebtoken::Validation;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use crate::{config, get_config};

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
}

pub struct JwtUtils {
    jwks_client: jwks::Jwks,
    config: config::Config,
}

impl JwtUtils {
    pub async fn new(jwks_url: String) -> Self {
        debug!("Config for JwtUtils: {:?}", get_config());
        Self {
            jwks_client: jwks::Jwks::from_jwks_url(jwks_url)
                .await
                .expect("JWKS_URL should be a valid URL pointing to JWKS"),
            config: get_config().clone(),
        }
    }

    pub fn verify(&self, jwt: &str) -> Result<String, Box<dyn Error>> {
        let header = match jsonwebtoken::decode_header(jwt) {
            Ok(v) => v,
            Err(e) => return Err(e.into()),
        };

        let kid = match header.kid {
            Some(v) => v,
            None => return Err("".into()),
        };

        let jwk = match self.jwks_client.keys.get(&kid) {
            Some(v) => v,
            None => return Err("".into()),
        };

        let mut validation = Validation::new(header.alg);
        validation.set_audience(&[self.config.jwks_iss.clone()]);

        let decode = match jsonwebtoken::decode::<Claims>(&jwt, &jwk.decoding_key, &validation) {
            Ok(v) => v,
            Err(e) => {
                error!("JWT verification failed: {:?}", e);
                return Err(e.into());
            },
        };

        Ok(decode.claims.sub)
    }
}
