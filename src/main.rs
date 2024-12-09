//! # Title: Simple HTTP Server with PostgreSQL Integration
//! This module implements a simple HTTP server for user management with a PostgreSQL database.
//!
//! ## Features
//! - User CRUD operations (`POST`, `GET`, `PUT`, `DELETE`).
//! - JSON-based HTTP responses.
//! - Environment variable management for configuration.
//!
//! ## Author
//! ChakravarthyN 

use postgres::{Client,NoTls};
use postgres::Error as PostgresError;
use std::net::{TcpListener,TcpStream};
use std::io::{Read,Write};
use std::env;
use dotenv::dotenv; 

#[macro_use]
extern crate serde_derive;

/// # Model: User
/// Represents a user in the system.
///
/// ## Fields
/// - `id`: Optional user ID (primary key in the database).
/// - `name`: Name of the user.
/// - `email`: Email address of the user.
#[derive(Serialize,Deserialize)]
struct User {
    id: Option<i32>,
    name: String,
    email: String,
}

/// ## Constants
/// - `OK_RESPONSE`: HTTP 200 OK response with JSON content.
/// - `NOT_FOUND`: HTTP 404 Not Found response.
/// - `INTERNAL_SERVER_ERROR`: HTTP 500 Internal Server Error response. 
const OK_RESPONSE:&str = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n";
const NOT_FOUND:&str = "HTTP/1.1 404 Not Found\r\n\r\n";
const INTERNAL_SERVER_ERROR:&str = "HTTP/1.1 500 Internal Server Error\r\n\r\n";

