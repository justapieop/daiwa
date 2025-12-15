use std::{error::Error, sync::OnceLock};

use jsonwebtoken::Validation;
use serde::{Deserialize, Serialize};

use crate::config;

#[derive(Serialize, Deserialize, Debug)]
struct Claims {
    sub: String,
}

pub struct JwtUtils {
    jwks_client: jwks::Jwks,
}

impl JwtUtils {
    pub async fn initialize(jwks_url: String) -> Self {
        Self {
            jwks_client: jwks::Jwks::from_jwks_url(jwks_url)
                .await
                .expect("JWKS_URL should be a valid URL pointing to JWKS"),
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
        validation.set_audience(&[config::get().jwks_iss.clone()]);

        let decode = match jsonwebtoken::decode::<Claims>(&jwt, &jwk.decoding_key, &validation) {
            Ok(v) => v,
            Err(e) => return Err(e.into()),
        };

        Ok(decode.claims.sub)
    }
}

static INSTANCE: OnceLock<JwtUtils> = OnceLock::new();

pub fn get_cell() -> &'static OnceLock<JwtUtils> {
    &INSTANCE
}

pub fn get() -> &'static JwtUtils {
    // jwt_utils is already initialized at launch
    &INSTANCE.get().expect("jwt_utils should be initialized")
}
