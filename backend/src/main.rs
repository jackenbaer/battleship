use serde::{Deserialize, Serialize};

use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder, Result};
use futures::StreamExt;
use hex;
use rand::rngs::OsRng;
use rand::RngCore;
use std::collections::{HashMap, HashSet};
use tokio::sync::{broadcast, RwLock};

use futures::stream::unfold;

#[derive(Debug, Deserialize)]
pub struct ShotRequest {
    player_id: String,
    x: u32,
    y: u32,
}

#[derive(Debug, Serialize, Clone)]
pub struct ShotResponse {
    shot: [u32; 2],
    hit: bool,
    sunk: bool,
    length: usize,
    fleet_destroyed: bool,
}

#[derive(Debug, Clone)]
pub struct Game {
    player_a: PlayerInfo,
    player_b: PlayerInfo,
    on_move: String,
}

#[derive(Debug, Clone)]
pub struct PlayerInfo {
    id: String,
    position: [Vec<[u32; 2]>; 5],
    hits: Vec<[u32; 2]>,
    misses: Vec<[u32; 2]>,
    //radar: Vec<[u32; 2]>,
    events: broadcast::Sender<ShotResponse>,
}

#[derive(Deserialize, Debug)]
pub struct PositionRequest {
    player_id: String,
    position: [Vec<[u32; 2]>; 5],
}

#[derive(Serialize, Debug)]
pub struct NewGameResponse {
    game_id: String,
    player_id: String,
}

fn generate_token() -> String {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    hex::encode(key)
}

fn are_same_ship(position: &[Vec<[u32; 2]>; 5], coord1: [u32; 2], coord2: [u32; 2]) -> bool {
    for ship in position {
        let ship_coords: Vec<_> = ship.iter().cloned().collect();
        if ship_coords.contains(&coord1) && ship_coords.contains(&coord2) {
            return true;
        }
    }
    false
}

fn ships_not_overlap(position: &[Vec<[u32; 2]>; 5]) -> bool {
    let mut position_set = HashSet::new();
    for ship in position {
        for &part in ship {
            position_set.insert(part);
        }
    }
    position_set.len() == 17
}

fn ships_not_neighbors(position: &[Vec<[u32; 2]>; 5]) -> bool {
    let mut positions_set = HashSet::new();

    for ship in position {
        for &part in ship {
            positions_set.insert(part);
        }
    }

    for &[x, y] in &positions_set {
        for dx in -1i32..=1 {
            for dy in -1i32..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                if positions_set.contains(&[(x as i32 + dx) as u32, (y as i32 + dy) as u32])
                    && !are_same_ship(
                        &position,
                        [x, y],
                        [(x as i32 + dx) as u32, (y as i32 + dy) as u32],
                    )
                {
                    return false;
                }
            }
        }
    }

    true
}

fn num_ships(position: &[Vec<[u32; 2]>; 5]) -> bool {
    position.len() == 5
}

fn ship_lengths(position: &[Vec<[u32; 2]>; 5]) -> bool {
    let required_lengths = [2, 3, 3, 4, 5];
    let mut found_lengths = Vec::new();

    for ship in position {
        found_lengths.push(ship.len());
    }

    found_lengths.sort();

    required_lengths == found_lengths.as_slice()
}

fn ship_on_field(position: &[Vec<[u32; 2]>; 5]) -> bool {
    for ship in position {
        for &[x, y] in ship {
            if x > 10 || y > 10 {
                return false;
            }
        }
    }
    true
}

fn ships_consistent(position: &[Vec<[u32; 2]>; 5]) -> bool {
    for ship in position {
        // Determine if the ship is placed horizontally or vertically by comparing the x-coordinates of the first two parts
        let is_horizontal = ship[0][0] == ship[1][0];

        // Sort the parts of the ship by their x-coordinate if the ship is placed horizontally, or by their y-coordinate if it's placed vertically
        let mut sorted_ship = ship.clone();
        sorted_ship.sort_by_key(|part| if is_horizontal { part[1] } else { part[0] });

        // Now check for gaps
        let mut previous_part: Option<[u32; 2]> = None;

        for &part in &sorted_ship {
            if let Some([prev_x, prev_y]) = previous_part {
                if !((part[0] == prev_x && part[1] == prev_y + 1)
                    || (part[1] == prev_y && part[0] == prev_x + 1))
                {
                    return false;
                }
            }
            previous_part = Some(part);
        }
    }
    true
}

fn verify_position(p: &[Vec<[u32; 2]>; 5]) -> bool {
    let crit_neighbors = ships_not_neighbors(&p);
    let crit_num = num_ships(&p); //5
    let crit_length = ship_lengths(&p); //5, 4, 3, 3, 2
    let crit_on_field = ship_on_field(&p); //10x10
    let crit_consistent = ships_consistent(&p); //no gaps inside one list
    let crit_overlap = ships_not_overlap(&p);
    return crit_neighbors
        && crit_num
        && crit_length
        && crit_on_field
        && crit_consistent
        && crit_overlap;
}

