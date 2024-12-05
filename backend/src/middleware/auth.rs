use actix_web::{dev::ServiceRequest, Error as ActixError, HttpMessage};
use actix_web_httpauth::extractors::{AuthenticationError, bearer::{BearerAuth, Config}};
use jsonwebtoken::{decode, DecodingKey, Validation};

#[derive(Debug, serde::Deserialize)]
struct Claims {
    sub: i32,
    exp: usize,
}

pub async fn jwt_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (ActixError, ServiceRequest)> {
    let config = Config::default().realm("Restricted area");
    let token = credentials.token();

    let decoding_key = DecodingKey::from_secret(std::env::var("ACCESS_TOKEN_SECRET").unwrap().as_ref());

    match decode::<Claims>(token, &decoding_key, &Validation::default()) {
        Ok(token_data) => {
            req.extensions_mut().insert(token_data.claims.sub);
            Ok(req)
        }
        Err(_) => {
            let err = AuthenticationError::from(config).into();
            Err((err, req))
        }
    }
}