use std::{env, fs, io::{BufReader, prelude::*}, net::{TcpListener, TcpStream}, thread};
use std::process::exit;
use std::time::Duration;
use serde_json::{json, Result, Value, to_string};
use serde::{Deserialize, Serialize};
use SteamApiHandler::ThreadPool;

// #[derive(Serialize, Deserialize)]
// struct SteamHttp {
//     name: String,
//     url: String,
// }
#[derive(Serialize, Deserialize)]
struct OwnedGames {
    response: GameResponse
}

#[derive(Serialize, Deserialize)]
struct GameResponse {
    game_count: i64,
    games: Vec<GameDetails>
}
#[derive(Serialize, Deserialize)]
struct ItemDataJson{
    ids: Vec<GameDetails>,
    context: ContextDetails,
    data_request: DataRequestDetails
}

#[derive(Serialize, Deserialize)]
struct ContextDetails {
    language: String,
    country_code: String,
}

#[derive(Serialize, Deserialize)]
struct DataRequestDetails {
    include_assets: bool,
}

#[derive(Serialize, Deserialize)]
struct GameDetails {
    appid: i64,
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    let key_var = "STEAMKEY";
    let steam_key;
    match env::var(key_var) {
        Ok(val) => {
            steam_key = val;
            println!("Loaded Steam Key");
        },
        Err(e) => {
            println!("Error loading Steam Key: {}", e);
            exit(1);
        },
    }



    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let copied_steam_key = steam_key.clone();
        pool.execute(|| {
            handle_connection(stream, copied_steam_key);
        })
    }
}



fn handle_connection(mut stream: TcpStream, steam_key: String) {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let parsed_request: Vec<&str> = request_line.split_whitespace().collect();
    if parsed_request.len() != 3 || parsed_request[0] != "GET" {panic!()}

    let parsed_http_resource: Vec<&str> = parsed_request[1].split("?id=").collect();
    if parsed_http_resource.len() != 2 {panic!()}

    let steam_id = &parsed_http_resource[1];

    let (status_line, contents) = match parsed_http_resource[0] {

        "/owned-games" => {
            let request_link ="https://api.steampowered.com/IPlayerService/GetOwnedGames/v1/".to_owned()
                + "?include_played_free_games=true&include_free_sub=true&key=" +
                &steam_key + "&steamid=" + &steam_id;

            println!("{}",request_link);
            let body = reqwest::blocking::get(request_link).unwrap().text().unwrap();

            // println!("{}", body);
            let games_list: OwnedGames = serde_json::from_str(&body.as_str()).unwrap();

            let game_ids: Vec<GameDetails> = (games_list.response.games).into_iter()
                .map(|games| GameDetails{
                    appid: games.appid
                }).rev().collect();

            let game_ids_json: ItemDataJson = ItemDataJson {
                ids: game_ids,
                context: ContextDetails {
                    language: "english".to_owned(),
                    country_code: "us".to_owned()
                },
                data_request: DataRequestDetails {
                    include_assets: true
                }
            };

            let request_link2 = "https://api.steampowered.com/IStoreBrowseService/GetItems/v1/?key=".to_owned() +
                &steam_key + "&input_json=" + to_string(&game_ids_json).unwrap().as_str();
            let body2 = reqwest::blocking::get(request_link2).unwrap().text().unwrap();


            ("HTTP/1.1 200 OK", body2)
        },
        "/badges-and-levels" => {
            let request_link =
                "https://api.steampowered.com/IPlayerService/GetBadges/v1/?key=".to_owned() +
                &steam_key + "&steamid="+ &steam_id;

            println!("{}", request_link);
            let body = reqwest::blocking::get(request_link).unwrap().text().unwrap();

            ("HTTP/1.1 200 OK", body)
        },
        "/recently-played-games" => {
            let request_link =
                "https://api.steampowered.com/IPlayerService/GetRecentlyPlayedGames/v1/?count=3&key=".to_owned() +
                    &steam_key + "&steamid="+ &steam_id;

            println!("{}", request_link);
            let body = reqwest::blocking::get(request_link).unwrap().text().unwrap();

            ("HTTP/1.1 200 OK", body)
        },


        _ => ("HTTP/1.1 404 NOT FOUND", fs::read_to_string("website/404.html").unwrap())
    };

    let length = contents.len();

    let response =
        format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}

fn fetch_badges_and_levels(){

}