mod graph;
#[macro_use]
extern crate lazy_static;

use std::hash::Hash;
use std::{fs, collections::HashMap};
use std::sync::Mutex;


lazy_static!{
    static ref RESOURCE_PATH : String = r"C:\Users\Administrator\Desktop\rust\photo-graph\src\resources\".to_string();
}


use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, HttpRequest, http::{StatusCode, header::{CacheControl, CacheDirective}}};
use actix_files::Files;
use graph::Graph;
use serde::Deserialize;


#[derive(Deserialize)]
struct param{
    x : String
}

struct AppState{
    graphs : Mutex<HashMap<u64,graph::Graph>>
}


#[get("/")]
async fn graphPageHtml() -> impl Responder {
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

#[get("/test.js")]
async fn graphPageJavascript()-> impl Responder {
    let contents = fs::read_to_string(RESOURCE_PATH.clone()+"test.js").unwrap();

    HttpResponse::Ok().content_type("text/javascript").body(contents)
}

#[post("/process")]
async fn processGraph(data: web::Data<AppState>)-> impl Responder {
    let mut graphs = data.graphs.lock().unwrap();
    graphs.get_mut(&0).unwrap().process().save(RESOURCE_PATH.clone()+r"images\output_0.png").unwrap();

    HttpResponse::Ok()
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
            .service(processGraph)
            .service(graphPageHtml)
            //.service(Files::new("/images", RESOURCE_PATH.clone()+"/images"))
            .service(web::resource("/images/{name}").route(web::get().to(images)))
            .service(graphPageJavascript)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}