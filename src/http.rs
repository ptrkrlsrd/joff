use url::Url;
use rocket::{Request, Data};
use rocket::handler::{Handler, Outcome};

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
        return Outcome::from(req, self.data.clone());
    }
}
