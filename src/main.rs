use anyhow::Error;
use anyhow::Result;
use chrono::prelude::*;
use chrono::Utc;
use chrono_tz::US::Eastern;
use prettytable::cell;
use prettytable::format;
use prettytable::row;
use prettytable::Table;
use std::fs::File;

const API_URL: &str = "https://api.wsj.net/api/dylan/quotes/v2/comp/quoteByDialect?dialect=official&needed=CompositeTrading&MaxInstrumentMatches=1&accept=application/json&EntitlementToken=cecc4267a0194af89ca343805a3e57af&ckey=cecc4267a0&dialects=Charting&id=";

const STOCKS: [&str; 13] = [
    "Stock-US-DDOG",
    "Stock-US-RDHL",
    "Stock-US-SPOT",
    "Stock-US-BMRA",
    "Stock-US-TSLA",
    "Stock-US-NFLX",
    "Stock-US-AMZN",
    "Stock-US-GOOG",
    "Stock-US-META",
    "Stock-US-SNOW",
    "Stock-US-WBA",
    "Index-US-DJIA",
    "Future-US-GOLD",
];

const FONT_SIZE: &str = "12";
const GREEN: &str = "#2CE035";
const RED: &str = "#FF6565";
const TOP_GREEN: &str = "#2CE035";
const TOP_RED: &str = "#FF6565";

fn main() -> Result<(), Error> {
    let save_file_path = home::home_dir().unwrap().as_path().join(".stocksave");

    let now = Utc::now().with_timezone(&Eastern);
    let current_weekday = now.weekday().num_days_from_monday();
    let current_time = now.time();

    let trading = current_weekday >= 5
        || current_time <= NaiveTime::from_hms_opt(9, 29, 0).unwrap()
        || current_time >= NaiveTime::from_hms_opt(16, 5, 0).unwrap();

    let (top_green, top_red) = if current_time >= NaiveTime::from_hms_opt(6, 0, 0).unwrap()
        && current_time < NaiveTime::from_hms_opt(8, 00, 0).unwrap()
    {
        ("#136417", "#901D1D")
    } else {
        (TOP_GREEN, TOP_RED)
    };

    let (data, prefix): (serde_json::Value, &str) = match (trading, File::open(save_file_path)) {
        (true, Ok(file)) => {
            // Read file as json
            (serde_json::from_reader(file)?, "☾")
        }
        (true, Err(_)) | (false, _) => {
            let url = format!("{}{}", API_URL, STOCKS.join(","));
            (reqwest::blocking::get(url)?.json()?, "")
        }
    };

    let mut table = Table::new();
    let format = format::FormatBuilder::new().padding(0, 1).build();
    table.set_format(format);

    for (i, stock) in data["InstrumentResponses"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        let matches = &stock["Matches"].as_array().unwrap()[0];
        let ct = matches["CompositeTrading"].as_object().unwrap();
        let ticker = matches["Instrument"].as_object().unwrap()["Ticker"]
            .as_str()
            .unwrap();

        let last_price = ct["Last"].as_object().unwrap()["Price"]
            .as_object()
            .unwrap()["Value"]
            .as_f64()
            .unwrap();
        let high = ct["High"].as_object().unwrap()["Value"].as_f64().unwrap();
        let low = ct["Low"].as_object().unwrap()["Value"].as_f64().unwrap();

        let change_value = ct["NetChange"].as_object().unwrap()["Value"]
            .as_f64()
            .unwrap();

        let pct = ct["ChangePercent"].as_f64().unwrap();
        let stype = matches["Instrument"].as_object().unwrap()["Types"]
            .as_array()
            .unwrap()[0]["Name"]
            .as_str()
            .unwrap();

        let (symbol, color, top_color) = if change_value > 0.0 {
            ("▲", GREEN, top_green)
        } else {
            ("▼", RED, top_red)
        };

        let mut row = row![
            prefix,
            ticker,
            r->last_price,
            symbol,];

        if change_value >= 0.0 {
            row.add_cell(cell!(r->format!("+{:.2}", change_value)));
            row.add_cell(cell!(r->format!("+{:.2}%", pct)));
        } else {
            row.add_cell(cell!(r->format!("{:.2}", change_value)));
            row.add_cell(cell!(r->format!("{:.2}%", pct)));
        };

        row.add_cell(cell!(format!("({:.2} - {:.2})", low, high)));
        row.add_cell(cell!("|"));
        row.add_cell(cell!(format!("size={}", FONT_SIZE)));

        if prefix.is_empty() {
            row.remove_cell(0);
        }

        if i == 0 {
            row.add_cell(cell!(format!("color={}", top_color)));
            table.add_row(row.clone());
            table.add_row(row![H2->"---"]);
            row.remove_cell(row.len() - 1);
        }
        row.add_cell(cell!(format!("font='Hack Nerd Font'")));
        row.add_cell(cell!(format!("color={}", color)));
        row.add_cell(cell!(format!(
            "href=https://www.marketwatch.com/investing/{}/{}",
            stype, ticker
        )));
        table.add_row(row);
    }
    table.printstd();

    Ok(())
}