/// # Main Function
/// Initializes the server and handles incoming client requests.
///
/// ## Steps
/// 1. Loads environment variables from a `.env` file.
/// 2. Retrieves the database URL from the environment.
/// 3. Sets up the PostgreSQL database.
/// 4. Starts the TCP server and listens on port 8080.
fn main() {
    dotenv().ok();
    let database_url = match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            println!("Error: DATABASE_URL must be set in environment");
            return;
        }
    };

    if let Err(e) = set_database(&database_url) { 
        println!("Error setting up database: {}", e);
        return;
    }

    let listener = TcpListener::bind(format!("0.0.0.0:8080")).unwrap();
    // Binds a TCP listener to the specified address and port and Dynamically creates the address string, program stops if the listener cannot bind.
    println!("Server listening on port 8080");

    //handle the client 
    for stream in listener.incoming() {
    // Loops over incoming TCP connections and handles each connection in turn.
        match stream {
            Ok(stream) => {
                let db_url = database_url.clone();
                handle_client(stream, &db_url);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}

/// # Function: handle_client
/// Handles an individual client connection.
///
/// ## Parameters
/// - `stream`: TCP connection stream from the client.
/// - `db_url`: Database connection string.
///
/// ## Behavior
/// Reads the client request, determines the HTTP method, and invokes the appropriate handler function.  
fn handle_client(mut stream: TcpStream, db_url: &str) {
// Here TCPStream means the Clients connection and it is mutable function reads from and writes to the stream.
    let mut buffer = [0; 1024]; // A fixed-size array to store raw data read from the stream
    let mut request = String::new(); // A String to build the client's request data as a readable format.
 
    match stream.read(&mut buffer){ 
    //reads data sent by the client into the buffer
        Ok(size) => { 
            request.push_str(String::from_utf8_lossy(&buffer[..size]).as_ref());
    //converts the raw data into a string and appends it to the request string. 
            let (status_line, content) = match &*request {
    //Determines the type of request and invokes the appropriate handler function.
                r if r.starts_with("POST /users") => handle_post_request(r, db_url),
    //r represents the input string which was constructed in the client's request data and if clause checks if the string is 
    // valid, starts_with method checks whether the string r begins with the specified substring.
                r if r.starts_with("GET /users/") => handle_get_request(r, db_url),
                r if r.starts_with("GET /users") => handle_get_all_request(r, db_url),
                r if r.starts_with("PUT /users") => handle_put_request(r, db_url),
                r if r.starts_with("DELETE /users/") => handle_delete_request(r, db_url),
                _ => (NOT_FOUND.to_string(),"404 Not Found".to_string()), // If No match found, returns a 404 Not Found response.
            };

            stream.write_all(format!("{}{}", status_line, content).as_bytes()).unwrap();
    //Combines the status_line and the response content into a single string and writes it back to the client.
        }
        // If reading from the stream fails, it logs the error but doesn't crash the program. 
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

/////////////////////////////////////// CONTROLLERS \\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\

//Handle_post_request function 
fn handle_post_request(request: &str, db_url: &str) -> (String, String) {
    match (get_user_request_body(&request), Client::connect(db_url, NoTls)) {
        // If the request body is successfully parsed and the database connection is established
        (Ok(user), Ok(mut client)) => {
            // Insert user data into the database
            client
                .execute(
                    "INSERT INTO users (name, email) VALUES ($1, $2)", // SQL query to insert user details
                    &[&user.name, &user.email],                        // Bind user values to the query
                )
                .unwrap(); // Ensure the program panics if the query fails
            // Return a successful HTTP response
            (OK_RESPONSE.to_string(), "user created".to_string())
        }
        // If parsing the request or connecting to the database fails
        _ => (
            INTERNAL_SERVER_ERROR.to_string(), // Return an HTTP 500 error
            "Error ".to_string(),              // Return a generic error message
        ),
    }
}


//Handle_get_request function
//Handles a GET request to retrieve a user by ID from the database and returns the result as JSON.
fn handle_get_request(request: &str, db_url: &str) -> (String, String) {
    match (get_id(request).parse::<i32>(), Client::connect(db_url, NoTls)){
        (Ok(id), Ok(mut client)) => 
        match client.query_one("SELECT * FROM users WHERE id = $1", &[&id]) {
            Ok(row) => {
                let user = User {
                    id: row.get(0),
                    name: row.get(1),
                    email: row.get(2),
                };
                (OK_RESPONSE.to_string(), serde_json::to_string(&user).unwrap())
            }
            _ => (NOT_FOUND.to_string(), "User not found".to_string()),
        }
        _ => (INTERNAL_SERVER_ERROR.to_string(), "Error".to_string()),
    }
}

// handle get all request function 
// Handles a GET request to retrieve all users from the database and returns the results as JSON. 
fn handle_get_all_request(_request: &str, db_url: &str) -> (String, String) {
    match Client::connect(db_url, NoTls) {
        Ok(mut client) => {
            let mut users = Vec::new();
            for row in client.query("SELECT * FROM users", &[]).unwrap() {
                users.push(User {
                    id: row.get(0),
                    name: row.get(1),
                    email: row.get(2),
                });
            }
            (OK_RESPONSE.to_string(), serde_json::to_string(&users).unwrap())
        }
        _ => (INTERNAL_SERVER_ERROR.to_string(), "Error".to_string()),
    }
} 

//Handle_put_request function
// Handles a PUT request to update an existing user's details in the database.
fn handle_put_request(request: &str, db_url: &str) -> (String, String) {
    match (get_id(&request).parse::<i32>(), // Parse the user ID from the request
    get_user_request_body(&request), // Extract user details from the request body
    Client::connect(db_url, NoTls)) { 
        (Ok(id), Ok(user), Ok(mut client)) => {
            client.execute("UPDATE users SET name = $1, email = $2 WHERE id = $3", &[&user.name, &user.email, &id]).unwrap();
            (OK_RESPONSE.to_string(), "User updated".to_string())
        }
        _ => (INTERNAL_SERVER_ERROR.to_string(), "Error".to_string()),
    }
}

//Handle_delete_request function
// Handles a DELETE request to remove a user by ID from the database.
fn handle_delete_request(request: &str, db_url: &str) -> (String, String) {
    match (get_id(&request).parse::<i32>(), Client::connect(db_url, NoTls)) {
        (Ok(id), Ok(mut client)) => {
             // Delete the user from the database 
            let rows_affected = client.execute(
                "DELETE FROM users WHERE id = $1", // SQL query for deletion
                 &[&id]  // Bind the user ID to the query
                ) 
                 .unwrap();
            if rows_affected == 0 {
                return (NOT_FOUND.to_string(), "User not found".to_string());
            }
            (OK_RESPONSE.to_string(), "User deleted".to_string())
        }
        _ => (INTERNAL_SERVER_ERROR.to_string(), "Error".to_string()),
    }
}

/// # Function: set_database
/// Sets up the PostgreSQL database.
///
/// ## Parameters
/// - `db_url`: Database connection string.
///
/// ## Behavior
/// - Creates a connection to the database.
/// - Ensures the `users` table exists.
///
/// ## Returns
/// - `Ok(())` if the setup is successful.
/// - `PostgresError` if there is an issue with the database. 
fn set_database(db_url: &str) -> Result<(), PostgresError> {
    //Connect to database
    let mut client = Client::connect(db_url, NoTls)?;
    //Create table
    client.batch_execute("
        CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            name VARCHAR NOT NULL,
            email VARCHAR NOT NULL
        )"
    )?;
    Ok(()) 
}

/// get_id function 
/// Extracts the ID from the request URL by splitting the string at '/' and retrieving the third segment.
fn get_id(request: &str) -> &str {
    request
        .split("/") // Split the request URL into parts using '/' as the delimiter
        .nth(2) // Get the third segment (index 2), which is the ID
        .unwrap_or_default() // Use an empty string if the segment doesn't exist
        .split_whitespace() // Further split the segment to isolate the ID (removing any trailing data)
        .next() // Get the first part after splitting (the actual ID)
        .unwrap_or_default() // Use an empty string if no valid part exists
}

/// Deserialize user from request body with the id 
//  Extracts the JSON body from the HTTP request and deserializes it into a `User` struct.
fn get_user_request_body(request: &str) -> Result<User, serde_json::Error> {
    serde_json::from_str(

        
        &request
            .split("\r\n\r\n") // Split the HTTP request into headers and body using the blank line delimiter
            .last() // Retrieve the last part, which is the body
            .unwrap_or_default(), // Use an empty string if no body exists
    )


}
