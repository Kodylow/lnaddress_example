use dotenv::dotenv;
use serde_derive::{Deserialize, Serialize};
use serde_json::from_str;
use serde_json::Value;
use warp::Filter;

#[derive(Debug, Deserialize)]
struct PayerDataDetails {
    mandatory: bool,
}

#[derive(Debug, Deserialize)]
struct PayerData {
    name: PayerDataDetails,
    email: PayerDataDetails,
    pubkey: PayerDataDetails,
}

#[derive(Debug, Deserialize)]
struct WellKnownResponse {
    status: String,
    tag: String,
    commentAllowed: u8,
    callback: String,
    metadata: String,
    minSendable: i64,
    maxSendable: i64,
    payerData: PayerData,
    nostrPubkey: String,
    allowsNostr: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct SuccessAction {
    tag: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct CallbackResponse {
    status: String,
    successAction: SuccessAction,
    verify: String,
    routes: Vec<Value>, // using Value here as the structure of routes is not provided
    pr: String,
}

#[tokio::main]
async fn main() {
    let api = warp::get()
        .and(warp::path!("api" / "invoice"))
        .and_then(handle_invoice);

    // GET / => index.html
    let index = warp::get()
        .and(warp::path::end())
        .and(warp::fs::file("./static/index.html"));

    let routes = index.or(api);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn handle_invoice() -> Result<impl warp::Reply, warp::Rejection> {
    // dotenv().ok();
    let lnaddress = String::from("kodylow@getalby.com"); // env::var("LNADDRESS").expect("LNADDRESS must be set");
    let (username, domain) = parse_lnaddress(&lnaddress);

    let client = reqwest::Client::new();
    let res = client
        .get(format!(
            "https://{}/.well-known/lnurlp/{}",
            domain, username
        ))
        .send()
        .await
        .expect("Failed to send request");

    let data: WellKnownResponse =
        from_str(&res.text().await.unwrap()).expect("Failed to parse response");

    let callback_res = client
        .get(format!("{}?amount=1000", data.callback))
        .send()
        .await
        .expect("Failed to send callback request");

    let response: CallbackResponse =
        from_str(&callback_res.text().await.unwrap()).expect("Failed to parse callback response");

    // return the response as json
    Ok(warp::reply::json(&response))
}

fn parse_lnaddress(lnaddress: &str) -> (&str, &str) {
    let mut parts = lnaddress.split('@');
    let username = parts.next().expect("Invalid LNADDRESS");
    let domain = parts.next().expect("Invalid LNADDRESS");
    (username, domain)
}
