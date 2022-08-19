use reqwest::header::{HeaderMap, COOKIE, HOST, SET_COOKIE};
use serde::Serialize;
use serde_json::Value;
use std::{error::Error, sync::Arc};
use tokio::sync::RwLock;

#[derive(Clone, Debug, Serialize)]
pub struct Trend<'a> {
    title: &'a str,
    data: Vec<Data>,
}

type TrendList<'a> = Vec<Trend<'a>>;

#[derive(Clone, Debug, Serialize)]
pub struct Data {
    time: u32,
    formatted_time: String,
    value: u8,
}
#[derive(Debug, Serialize)]
pub struct ZippedData {
    time: u32,
    formatted_time: String,
    value: Vec<u8>,
}
#[derive(Debug, Serialize)]
pub struct TrendResponse<'a> {
    titles: Vec<&'a str>,
    data: Vec<ZippedData>,
}
impl<'a> Trend<'a> {
    fn new() -> Self {
        Trend {
            title: "",
            data: Vec::<Data>::new(),
        }
    }
    fn find_max_value(&self) -> u8 {
        let mut max_value: u8 = 0;
        for data in self.data.iter() {
            if data.value > max_value {
                max_value = data.value;
            }
        }
        max_value
    }
}
#[derive(Debug)]
pub struct ReqParams<'a> {
    pub base_url: &'a str,
    pub tz: &'a str,
    pub hl: &'a str,
    pub time: &'a str,
    pub geo: &'a str,
    pub keywords: Option<Vec<&'a str>>
}
#[derive(Debug)]
pub struct TrendRequest<'a> {
    pub req_params: ReqParams<'a>,
    pub req_list: Vec<&'a str>,
}
impl<'a> TrendRequest<'a> {
    pub async fn query(self) -> Result<TrendResponse<'a>, Box<dyn Error + Send + Sync>> {
        let timing = Arc::new(RwLock::new(Vec::<(u32, String)>::new()));
        let set_timing = Arc::new(RwLock::new(false));
        let lists_main: Vec<Vec<&str>> = if self.req_list.len() > 5 {
            self.req_list.chunks(4).map(|s| s.into()).collect()
        } else {
            self.req_list.chunks(5).map(|s| s.into()).collect()
        };
        let mut max_trend_list = TrendList::new();
        let mut headers = HeaderMap::new();
        headers.insert(HOST, "trends.google.com".parse()?);
        let headers_clone = headers.clone();
        let client = reqwest::Client::new();
        let resp = client
            .get("https://trends.google.com")
            .headers(headers_clone)
            .send()
            .await?;
        let mut trend_list_full = TrendList::new();
        if let Some(set_cookie) = resp.headers().get(SET_COOKIE) {
            headers.insert(COOKIE, set_cookie.to_owned());
            for list in &lists_main {
                let headers_clone = headers.clone();
                let timing_clone = timing.clone();
                let set_timing_clone = set_timing.clone();
                let trend_list = request_data(
                    headers_clone,
                    &self.req_params,
                    list,
                    timing_clone,
                    set_timing_clone,
                )
                .await?;
                if self.req_list.len() <= 5 {
                    let mut titles = Vec::<&str>::new();
                    let mut zipped_data = Vec::<ZippedData>::new();
                    let len = trend_list[0].data.len();
                    for index in 0..len {
                        let mut data_vec = Vec::<u8>::new();
                        for trend in &*trend_list {
                            data_vec.push(trend.data[index].value);
                            if !titles.contains(&trend.title) {
                                titles.push(trend.title);
                            }
                        }
                        let time = timing.read().await[index].0;
                        let formatted_time = &timing.read().await[index].1;
                        zipped_data.push(ZippedData {
                            time,
                            formatted_time: formatted_time.to_string(),
                            value: data_vec,
                        });
                    }

                    let trend_response = TrendResponse {
                        titles,
                        data: zipped_data,
                    };
                    return Ok(trend_response);
                }
                let mut max_trend = Trend::new();
                for trend in trend_list {
                    if trend.find_max_value() > max_trend.find_max_value() {
                        max_trend = trend;
                    }
                }
                max_trend_list.push(max_trend);
            }
            while max_trend_list.len() > 1 {
                let max_trend_list_clone = max_trend_list.clone();
                let newlist: Vec<&str> =
                    max_trend_list_clone.into_iter().map(|a| a.title).collect();
                max_trend_list.clear();
                
                let lists: Vec<Vec<&str>> = if newlist.len() > 5 {
                    newlist.chunks(4).map(|s| s.into()).collect()
                } else {
                    newlist.chunks(5).map(|s| s.into()).collect()
                };
                for list in &lists {
                    let headers_clone = headers.clone();
                    let timing_clone = timing.clone();
                    let set_timing_clone = set_timing.clone();
                    let trend_list = request_data(
                        headers_clone,
                        &self.req_params,
                        list,
                        timing_clone,
                        set_timing_clone,
                    )
                    .await?;
                    let mut max_trend = Trend::new();
                    for trend in trend_list {
                        if trend.find_max_value() > max_trend.find_max_value() {
                            max_trend = trend;
                        }
                    }
                    max_trend_list.push(max_trend);
                }
            }
            for mut list in lists_main {
                let max_title = &max_trend_list[0].title;
                if !list.contains(max_title) {
                    list.push(max_title);
                }

                
                let headers_clone = headers.clone();
                let timing_clone = timing.clone();
                let set_timing_clone = set_timing.clone();
                let mut trend_list = request_data(
                    headers_clone,
                    &self.req_params,
                    &list,
                    timing_clone,
                    set_timing_clone,
                )
                .await?;

                trend_list_full.append(&mut trend_list);
                while trend_list_full
                    .iter()
                    .filter(|trend| &trend.title == max_title)
                    .count()
                    > 1
                {
                    if let Some(pos) = trend_list_full
                        .iter()
                        .position(|trend| &trend.title == max_title)
                    {
                        trend_list_full.remove(pos);
                    }
                }
            }
        }
        let mut titles = Vec::<&str>::new();
        let mut zipped_data = Vec::<ZippedData>::new();

