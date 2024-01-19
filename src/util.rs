use std::fs;


lazy_static!{
    pub static ref RESOURCE_PATH : String = r"C:\Users\Administrator\Desktop\rust\photo-graph-main\photo-graph\src\resources\".to_string();
    pub static ref SECRET : String = fs::read_to_string(r"C:\Users\Administrator\Desktop\secret.txt").unwrap();
    pub static ref DOMAIN: String = "http://127.0.0.1:8080".to_owned();
}

pub fn sanitize(dirty:&str,isDir:bool)->String{

    let mut clean= dirty.to_owned();

    let mut firstPeriod = true;
    clean = clean.chars().rev().filter(|c|{if *c == '.'{if firstPeriod && !isDir{firstPeriod=false;true}else{false}}else{true}}).rev().collect();
    clean = clean.replace(r"\", "");
    
    clean
}
