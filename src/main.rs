mod graph;
#[macro_use]
extern crate lazy_static;

use std::hash::Hash;
use std::io::Cursor;
use std::{fs, collections::HashMap};
use std::sync::Mutex;


lazy_static!{
    static ref RESOURCE_PATH : String = r"C:\Users\Administrator\Desktop\rust\photo-graph\src\resources\".to_string();
}


use actix_web::web::Json;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, HttpRequest, http::{StatusCode, header::{CacheControl, CacheDirective}}};
use actix_files::Files;
use graph::Graph;
use serde::Deserialize;

use crate::graph::node::NodeStatic;


#[derive(Deserialize)]
struct param{
    x : String
}

struct AppState{
    graphs : Mutex<HashMap<u64,graph::Graph>>
}


#[get("/")]
async fn graph_page_html() -> impl Responder {
    let contents = fs::read_to_string(RESOURCE_PATH.clone()+"test.html").unwrap();

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
async fn process_graph(data: web::Data<AppState>)-> impl Responder {
    let mut graphs = data.graphs.lock().unwrap();
    let mut outputImage = graphs.get_mut(&0).unwrap().process();
    //outputImage.save(RESOURCE_PATH.clone()+r"images\output_0.png").unwrap();
    let mut bytes: Vec<u8> = Vec::new();
    outputImage.write_to(&mut Cursor::new(&mut bytes), image::ImageOutputFormat::Png).unwrap();
    HttpResponse::Ok().content_type("image/png").body(bytes)
}



#[post("/command")]
async fn command_graph(data: web::Data<AppState>, commands: Json<graph::Commands>)-> impl Responder{
    let mut graphs = data.graphs.lock().unwrap();
    HttpResponse::Ok().content_type("text").body(match graphs.get_mut(&0).unwrap().execute_commands(commands.clone()).err(){
        Some(error) => match error{
            graph::GraphError::Cycle=>"cycle",
            graph::GraphError::EdgeNotFound => "edge",
            graph::GraphError::NodeNotFound => "node",
            graph::GraphError::MismatchedNodes => "mismatched",
            graph::GraphError::UnknownCommand => "unknown",
            graph::GraphError::IllFormedCommand => "ill-formed"
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
    HttpResponse::Ok().content_type("text").body(serde_json::to_string(&descriptors).unwrap())
}

#[derive(Deserialize)]
struct Info{
    name:String
}

async fn images(req: HttpRequest, info: web::Path<Info>) -> impl Responder {

    let name = info.name.clone();
    HttpResponse::Ok()
    .insert_header(CacheControl(vec![CacheDirective::NoCache, CacheDirective::NoStore, CacheDirective::MustRevalidate]))
    .content_type("image/png")
    .body(web::block(move || std::fs::read(RESOURCE_PATH.clone()+r"images\" + &name)).await.unwrap().expect(&info.name))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {


    let mut graphs = HashMap::new();
    graphs.insert(0, Graph::new());
    let appState = web::Data::new(AppState{
        graphs: Mutex::new(graphs)
    });
    HttpServer::new(move || {
        App::new()
            .app_data(appState.clone())
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