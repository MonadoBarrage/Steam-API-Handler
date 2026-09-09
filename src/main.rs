mod structs;

use std::{env, io::{BufReader, prelude::*}, net::{TcpListener, TcpStream}};
use std::result::Result;
use serde_json::to_string;
use steam_api_handler::ThreadPool;
use structs::*;
const WORKERS_SIZE: usize = 8;

fn main() {

    let port_var = "STEAM_API_HANDLER_IP_PORT";
    let tcp_listener_address = match env::var(port_var){
        Ok(val) => {
            println!("Loaded port value");
            val
        },
        Err(e) => {
            panic!("Error loading port value. Set port in environment variable STEAM_API_HANDLER_PORT: {:#?}", e);
        },
    };

    let key_var = "STEAM_KEY";
    let steam_key= match env::var(key_var) {
        Ok(val) => {
            println!("Loaded Steam Key");
            val
        },
        Err(e) => {
            panic!("Error loading Steam Key. Set Steam key in environmental variable STEAM_KEY: {:#?}", e);
        },
    };

    let listener_result = TcpListener::bind("127.0.0.1:".to_owned() + &*tcp_listener_address.to_owned());
    let listener = match listener_result {
        Ok(listener) => listener,
        Err(error) => panic!("Problem creating the TCP Listener {:#?}", error)
    };
    let pool = ThreadPool::new(WORKERS_SIZE);

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(stream) => stream,
            Err(error  ) => {
                eprintln!("Error processing incoming connection: {:#?}", error);
                continue;
            }
        };

        let copied_steam_key = steam_key.clone();
        pool.execute(|| {
            handle_connection(stream, copied_steam_key);
        })
    }
}


