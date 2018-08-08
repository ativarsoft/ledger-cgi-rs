extern crate ledger;
extern crate postgres;
extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate uuid;

use std::io::Write;
use ledger::*;

#[derive(Deserialize)]
struct InputPost {
	balance: f64,
	bolao: f64,
	bills: f64,
	coins: f64,
	checks: f64,
	armored_car: f64,
}

#[derive(Serialize)]
struct OutputError {
	success: bool,
	msg: &'static str,
}

#[derive(Serialize)]
struct OutputSuccessGet {
	success: bool,
	balance: f64,
	bolao: f64,
	bills: f64,
	coins: f64,
	checks: f64,
	armored_car: f64,
}

#[derive(Serialize)]
struct OutputSuccessPost {
	success: bool,
}

fn main() {
	// get environment variables
	let method = get_env("REQUEST_METHOD");
	let token = get_env("HTTP_X_AUTH_TOKEN");

	println!("Content-Type: application/json");
	println!("");
	
	// generate session token
    let token: uuid::Uuid = uuid::Uuid::parse_str(&*token).unwrap();

    // connect to database
	let conn_params = CONN_PARAMS;
    let conn = postgres::Connection::connect(conn_params, postgres::TlsMode::None).unwrap();
    
    // get user information
    let sql = "SELECT user_id FROM users INNER JOIN sessions ON users.id = sessions.user_id WHERE sessions.token = $1 AND time + '10 hours' > NOW()";
    let rows = conn.query(sql, &[&token]).unwrap();
    if rows.len() != 1 {
		let output = OutputError {
			success: false,
			msg: "Expired session. Please login again.",
		};
		let output: Vec<u8> = serde_json::to_vec(&output).unwrap();
		std::io::stdout().write(&output).unwrap();
		std::process::exit(1);
    }
    
    let row = rows.get(0);
    let user_id: i32 = row.get(0);
    
    if method == "GET" {
		// get products for company
		let sql = "SELECT balance, bolao, bills, coins, checks, armored_car FROM balance WHERE user_id=$1 AND data=current_date AND horario=(SELECT MAX(horario) FROM balance WHERE data=current_date)";
		let rows = conn.query(sql, &[&user_id]).unwrap();
		
		// output json
		let output = match rows.len() {
			0 => {
				let output = OutputSuccessGet {
					success: true,
					balance: 0.0,
					bolao: 0.0,
					bills: 0.0,
					coins: 0.0,
					checks: 0.0,
					armored_car: 0.0,
				};
				serde_json::to_vec(&output).unwrap()
			},
			1 => {
				let row = rows.get(0);
				let output = OutputSuccessGet {
					success: true,
					balance: row.get(0),
					bolao: row.get(1),
					bills: row.get(2),
					coins: row.get(3),
					checks: row.get(4),
					armored_car: row.get(5),
				};
				serde_json::to_vec(&output).unwrap()
			},
			_ => {
				let output = OutputError {
					success: false,
					msg: "Mais de um balanço encontrado.",
				};
				serde_json::to_vec(&output).unwrap()
			}
		};
		//let token = format!("{{{}}}", token);
		std::io::stdout().write(&output).unwrap();
	} else if method == "POST" {
		// retrieve and verify input
		let input: InputPost = serde_json::from_reader(std::io::stdin()).unwrap();
		
		let sql = "INSERT INTO balance (user_id, data, horario, balance, bolao, bills, coins, checks, armored_car) VALUES ($1, current_date, current_time, $2, $3, $4, $5, $6, $7)";
		conn.execute(sql, &[&user_id, &input.balance, &input.bolao, &input.bills, &input.coins, &input.checks, &input.armored_car]).unwrap();
		
		// output json
		let output = OutputSuccessPost {
			success: true,
		};
		let output = serde_json::to_vec(&output).unwrap();
		std::io::stdout().write(&output).unwrap();
	} else {
		println!("Status: 404");
		println!("");
		println!("Page not found.");
		std::process::exit(1);
	}
	println!("");
}
