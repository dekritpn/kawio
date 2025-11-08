use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // player name
    pub exp: usize,  // expiration time
}

pub struct Auth;

impl Auth {
    const SECRET: &'static str = "your-secret-key"; // In production, use env var

    pub fn generate_token(player: &str) -> Result<String, jsonwebtoken::errors::Error> {
        let expiration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize
            + 3600; // 1 hour

        let claims = Claims {
            sub: player.to_string(),
            exp: expiration,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(Self::SECRET.as_ref()),
        )
    }

    pub fn validate_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(Self::SECRET.as_ref()),
            &Validation::default(),
        )?;
        Ok(token_data.claims)
    }
}