        for index in 0..trend_list_full[0].data.len() {
            let mut data_vec = Vec::<u8>::new();
            for trend in &trend_list_full {
                data_vec.push(trend.data[index].value);
                if !titles.contains(&trend.title) {
                    titles.push(trend.title);
                }
            }
            let time = timing.read().await[index].0;
            let formatted_time = &timing.read().await[index].1;
            zipped_data.push(ZippedData {
                time,
                formatted_time: formatted_time.to_string(),
                value: data_vec,
            });
        }

        let trend_response = TrendResponse {
            titles,
            data: zipped_data,
        };
        Ok(trend_response)
    }
}

fn req_explore(list: &[&str], geo: &str, time: &str) -> String {
    let comparison_item: String = list
        .iter()
        .map(|s| {
            format!(
                "{{\"keyword\":\"{}\",\"geo\":\"{}\",\"time\":\"{}\"}},",
                s, geo, time
            )
        })
        .collect();
    format!(
        "{{\"comparisonItem\":[{}],\"category\":0,\"property\":\"\"}}",
        &comparison_item[..&comparison_item.len() - 1]
    )
}
async fn request_data<'a, 'b>(
    headers: HeaderMap,
    request_params: &ReqParams<'b>,
    list: &[&'a str],
    timing: Arc<RwLock<Vec<(u32, String)>>>,
    set_timing: Arc<RwLock<bool>>,
) -> Result<TrendList<'a>, Box<dyn Error + Send + Sync>>
where
    'b: 'a,
{
    let client = reqwest::Client::new();
    let headers_clone = headers.clone();
    let explore_url = format!(
        "{}explore?hl={}&tz={}&req={}",
        request_params.base_url,
        request_params.hl,
        request_params.tz,
        req_explore(list, request_params.geo, request_params.time)
    );
    let explore_resp = client
        .get(explore_url)
        .headers(headers_clone)
        .send()
        .await?;
    let raw_explore_resp = explore_resp.text().await?;
    let formatted_explore_resp = &raw_explore_resp[5..];
    let json_explore_resp: Value = serde_json::from_str(formatted_explore_resp)?;
    let token_raw = json_explore_resp["widgets"][0]["token"].to_string();
    let token = token_raw.trim_matches('"');
    let req = &json_explore_resp["widgets"][0]["request"];
    let data_url = format!(
        "{}widgetdata/multiline?hl={}&tz={}&req={}&token={}",
        request_params.base_url, request_params.hl, request_params.tz, req, token
    );
    let data_resp = client.get(data_url).headers(headers).send().await?;
    let raw_data_resp = data_resp.text().await?;
    let formatted_data_resp = &raw_data_resp[5..];
    let json_data_resp: Value = serde_json::from_str(formatted_data_resp)?;
    let points = json_data_resp["default"]["timelineData"]
        .as_array()
        .unwrap();
    if !*set_timing.read().await {
        for point in points {
            let point_time: u32 = point["time"].to_string().trim_matches('"').parse()?;
            let point_formatted_time = point["formattedAxisTime"].as_str().unwrap().trim_matches('\"').to_string();
            timing
                .write()
                .await
                .push((point_time, point_formatted_time));
        }
    }
    timing.write().await.sort();
    *set_timing.write().await = true;
    let mut trend_list = TrendList::new();
    for (index, keyword) in list.iter().enumerate() {
        let mut point_map_full = Vec::<Data>::new();
        for (point_index, point) in points.iter().enumerate() {
            let point_int: u8 = point["value"].as_array().unwrap()[index]
                .to_string()
                .parse()?;
            let point_time: u32 = timing.read().await[point_index].0;
            let point_formatted_time = timing.read().await[point_index].1.to_owned();
            let point_data = Data {
                time: point_time,
                formatted_time: point_formatted_time,
                value: point_int,
            };
            point_map_full.push(point_data);
        }
        let trend = Trend {
            title: keyword,
            data: point_map_full,
        };
        trend_list.push(trend);
    }
    Ok(trend_list)
}
