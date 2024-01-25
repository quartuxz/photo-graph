use std::fs;
use dotenv::dotenv;
use std::env;


fn get_env_var(varName: &str)->String{
    dotenv().ok();
    env::var(varName).unwrap()
}

lazy_static!{
    pub static ref RESOURCE_PATH : String = "resources/".to_string();
    pub static ref SECRET : String = fs::read_to_string("secret.txt").unwrap();
    pub static ref HOST : String = get_env_var("HOST");
    pub static ref MEM_THRESHOLD : usize = get_env_var("MEMORY_THRESHOLD").parse().unwrap();
}

pub fn sanitize(dirty:&str,isDir:bool)->String{

    let mut clean= dirty.to_owned();

    let mut firstPeriod = true;
    clean = clean.chars().rev().filter(|c|{if *c == '.'{if firstPeriod && !isDir{firstPeriod=false;true}else{false}}else{true}}).rev().collect();
    clean = clean.replace(r"\", "");
    clean = clean.replace("/", "");
    
    clean
}
