use actix_web::{dev::ServiceRequest, Error as ActixError, HttpMessage};
use actix_web_httpauth::extractors::{AuthenticationError, bearer::{BearerAuth, Config}};
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::middleware::roles::Role;

#[derive(Debug, serde::Deserialize)]
struct Claims {
    sub: i32,
    role: String,
    exp: usize,
}

pub async fn jwt_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (ActixError, ServiceRequest)> {
    let token = credentials.token();

    let decoding_key = DecodingKey::from_secret(std::env::var("ACCESS_TOKEN_SECRET").unwrap().as_ref());

    match decode::<Claims>(token, &decoding_key, &Validation::default()) {
        Ok(token_data) => {
            req.extensions_mut().insert(token_data.claims.sub);

            let role = match token_data.claims.role.as_str() {
                "admin" => Role::Admin,
                "user" => Role::User,
                _ => Role::Guest,
            };
            req.extensions_mut().insert(role);

            req.extensions_mut().insert(token_data.claims.exp);

            Ok(req)
        }
        Err(_) => {
            let config = Config::default();
            let err = AuthenticationError::from(config).into();
            Err((err, req))
        }
    }
}