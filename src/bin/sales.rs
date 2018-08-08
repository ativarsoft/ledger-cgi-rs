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
	product_id: i32,
	amount: i32,
}

#[derive(Serialize)]
struct OutputError {
	success: bool,
	msg: &'static str,
}

#[derive(Serialize)]
struct OutputSuccessGet {
	date: String,
	product: String,
	amount: i32,
	id: i32,
	value: f64,
}

#[derive(Serialize)]
struct OutputSuccessPost {
	success: bool,
	sale_id: i32,
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
    let sql = "SELECT user_id, users.company, users.flags FROM users INNER JOIN sessions ON users.id = sessions.user_id WHERE sessions.token = $1 AND time + '10 hours' > NOW()";
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
    let company_id: i32 = row.get(1);
    let flags: i32 = row.get(2);
    
    if method == "GET" {
		let path_info = get_env("PATH_INFO");
		let mut path = path_info.split('/');
		let request_set = path.nth(1).unwrap();
		let request_user = path.next().unwrap();
		let request_date_start = path.next().unwrap();
		let request_date_end = path.next().unwrap();

		// get products for company
		/*let sql = "SELECT products.name, sales.amount, sales.id, products.value FROM sales INNER JOIN products ON sales.product_id = products.id WHERE user_id=$1 AND data=current_date ORDER BY horario";
		let rows = conn.query(sql, &[&user_id]).unwrap();*/
		
		let rows = if request_set == "user" {
			let sql = "SELECT to_char(data, 'YYYY-MM-DD'), products.name, sales.amount, sales.id, products.value FROM sales INNER JOIN products ON sales.product_id = products.id WHERE user_id = (SELECT id FROM users WHERE username = $1) AND data >= to_date($2, 'YYYY-MM-DD') AND data <= to_date($3, 'YYYY-MM-DD') ORDER BY data, products.name";
			conn.query(sql, &[&request_user, &request_date_start, &request_date_end]).unwrap()
		} else if request_set == "group" {
			return;
		} else if request_set == "company" && (flags & 1 != 0) {
			let sql = "SELECT products.name, sales.amount, sales.id, products.value FROM sales INNER JOIN products ON sales.product_id = products.id WHERE company_id = $1 AND data >= to_date($2, 'YYYY-MM-DD') AND data <= to_date($3, 'YYYY-MM-DD') ORDER BY horario";
			conn.query(sql, &[&company_id, &request_date_start, &request_date_end]).unwrap()
		} else {
			return;
		};
		
		// output json
		let mut output: Vec<OutputSuccessGet> = Vec::with_capacity(rows.len());
		for row in rows.iter() {
			output.push(OutputSuccessGet {
				date: row.get(0),
				product: row.get(1),
				amount: row.get(2),
				id: row.get(3),
				value: row.get(4),
			});
		}
		//let token = format!("{{{}}}", token);
		let output = serde_json::to_vec(&output).unwrap();
		std::io::stdout().write(&output).unwrap();
	} else if method == "POST" {
		// retrieve and verify input
		let input: InputPost = serde_json::from_reader(std::io::stdin()).unwrap();
		
		let sql = "INSERT INTO sales(id, data, horario, user_id, product_id, amount) VALUES (DEFAULT, current_date, current_time, $1, $2, $3) RETURNING id";
		conn.execute(sql, &[&user_id, &input.product_id, &input.amount]).unwrap();
		let sql = "UPDATE inventory SET amount = amount - $3 WHERE user_id = $1 AND product_id = $2 AND amount > 0";
		conn.execute(sql, &[&user_id, &input.product_id, &input.amount]).unwrap();
		
		// output json
		let output = OutputSuccessPost {
			success: true,
			sale_id: rows.get(0).get(0),
		};
		let output = serde_json::to_vec(&output).unwrap();
		std::io::stdout().write(&output).unwrap();
	} else if method == "DELETE" {
		let path_info = get_env("PATH_INFO");
		let mut path = path_info.split('/');
		let sales_id = path.nth(1).unwrap();
		let sales_id: i32 = sales_id.parse().unwrap();
		
		let sql = "DELETE FROM sales WHERE user_id=$1 AND id=$2 AND data=current_date";
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
