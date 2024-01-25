mod graph;
mod util;
mod image_utils;
#[macro_use]
extern crate lazy_static;

use std::fs::File;

use memory_stats::memory_stats;

use std::io::Cursor;
use std::path::PathBuf;
use std::{env, thread};
use std::{fs, collections::HashMap,io::Write};
use std::sync::{Arc, Mutex, RwLock};


use thiserror::Error;
use uuid::Uuid;


use std::time;

use actix_web::HttpResponseBuilder;
use actix_web::body::MessageBody;
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};

use actix_multipart::Multipart;
use actix_web::web::{Json, Redirect};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, HttpRequest, http::{header::{CacheControl, CacheDirective}}};

use cookie::Cookie;
use tokio_stream::StreamExt;

use chrono::{Utc, Duration};
use graph::Graph;
use jsonwebtoken::{encode, Header, EncodingKey, decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::graph::node::NodeStatic;


use sqlx::{migrate::MigrateDatabase, FromRow, Row, Sqlite, SqlitePool};
const DB_URL: &str = "sqlite://sqlite.db";



#[derive(Clone, FromRow, Debug)]
struct User {
    username: String,
    password_hash: String,
}

#[derive(Deserialize)]
struct UserCredentials{
    username: String,
    password: String
}


#[derive(Deserialize)]
struct param{
    x : String
}

struct AppState{
    db : sqlx::Pool<Sqlite>,
    graphs: RwLock<HashMap<String,Arc<Mutex<Graph>>>>

}

fn sanitize(dirty: &str)->String{
    let mut clean = dirty.to_owned();   
    clean
}


fn build_session_cookie(username:&str,graphName:&str)->Cookie<'static>{
    let my_iat = Utc::now().timestamp();
    let my_exp = Utc::now()
        .checked_add_signed(Duration::days(1))
        .expect("invalid timestamp")
        .timestamp();
    let loginClaim = LoginClaim {
        sub: "".to_owned(),
        iat: my_iat as usize,
        exp: my_exp as usize,
        username: username.to_string().clone(),
        graphName:graphName.to_string().clone()
    };

    let token = match encode(
        &Header::default(),
        &loginClaim,
        &EncodingKey::from_secret(util::SECRET.as_bytes()),
    ) {
        Ok(t) => t,
        Err(_) => panic!(),
    };

    Cookie::build("session", token).http_only(true).finish()
}

async fn authenticate(db:&sqlx::Pool<Sqlite>,userCred:&UserCredentials)->bool{
    let username = sanitize(&userCred.username);
    let user_results = sqlx::query_as::<_, User>("SELECT username, password_hash FROM users WHERE username=$1")
        .bind(username)
        .fetch_all(db)
        .await
        .unwrap();
    if user_results.len() == 0{
        return false;
    }
    let parsed_hash = PasswordHash::new(&user_results[0].password_hash).unwrap();
    if  Argon2::default().verify_password(userCred.password.as_bytes(), &parsed_hash).is_ok(){
        return true;
    }
    false
}



fn get_claim_from_token(token:String)->Option<LoginClaim>{

    let mut validator = Validation::default();
    validator.leeway = 1;
    validator.validate_exp = true;
    let token_data = match decode::<LoginClaim>(
        &token,
        &DecodingKey::from_secret(util::SECRET.as_bytes()),
        &validator,
    ) {
        Ok(c) => c,
        Err(_) => {
            return None;
        }
    };

    
    return Some(token_data.claims);
}


