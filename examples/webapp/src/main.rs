//! In this example, the token is saved into a cache file. If you are building a real-world web
//! app, it's easy to save token into database, by calling the function
//! `util::get_token_without_cache()`, instead of `util::get_token()`, which saves token by
//! default.

#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use rocket::http::{Cookie, Cookies};
use rocket::response::Redirect;
use rocket_contrib::json;
use rocket_contrib::json::JsonValue;
use rocket_contrib::templates::Template;
use rspotify::client::{ClientError, SpotifyBuilder};

use rspotify::oauth2::{CredentialsBuilder, OAuthBuilder, TokenBuilder};
use rspotify::util;

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Responder)]
pub enum AppResponse {
    Template(Template),
    Redirect(Redirect),
    Json(JsonValue),
}

const CACHE_PATH: &str = ".spotify_cache/";

fn create_cache_path_if_absent(cookies: &Cookies) -> PathBuf {
    let (exist, cache_path) = check_cache_path_exists(cookies);
    if !exist {
        let mut path = cache_path.clone();
        path.pop();
        fs::create_dir_all(path).unwrap();
    }
    cache_path.clone()
}

fn remove_cache_path(mut cookies: Cookies) {
    let (exist, cache_path) = check_cache_path_exists(&cookies);
    if exist {
        fs::remove_file(cache_path).unwrap()
    }
    cookies.remove(Cookie::named("uuid"))
}

fn check_cache_path_exists(cookies: &Cookies) -> (bool, PathBuf) {
    let project_dir_path = env::current_dir().unwrap();
    let mut cache_path = project_dir_path;
    cache_path.push(CACHE_PATH);
    cache_path.push(cookies.get("uuid").unwrap().value());
    (cache_path.exists(), cache_path)
}

fn init_spotify() -> SpotifyBuilder {
    // Please notice that protocol of redirect_uri, make sure it's http
    // (or https). It will fail if you mix them up.
    let oauth = OAuthBuilder::default()
        .redirect_uri("http://localhost:8000/callback")
        .scope("user-read-currently-playing playlist-modify-private")
        .build()
        .unwrap();

    // Replacing client_id and client_secret with yours.
    let creds = CredentialsBuilder::default()
        .id("e1dce60f1e274e20861ce5d96142a4d3")
        .secret("0e4e03b9be8d465d87fc32857a4b5aa3")
        .build()
        .unwrap();

    SpotifyBuilder::default()
        .credentials(creds)
        .oauth(oauth)
        .clone()
}

#[get("/callback?<code>")]
fn callback(cookies: Cookies, code: String) -> AppResponse {
    let mut spotify = init_spotify();
    let mut spotify = spotify
        .cache_path(create_cache_path_if_absent(&cookies))
        .build()
        .unwrap();
    return match spotify.request_user_token(code.as_str()) {
        Ok(_) => {
            println!("request user token successful");
            AppResponse::Redirect(Redirect::to("/"))
        }
        Err(err) => {
            println!("Failed to get user token {:?}", err);
            let mut context = HashMap::new();
            context.insert("err_msg", "Failed to get token!");
            AppResponse::Template(Template::render("error", context))
        }
    };
}

#[get("/")]
fn index(mut cookies: Cookies) -> AppResponse {
    let cookie = cookies.get("uuid");
    let mut spotify_builder = init_spotify();
    let spotify = match cookie.is_none() {
        true => {
            cookies.add(Cookie::new("uuid", util::generate_random_string(64)));
            spotify_builder
                .cache_path(create_cache_path_if_absent(&cookies))
                .build()
                .unwrap()
        }
        false => {
            let (_exist, cache_path) = check_cache_path_exists(&cookies);
            let token = TokenBuilder::from_cache(cache_path).build().unwrap();
            spotify_builder.token(token).build().unwrap()
        }
    };
    let mut context = HashMap::new();
    match spotify.me() {
        Ok(user_info) => {
            context.insert(
                "display_name",
                user_info
                    .display_name
                    .unwrap_or_else(|| String::from("Dear")),
            );
            AppResponse::Template(Template::render("index", context.clone()))
        }
        Err(ClientError::InvalidAuth(msg)) => {
            println!("InvalidAuth msg {:?}", msg);
            let auth_url = spotify.get_authorize_url(true).unwrap();
            context.insert("auth_url", auth_url);
            AppResponse::Template(Template::render("authorize", context))
        }
        Err(err) => {
            let mut context = HashMap::new();
            context.insert("err_msg", format!("Failed for {}!", err));
            AppResponse::Template(Template::render("error", context))
        }
    }
}

#[get("/sign_out")]
fn sign_out(cookies: Cookies) -> AppResponse {
    remove_cache_path(cookies);
    AppResponse::Redirect(Redirect::to("/"))
}

#[get("/playlists")]
fn playlist(cookies: Cookies) -> AppResponse {
    let mut spotify = init_spotify();
    let (exist, cache_path) = check_cache_path_exists(&cookies);
    if !exist {
        return AppResponse::Redirect(Redirect::to("/"));
    }

    let token = TokenBuilder::from_cache(cache_path).build().unwrap();
    let spotify = spotify.token(token).build().unwrap();
    match spotify.current_user_playlists(Some(20), Some(0)) {
        Ok(playlists) => AppResponse::Json(json!(playlists)),
        Err(_) => AppResponse::Redirect(Redirect::to("/")),
    }
}

#[get("/me")]
fn me(cookies: Cookies) -> AppResponse {
    let mut spotify = init_spotify();
    let (exist, cache_path) = check_cache_path_exists(&cookies);
    if !exist {
        return AppResponse::Redirect(Redirect::to("/"));
    }

    let token = TokenBuilder::from_cache(cache_path).build().unwrap();
    let spotify = spotify.token(token).build().unwrap();
    match spotify.me() {
        Ok(user_info) => AppResponse::Json(json!(user_info)),
        Err(_) => AppResponse::Redirect(Redirect::to("/")),
    }
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index, callback, sign_out, me, playlist])
        .attach(Template::fairing())
        .launch();
}
