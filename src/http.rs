use std::io::Cursor;
use rocket::Response;
use url::Url;
use rocket::{Request, Data};
use rocket::handler::{Handler, Outcome};
use rocket::response::content;
use rocket::http::ContentType;

type Error = Box<dyn std::error::Error>;
type Result<T, E = Error> = std::result::Result<T, E>;

pub async fn get_json(url: &String) -> Result<serde_json::Value> {
    let url = Url::parse(&url)?;

    let resp = reqwest::get(url)
        .await?
        .json::<serde_json::Value>()
        .await?;

    return Ok(resp);
}

#[derive(Clone)]
pub struct JSONHandler{ 
    pub data: String 
}

impl Handler for JSONHandler {
    fn handle<'r>(&self, req: &'r Request, _data: Data) -> Outcome<'r> {
        let mut response = Response::new();
        response.set_header(ContentType::JSON);
        response.set_sized_body(Cursor::new(self.data.clone()));
        return Outcome::from(req, response);
    }
}