#[post("/createAccount")]
async fn create_account(data: web::Data<AppState>, userCred:Json<UserCredentials>)->impl Responder{
    if util::sanitize(&userCred.username,true).len()< userCred.username.len(){
        return HttpResponse::BadRequest().content_type("text").body("invalid username");
    }
    let user_results = sqlx::query_as::<_, User>("SELECT username, password_hash FROM users WHERE username=$1")
        .bind(userCred.username.clone())
        .fetch_all(&data.db)
        .await
        .unwrap();
    if user_results.len() != 0{
        return HttpResponse::BadRequest().content_type("text").body("taken");
    }

    let salt = SaltString::generate(&mut OsRng);

    // Argon2 with default params (Argon2id v19)
    let argon2 = Argon2::default();

    // Hash password to PHC string ($argon2id$v=19$...)
    let password_hash = argon2.hash_password(userCred.password.as_bytes(), &salt).unwrap().to_string();

    let _result = sqlx::query("INSERT INTO users (username,password_hash) VALUES ($1,$2)")
        .bind(userCred.username.clone())
        .bind(password_hash)
        .execute(&data.db)
        .await
        .unwrap();

    let _ = fs::create_dir_all(PathBuf::from_iter([util::RESOURCE_PATH.clone(),"images".to_owned(),userCred.username.clone()]));

    HttpResponse::Ok().content_type("text").body("ok")
}


#[derive(Deserialize, Serialize, Debug)]
struct LoginClaim {
    sub: String,
    iat: usize,
    exp: usize,
    username: String,
    graphName:String
}

#[post("/login")]
async fn login(data: web::Data<AppState>, userCredentials:Json<UserCredentials>)->impl Responder{
    if !authenticate(&data.db, &userCredentials).await{
        return HttpResponse::Unauthorized().into();
    }


    HttpResponse::Ok().cookie(build_session_cookie(&userCredentials.username, "")).finish()
}


#[get("/retrieveGraphFileList")]
async fn retrieve_graph_file_list(request: HttpRequest,data: web::Data<AppState>)->impl Responder{
    let claim = match get_claim_from_token(match request.cookie("session"){Some(val)=>val,None=>return HttpResponse::Unauthorized().into()}.value().to_owned()){Some(val)=>val, None=>return HttpResponse::Unauthorized().into()};
    let results = sqlx::query("SELECT graphName FROM graphs WHERE username=$1")
        .bind(claim.username.clone())
        .fetch_all(&data.db)
        .await
        .unwrap();
    let mut out = vec![];
    for result in  results{
        out.push(result.get::<String,&str>("graphName"));
    }
    HttpResponse::Ok().content_type("application/json").body(serde_json::to_string(&out).unwrap())
}

#[derive(Error, Debug,PartialEq)]
pub enum LoadError{
    #[error("graph not found!")]
    graphNotFound,
    #[error("graph not valid!")]
    graphNotValid
}

async fn load_graph(data: &web::Data<AppState>, claim :&LoginClaim)->Result<Graph,LoadError>{

    let result = sqlx::query("SELECT graph FROM graphs WHERE username=$1 AND graphName=$2")
        .bind(claim.username.clone())
        .bind(claim.graphName.clone())
        .fetch_all(&data.db)
        .await
        .unwrap();
    let contents;
    if(!result.is_empty()){
        contents= result[0].get::<String,&str>("graph");
    }else{
        return Err(LoadError::graphNotFound);
    }
    //username is guaranteed to be valid because it was found in the database
    let mut loadedGraph = Graph::new(&claim.username);
    match loadedGraph.execute_commands(match serde_json::from_str(&contents){Ok(val)=>val, Err(_)=>return Err(LoadError::graphNotValid)}){Ok(_)=>(),Err(_)=>return Err(LoadError::graphNotValid)};
    
    return Ok(loadedGraph);
}

#[post("/retrieveGraph")]
async fn retrieve_graph(request: HttpRequest,data: web::Data<AppState>, body: web::Bytes)->impl Responder{
    let mut claim = match get_claim_from_token(match request.cookie("session"){Some(val)=>val, None=>return HttpResponse::Unauthorized().into()}.value().to_owned()){Some(val)=>val,None=>return HttpResponse::Unauthorized().into()};
    claim.graphName = match String::from_utf8(body.to_vec()){Ok(val)=>val,Err(_)=>return HttpResponse::BadRequest().into()};
    claim.graphName = sanitize(&claim.graphName);
    let graph = match load_graph(&data, &claim).await{Ok(val)=>val, Err(_)=>return HttpResponse::Unauthorized().into()};
    let _ = load_cached_graph(&claim, &data).await;
    HttpResponse::Ok().cookie(build_session_cookie(&claim.username, &claim.graphName)).content_type("application/json").body(serde_json::to_string(&graph.get_command_history()).unwrap())
}

