# SkillXchange

1. error[E0277]: the trait bound `register_user::{closure#0}::Record: handlers::auth::_::_serde::Serialize` is not satisfied
   --> src\handlers\auth.rs:35:45
    |
35  |         Ok(user) => HttpResponse::Ok().json(user),
    |                                        ---- ^^^^ the trait `handlers::auth::_::_serde::Serialize` is not implemented for `register_user::{closure#0}::Record`
    |                                        |
    |                                        required by a bound introduced by this call
    |
    = note: for local types consider adding `#[derive(serde::Serialize)]` to your `register_user::{closure#0}::Record` type
    = note: for types from other crates check whether the crate offers a `serde` feature flag
    = help: the following other types implement trait `handlers::auth::_::_serde::Serialize`:
              &'a T
              &'a mut T
              ()
              (T,)
              (T0, T1)
              (T0, T1, T2)
              (T0, T1, T2, T3)
              (T0, T1, T2, T3, T4)
            and 173 others
note: required by a bound in `HttpResponseBuilder::json`
   --> C:\Users\Nikita\.cargo\registry\src\index.crates.io-6f17d22bba15001f\actix-web-4.9.0\src\response\builder.rs:333:40
    |
333 |     pub fn json(&mut self, value: impl Serialize) -> HttpResponse {
    |                                        ^^^^^^^^^ required by this bound in `HttpResponseBuilder::json`

2. error[E0277]: the trait bound `actix_web_httpauth::extractors::bearer::Error: std::convert::From<AuthenticationError<actix_web_httpauth::headers::www_authenticate::bearer::Bearer>>` is not satisfied
  --> src\middleware\auth.rs:25:57
   |
25 |         Err(_) => Err(AuthenticationError::from(config).into()),
   |                                                         ^^^^ the trait `std::convert::From<AuthenticationError<actix_web_httpauth::headers::www_authenticate::bearer::Bearer>>` is not implemented for `actix_web_httpauth::extractors::bearer::Error`, which is required by `AuthenticationError<actix_web_httpauth::headers::www_authenticate::bearer::Bearer>: Into<_>`
   |
   = note: required for `AuthenticationError<actix_web_httpauth::headers::www_authenticate::bearer::Bearer>` to implement `Into<actix_web_httpauth::extractors::bearer::Error>`

В всех моделях есть эти ошибки:
3. error[E0277]: the trait bound `NaiveDateTime: handlers::auth::_::_serde::Serialize` is not satisfied
    --> src\models\category.rs:3:17
     |
3    | #[derive(Debug, Serialize, Deserialize)]
     |                 ^^^^^^^^^ the trait `handlers::auth::_::_serde::Serialize` is not implemented for `NaiveDateTime`
...
8    |     pub created_at: chrono::NaiveDateTime,
     |     --- required by a bound introduced by this call
     |
     = note: for local types consider adding `#[derive(serde::Serialize)]` to your `NaiveDateTime` type
     = note: for types from other crates check whether the crate offers a `serde` feature flag
     = help: the following other types implement trait `handlers::auth::_::_serde::Serialize`:
               &'a T
               &'a mut T
               ()
               (T,)
               (T0, T1)
               (T0, T1, T2)
               (T0, T1, T2, T3)
               (T0, T1, T2, T3, T4)
             and 172 others
note: required by a bound in `handlers::auth::_::_serde::ser::SerializeStruct::serialize_field`
    --> C:\Users\Nikita\.cargo\registry\src\index.crates.io-6f17d22bba15001f\serde-1.0.215\src\ser\mod.rs:1867:21
     |
1865 |     fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
     |        --------------- required by a bound in this associated function
1866 |     where
1867 |         T: ?Sized + Serialize;
     |                     ^^^^^^^^^ required by this bound in `SerializeStruct::serialize_field`
     = note: this error originates in the derive macro `Serialize` (in Nightly builds, run with -Z macro-backtrace for more info)
4. error[E0277]: the trait bound `NaiveDateTime: handlers::auth::_::_serde::Deserialize<'_>` is not satisfied
    --> src\models\category.rs:8:21
     |
8    |     pub created_at: chrono::NaiveDateTime,
     |                     ^^^^^^^^^^^^^^^^^^^^^ the trait `handlers::auth::_::_serde::Deserialize<'_>` is not implemented for `NaiveDateTime`
     |
     = note: for local types consider adding `#[derive(serde::Deserialize)]` to your `NaiveDateTime` type
     = note: for types from other crates check whether the crate offers a `serde` feature flag
     = help: the following other types implement trait `handlers::auth::_::_serde::Deserialize<'de>`:
               &'a JsonRawValue
               &'a [u8]
               &'a std::path::Path
               &'a str
               ()
               (T,)
               (T0, T1)
               (T0, T1, T2)
             and 185 others
note: required by a bound in `next_element`
    --> C:\Users\Nikita\.cargo\registry\src\index.crates.io-6f17d22bba15001f\serde-1.0.215\src\de\mod.rs:1732:12
     |
1730 |     fn next_element<T>(&mut self) -> Result<Option<T>, Self::Error>
     |        ------------ required by a bound in this associated function
1731 |     where
1732 |         T: Deserialize<'de>,
     |            ^^^^^^^^^^^^^^^^ required by this bound in `SeqAccess::next_element`

5. error[E0271]: expected `impl Future<Output = Result<ServiceRequest, Error>>` to be a future that resolves to `Result<ServiceRequest, (Error, ServiceRequest)>`, but it resolves to `Result<ServiceRequest, Error>`
   --> src/main.rs:22:20
    |
22  |         let auth = HttpAuthentication::bearer(middleware::auth::jwt_validator);
    |                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `Result<ServiceRequest, (..., ...)>`, found `Result<ServiceRequest, Error>`
    |
    = note: expected enum `Result<_, (actix_web::Error, ServiceRequest)>`
               found enum `Result<_, actix_web_httpauth::extractors::bearer::Error>`
note: required by a bound in `HttpAuthentication::<BearerAuth, F>::bearer`
   --> C:\Users\Nikita\.cargo\registry\src\index.crates.io-6f17d22bba15001f\actix-web-httpauth-0.8.2\src\middleware.rs:135:15
    |
135 |     O: Future<Output = Result<ServiceRequest, (Error, ServiceRequest)>>,
    |               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `HttpAuthentication::<BearerAuth, F>::bearer`
...
164 |     pub fn bearer(process_fn: F) -> Self {
    |            ------ required by a bound in this associated function

6. error[E0271]: expected `impl Future<Output = Result<ServiceRequest, Error>>` to be a future that resolves to `Result<ServiceRequest, (Error, ServiceRequest)>`, but it resolves to `Result<ServiceRequest, Error>`
   --> src/main.rs:38:27
    |
38  |                     .wrap(auth)
    |                      ---- ^^^^ expected `Result<ServiceRequest, (..., ...)>`, found `Result<ServiceRequest, Error>`
    |                      |
    |                      required by a bound introduced by this call
    |
    = note: expected enum `Result<_, (actix_web::Error, ServiceRequest)>`
               found enum `Result<_, actix_web_httpauth::extractors::bearer::Error>`
    = note: required for `HttpAuthentication<BearerAuth, fn(ServiceRequest, BearerAuth) -> impl Future<Output = ...> {jwt_validator}>` to implement `Transform<actix_web::scope::ScopeService, ServiceRequest>`
note: required by a bound in `actix_web::Scope::<T>::wrap`
   --> C:\Users\Nikita\.cargo\registry\src\index.crates.io-6f17d22bba15001f\actix-web-4.9.0\src\scope.rs:308:12
    |
295 |       pub fn wrap<M, B>(
    |              ---- required by a bound in this associated function
...
308 |           M: Transform<
    |  ____________^
309 | |                 T::Service,
310 | |                 ServiceRequest,
311 | |                 Response = ServiceResponse<B>,
312 | |                 Error = Error,
313 | |                 InitError = (),
314 | |             > + 'static,
    | |_____________^ required by this bound in `Scope::<T>::wrap`
    = note: the full name for the type has been written to 'G:\Программирование\Rust\SkillXchange\backend\target\debug\deps\backend.long-type-17974427427348645469.txt'
    = note: consider using `--verbose` to print the full type name to the console

7. error[E0271]: expected `impl Future<Output = Result<ServiceRequest, Error>>` to be a future that resolves to `Result<ServiceRequest, (Error, ServiceRequest)>`, but it resolves to `Result<ServiceRequest, Error>`
   --> src/main.rs:38:27
    |
38  |                     .wrap(auth)
    |                      ---- ^^^^ expected `Result<ServiceRequest, (..., ...)>`, found `Result<ServiceRequest, Error>`
    |                      |
    |                      required by a bound introduced by this call
    |
    = note: expected enum `Result<_, (actix_web::Error, ServiceRequest)>`
               found enum `Result<_, actix_web_httpauth::extractors::bearer::Error>`
    = note: required for `HttpAuthentication<BearerAuth, fn(ServiceRequest, BearerAuth) -> impl Future<Output = ...> {jwt_validator}>` to implement `Transform<actix_web::scope::ScopeService, ServiceRequest>`
note: required by a bound in `actix_web::Scope::<T>::wrap`
   --> C:\Users\Nikita\.cargo\registry\src\index.crates.io-6f17d22bba15001f\actix-web-4.9.0\src\scope.rs:308:12
    |
295 |       pub fn wrap<M, B>(
    |              ---- required by a bound in this associated function
...
308 |           M: Transform<
    |  ____________^
309 | |                 T::Service,
310 | |                 ServiceRequest,
311 | |                 Response = ServiceResponse<B>,
312 | |                 Error = Error,
313 | |                 InitError = (),
314 | |             > + 'static,
    | |_____________^ required by this bound in `Scope::<T>::wrap`
    = note: the full name for the type has been written to 'G:\Программирование\Rust\SkillXchange\backend\target\debug\deps\backend.long-type-17974427427348645469.txt'
    = note: consider using `--verbose` to print the full type name to the console

8. error[E0271]: expected `impl Future<Output = Result<ServiceRequest, Error>>` to be a future that resolves to `Result<ServiceRequest, (Error, ServiceRequest)>`, but it resolves to `Result<ServiceRequest, Error>`
   --> src/main.rs:24:9
    |
24  | /         App::new()
25  | |             .app_data(web::Data::new(pool.clone()))
26  | |             // Обслуживаем статические файлы из директории frontend/dist
27  | |             .service(Files::new("/", "./frontend/dist").index_file("index.html"))
...   |
39  | |                 // Добавьте ваши защищенные маршруты здесь
40  | |             )
    | |_____________^ expected `Result<ServiceRequest, (..., ...)>`, found `Result<ServiceRequest, Error>`
    |
    = note: expected enum `Result<_, (actix_web::Error, ServiceRequest)>`
               found enum `Result<_, actix_web_httpauth::extractors::bearer::Error>`
    = note: required for `HttpAuthentication<BearerAuth, fn(ServiceRequest, BearerAuth) -> impl Future<Output = ...> {jwt_validator}>` to implement `Transform<actix_web::scope::ScopeService, ServiceRequest>`
note: required by a bound in `actix_web::Scope::<T>::wrap`
   --> C:\Users\Nikita\.cargo\registry\src\index.crates.io-6f17d22bba15001f\actix-web-4.9.0\src\scope.rs:308:12
    |
295 |       pub fn wrap<M, B>(
    |              ---- required by a bound in this associated function
...
308 |           M: Transform<
    |  ____________^
309 | |                 T::Service,
310 | |                 ServiceRequest,
311 | |                 Response = ServiceResponse<B>,
312 | |                 Error = Error,
313 | |                 InitError = (),
314 | |             > + 'static,
    | |_____________^ required by this bound in `Scope::<T>::wrap`
    = note: the full name for the type has been written to 'G:\Программирование\Rust\SkillXchange\backend\target\debug\deps\backend.long-type-17974427427348645469.txt'
    = note: consider using `--verbose` to print the full type name to the console

