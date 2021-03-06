#!./venv/bin/python 
import requests
import datetime
import time
import os
import os.path
import time
import json
import sys
from pathlib import Path

STOCKS = ["Stock-US-DDOG", "Stock-US-SPOT", "Stock-US-BMRA", "Stock-US-TSLA", "Stock-US-NFLX",
          "Stock-US-AMZN", "Stock-US-GOOG", "Stock-US-FB", "Stock-US-NEWR",
          "Stock-US-WBA", "Index-US-DJIA", "Future-US-GOLD"]

FONT_SIZE = 12

home = str(Path.home())
save_file = "{:s}/.stocksave".format(home)

day = datetime.datetime.today().weekday()
now_time = int(datetime.datetime.now().time().strftime("%H%M"))

if os.path.exists(save_file) and (day >= 5 or now_time <= 829 or now_time >= 1530) and not time.time() - os.path.getmtime(save_file) > 12 * 3600:
    with open(save_file, "r") as f:
        data = json.load(f)
        prefix = "☾ "
else:
    try:
        data = requests.get('https://api.wsj.net/api/dylan/quotes/v2/comp/quoteByDialect?dialect=official&needed=CompositeTrading&MaxInstrumentMatches=1&accept=application/json&EntitlementToken=cecc4267a0194af89ca343805a3e57af&ckey=cecc4267a0&dialects=Charting&id={}'.format(",".join(STOCKS))).json()
    except:
        sys.exit("Unable to connect")

    with open(save_file, "w") as f:
        json.dump(data, f)
    prefix = ""

for i in data.get("InstrumentResponses"):
    matches = i.get("Matches")[0]
    ct = matches.get("CompositeTrading")
    ticker = matches.get("Instrument").get("Ticker")
    change_value = ct.get("NetChange").get("Value")
    last_price = ct.get("Last").get("Price").get("Value")
    pct = round(ct.get("ChangePercent"), 2)
    stype = matches.get("Instrument").get("Types")[0].get("Name")

    if change_value > 0:
        symbol = "▲"
        color = "#136417"
    else:
        symbol = "▼"
        color = "#901D1D"

    line = "{}{} {:.2f} {} {:.2f} {:.2f}% | color={} size={}".format(
        prefix, ticker, last_price, symbol, change_value, pct, color, FONT_SIZE)
    if i.get("RequestId") == STOCKS[0]:
        print(line)
        print("---")

    href = "https://www.marketwatch.com/investing/{}/{}".format(stype, ticker)
    print("{} href={}".format(line, href))