#[derive(Deserialize)]
struct SaveInfo{
    fileName: String
}

async fn save_graph(data: &web::Data<AppState>, graphName:&str, graph:&Graph, username:&str){


    let results = sqlx::query("SELECT graph FROM graphs WHERE username=$1 AND graphName=$2")
        .bind(username)
        .bind(sanitize(&graphName))
        .fetch_all(&data.db)
        .await
        .unwrap();
    if !results.is_empty(){
        let _result = sqlx::query("UPDATE graphs SET graph=$3 WHERE username=$1 AND graphName=$2")
            .bind(username)
            .bind(sanitize(&graphName))
            .bind(serde_json::to_string(graph.get_command_history()).unwrap())
            .execute(&data.db)
            .await
            .unwrap();
    }else{
        let _result = sqlx::query("INSERT INTO graphs (username,graphName,graph) VALUES ($1,$2,$3)")
            .bind(username)
            .bind(sanitize(&graphName))
            .bind(serde_json::to_string(graph.get_command_history()).unwrap())
            .execute(&data.db)
            .await
            .unwrap();
    }
}

#[post("/createGraph")]
async fn create_graph(request: HttpRequest,data: web::Data<AppState>,body:web::Bytes)->impl Responder{
    let claim = match get_claim_from_token(match request.cookie("session"){Some(val)=>val, None=>return HttpResponse::Unauthorized().into()}.value().to_owned()){Some(val)=>val,None=>return HttpResponse::Unauthorized().into()};
    let mut graphName = match String::from_utf8(body.to_vec()){Ok(val)=>val,Err(_)=>return HttpResponse::BadRequest().into()};
    graphName = sanitize(&graphName);

    let newGraph = Graph::new(&claim.username);


    save_graph(&data, &graphName,&newGraph,&claim.username).await;

    HttpResponse::Ok().cookie(build_session_cookie(&claim.username, &graphName)).finish()
}


async fn load_cached_graph(claim:&LoginClaim,data:&web::Data<AppState>)->Result<Arc<Mutex<Graph>>,LoadError>{

    {
        let mut graphs = data.graphs.write().unwrap();
        if let Some(usage) = memory_stats() {
            if usage.physical_mem > util::MEM_THRESHOLD.clone(){
                println!("CLEARED GRAPH CACHE!!!");
                graphs.clear();
            }
            println!("Current physical memory usage: {}", usage.physical_mem as f64/1024_f64.powi(2));
            println!("Current virtual memory usage: {}", usage.virtual_mem as f64/1024_f64.powi(2));
        }
    }

    //loads graph into cache on-demand
    let cacheKey = (claim.username.clone()+&claim.graphName);

    {
        let mut graphs = data.graphs.write().unwrap();
        if !graphs.contains_key(&cacheKey){
            graphs.insert(cacheKey.clone(),match load_graph(&data, &claim).await{Ok(val)=>Arc::new(Mutex::new(val)),Err(err)=>return Err(err)});
        }
    }
    
    let cachedGraphArc = {
        let mut graphs = data.graphs.read().unwrap();
        graphs.get(&cacheKey).unwrap().clone()
    };
    Ok(cachedGraphArc)
}

