use worker::*;

mod openweathermapapi;
mod utils;

use openweathermapapi::OneCall;

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or("unknown region".into())
    );
}

struct GeographicData {
    lat: f32,
    long: f32,
    city: String,
    country: String,
    region: String,
}

fn parse_geographic_cf_data(req: &Request) -> Option<GeographicData> {
    let city = req.cf().city()?;
    let country = req.cf().country()?;
    let region = req.cf().region()?;
    let (lat, long) = req.cf().coordinates()?;

    Some(GeographicData {
        lat,
        long,
        city,
        region,
        country,
    })
}

async fn get_geographic_data(geo: &GeographicData, api_key: &str) -> Option<OneCall> {
    const WEATHER_ENDPOINT: &str = "https://api.openweathermap.org/data/2.5/onecall";
    let lat = geo.lat;
    let lon = geo.long;
    let unit_type = "imperial";
    let exclude_parts = "minutely,hourly,alerts";
    let weather_query_string = format!("{WEATHER_ENDPOINT}?lat={lat}&lon={lon}&units={unit_type}&exclude={exclude_parts}&appid={api_key}");

    let resp = match reqwest::get(weather_query_string).await {
        Ok(s) => s,
        Err(_) => return None,
    };
    resp.json().await.unwrap()
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);

    // Optionally, get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    let mut known_location = true;
    let default_geographic_data = GeographicData {
        city: "Ames".parse().unwrap(),
        country: "US".parse().unwrap(),
        region: "Iowa".parse().unwrap(),
        lat: 42.03,
        long: -93.62,
    };
    let geographic_data = match parse_geographic_cf_data(&req) {
        Some(g_data) => g_data,
        None => {
            known_location = false;
            default_geographic_data
        }
    };

    let weather_api_token = env.secret("WEATHER_API_KEY")?.to_string();

    let weather_data = match get_geographic_data(&geographic_data, &weather_api_token).await {
        Some(weather_data) => weather_data,
        None => return Response::error("no weather data available", 400),
    };

    let html_style = r##"body{padding:6em; font-family: sans-serif;} h1{color:#f6821f}"##;

    let geo = geographic_data;
    let first_day = &weather_data.daily.unwrap()[0];
    let mut html_content = String::with_capacity(1024);
    html_content.push_str(r"<h1>Weather 🌦: Rust + WASM + Cloudflare Workers</h1>");
    html_content.push_str(r#"<p>This demo uses weather data from <a href="https://openweathermap.org" target="_blank">openweathermap.org</a>.</p>"#);
    html_content.push_str(r#"<p>Source for the demo available <a href="https://github.com/gurgalex/rustwasm-cf-workers-openweathermap" target="_blank">here</a>.</p>"#);
    if !known_location {
        html_content.push_str("<p><b>Unable to determine location, using Ames as default.</b></p>");
    } else {
        html_content.push_str("<p>Your location is estimated using Cloudflare edge servers.</p>");
    };

    html_content.push_str(&*format!(
        "Showing weather data for: Lat: {}, Long: {}.</p>",
        weather_data.lat, weather_data.lon
    ));
    html_content.push_str(&*format!(
        "<p>Weather data for city: {}, region: {}, country: {}</p>",
        geo.city, geo.region, geo.country
    ));
    html_content.push_str(&*format!(
        "<p>The current temperature is: {}°F.</p>",
        weather_data.current.unwrap().temp
    ));
    html_content.push_str(&*format!(
        "<p>Daily forecast: morning: {}, day: {}, evening: {}, night: {}",
        first_day.temp.morn, first_day.temp.day, first_day.temp.eve, first_day.temp.night
    ));
    let html = format!(
        r###"
    <!DOCTYPE html>
        <head>
        <title>Geolocation: Weather</title>
        <meta charset="UTF-8">
        </head>
        <body>
        <style>{html_style}</style>
        <div id="container">
    {html_content}
    </div>
        </body>
    "###
    );

    return Response::from_html(html);
}
