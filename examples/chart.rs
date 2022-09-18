use chrono::{Date, DateTime, TimeZone, Utc};
use plotters::prelude::LabelAreaPosition::Bottom;
use plotters::prelude::*;
use serde_json::Value;
use std::{error::Error, fs};

#[derive(Debug)]
struct Trend<'a> {
    keyword: &'a str,
    data: Vec<i64>,
}
type TrendList<'a> = Vec<Trend<'a>>;
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = reqwest::Client::new();
    let _keywords = fs::read("keywords.json").unwrap();
    /* let response = client
    .post("http://127.0.0.1:8000/api/?time=today+12-m")
    .body(keywords)
    .send(); */
    let response = client
        .get("http://127.0.0.1:8000/api/?time=today+5-y&keywords=Rust,Blockchain")
        .send();

    let raw_response = response.await.unwrap().text().await.unwrap();
    println!("{}", raw_response);
    let json_data: Value = serde_json::from_str(raw_response.as_str()).unwrap();
    let data = json_data["data"].as_array().unwrap();
    let titles = json_data["titles"].as_array().unwrap();
    let mut time = Vec::<i64>::new();
    //let mut time = Vec::<&str>::new();
    let mut formatted_time = Vec::<DateTime<Utc>>::new();
    let mut trendlist = TrendList::new();
    for (index, title) in titles.iter().enumerate() {
        let mut trend = Trend {
            keyword: title.as_str().unwrap(),
            data: Vec::<i64>::new(),
        };
        for record in data {
            trend
                .data
                .push(record["value"].as_array().unwrap()[index].as_i64().unwrap());
        }
        trendlist.push(trend);
    }
    for record in data {
        time.push(record["time"].as_i64().unwrap());
        formatted_time.push(Utc.timestamp(record["time"].as_i64().unwrap(), 0));
    }
    //println!("{:?}", trendlist);
    let root_area = SVGBackend::new("chart.svg", (800, 480)).into_drawing_area();
    root_area.fill(&WHITE).unwrap();
    let x_range = formatted_time[0]..formatted_time[formatted_time.len() - 1];
    let mut ctx = ChartBuilder::on(&root_area)
        .margin(25)
        .set_label_area_size(LabelAreaPosition::Left, 40)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .build_cartesian_2d(x_range, 0..100)
        .unwrap();
    let style = TextStyle::from(("sans-serif", 10).into_font());
    ctx.configure_mesh()
        .light_line_style(&WHITE)
        .x_desc("Time")
        .y_desc("Popularity")
        .axis_desc_style(("sans-serif", 20))
        .x_label_style(style)
        .x_labels(6)
        .draw()
        .unwrap();
    for (idx, trend) in trendlist.iter().enumerate() {
        let color = Palette99::pick(idx).mix(0.9);
        ctx.draw_series(LineSeries::new(
            formatted_time
                .iter()
                .zip(trend.data.iter())
                .map(|(x, y)| (*x, *y as i32)),
            color.stroke_width(3),
        ))
        .unwrap()
        .label(trend.keyword)
        .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 25, y)], color.filled()));
    }
    ctx.configure_series_labels()
        .label_font(("sans-serif", 20))
        .background_style(&WHITE)
        .border_style(&BLACK)
        .draw()
        .unwrap();
    Ok(())
}