#[post("/process")]
async fn process_graph(request: HttpRequest, data: web::Data<AppState>)-> impl Responder {
    let claim = match get_claim_from_token(match request.cookie("session"){Some(val)=>val, None=>return HttpResponse::Unauthorized().into()}.value().to_owned()){Some(val)=>val,None=>return HttpResponse::Unauthorized().into()};



    let cachedGraphArc = match load_cached_graph(&claim, &data).await{Ok(val)=>val,Err(_)=>return HttpResponse::Unauthorized().into()};
    let mut cachedGraph = cachedGraphArc.lock().unwrap();

    let outputImage = match cachedGraph.process(){Ok(val)=>val,Err(_)=>return HttpResponse::BadRequest().into()};
    let mut bytes: Vec<u8> = Vec::new();
    outputImage.write_to(&mut Cursor::new(&mut bytes), image::ImageOutputFormat::Png).unwrap();
    HttpResponse::Ok().content_type("image/png").body(bytes)
}



#[derive(Deserialize)]
struct commandGraphData{
    commands: graph::Commands
}

#[get("/")]
async fn landing()->impl Responder{
    Redirect::to("/web/main.html").see_other()
}

#[post("/command")]
async fn command_graph(request: HttpRequest, data: web::Data<AppState>, commands: Json<commandGraphData>)-> impl Responder{
    let claim = match get_claim_from_token(match request.cookie("session"){Some(val)=>val, None=>return HttpResponse::Unauthorized().into()}.value().to_owned()){Some(val)=>val,None=>return HttpResponse::Unauthorized().into()};
    
    let cachedGraphArc = match load_cached_graph(&claim, &data).await{Ok(val)=>val,Err(_)=>return HttpResponse::Unauthorized().into()};
    let mut cachedGraph = cachedGraphArc.lock().unwrap();

    for command in &commands.commands{
        println!("{}",command);
    }



    HttpResponse::Ok().content_type("text").body(match cachedGraph.execute_commands(commands.commands.clone()).err(){
        Some(error) => match error{
            graph::GraphError::Cycle=>"cycle",
            graph::GraphError::EdgeNotFound => "edge not found",
            graph::GraphError::NodeNotFound => "node not found",
            graph::GraphError::MismatchedNodes => "mismatched sockets",
            graph::GraphError::UnknownCommand => "unknown command",
            graph::GraphError::IllFormedCommand => "ill-formed command",
            graph::GraphError::NError(_) => "node error"
             },
        None => { save_graph(&data, &claim.graphName,&cachedGraph,&claim.username).await; "ok"}
    })
}

//all of the node templates available are sent client-side
#[post("/retrieveNodeTemplates")]
async fn retrieve_node_templates()->impl Responder{
    let mut descriptors = vec![];
    descriptors.push(graph::node::mathNode::MathNode::get_node_descriptor());
    descriptors.push(graph::node::finalNode::FinalNode::get_node_descriptor());
    descriptors.push(graph::node::imageInputNode::ImageInputNode::get_node_descriptor());
    descriptors.push(graph::node::colorToImageNode::ColorToImageNode::get_node_descriptor());
    descriptors.push(graph::node::composeNode::ComposeNode::get_node_descriptor());
    descriptors.push(graph::node::blendNode::BlendNode::get_node_descriptor());
    descriptors.push(graph::node::moveNode::MoveNode::get_node_descriptor());
    descriptors.push(graph::node::rotationNode::RotationNode::get_node_descriptor());
    descriptors.push(graph::node::resizeNode::ResizeNode::get_node_descriptor());
    descriptors.push(graph::node::scaleNode::ScaleNode::get_node_descriptor());
    HttpResponse::Ok().content_type("application/json").body(serde_json::to_string(&descriptors).unwrap())
}


