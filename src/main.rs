mod graph;
#[macro_use]
extern crate lazy_static;

use std::fs::File;

use std::io::Cursor;
use std::thread;
use std::{fs, collections::HashMap,io::Write};
use std::sync::Mutex;


lazy_static!{
    static ref RESOURCE_PATH : String = r"C:\Users\Administrator\Desktop\rust\photo-graph\src\resources\".to_string();
    static ref SECRET : String = fs::read_to_string(r"C:\Users\Administrator\Desktop\secret.txt").unwrap();
}

use std::time;

use actix_web::body::MessageBody;
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};

use actix_web::web::Json;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, HttpRequest, http::{header::{CacheControl, CacheDirective}}};

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



async fn authenticate(db:&sqlx::Pool<Sqlite>,userCred:&UserCredentials)->bool{
    let user_results = sqlx::query_as::<_, User>("SELECT username, password_hash FROM users WHERE username=$1")
        .bind(userCred.username.clone())
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



fn get_username_from__encoded_claim(token:String)->Option<String>{

    let mut validator = Validation::default();
    validator.leeway = 1;
    validator.validate_exp = true;
    let token_data = match decode::<LoginClaim>(
        &token,
        &DecodingKey::from_secret(SECRET.as_bytes()),
        &validator,
    ) {
        Ok(c) => c,
        Err(_) => {
            return None;
        }
    };

    
    return Some(token_data.claims.username);
}

async fn corroborate_claim(grph : &graph::Graph, encoded_claim:String)->bool{
    let res = get_username_from__encoded_claim(encoded_claim);
    match res{
        Some(username) => grph.get_user() == username,
        None => false
    }
}

#[post("/createAccount")]
async fn create_account(data: web::Data<AppState>, userCred:Json<UserCredentials>)->impl Responder{
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
    let user_results = sqlx::query_as::<_, User>("SELECT username, password_hash FROM users WHERE username=$1")
        .bind(userCred.username.clone())
        .fetch_all(&data.db)
        .await
        .unwrap();

    if user_results.len() != 0{
        println!("{0}, {1}", user_results[0].username, user_results[0].password_hash);
    }
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
        &EncodingKey::from_secret(SECRET.as_bytes()),
    ) {
        Ok(t) => t,
        Err(_) => panic!(),
    };

    if !authenticate(&data.db, &userCredentials).await{
        return HttpResponse::Unauthorized().content_type("text").body("fail");
    }

    HttpResponse::Ok().content_type("text").body(token)
}


#[get("/retrieveGraphFileList")]
async fn retrieve_graph_file_list()->impl Responder{
    let paths = fs::read_dir(RESOURCE_PATH.clone()+r"\graphs").unwrap();
    let mut out = vec![];
    for path in paths {
        out.push(path.unwrap().file_name().to_str().unwrap().to_string());
    }
    HttpResponse::Ok().content_type("application/json").body(serde_json::to_string(&out).unwrap())
}

