use reqwest::Url;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::{Request, Response};

pub struct Cors;

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        if request.method() != rocket::http::Method::Options {
            return;
        }
        eprintln!("CORS request");

        let origin = request.headers().get_one("Origin");
        response.set_header(Header::new(
            "Access-Control-Allow-Origin",
            "retronomicon.dev",
        ));

        if let Some(origin) = origin {
            if let Ok(origin_url) = Url::parse(origin) {
                let host = origin_url
                    .host_str()
                    .map_or(String::new(), |s| s.to_string());
                match host.as_str() {
                    "retronomicon.dev"
                    | "retronomicon.com"
                    | "api.retronomicon.com"
                    | "retronomicon.land"
                    | "www.retronomicon.land" => {
                        response.set_header(Header::new("Access-Control-Allow-Origin", host));
                    }
                    #[cfg(debug_assertions)]
                    "localhost" => {
                        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
                    }
                    _ => {}
                }
            }
        }

        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}