#[post("/upload")]
async fn upload_image(request: HttpRequest,mut payload: Multipart)->impl Responder{
    let mut file_data = Vec::<u8>::new();
    let mut fileName: Option<String>= None;
    let claim = match get_claim_from_token(match request.cookie("session"){Some(val)=>val,None=>return Redirect::to("/web/login.html").see_other()}.value().to_owned()){Some(val)=>val,None=>return Redirect::to("/web/login.html").see_other()};
    while let Some(mut field) = payload.try_next().await.unwrap() {
        let content_disposition = field.content_disposition();
        let field_name = content_disposition.get_name().unwrap();
        match field_name {
            "file" => {
                while let Some(chunk) = match field.try_next().await{Ok(val)=>val,Err(_)=>return Redirect::to("/web/upload_image.html?bad_image").see_other()} {
                    file_data.extend_from_slice(&chunk);
                }
            }
            "fileName" => {
                let bytes = match field.try_next().await{
                    Ok(val)=>val,
                    Err(_)=>return Redirect::to("web//upload_image.html?bad_name").see_other()
                };
                fileName = String::from_utf8(bytes.unwrap().to_vec()).ok();
            }
            _ => {}
        }
    }
    let cleanFileName = util::sanitize(&match fileName{Some(val)=>val,None=>return Redirect::to("/web/upload_image.html?bad_name").see_other()},true);
    if(cleanFileName.is_empty()){
        return Redirect::to("/web/upload_image.html?bad_name").see_other();
    }
    let image = match image::load_from_memory(&file_data){Ok(val)=>val, Err(_)=>return Redirect::to("/web/upload_image.html?bad_image").see_other()};
    match image.save_with_format(PathBuf::from_iter([util::RESOURCE_PATH.clone(),"images".to_owned(),util::sanitize(&claim.username,true),cleanFileName+".png"]), image::ImageFormat::Png){
        Ok(_)=>(),
        Err(_)=>return Redirect::to("/web/upload_image.html?bad_image").see_other()
    };

    Redirect::to("/web/main.html").see_other()
}

#[derive(Deserialize)]
struct Info{
    name:String
}

async fn sites(_req: HttpRequest, info: web::Path<Info>) -> impl Responder {

    let name = info.name.clone();
    let cleanName = util::sanitize(&name, false);
    if cleanName.split(".").last() == Some("png"){
        return HttpResponse::Ok().content_type("image/png").body(match std::fs::read(PathBuf::from_iter([util::RESOURCE_PATH.clone(),"web".to_string() ,cleanName.clone()])){Ok(val)=>val,Err(_)=>return HttpResponse::BadRequest().into()});
    }
    let mut contentType = "text/html";
    if cleanName.split(".").last() == Some("css"){
        contentType = "text/css";
    }
    HttpResponse::Ok()
    .content_type(contentType)
    .body(match std::fs::read_to_string(PathBuf::from_iter([util::RESOURCE_PATH.clone(),"web".to_string() ,cleanName.clone()])){Ok(val)=>val,Err(_)=>return HttpResponse::BadRequest().into()})
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {


    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        println!("Creating database {}", DB_URL);
        match Sqlite::create_database(DB_URL).await {
            Ok(_) => println!("Create db success"),
            Err(error) => panic!("error: {}", error),
        }
    } else {
        println!("Database already exists");
    }

    let db: sqlx::Pool<Sqlite> = SqlitePool::connect(DB_URL).await.unwrap();

    let result = sqlx::query("CREATE TABLE IF NOT EXISTS users (username VARCHAR(64) NOT NULL, password_hash VARCHAR(255) NOT NULL);").execute(&db).await.unwrap();
    println!("Create user table result: {:?}", result);
    
    let result = sqlx::query("CREATE TABLE IF NOT EXISTS graphs (username VARCHAR(64) NOT NULL, graphName VARCHAR(128), graph TEXT NOT NULL);").execute(&db).await.unwrap();
    println!("Create user table result: {:?}", result);

    

    let appState = web::Data::new(AppState{
        db: db,
        graphs: RwLock::new(HashMap::new())
    });

    HttpServer::new(move || {
        App::new()
            .app_data(appState.clone())
            .service(login)
            .service(create_account)
            .service(retrieve_graph)
            .service(create_graph)
            .service(retrieve_graph_file_list)
            .service(process_graph)
            .service(landing)
            //.service(Files::new("/images", RESOURCE_PATH.clone()+"/images"))
            .service(web::resource("/web/{name}").route(web::get().to(sites)))
            .service(retrieve_node_templates)
            .service(command_graph)
            .service(upload_image)
    })
    .bind((util::HOST.to_owned(), 8088))?
    .run()
    .await
}