async fn new_game(data: web::Data<RwLock<HashMap<String, Game>>>) -> impl Responder {
    let game_id = generate_token();
    let player_id = generate_token();

    let (player_a_events, _) = broadcast::channel::<ShotResponse>(10);
    let (player_b_events, _) = broadcast::channel::<ShotResponse>(10);

    let game = Game {
        player_a: PlayerInfo {
            id: player_id.clone(),
            position: Default::default(),
            hits: Default::default(),
            misses: Default::default(),
            //radar: Default::default(),
            events: player_a_events,
        },
        player_b: PlayerInfo {
            id: Default::default(),
            position: Default::default(),
            hits: Default::default(),
            misses: Default::default(),
            //radar: Default::default(),
            events: player_b_events,
        },
        on_move: player_id.clone(),
    };

    let mut hashmap = data.write().await;
    hashmap.insert(game_id.clone(), game.clone());
    let response = NewGameResponse { game_id, player_id };
    //println!("{:?}", response);
    //print!("{:#?}", game);
    HttpResponse::Ok().json(response)
}

async fn join_game(
    game_id: web::Path<String>,
    data: web::Data<RwLock<HashMap<String, Game>>>,
) -> impl Responder {
    let game_id = game_id.into_inner();
    let player_id = generate_token();

    let mut hashmap = data.write().await;
    if let Some(game) = hashmap.get_mut(&game_id) {
        game.player_b.id = player_id.clone();
        // Perform any other necessary operations with the game

        let response = NewGameResponse {
            game_id: game_id.clone(),
            player_id: player_id.clone(),
        };

        //println!("{:#?}", game);
        //println!("{:?}", response);

        return HttpResponse::Ok().json(response);
    }

    // Game not found
    return HttpResponse::NotFound().finish();
}

async fn position(
    game_id: web::Path<String>,
    data: web::Data<RwLock<HashMap<String, Game>>>,
    req: web::Json<PositionRequest>,
) -> impl Responder {
    let game_id = game_id.into_inner();
    let position_request = req.into_inner();
    println!("position {:?}", position_request);

    if verify_position(&position_request.position) {
        let mut hashmap = data.write().await;
        if let Some(game) = hashmap.get_mut(&game_id) {
            let player = if game.player_a.id == position_request.player_id {
                &mut game.player_a
            } else if game.player_b.id == position_request.player_id {
                &mut game.player_b
            } else {
                return HttpResponse::NotFound();
            };

            player.position = position_request.position;
            return HttpResponse::Ok();
        } else {
            return HttpResponse::NotFound();
        }
    } else {
        // Game not found
        return HttpResponse::NotFound();
    }
}

fn i_am_player_a(game: &Game, player_id: &str) -> Result<bool, &'static str> {
    if game.player_a.id == player_id {
        Ok(true)
    } else if game.player_b.id == player_id {
        Ok(false)
    } else {
        Err("Player not found")
    }
}

fn is_hit(position: &[Vec<[u32; 2]>; 5], s: &[u32; 2]) -> (bool, usize) {
    for ship in position {
        for &part in ship {
            if part == *s {
                return (true, ship.len());
            }
        }
    }
    return (false, 0);
}

fn is_sunk(position: &[Vec<[u32; 2]>; 5], hits: &Vec<[u32; 2]>, s: &[u32; 2]) -> bool {
    for ship in position {
        if ship.contains(s) {
            return ship.iter().all(|part| hits.contains(part));
        }
    }
    false
}

fn is_fleet_destroyed(position: &[Vec<[u32; 2]>; 5], hits: &Vec<[u32; 2]>) -> bool {
    for ship in position {
        for &part in ship {
            if !(hits.contains(&part)) {
                return false;
            }
        }
    }
    return true;
}

