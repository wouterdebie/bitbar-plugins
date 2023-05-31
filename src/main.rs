use anyhow::Error;
use anyhow::Result;
use chrono::prelude::*;
use chrono::Utc;
use chrono_tz::US::Eastern;
use prettytable::cell;
use prettytable::format;
use prettytable::row;
use prettytable::Table;

const API_URL: &str = "https://api.wsj.net/api/dylan/quotes/v2/comp/quoteByDialect?dialect=official&needed=CompositeTrading|TimeZoneInfo&MaxInstrumentMatches=1&accept=application/json&EntitlementToken=cecc4267a0194af89ca343805a3e57af&ckey=cecc4267a0&dialects=Charting&id=";

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
    let now = Utc::now().with_timezone(&Eastern);
    let current_time = now.time();

    let (top_green, top_red) = if current_time >= NaiveTime::from_hms_opt(6, 0, 0).unwrap()
        && current_time < NaiveTime::from_hms_opt(8, 00, 0).unwrap()
    {
        ("#136417", "#901D1D")
    } else {
        (TOP_GREEN, TOP_RED)
    };

    let url = format!("{}{}", API_URL, STOCKS.join(","));
    let data: serde_json::Value = reqwest::blocking::get(url)?.json()?;

    let mut table = Table::new();
    let format = format::FormatBuilder::new().padding(0, 1).build();
    table.set_format(format);

    for (i, stock) in data["InstrumentResponses"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        let matches = &stock["Matches"][0];
        let ct = &matches["CompositeTrading"];
        let ticker = matches["Instrument"]["Ticker"].as_str().unwrap();
        let last_price = &ct["Last"].as_object().unwrap()["Price"]["Value"];
        let high = &ct["High"]["Value"];
        let low = &ct["Low"]["Value"];
        let change_value = ct["NetChange"]["Value"].as_f64().unwrap();
        let pct = ct["ChangePercent"].as_f64().unwrap();
        let stype = matches["Instrument"].as_object().unwrap()["Types"][0]["Name"]
            .as_str()
            .unwrap();
        let last_trade = &ct["Last"]["Time"].as_str().unwrap();
        let tzi = &matches["TimeZoneInfo"];
        let offset_hours = tzi["UtcOffsetHours"].as_i64().unwrap();
        let offset_minutes = tzi["UtcOffsetMinutes"].as_i64().unwrap();

        let (symbol, color, top_color) = if change_value > 0.0 {
            ("▲", GREEN, top_green)
        } else {
            ("▼", RED, top_red)
        };

        let last_trade_with_offset =
            format!("{}{:+03}:{:02}", last_trade, offset_hours, offset_minutes);
        let naive_dt = DateTime::parse_from_rfc3339(&last_trade_with_offset).unwrap();
        let utc_dt = naive_dt.with_timezone(&Utc);

        // Check if the last trade was within the last minute
        let trading = Utc::now() - utc_dt < chrono::Duration::minutes(1);
        let mut row = if trading { row![" "] } else { row!["☾"] };

        row.add_cell(cell!(ticker));
        row.add_cell(cell!(r->format!("{:.2}", last_price)));
        row.add_cell(cell!(symbol));

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
