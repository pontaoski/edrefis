// SPDX-FileCopyrightText: 2024 Janet Blackquill <uhhadd@gmail.com>
//
// SPDX-License-Identifier: MPL-2.0

use std::marker::PhantomData;
use nanoserde::{DeJson, DeJsonErr, SerJson};
use quad_net::{http_request::{HttpError, Request, RequestBuilder}, web_socket::WebSocket};
use urlencoding::encode;

pub struct DecoderRequest<T: DeJson> {
    request: Request,

    marker: PhantomData<T>
}
#[derive(Debug)]
pub enum RequestError {
    Json(DeJsonErr),
    Http(HttpError),
}
impl<T: DeJson> DecoderRequest<T> {
    pub fn try_recv(&mut self) -> Option<Result<T, RequestError>> {
        self.request.try_recv()
            .map(|res| {
                res
                    .map_err(|it| RequestError::Http(it))
                    .and_then(|data| { T::deserialize_json(&data).map_err(|it| RequestError::Json(it)) })
            })
    }
}
impl<T: DeJson> From<Request> for DecoderRequest<T> {
    fn from(request: Request) -> Self {
        DecoderRequest { request, marker: PhantomData }
    }
}
pub struct Nakama {
    key: String,
    base_url: String,
}
#[derive(DeJson, Debug)]
pub struct Session {
    pub token: String,
    pub refresh_token: String,
}
impl Nakama {
    pub fn new(key: &str, base_url: &str) -> Nakama {
        Nakama {
            key: key.to_string(),
            base_url: base_url.to_string(),
        }
    }
    fn make_uri(&self, uri: &str) -> String {
        self.base_url.clone() + uri
    }
    pub fn authenticate_email(&self, email: &str, password: &str) -> DecoderRequest<Session> {
        #[derive(SerJson)]
        struct AuthenticateEmail<'a> {
            email: &'a str,
            password: &'a str,
        }

        RequestBuilder::new(&self.make_uri("/v2/account/authenticate/email"))
            .header("Content-Type", "application/json")
            .header("Authorization", "Basic ZGVmYXVsdGtleTo=")
            .method(quad_net::http_request::Method::Post)
            .body(
                &AuthenticateEmail { email, password }
                .serialize_json(),
            )
            .send()
            .into()
    }
}
