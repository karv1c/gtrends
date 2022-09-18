use chrono::{DateTime, TimeZone, Utc};
use plotters::prelude::*;
use serde_json::Value;
use std::{error::Error};

#[derive(Debug)]
struct TrendChart<'a> {
    keyword: &'a str,
    data: Vec<i64>,
}
type TrendListChart<'a> = Vec<TrendChart<'a>>;
pub async fn make_chart(params: &str, filename: &str) -> Result<(), Box<dyn Error>> {
    //println!("{}",params);
    let client = reqwest::Client::new();
    //let keywords = fs::read("keywords.json").unwrap();
    let response = client.get(params).send().await?;
    let raw_response = response.text().await?;
    //println!("{}", raw_response);
    let json_data: Value = serde_json::from_str(raw_response.as_str())?;
    let data = json_data["data"].as_array().unwrap();
    let titles = json_data["titles"].as_array().unwrap();
    let mut time = Vec::<i64>::new();
    let mut formatted_time = Vec::<DateTime<Utc>>::new();
    let mut trendlist = TrendListChart::new();
    for (index, title) in titles.iter().enumerate() {
        let mut trend = TrendChart {
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
    //println!("{:?}", time);
    //let resopnse_chart_file = format!("{}.svg", id);
    let root_area = SVGBackend::new(&filename, (800, 480)).into_drawing_area();
    root_area.fill(&WHITE)?;
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
        .draw()?;
    for (idx, trend) in trendlist.iter().enumerate() {
        let color = Palette99::pick(idx).mix(0.9);
        ctx.draw_series(LineSeries::new(
            formatted_time
                .iter()
                .zip(trend.data.iter())
                .map(|(x, y)| (*x, *y as i32)),
            color.stroke_width(3),
        ))?
        .label(trend.keyword)
        .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 25, y)], color.filled()));
    }
    ctx.configure_series_labels()
        .label_font(("sans-serif", 20))
        .background_style(&WHITE)
        .border_style(&BLACK)
        .draw()?;
    Ok(())
}
