use std::rc::Rc;
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    Error as ActixError, HttpMessage,
    body::EitherBody,
    dev::{forward_ready, Service, Transform},
    http::StatusCode,
    HttpResponse,
};
use futures_util::future::{LocalBoxFuture, Ready, ready};

#[derive(Clone, PartialEq)]
pub enum Role {
    Guest,
    Admin,
    User,
}

pub struct RequireRole {
    required_role: Role,
}

impl RequireRole {
    pub fn new(role: Role) -> Self {
        Self {
            required_role: role,
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RequireRole
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = ActixError;
    type Transform = RequireRoleMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequireRoleMiddleware {
            service: Rc::new(service),
            required_role: self.required_role.clone(),
        }))
    }
}

pub struct RequireRoleMiddleware<S> {
    service: Rc<S>,
    required_role: Role,
}

impl<S, B> Service<ServiceRequest> for RequireRoleMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = ActixError;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let required_role = self.required_role.clone();
        let svc = self.service.clone();

        Box::pin(async move {
            let user_role = req
                .extensions()
                .get::<Role>()
                .cloned()
                .unwrap_or(Role::Guest);

            let access_granted = match user_role {
                Role::Guest => true,
                Role::User => matches!(required_role, Role::User | Role::Admin),
                Role::Admin => matches!(required_role, Role::Admin),
            };

            if access_granted {
                let res = svc.call(req).await?;
                Ok(res.map_into_left_body())
            } else {
                let (http_req, _) = req.into_parts();
                let res = HttpResponse::build(StatusCode::FORBIDDEN)
                    .body("Доступ запрещен");
                let res = ServiceResponse::new(http_req, res);
                Ok(res.map_into_right_body())  // Изменено на map_into_right_body()
            }
        })
    }
}