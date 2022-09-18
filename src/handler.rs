use crate::chart::make_chart;
use crate::trend_request::*;
use hyper::{Body, Method, Request, Response, StatusCode};
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde_json::{json, Value};
use std::error::Error;
use std::{collections::HashMap, fs};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub async fn handler(req: Request<Body>) -> Result<Response<Body>, Box<dyn Error + Send + Sync>> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/api/") => {
            //println!("{}", req.uri().query().unwrap());
            //println!("{:?}", req);
            let params: HashMap<String, String> = req
                .uri()
                .query()
                .map(|query| {
                    url::form_urlencoded::parse(query.as_bytes())
                        .into_owned()
                        .collect()
                })
                .unwrap_or_else(HashMap::new);
            //println!("{:?}", params);
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
                //keywords: None
            };
            //println!("{:?}", &req_params);

            let req_bytes = hyper::body::to_bytes(req.into_body()).await?;
            if let Ok(req_json) =
                serde_json::from_slice(&req_bytes) as Result<Value, serde_json::Error>
            {
                if let Some(req_list) = list_keywords(&req_json) {
                    //println!("{:?}", req_list);
                    if !req_list.is_empty() && !req_list.contains(&"") {
                        let trend_req = TrendRequest {
                            req_params,
                            req_list,
                        };
                        let trend_resp = trend_req.query().await?;
                        //println!("{}", json!(trend_resp));
                        let body = json!(trend_resp).to_string();
                        //println!("{}", trend_req.query().await.unwrap());
                        Ok(Response::builder()
                            .status(StatusCode::OK)
                            .body(body.into())?)
                    } else {
                        //println!("{:?}", req_list);
                        let body = "JSON contains empty keywords. Post request should contain json: {\"keywords\": [your \"quoted\" keywords comma separated]}";
                        Ok(Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(body.into())?)
                    }
                } else {
                    let body = "No keywords. Post request should contain json: {\"keywords\": [your \"quoted\" keywords comma separated]}";
                    Ok(Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(body.into())
                        .unwrap())
                }
            } else {
                let body = "JSON format error. Post request should contain json: {\"keywords\": [your \"quoted\" keywords comma separated]}";
                Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(body.into())
                    .unwrap())
            }
        }

        (&Method::GET, "/api/") => {
            //println!("{:?}", req);
            let params: HashMap<String, String> = req
                .uri()
                .query()
                .map(|query| {
                    url::form_urlencoded::parse(query.as_bytes())
                        .into_owned()
                        .collect()
                })
                .unwrap_or_else(HashMap::new);
            //println!("{:?}", params);

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
            };
            if let Some(keywords) = params.get("keywords") {
                let req_list: Vec<&str> = keywords
                    .split(',')
                    .map(|word| word.trim_matches('"'))
                    .collect();
                //println!("{:?}", req_list);
                if !req_list.contains(&"") {
                    let trend_req = TrendRequest {
                        req_params,
                        req_list,
                    };
                    let trend_resp = trend_req.query().await?;
                    let body = json!(trend_resp).to_string();
                    Ok(Response::builder()
                        .status(StatusCode::OK)
                        .body(body.into())?)
                } else {
                    let body = "Keyword format error. Get request sould contain ../api/?keywords=your keywords comma separated";
                    Ok(Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(body.into())?)
                }
            } else {
                let body = "No keywords. Get request sould contain ../api/?keywords=your keywords comma separated";
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .body(body.into())?)
            }
        }

        (&Method::GET, "/") => {
            let page = fs::read_to_string("index.html").unwrap();
            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(page.into())
                .unwrap())
        }
        (&Method::GET, "/use") => {
            let page = fs::read_to_string("use.html").unwrap();
            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(page.into())
                .unwrap())
        }
        (&Method::GET, "/about") => {
            let page = fs::read_to_string("about.html").unwrap();
            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(page.into())
                .unwrap())
        }
        (&Method::GET, "/request") => {
            //let client = reqwest::Client::new();
            //println!("{:?}",req);
            let filename = format!("{}.svg", generate_string());
            let web_request = format!("http://127.0.0.1:8000/api/?{}", req.uri().query().unwrap());
            match make_chart(&web_request, &filename).await {
                Ok(_) => {
                    let page = fs::read_to_string("index.html").unwrap();
                    let new_page = page.replace("chart.svg", &filename);
                    Ok(Response::builder()
                        .status(StatusCode::OK)
                        .body(new_page.into())
                        .unwrap())
                }
                Err(_) => {
                    let page = fs::read_to_string("index.html").unwrap();
                    Ok(Response::builder()
                        .status(StatusCode::OK)
                        .body(page.into())
                        .unwrap())
                }
            }
        }
        (&Method::GET, "/chart.svg") => {
            let mut f = File::open("chart.svg").await.unwrap();
            let mut source = Vec::new();
            f.read_to_end(&mut source).await?;
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "image/svg+xml")
                .body(source.into())
                .unwrap())
        }
        (&Method::GET, path) => {
            if path.contains(".svg") && path != "chart.svg" {
                //println!("{}",&path.to_string()[1..]);
                let mut file = File::open(&path.to_string()[1..]).await?;
                let mut source = Vec::new();
                file.read_to_end(&mut source).await?;
                fs::remove_file(&path.to_string()[1..])?;
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "image/svg+xml")
                    .body(source.into())?)
            } else {
                let page = fs::read_to_string("index.html").unwrap();
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .body(page.into())
                    .unwrap())
            }
        }
        _ => Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
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
fn generate_string() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect()
}
