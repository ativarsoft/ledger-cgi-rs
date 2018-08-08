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
	description: String,
	value: f64,
}

#[derive(Serialize)]
struct OutputError {
	success: bool,
	msg: &'static str,
}

#[derive(Serialize)]
struct OutputSuccessGet {
	id: i32,
	date: String,
	description: String,
	value: f64,
}

#[derive(Serialize)]
struct OutputSuccessPost {
	success: bool,
	payment_id: i32,
}

#[derive(Serialize)]
struct OutputSuccessDelete {
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
		let path_info = get_env("PATH_INFO");
		let mut path = path_info.split('/');
		let request_set = path.nth(1).unwrap();
		let request_user = path.next().unwrap();
		let request_date_start = path.next().unwrap();
		let request_date_end = path.next().unwrap();

		let rows = if request_set == "user" {
			let sql = "SELECT id, to_char(data, 'YYYY-MM-DD'), description, value FROM payments WHERE user_id = (SELECT id FROM users WHERE username = $1) AND data >= to_date($2, 'YYYY-MM-DD') AND data <= to_date($3, 'YYYY-MM-DD') ORDER BY data, horario";
			conn.query(sql, &[&request_user, &request_date_start, &request_date_end]).unwrap()
		} else {
			return;
		};
		
		// output json
		let mut output: Vec<OutputSuccessGet> = Vec::with_capacity(rows.len());
		for row in rows.iter() {
			output.push(OutputSuccessGet {
				id: row.get(0),
				date: row.get(1),
				description: row.get(2),
				value: row.get(3),
			});
		}
		//let token = format!("{{{}}}", token);
		let output = serde_json::to_vec(&output).unwrap();
		std::io::stdout().write(&output).unwrap();
	} else if method == "POST" {
		// retrieve and verify input
		let input: InputPost = serde_json::from_reader(std::io::stdin()).unwrap();
		
		let sql = "INSERT INTO payments(id, data, horario, user_id, description, value) VALUES (DEFAULT, current_date, current_time, $1, $2, $3) RETURNING id";
		let rows = conn.query(sql, &[&user_id, &input.description, &input.value]).unwrap();
		
		// output json
		let output = OutputSuccessPost {
			success: true,
			payment_id: rows.get(0).get(0),
		};
		let output = serde_json::to_vec(&output).unwrap();
		std::io::stdout().write(&output).unwrap();
	} else if method == "DELETE" {
		let path_info = get_env("PATH_INFO");
		let mut path = path_info.split('/');
		let sales_id = path.nth(1).unwrap();
		let sales_id: i32 = sales_id.parse().unwrap();
		
		let sql = "DELETE FROM payments WHERE user_id=$1 AND id=$2 AND data=current_date";
		conn.execute(sql, &[&user_id, &sales_id]).unwrap();
		
		// output json
		let output = OutputSuccessDelete {
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