fn handle_connection(stream: TcpStream, steam_key: String) {

    let buf_reader = BufReader::new(&stream);

    let request_result = match buf_reader.lines().next() {
        Some(request_result) => request_result,
        None => {
            eprintln!("Error reading from stream.");
            send_response(stream, "HTTP/1.1 400 BAD REQUEST".to_owned(), "Request is invalid".to_owned());
            return
        }
    };

    let request_line = match request_result {
        Ok(request_line) => request_line,
        Err(error) => {
            eprintln!("Error reading from stream: {:#?}", error);
            send_response(stream, "HTTP/1.1 400 BAD REQUEST".to_owned(), "Request is invalid".to_owned());
            return;
        }
    };

    let parsed_request: Vec<&str> = request_line.split_whitespace().collect();

    if parsed_request.len() != 3 {
        eprintln!("Error parsing through HTTP request.");
        send_response(stream, "HTTP/1.1 400 BAD REQUEST".to_owned(), "Parameters invalid".to_owned());
        return;
    }

    if parsed_request[0] != "GET" {
        eprintln!("Error parsing through HTTP request. Only GET is supported.");
        send_response(stream, "HTTP/1.1 405 METHOD NOT ALLOWED".to_owned(), "Only GET is allowed".to_owned());
        return;
    }

    let parsed_http_resource: Vec<&str> = parsed_request[1].split("?id=").collect();

    if parsed_http_resource.len() != 2 {
        eprintln!("Error parsing through HTTP request. id parameter not found within request");
        send_response(stream, "HTTP/1.1 400 BAD REQUEST".to_owned(), "Parameters invalid".to_owned());
        return;
    }

    let steam_id = &parsed_http_resource[1];

    let (status_line, contents) = match parsed_http_resource[0] {
        "/owned-games" => {

            let request_link = "https://api.steampowered.com/IPlayerService/GetOwnedGames/v1/".to_owned()
                + "?include_played_free_games=true&include_free_sub=true&key=" +
                &steam_key + "&steamid=" + &steam_id;
            let body = match send_steam_request(request_link){
                Ok(body) => body,
                Err(e) => {
                    eprintln!("Error sending steam request. {:#?}", e);
                    send_response(stream, "HTTP/1.1 502 BAD GATEWAY".to_owned(), e.to_owned());
                    return
                }
            };

            let games_list: OwnedGames = match serde_json::from_str(&body.to_owned()) {
                Ok(games_list) => games_list,
                Err(e) => {
                    eprintln!("Error deserializing steam response. {:#?}", e);
                    send_response(stream, "HTTP/1.1 500 INTERNAL SERVER ERROR".to_owned(),
                                  "Cannot process game data".to_owned());
                    return;
                }
            };

            let game_ids: Vec<GameDetails> = games_list.response.games.into_iter()
                .map(|games| GameDetails {
                    appid: games.appid
                }).collect();
            let input_json: ItemDataJson = ItemDataJson {
                ids: game_ids,
                context: ContextDetails {
                    language: "english".to_owned(),
                    country_code: "us".to_owned()
                },
                data_request: DataRequestDetails {
                    include_assets: true
                }
            };
            let input_json_string = match to_string(&input_json){
                Ok(games_ids_json_string) => {
                    games_ids_json_string
                },
                Err(error) => {
                    eprintln!("Error converting input json to string: {:#?}", error);
                    send_response(stream, "HTTP/1.1 500 INTERNAL SERVER ERROR".to_owned(),
                                  "Cannot process game data".to_owned());
                    return;
                }
            };

            let request_link2 = "https://api.steampowered.com/IStoreBrowseService/GetItems/v1/?key=".to_owned() +
                &steam_key + "&input_json=" + input_json_string.as_str();
            match send_steam_request(request_link2){
                Ok(body) => ("HTTP/1.1 200 OK", body),
                Err(e) => {
                    eprintln!("Error sending steam request. {:#?}", e);
                    send_response(stream, "HTTP/1.1 502 BAD GATEWAY".to_owned(), e.to_owned());
                    return;
                }
            }
        },

        "/badges-and-levels" => {
            let request_link =
                "https://api.steampowered.com/IPlayerService/GetBadges/v1/?key=".to_owned() +
                    &steam_key + "&steamid=" + &steam_id;
            match send_steam_request(request_link){
                Ok(body) => ("HTTP/1.1 200 OK", body),
                Err(e) => {
                    eprintln!("Error sending steam request. {:#?}", e);
                    send_response(stream, "HTTP/1.1 502 BAD GATEWAY".to_owned(), e.to_owned());
                    return
                }
            }
        },

        "/recently-played-games" => {
            let request_link =
                "https://api.steampowered.com/IPlayerService/GetRecentlyPlayedGames/v1/?count=3&key=".to_owned() +
                    &steam_key + "&steamid=" + &steam_id;

            match send_steam_request(request_link){
                Ok(body) => ("HTTP/1.1 200 OK", body),
                Err(e) => {
                    eprintln!("Error sending steam request. {:#?}", e);
                    send_response(stream, "HTTP/1.1 502 BAD GATEWAY".to_owned(), e.to_owned());
                    return
                }
            }
        },
        _ => {
            eprintln!("Error parsing through HTTP request.");
            send_response(stream, "HTTP/1.1 404 NOT FOUND".to_owned(), "Path cannot be found".to_owned());
            return
        }
    };
    send_response(stream, status_line.to_owned(), contents);
}


fn send_steam_request(request: String) -> Result<String, String>{
    let body_response = match reqwest::blocking::get(request) {
        Ok(response) => response,
        Err(e) => {
            return Err(e.to_string())
        }
    };

    match body_response.text() {
        Ok(body) => Ok(body),
        Err(e) => {
            Err(e.to_string())
        }
    }
}

fn send_response(mut stream: TcpStream, status:String, content: String) {
    let content_length = content.len();
    let response =
        format!("{status}\r\nContent-Length: {content_length}\r\n\r\n{content}");

    match stream.write_all(response.as_bytes()){
        Ok(_) => (),
        Err(error) => {
            eprintln!("Error responding to the request: {:#?}", error);
        }
    }
}