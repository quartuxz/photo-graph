mod graph;
mod util;
mod image_utils;
#[macro_use]
extern crate lazy_static;

use std::fs::File;

use std::io::Cursor;
use std::thread;
use std::{fs, collections::HashMap,io::Write};
use std::sync::Mutex;




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
    graphs : Mutex<HashMap<u64,graph::Graph>>,
    currentID : Mutex<u64>,
    db : sqlx::Pool<Sqlite>

}

fn sanitize(dirty: &str)->String{
    let mut clean = dirty.to_owned();   
    clean
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



fn get_username_from_token(token:String)->Option<String>{

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

    
    return Some(token_data.claims.username);
}

async fn corroborate_claim(grph : &graph::Graph, request: &HttpRequest)->bool{
    let usernameToken = match request.cookie("session"){Some(val)=>val, None=> return false}.value().to_owned();
    let res = get_username_from_token(usernameToken);
    match res{
        Some(username) => grph.get_user() == username,
        None => false
    }
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

    let _ = fs::create_dir_all(util::RESOURCE_PATH.clone()+r"\images\"+r"\"+&userCred.username);

    HttpResponse::Ok().content_type("text").body("ok")
}


#[derive(Deserialize, Serialize, Debug)]
struct LoginClaim {
    sub: String,
    iat: usize,
    exp: usize,
    username: String,
}

#[post("/login")]
async fn login(data: web::Data<AppState>, userCredentials:Json<UserCredentials>)->impl Responder{
    if !authenticate(&data.db, &userCredentials).await{
        return HttpResponse::Unauthorized().into();
    }

    let my_iat = Utc::now().timestamp();
    let my_exp = Utc::now()
        .checked_add_signed(Duration::days(1))
        .expect("invalid timestamp")
        .timestamp();
    let loginClaim = LoginClaim {
        sub: "".to_owned(),
        iat: my_iat as usize,
        exp: my_exp as usize,
        username: userCredentials.username.clone(),
    };

    let token = match encode(
        &Header::default(),
        &loginClaim,
        &EncodingKey::from_secret(util::SECRET.as_bytes()),
    ) {
        Ok(t) => t,
        Err(_) => panic!(),
    };

    let session : Cookie = Cookie::build("session", token).http_only(true).finish();
    HttpResponse::Ok().cookie(session).finish()
}


#[get("/retrieveGraphFileList")]
async fn retrieve_graph_file_list(request: HttpRequest,data: web::Data<AppState>)->impl Responder{
    let username = match get_username_from_token(match request.cookie("session"){Some(val)=>val,None=>return HttpResponse::Unauthorized().into()}.value().to_owned()){Some(val)=>val, None=>return HttpResponse::Unauthorized().into()};
    let results = sqlx::query("SELECT graphName FROM graphs WHERE username=$1")
        .bind(username.clone())
        .fetch_all(&data.db)
        .await
        .unwrap();
    let mut out = vec![];
    for result in  results{
        out.push(result.get::<String,&str>("graphName"));
    }
    HttpResponse::Ok().content_type("application/json").body(serde_json::to_string(&out).unwrap())
}

#[post("/loadGraph")]
async fn load_graph(request: HttpRequest,data: web::Data<AppState>, body:web::Bytes)->impl Responder{
    let mut graphs = data.graphs.lock().unwrap();
    let mut currentID = data.currentID.lock().unwrap();
    let username = match get_username_from_token(match request.cookie("session"){Some(val)=>val, None=>return HttpResponse::Unauthorized().into()}.value().to_owned()){
        Some(value) => value,
        None => return HttpResponse::Unauthorized().into()
    };
    let mut graphName = match String::from_utf8(body.to_vec()){Ok(val)=>val,Err(_)=>return HttpResponse::BadRequest().into()};
    graphName = sanitize(&graphName);
    let result = sqlx::query("SELECT graph FROM graphs WHERE username=$1 AND graphName=$2")
        .bind(username.clone())
        .bind(graphName.clone())
        .fetch_all(&data.db)
        .await
        .unwrap();
    let contents;
    if(!result.is_empty()){
        contents= result[0].get::<String,&str>("graph");
    }else{
        return HttpResponse::NotFound().body("graph not found");
    }
    //username is guaranteed to be valid because it was found in the database
    let mut loadedGraph = Graph::new(username);
    match loadedGraph.execute_commands(match serde_json::from_str(&contents){Ok(val)=>val, Err(_)=>return HttpResponse::InternalServerError().into()}){Ok(_)=>(),Err(_)=>return HttpResponse::BadRequest().into()};
    graphs.insert(*currentID, loadedGraph);
    *currentID += 1;
    HttpResponse::Ok().content_type("text").body((*currentID-1).to_string())
}

#[post("/retrieveGraph")]
async fn retrieve_graph(request: HttpRequest,data: web::Data<AppState>, body:web::Bytes)->impl Responder{
    
    let graphs = data.graphs.lock().unwrap();
    
    let graph = match graphs.get(match &(match String::from_utf8(body.to_vec()){Ok(val)=>val,Err(_)=>return HttpResponse::BadRequest().into()}).parse(){Ok(val)=>val, Err(_)=> return HttpResponse::BadRequest().into()}){Some(val)=>val, None=>return HttpResponse::Unauthorized().into()};
    if !corroborate_claim(graph, &request).await{
        return HttpResponse::Unauthorized().into()
    }
    HttpResponse::Ok().content_type("application/json").body(serde_json::to_string(&graph.get_command_history()).unwrap())
}

#[derive(Deserialize)]
struct SaveInfo{
    fileName: String,
    graphID : u64
}

#[post("/saveGraph")]
async fn save_graph(request: HttpRequest,data: web::Data<AppState>, saveInfo:Json<SaveInfo>)->impl Responder{
    let graphs = data.graphs.lock().unwrap();
    let graph = match graphs.get(&saveInfo.graphID){Some(val)=>val,None=>return HttpResponse::BadRequest().into()};
    if !corroborate_claim(graph, &request).await{
        return HttpResponse::Unauthorized();
    }
    let username = match get_username_from_token(match request.cookie("session"){Some(val)=>val, None=>return HttpResponse::Unauthorized().into()}.value().to_owned()){Some(val)=>val,None=>return HttpResponse::BadRequest().into()};

    let results = sqlx::query("SELECT graph FROM graphs WHERE username=$1 AND graphName=$2")
        .bind(username.clone())
        .bind(sanitize(&saveInfo.fileName))
        .fetch_all(&data.db)
        .await
        .unwrap();
    if !results.is_empty(){
        let _result = sqlx::query("UPDATE graphs SET graph=$3 WHERE username=$1 AND graphName=$2")
            .bind(username)
            .bind(sanitize(&saveInfo.fileName))
            .bind(serde_json::to_string(graph.get_command_history()).unwrap())
            .execute(&data.db)
            .await
            .unwrap();
    }else{
        let _result = sqlx::query("INSERT INTO graphs (username,graphName,graph) VALUES ($1,$2,$3)")
            .bind(username)
            .bind(sanitize(&saveInfo.fileName))
            .bind(serde_json::to_string(graph.get_command_history()).unwrap())
            .execute(&data.db)
            .await
            .unwrap();
    }



    HttpResponse::Ok()
}

#[post("/createGraph")]
async fn create_graph(request: HttpRequest,data: web::Data<AppState>)->impl Responder{
    let mut graphs = data.graphs.lock().unwrap();
    let mut currentID = data.currentID.lock().unwrap();
    let newGraph = Graph::new(match get_username_from_token(match request.cookie("session"){Some(val)=>val,None=> return HttpResponse::Unauthorized().into()}.value().to_owned()){
        Some(val) => val,
        None => return HttpResponse::Unauthorized().into()
    });
    graphs.insert(*currentID, newGraph);
    *currentID += 1;
    HttpResponse::Ok().content_type("text").body((*currentID-1).to_string())
}

#[get("/")]
async fn graph_selector_html()-> impl Responder{
    let contents = fs::read_to_string(util::RESOURCE_PATH.clone()+"graph_selector.html").unwrap();

    HttpResponse::Ok().content_type("text/html").body(contents)
}

#[get("/utils.js")]
async fn utils_javascript()->impl Responder{
    let contents = fs::read_to_string(util::RESOURCE_PATH.clone()+"utils.js").unwrap();
    HttpResponse::Ok().content_type("text/javascript").body(contents)
}

#[get("/graph_selector.js")]
async fn graph_selector_javascript()->impl Responder{
    let contents = fs::read_to_string(util::RESOURCE_PATH.clone()+"graph_selector.js").unwrap();

    HttpResponse::Ok().content_type("text/javascript").body(contents)
}

#[get("/create_account")]
async fn create_account_html()->impl Responder{
    let contents = fs::read_to_string(util::RESOURCE_PATH.clone()+"create_account.html").unwrap();

    HttpResponse::Ok().content_type("text/html").body(contents)
}

#[get("/create_account.js")]
async fn create_account_javascript()->impl Responder{
    let contents = fs::read_to_string(util::RESOURCE_PATH.clone()+"create_account.js").unwrap();

    HttpResponse::Ok().content_type("text/html").body(contents)
}

#[get("/login")]
async fn login_html()->impl Responder{
    let contents = fs::read_to_string(util::RESOURCE_PATH.clone()+"login.html").unwrap();

    HttpResponse::Ok().content_type("text/html").body(contents)
}

#[get("/login.js")]
async fn login_javascript()->impl Responder{
    let contents = fs::read_to_string(util::RESOURCE_PATH.clone()+"login.js").unwrap();

    HttpResponse::Ok().content_type("text/javascript").body(contents)
}

#[get("/graph")]
async fn graph_page_html() -> impl Responder {
    let contents = fs::read_to_string(util::RESOURCE_PATH.clone()+"main.html").unwrap();

    HttpResponse::Ok().content_type("text/html").body(contents)
    //HttpResponse::Ok().body(r#"<script>
    //function setCookie(name,value,days) {
    //    var expires = "";
    //    if (days) {
    //         var date = new Date();
    //         date.setTime(date.getTime() + (days*24*60*60*1000));
    //         expires = "; expires=" + date.toUTCString();
    //     }
    //     document.cookie = name + "=" + (value || "")  + expires + "; path=/";
    // }

    // const data = { x : " hello \<script\>console.log(\"hello\");\</script\>" };
    // const options = {
    //   method: "POST",
    //   headers: {
    //     "Content-Type": "application/json",
    //   },
    //   body: JSON.stringify(data),
    //   credentials: 'include'
    // };
    
    // setCookie("asd", "bye", 3);
    // fetch("/api", options).then(response => {    response.text().then(final => {document.body.innerHTML=final;})});
    // console.log("bye");
    // </script><body></body>"#)
}

#[get("/main.js")]
async fn graph_page_javascript_main()-> impl Responder {
    let contents = fs::read_to_string(util::RESOURCE_PATH.clone()+"main.js").unwrap();

    HttpResponse::Ok().content_type("text/javascript").body(contents)
}

#[get("/style.css")]
async fn style()-> impl Responder {
    let contents = fs::read_to_string(util::RESOURCE_PATH.clone()+"style.css").unwrap();

    HttpResponse::Ok().content_type("text/css").body(contents)
}

#[get("/UI.js")]
async fn graph_page_javascript_UI()-> impl Responder {
    let contents = fs::read_to_string(util::RESOURCE_PATH.clone()+"UI.js").unwrap();

    HttpResponse::Ok().content_type("text/javascript").body(contents)
}

#[get("/matrix.js")]
async fn graph_page_javascript_matrix()-> impl Responder {
    let contents = fs::read_to_string(util::RESOURCE_PATH.clone()+"matrix.js").unwrap();

    HttpResponse::Ok().content_type("text/javascript").body(contents)
}

#[get("/graph.js")]
async fn graph_page_javascript_graph()-> impl Responder {
    let contents = fs::read_to_string(util::RESOURCE_PATH.clone()+"graph.js").unwrap();

    HttpResponse::Ok().content_type("text/javascript").body(contents)
}


#[post("/process")]
async fn process_graph(request: HttpRequest, data: web::Data<AppState>, body:web::Bytes)-> impl Responder {
    let mut graphs = data.graphs.lock().unwrap();
    let graph = match graphs.get_mut(&match(match String::from_utf8(body.to_vec()){Ok(val)=>val, Err(_)=>return HttpResponse::BadRequest().into()}).parse(){Ok(val)=>val, Err(_)=>return HttpResponse::BadRequest().into()}){Some(val)=>val, None=>return HttpResponse::BadRequest().into()};
    if !corroborate_claim(graph, &request).await{
        return HttpResponse::Unauthorized().into();
    }
    let outputImage = graph.process();
    let mut bytes: Vec<u8> = Vec::new();
    outputImage.write_to(&mut Cursor::new(&mut bytes), image::ImageOutputFormat::Png).unwrap();
    HttpResponse::Ok().content_type("image/png").body(bytes)
}



#[derive(Deserialize)]
struct commandGraphData{
    commands: graph::Commands,
    graphID : u64
}

#[post("/command")]
async fn command_graph(request: HttpRequest, data: web::Data<AppState>, commands: Json<commandGraphData>)-> impl Responder{
    let mut graphs = data.graphs.lock().unwrap();
    let graph = match graphs.get_mut(&commands.graphID){Some(val)=>val, None=> return HttpResponse::BadRequest().into()};
    if !corroborate_claim(graph, &request).await{
        return HttpResponse::Unauthorized().into();
    }
    HttpResponse::Ok().content_type("text").body(match graph.execute_commands(commands.commands.clone()).err(){
        Some(error) => match error{
            graph::GraphError::Cycle=>"cycle",
            graph::GraphError::EdgeNotFound => "edge not found",
            graph::GraphError::NodeNotFound => "node not found",
            graph::GraphError::MismatchedNodes => "mismatched sockets",
            graph::GraphError::UnknownCommand => "unknown command",
            graph::GraphError::IllFormedCommand => "ill-formed command"
             },
        None => "ok"
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
    HttpResponse::Ok().content_type("application/json").body(serde_json::to_string(&descriptors).unwrap())
}

#[derive(Deserialize)]
struct UploadData{
    fileName:String,
    fileType:String,
    file:String
}

#[post("/upload")]
async fn upload_image(request: HttpRequest,mut payload: Multipart)->impl Responder{
    let mut file_data = Vec::<u8>::new();
    let mut fileName: Option<String>= None;
    let username = match get_username_from_token(match request.cookie("session"){Some(val)=>val,None=>return Redirect::to("/login").see_other()}.value().to_owned()){Some(val)=>val,None=>return Redirect::to("/login").see_other()};
    while let Some(mut field) = payload.try_next().await.unwrap() {
        let content_disposition = field.content_disposition();
        let field_name = content_disposition.get_name().unwrap();
        match field_name {
            "file" => {
                while let Some(chunk) = match field.try_next().await{Ok(val)=>val,Err(_)=>return Redirect::to("/upload_image?bad_image").see_other()} {
                    file_data.extend_from_slice(&chunk);
                }
            }
            "fileName" => {
                let bytes = match field.try_next().await{
                    Ok(val)=>val,
                    Err(_)=>return Redirect::to("/upload_image?bad_name").see_other()
                };
                fileName = String::from_utf8(bytes.unwrap().to_vec()).ok();
            }
            _ => {}
        }
    }
    let cleanFileName = util::sanitize(&match fileName{Some(val)=>val,None=>return Redirect::to("/upload_image?bad_name").see_other()},true);
    if(cleanFileName.is_empty()){
        return Redirect::to("/upload_image?bad_name").see_other();
    }
    let image = match image::load_from_memory(&file_data){Ok(val)=>val, Err(_)=>return Redirect::to("/upload_image?bad_image").see_other()};
    match image.save_with_format(util::RESOURCE_PATH.clone()+r"\images\"+r"\"+&util::sanitize(&username,true)+r"\"+&cleanFileName+".png", image::ImageFormat::Png){
        Ok(_)=>(),
        Err(_)=>return Redirect::to("/upload_image?bad_image").see_other()
    };

    Redirect::to("/graph").see_other()
}

#[get("/upload_image")]
async fn upload_image_html()->impl Responder{
    let contents = fs::read_to_string(util::RESOURCE_PATH.clone()+"upload_image.html").unwrap();

    HttpResponse::Ok().content_type("text/html").body(contents)
}

#[get("/upload_image.js")]
async fn upload_image_javascript()->impl Responder{
    let contents = fs::read_to_string(util::RESOURCE_PATH.clone()+"upload_image.js").unwrap();

    HttpResponse::Ok().content_type("text/html").body(contents)
}

#[derive(Deserialize)]
struct Info{
    name:String
}

async fn images(_req: HttpRequest, info: web::Path<Info>) -> impl Responder {

    let name = info.name.clone();
    HttpResponse::Ok()
    .insert_header(CacheControl(vec![CacheDirective::NoCache, CacheDirective::NoStore, CacheDirective::MustRevalidate]))
    .content_type("image/png")
    .body(web::block(move || std::fs::read(util::RESOURCE_PATH.clone()+r"images\" + &name)).await.unwrap().expect(&info.name))
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
    
    let result = sqlx::query("CREATE TABLE IF NOT EXISTS graphIDs (id INTEGER NOT NULL, username VARCHAR(64) NOT NULL);").execute(&db).await.unwrap();
    println!("Create user table result: {:?}", result);

    let result = sqlx::query("CREATE TABLE IF NOT EXISTS graphs (username VARCHAR(64) NOT NULL, graphName VARCHAR(128), graph TEXT NOT NULL);").execute(&db).await.unwrap();
    println!("Create user table result: {:?}", result);

    

    let graphs = HashMap::new();
    let appState = web::Data::new(AppState{
        graphs: Mutex::new(graphs),
        currentID: Mutex::new(0),
        db: db
    });

    HttpServer::new(move || {
        App::new()
            .app_data(appState.clone())
            .service(login)
            .service(login_html)
            .service(create_account)
            .service(create_account_html)
            .service(create_account_javascript)
            .service(login_javascript)
            .service(utils_javascript)
            .service(retrieve_graph)
            .service(save_graph)
            .service(load_graph)
            .service(create_graph)
            .service(retrieve_graph_file_list)
            .service(graph_selector_html)
            .service(graph_selector_javascript)
            .service(process_graph)
            .service(graph_page_html)
            //.service(Files::new("/images", RESOURCE_PATH.clone()+"/images"))
            //.service(web::resource("/images/{name}").route(web::get().to(images)))
            .service(graph_page_javascript_main)
            .service(graph_page_javascript_matrix)
            .service(graph_page_javascript_graph)
            .service(graph_page_javascript_UI)
            .service(style)
            .service(retrieve_node_templates)
            .service(command_graph)
            .service(upload_image_html)
            .service(upload_image_javascript)
            .service(upload_image)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}