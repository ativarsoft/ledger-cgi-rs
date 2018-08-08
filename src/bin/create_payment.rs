extern crate curl;
extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;

use std::io::Write;
use std::io::Read;

#[derive(Deserialize)]
struct PaypalAuthResponse {
	scope: String,
	access_token: String,
	token_type: String,
	app_id: String,
	expires_in: i32,
}

#[derive(Serialize)]
struct PaypalAmount {
	currency: &'static str,
	total: String,
}

#[derive(Serialize)]
struct PaypalItemList {
	items: Vec<PaypalItems>,
}

#[derive(Serialize)]
struct PaypalItems {
	name: &'static str,
	currency: &'static str,
	quantity: i32,
	price: String,
}

#[derive(Serialize)]
struct PaypalTransaction {
	amount: PaypalAmount,
	item_list: PaypalItemList,
	description: &'static str,
}

#[derive(Serialize)]
struct PaypalPayer {
	payment_method: &'static str,
}

#[derive(Serialize)]
struct PaypalRedirectUrls {
	return_url: &'static str,
	cancel_url: &'static str,
}

#[derive(Serialize)]
struct PaypalPayment {
	intent: &'static str,
	payer: PaypalPayer,
	redirect_urls: PaypalRedirectUrls,
	transactions: Vec<PaypalTransaction>,
}

/*{
    "intent": "sale",
    "payer": {
        "payment_method": "paypal"
    },
    "redirect_urls": {
        "return_url": "http://localhost",
        "cancel_url": "http://localhost"
    },
    "transactions": [
        {
            "amount": {
                "currency": "BRL",
                "total": "%.2f"
            },
            "item_list": {
                "items": [
                    {
                        "name": "Hospedagem de website",
                        "currency": "BRL",
                        "quantity": 1,
                        "price": "%.2f"
                    }
                ]
            },
            "description": "Serviço de hospedagem"
        }
    ]
}*/

fn get_env(key: &'static str) -> String {
	match std::env::var(key) {
		Ok(val) => val,
		Err(e) => {
			writeln!(std::io::stderr(), "Undefined environment variable {:?}: {}", key, e).unwrap();
			std::process::exit(1);
		},
	}
}

fn main() {
	let total = "1".to_string();
	let price = "1".to_string();
	let json = PaypalPayment {
		intent: "sale",
		payer: PaypalPayer {
			payment_method: "paypal"
		},
		redirect_urls: PaypalRedirectUrls {
			return_url: "http://localhost",
			cancel_url: "http://localhost",
		},
		transactions: vec![
			PaypalTransaction {
				amount: PaypalAmount {
					currency: "BRL",
					total: total
				},
				item_list: PaypalItemList {
					items: vec![
						PaypalItems {
							name: "Crédito",
							currency: "BRL",
							quantity: 1,
							price: price
						}
					],
				},
				description: "Crédito para jogo online",
			},
		],
	};
	
	// get environment variables
	let method = get_env("REQUEST_METHOD");

	println!("Content-Type: application/json");
	println!("");
	
	if method == "POST" {
		let mut response_data = Vec::new();
		let request_data = "grant_type=client_credentials";
		let mut request_data = request_data.as_bytes();

		// debug https
		//println!("request: {}", std::str::from_utf8(&request_data).unwrap());

		{
			let mut easy = curl::easy::Easy::new();
			easy.url("https://api.sandbox.paypal.com/v1/oauth2/token").unwrap();
			//easy.url("http://localhost").unwrap();
			let mut list = curl::easy::List::new();
			list.append("Accept: application/json").unwrap();
			list.append("Accept-Language: en_US").unwrap();
			easy.http_headers(list).unwrap();
			easy.username("AdMvhJjx8eNg7XDn1rnAh5kt-LAzf3_PMbeT7L9DYjVz_NVz0ZiXqHp7blIx5wzG-8Px0sg7lpE9bIvr").unwrap();
			easy.password("EH1fAebF3Ye380cFVozYc28kofy3RJhCSalRY3vNr49576uUjpgv45CEDXEz8CKQ3T-oyqkuKULU2pNj").unwrap();
			easy.post_field_size(request_data.len() as u64).unwrap(); // required: send Content-Length
			easy.post(true).unwrap();
			let mut transfer = easy.transfer();
			transfer.read_function(|buffer| {
				Ok(request_data.read(buffer).unwrap_or(0))
			}).unwrap();
			transfer.write_function(|buffer| {
				response_data.extend_from_slice(buffer);
				Ok(buffer.len())
			}).unwrap();
			transfer.perform().unwrap();
		}
		
		// debug https
		//println!("response: {}", std::str::from_utf8(&response_data).unwrap());

		let auth_response: PaypalAuthResponse = serde_json::from_slice(&response_data).unwrap();

		let mut response_data = Vec::new();
		let request_data: String = serde_json::to_string(&json).unwrap();
		let mut request_data = request_data.as_bytes();

		let authorization = "Authorization: Bearer ".to_string() + auth_response.access_token.as_str();
		let authorization = authorization.as_str();

		// debug https
		/*println!("authorization: {}", authorization);
		println!("request: {}", std::str::from_utf8(&request_data).unwrap());*/

		{
			let mut easy = curl::easy::Easy::new();
			easy.url("https://api.sandbox.paypal.com/v1/payments/payment").unwrap();
			//easy.url("http://localhost").unwrap();
			let mut list = curl::easy::List::new();
			list.append("Content-Type: application/json").unwrap();
			list.append(authorization).unwrap();
			easy.http_headers(list).unwrap();
			easy.post_field_size(request_data.len() as u64).unwrap(); // required: send Content-Length
			easy.post(true).unwrap();
			let mut transfer = easy.transfer();
			transfer.read_function(|buffer| {
				Ok(request_data.read(buffer).unwrap_or(0))
			}).unwrap();
			transfer.write_function(|buffer| {
				response_data.extend_from_slice(buffer);
				Ok(buffer.len())
			}).unwrap();
			transfer.perform().unwrap();
		}
		
		// output json
		std::io::stdout().write(&response_data).unwrap();
	} else {
		println!("Status: 404");
		println!("");
		println!("Page not found.");
		std::process::exit(1);
	}
	println!("");
}