async fn shot(
    game_id: web::Path<String>,
    data: web::Data<RwLock<HashMap<String, Game>>>,
    req: web::Json<ShotRequest>,
) -> HttpResponse {
    let shot_req = req.into_inner();
    let mut hashmap = data.write().await;
    if let Some(game) = hashmap.get_mut(&game_id.into_inner()) {
        if game.on_move != shot_req.player_id {
            return HttpResponse::NotFound().finish();
        }
        let response;
        match i_am_player_a(&game, &shot_req.player_id) {
            Ok(is_player_a) => {
                let (hit, sunk, fleet_destroyed, l);

                if is_player_a {
                    (hit, l) = is_hit(&game.player_b.position, &[shot_req.x, shot_req.y]);
                    if hit {
                        game.player_b.hits.push([shot_req.x, shot_req.y]);
                    } else {
                        game.player_b.misses.push([shot_req.x, shot_req.y]);
                    }

                    sunk = is_sunk(
                        &game.player_b.position,
                        &game.player_b.hits,
                        &[shot_req.x, shot_req.y],
                    );

                    fleet_destroyed =
                        is_fleet_destroyed(&game.player_b.position, &game.player_b.hits);
                    game.on_move = game.player_b.id.clone();
                    response = ShotResponse {
                        shot: [shot_req.x, shot_req.y],
                        hit: hit,
                        sunk: sunk,
                        length: l,
                        fleet_destroyed: fleet_destroyed,
                    };
                    match game.player_b.events.send(response.clone()) {
                        Ok(_) => {} // Successfully sent the message, do nothing
                        Err(e) => println!("Failed to send the message: {:?}", e), // Handle the error here
                    }
                    return HttpResponse::Ok().json(response);
                } else {
                    (hit, l) = is_hit(&game.player_a.position, &[shot_req.x, shot_req.y]);
                    if hit {
                        game.player_a.hits.push([shot_req.x, shot_req.y]);
                    } else {
                        game.player_a.misses.push([shot_req.x, shot_req.y]);
                    }

                    sunk = is_sunk(
                        &game.player_a.position,
                        &game.player_a.hits,
                        &[shot_req.x, shot_req.y],
                    );
                    fleet_destroyed =
                        is_fleet_destroyed(&game.player_a.position, &game.player_a.hits);

                    game.on_move = game.player_a.id.clone();
                    println!("{:?}", hit);
                    println!("{:?}", sunk);
                    response = ShotResponse {
                        shot: [shot_req.x, shot_req.y],
                        hit: hit,
                        sunk: sunk,
                        length: l,
                        fleet_destroyed: fleet_destroyed,
                    };

                    match game.player_a.events.send(response.clone()) {
                        Ok(_) => {} // Successfully sent the message, do nothing
                        Err(e) => println!("Failed to send the message: {:?}", e), // Handle the error here
                    }
                    return HttpResponse::Ok().json(response);
                }
            }
            Err(err) => {
                println!("error {:?}", err);
                return HttpResponse::NotFound().finish();
            }
        }
    }
    return HttpResponse::NotFound().finish();
}

#[derive(Clone)]
struct AppState {
    clients: web::Data<RwLock<HashMap<String, Game>>>,
}

#[derive(Debug, Serialize)]
struct MyData {
    field1: String,
    field2: i32,
}

async fn events(
    data: web::Data<RwLock<HashMap<String, Game>>>,
    info: web::Path<(String, String)>,
) -> Result<HttpResponse, actix_web::Error> {
    println!("eventsnnnnnnnnnnn");

    let game_id = &info.0;
    let player_id = &info.1;

    let mut hashmap = data.write().await;
    let game;
    let receiver: Option<broadcast::Receiver<ShotResponse>>;
    if let Some(g) = hashmap.get_mut(game_id) {
        game = g;
    } else {
        println!("game not found ");
        return Ok(HttpResponse::NotFound().finish());
    }

    if game.player_a.id == player_id.to_string() {
        receiver = Some(game.player_a.events.subscribe());
    } else if game.player_b.id == player_id.to_string() {
        receiver = Some(game.player_b.events.subscribe());
    } else {
        println!("player not found ");
        return Ok(HttpResponse::NotFound().finish());
    }

    let receiver = receiver.expect("No receiver initialized");

    let stream = unfold(receiver, |mut receiver| async move {
        match receiver.recv().await {
            Ok(msg) => Some((
                Ok(web::Bytes::from(format!(
                    "data: {}\n\n",
                    serde_json::to_string(&msg).unwrap()
                ))),
                receiver,
            )),
            Err(broadcast::error::RecvError::Lagged(_)) => {
                // Handle the case where we couldn't keep up with the message rate and lost some messages.
                // You may want to return an error to the client here.
                let error = actix_web::error::ErrorInternalServerError("Message lagged");
                Some((Err(error), receiver))
            }
            Err(broadcast::error::RecvError::Closed) => None,
        }
    })
    .boxed();

    return Ok(HttpResponse::Ok()
        .append_header(("content-type", "text/event-stream"))
        .streaming(stream));
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let data = web::Data::new(RwLock::new(HashMap::new()));
    let state = AppState {
        clients: data.clone(),
    };

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials();

        App::new()
            .wrap(cors)
            .app_data(state.clients.clone())
            .route("/shot/{game_id}", web::post().to(shot))
            .route("/new_game/", web::get().to(new_game))
            .route("/events/{game_id}/{player_id}", web::get().to(events))
            .route("/join_game/{game_id}", web::get().to(join_game))
            .route("/position/{game_id}", web::post().to(position))
    })
    .bind("2a01:4f8:c0c:9f3e::2:5000")?
    .run()
    .await
}

/*
TODO
what if  an api endpoint is called multiple times
what if a json doesnt match structs
wrong player makes a move
restrict cors
use latest version of crates
 */