#[post("/loadGraph")]
async fn load_graph(data: web::Data<AppState>, body:web::Bytes)->impl Responder{
    let mut graphs = data.graphs.lock().unwrap();
    let mut currentID = data.currentID.lock().unwrap();
    let contents = fs::read_to_string(RESOURCE_PATH.clone()+r"\graphs\"+&String::from_utf8(body.to_vec()).unwrap()).unwrap();

    let mut loadedGraph = Graph::new("".to_string());
    loadedGraph.execute_commands(serde_json::from_str(&contents).unwrap()).unwrap();
    graphs.insert(*currentID, loadedGraph);
    *currentID += 1;
    HttpResponse::Ok().content_type("text").body((*currentID-1).to_string())
}

#[post("/retrieveGraph")]
async fn retrieve_graph(data: web::Data<AppState>, body:web::Bytes)->impl Responder{
    let graphs = data.graphs.lock().unwrap();
    if(!graphs.contains_key(&(String::from_utf8(body.to_vec()).unwrap()).parse().unwrap())){
        return HttpResponse::Ok().content_type("application/json").body("{\"isValid\":\"no\"}");
    }
    HttpResponse::Ok().content_type("application/json").body(serde_json::to_string(&graphs.get(&(String::from_utf8(body.to_vec()).unwrap()).parse().unwrap()).unwrap().get_command_history()).unwrap())
}

#[derive(Deserialize)]
struct SaveInfo{
    fileName: String,
    graphID : u64
}

#[post("/saveGraph")]
async fn save_graph(data: web::Data<AppState>, saveInfo:Json<SaveInfo>)->impl Responder{
    let graphs = data.graphs.lock().unwrap();
    let mut output = File::create(RESOURCE_PATH.clone()+r"\graphs\"+&saveInfo.fileName).unwrap();
    write!(output,"{}",serde_json::to_string(&graphs.get(&saveInfo.graphID).unwrap().get_command_history()).unwrap()).unwrap();
    HttpResponse::Ok()
}

#[post("/createGraph")]
async fn create_graph(data: web::Data<AppState>)->impl Responder{
    let mut graphs = data.graphs.lock().unwrap();
    let mut currentID = data.currentID.lock().unwrap();
    let newGraph = Graph::new("".to_string());
    graphs.insert(*currentID, newGraph);
    *currentID += 1;
    HttpResponse::Ok().content_type("text").body((*currentID-1).to_string())
}

#[get("/")]
async fn graph_selector_html()-> impl Responder{
    let contents = fs::read_to_string(RESOURCE_PATH.clone()+"graph_selector.html").unwrap();

    HttpResponse::Ok().content_type("text/html").body(contents)
}

#[get("/utils.js")]
async fn utils_javascript()->impl Responder{
    let contents = fs::read_to_string(RESOURCE_PATH.clone()+"utils.js").unwrap();
    HttpResponse::Ok().content_type("text/javascript").body(contents)
}

#[get("/graph_selector.js")]
async fn graph_selector_javascript()->impl Responder{
    let contents = fs::read_to_string(RESOURCE_PATH.clone()+"graph_selector.js").unwrap();

    HttpResponse::Ok().content_type("text/javascript").body(contents)
}

#[get("/create_account")]
async fn create_account_html()->impl Responder{
    let contents = fs::read_to_string(RESOURCE_PATH.clone()+"create_account.html").unwrap();

    HttpResponse::Ok().content_type("text/html").body(contents)
}

#[get("/create_account.js")]
async fn create_account_javascript()->impl Responder{
    let contents = fs::read_to_string(RESOURCE_PATH.clone()+"create_account.js").unwrap();

    HttpResponse::Ok().content_type("text/html").body(contents)
}

#[get("/login")]
async fn login_html()->impl Responder{
    let contents = fs::read_to_string(RESOURCE_PATH.clone()+"login.html").unwrap();

    HttpResponse::Ok().content_type("text/html").body(contents)
}

#[get("/login.js")]
async fn login_javascript()->impl Responder{
    let contents = fs::read_to_string(RESOURCE_PATH.clone()+"login.js").unwrap();

    HttpResponse::Ok().content_type("text/javascript").body(contents)
}

#[get("/graph")]
async fn graph_page_html() -> impl Responder {
    let contents = fs::read_to_string(RESOURCE_PATH.clone()+"main.html").unwrap();

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
    let contents = fs::read_to_string(RESOURCE_PATH.clone()+"main.js").unwrap();

    HttpResponse::Ok().content_type("text/javascript").body(contents)
}

#[get("/style.css")]
async fn style()-> impl Responder {
    let contents = fs::read_to_string(RESOURCE_PATH.clone()+"style.css").unwrap();

    HttpResponse::Ok().content_type("text/css").body(contents)
}

#[get("/UI.js")]
async fn graph_page_javascript_UI()-> impl Responder {
    let contents = fs::read_to_string(RESOURCE_PATH.clone()+"UI.js").unwrap();

    HttpResponse::Ok().content_type("text/javascript").body(contents)
}

#[get("/matrix.js")]
async fn graph_page_javascript_matrix()-> impl Responder {
    let contents = fs::read_to_string(RESOURCE_PATH.clone()+"matrix.js").unwrap();

    HttpResponse::Ok().content_type("text/javascript").body(contents)
}

#[get("/graph.js")]
async fn graph_page_javascript_graph()-> impl Responder {
    let contents = fs::read_to_string(RESOURCE_PATH.clone()+"graph.js").unwrap();

    HttpResponse::Ok().content_type("text/javascript").body(contents)
}


#[post("/process")]
async fn process_graph(data: web::Data<AppState>, body:web::Bytes)-> impl Responder {
    let mut graphs = data.graphs.lock().unwrap();
    let outputImage = graphs.get_mut(&(String::from_utf8(body.to_vec()).unwrap()).parse().unwrap()).unwrap().process();
    //outputImage.save(RESOURCE_PATH.clone()+r"images\output_0.png").unwrap();
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
async fn command_graph(data: web::Data<AppState>, commands: Json<commandGraphData>)-> impl Responder{
    let mut graphs = data.graphs.lock().unwrap();
    HttpResponse::Ok().content_type("text").body(match graphs.get_mut(&commands.graphID).unwrap().execute_commands(commands.commands.clone()).err(){
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
    //println!("{}",serde_json::to_string(&descriptors).unwrap());
    HttpResponse::Ok().content_type("application/json").body(serde_json::to_string(&descriptors).unwrap())
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
    .body(web::block(move || std::fs::read(RESOURCE_PATH.clone()+r"images\" + &name)).await.unwrap().expect(&info.name))
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
    
    let result = sqlx::query("CREATE TABLE IF NOT EXISTS graphIDs (id NOT NULL, username VARCHAR(64) NOT NULL);").execute(&db).await.unwrap();
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
            .service(web::resource("/images/{name}").route(web::get().to(images)))
            .service(graph_page_javascript_main)
            .service(graph_page_javascript_matrix)
            .service(graph_page_javascript_graph)
            .service(graph_page_javascript_UI)
            .service(style)
            .service(retrieve_node_templates)
            .service(command_graph)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}