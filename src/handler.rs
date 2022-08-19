use std::collections::HashMap;
use std::error::Error;
use crate::trend_request::*;
use hyper::{Body, Method, Request, Response, StatusCode};
use serde_json::{json, Value};

pub async fn handler(req: Request<Body>) -> Result<Response<Body>, Box<dyn Error + Send + Sync>> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/") => {
            println!("{}", req.uri().query().unwrap());
            println!("{:?}", req);
            let params: HashMap<String, String> = req
                .uri()
                .query()
                .map(|query| {
                    url::form_urlencoded::parse(query.as_bytes())
                        .into_owned()
                        .collect()
                })
                .unwrap_or_else(HashMap::new);
            println!("{:?}", params);
            let default_tz = &String::from("0");
            let default_hl = &String::from("en");
            let default_time = &String::from("now 7-d");
            let default_geo = &String::from("");
            //let default_keywords = &String::from("trender+plus");
            let req_params = ReqParams {
                base_url: "https://trends.google.com/trends/api/",
                tz: params.get("tz").unwrap_or(default_tz),
                hl: params.get("hl").unwrap_or(default_hl),
                time: params.get("time").unwrap_or(default_time),
                geo: params.get("geo").unwrap_or(default_geo),
                keywords: None
            };
            println!("{:?}", &req_params);

            let req_bytes = hyper::body::to_bytes(req.into_body()).await?;
            let req_json: Value = serde_json::from_slice(&req_bytes)?;
            if let Some(req_list) = list_keywords(&req_json) {
                println!("{:?}", &req_list);
                let trend_req = TrendRequest {
                    req_params,
                    req_list,
                };
                let trend_resp = trend_req.query().await?;
                //println!("{}", json!(trend_resp));
                let body = json!(trend_resp).to_string();
                //println!("{}", trend_req.query().await.unwrap());
                return Ok(Response::builder()
                    .status(StatusCode::OK)
                    .body(body.into())
                    .unwrap());
            } else {
                Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::empty())
                .unwrap())
            }

            
        }
        (&Method::GET, "/") => {
            println!("{}", req.uri().query().unwrap());
            println!("{:?}", req);
            let params: HashMap<String, String> = req
                .uri()
                .query()
                .map(|query| {
                    url::form_urlencoded::parse(query.as_bytes())
                        .into_owned()
                        .collect()
                })
                .unwrap_or_else(HashMap::new);
            println!("{:?}", params);
            let default_tz = &String::from("0");
            let default_hl = &String::from("en");
            let default_time = &String::from("now 7-d");
            let default_geo = &String::from("");
            let req_params = ReqParams {
                base_url: "https://trends.google.com/trends/api/",
                tz: params.get("tz").unwrap_or(default_tz),
                hl: params.get("hl").unwrap_or(default_hl),
                time: params.get("time").unwrap_or(default_time),
                geo: params.get("geo").unwrap_or(default_geo),
                keywords: None,
            };
            println!("{:?}", &req_params);

            let req_bytes = hyper::body::to_bytes(req.into_body()).await?;
            let req_json: Value = serde_json::from_slice(&req_bytes)?;
            if let Some(req_list) = list_keywords(&req_json) {
                println!("{:?}", &req_list);
                let trend_req = TrendRequest {
                    req_params,
                    req_list,
                };
                let trend_resp = trend_req.query().await?;
                //println!("{}", json!(trend_resp));
                let body = json!(trend_resp).to_string();
                //println!("{}", trend_req.query().await.unwrap());
                return Ok(Response::builder()
                    .status(StatusCode::OK)
                    .body(body.into())
                    .unwrap());
            } else {
                Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::empty())
                .unwrap())
            }

            
        }
        _ => Ok(Response::builder()
            .status(StatusCode::OK)
            .body(Body::empty())
            .unwrap()),
    }
}

fn list_keywords(req_json: &Value) -> Option<Vec<&str>> {
    Some(
        req_json
            .get("keywords")?
            .as_array()?
            .iter()
            .map(|value| value.as_str().unwrap_or("").trim_matches('"'))
            .collect(),
    )
}
