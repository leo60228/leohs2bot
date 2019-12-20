use anyhow::{bail, Result};
use base64::write::EncoderWriter;
use http_client::HttpClient;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::io::prelude::*;
use surf::Client;

struct BoxedError(pub surf::Exception);

impl fmt::Display for BoxedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl fmt::Debug for BoxedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl Error for BoxedError {
    fn description(&self) -> &str {
        self.0.description()
    }

    #[allow(deprecated)]
    fn cause(&self) -> Option<&dyn Error> {
        self.0.cause()
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.0.source()
    }
}

pub fn http_basic(user: &str, pass: &str) -> String {
    let mut buf = Vec::from("Basic ".to_string());
    let mut b64 = EncoderWriter::new(&mut buf, base64::STANDARD);
    write!(b64, "{}:{}", user, pass).expect("Writing to Vec should never fail");
    drop(b64);
    String::from_utf8(buf).unwrap()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Token(pub String);

pub async fn token(client: &Client<impl HttpClient>) -> Result<Token> {
    #[derive(Deserialize)]
    struct Response {
        pub access_token: Token,
    }

    #[derive(Serialize)]
    struct Body<'a> {
        pub grant_type: &'a str,
        pub username: &'a str,
        pub password: &'a str,
    }

    let username = dotenv::var("REDDIT_USERNAME")?;
    let password = dotenv::var("REDDIT_PASSWORD")?;

    let body = Body {
        grant_type: "password",
        username: &username,
        password: &password,
    };

    let mut resp = client
        .post("https://www.reddit.com/api/v1/access_token")
        .set_header("User-Agent", "leo60228's Homestuck^2 update bot")
        .set_header(
            "Authorization",
            http_basic(
                &dotenv::var("REDDIT_CLIENT_ID")?,
                &dotenv::var("REDDIT_CLIENT_SECRET")?,
            ),
        )
        .body_form(&body)?
        .await
        .map_err(BoxedError)?;

    if resp.status() == 200 {
        let Response { access_token } = resp.body_json().await?;
        Ok(access_token)
    } else {
        bail!(
            "http error: {}",
            resp.body_string().await.map_err(BoxedError)?
        )
    }
}
