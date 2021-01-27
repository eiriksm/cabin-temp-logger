use core::{panic};
use std::{clone, env, fmt::{self, format}};
use env::var;
use serde::Deserialize;

#[derive(Deserialize)]
struct AuthCode {
  authorization_code : String
}

#[derive(Deserialize)]
struct AuthData {
  success: bool,
  data: AuthCode
}

#[derive(Deserialize)]
struct DeviceData {
  currentTemp : f32
}

#[derive(Deserialize)]
struct deviceListData {
  deviceList : Vec<DeviceData>
}

#[derive(Deserialize)]
struct RoomData {
  success: bool,
  data: deviceListData
}

#[derive(Deserialize)]
struct TokenData {
  success: bool,
  data: Token
}

#[derive(Deserialize)]
struct Token {
  access_token: String
}

#[derive(Deserialize)]
struct SeriesInstantDetailsData {
  air_temperature: f32
}

#[derive(Deserialize)]
struct SeriesInstantData {
  details: SeriesInstantDetailsData
}

#[derive(Deserialize)]
struct SeriesData {
  instant: SeriesInstantData
}

#[derive(Deserialize)]
struct TimeSeries {
  time: String,
  data: SeriesData
}

#[derive(Deserialize)]
struct TempProperties {
  timeseries: Vec<TimeSeries>
}

#[derive(Deserialize)]
struct TempData {
  properties: TempProperties
}

fn main() {
  let client = reqwest::blocking::Client::new();
  let access_key = env::var("MILL_ACCESS_KEY");
  let secret_token = env::var("MILL_SECRET_TOKEN");
  if access_key.is_err() {
    panic!("No MILL_ACCESS_KEY env var found");
  }
  if secret_token.is_err() {
    panic!("No MILL_SECRET_TOKEN env var found");
  }
  let resp = client.post("https://api.millheat.com/share/applyAuthCode")
    .header("access_key", access_key.unwrap())
    .header("secret_token", secret_token.unwrap())
    .send();
  if resp.is_err() {
    panic!("No auth code found in applyAuthCode request");
  }
  let response = resp.unwrap();
  let json = response.json::<AuthData>();
  let code = json.unwrap().data.authorization_code;
  let mill_password = env::var("MILL_PASSWORD");
  if mill_password.is_err() {
    panic!("No MILL_PASSWORD env var found");
  }
  let mill_username = env::var("MILL_USERNAME");
  if mill_username.is_err() {
    panic!("NO MILL_USERNAME env var found");
  }
  let body = &[("password", mill_password.unwrap()), ("username", mill_username.unwrap())];
  let resp = client.post("https://api.millheat.com/share/applyAccessToken")
    .header("authorization_code", code)
    .query(body)
    .send();
  if resp.is_err() {
    panic!("No access token found in applyAccessToken request");
  }
  let response = resp.unwrap();
  let json = response.json::<TokenData>();
  let code = json.unwrap().data.access_token;
  let room_id = env::var("MILL_ROOM_ID");
  if room_id.is_err() {
    panic!("No MILL_ROOM_ID env var");
  }
  let url = "https://api.millheat.com/uds/selectDevicebyRoom";
  let body = &[("roomId", room_id.unwrap())];
  let resp = client.post(url)
    .header("access_token", code)
    .query(body)
    .send();
  let data = resp.unwrap().json::<RoomData>().unwrap();
  // Now find the temperature in yr.
  let lon = env::var("YR_LON");
  if lon.is_err() {
    panic!("There was no YR_LON env var set");
  }
  let lat = env::var("YR_LAT");
  if lat.is_err() {
    panic!("There was no YR_LAT env var set");
  }
  let mut altitude = env::var("YR_ALTITUDE").unwrap_or_default();
  if altitude == "" {
    altitude = format!("0");
  }
  let url = "https://api.met.no/weatherapi/locationforecast/2.0/compact";
  let yr_body = &[("lat", lat.unwrap()), ("lon", lon.unwrap()), ("altitude", altitude)];
  let resp = client.get(url)
    .header("User-Agent", "Logging the cabin temperature with https://github.com/eiriksm/cabin-temp-logger")
    .query(yr_body)
    .send();
  let response = resp.unwrap();
  let json = response.json::<TempData>().unwrap();
  let temp = json.properties.timeseries[0].data.instant.details.air_temperature;
  // Right, now we should have all the details we need?
  let roomTemp = data.data.deviceList[0].currentTemp;
  let api_key = env::var("THINGSPEAK_API_KEY");
  if api_key.is_err() {
    panic!("No THINGSPEAK_API_KEY env var found");
  }
  let body = &[("api_key", api_key.unwrap()), ("field1", temp.to_string()), ("field2", roomTemp.to_string())];
  let tResp = client.get("https://api.thingspeak.com/update")
    .query(body)
    .send();
  let data = tResp.unwrap().text().unwrap();
}
