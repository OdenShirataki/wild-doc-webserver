use std::{fs::File, io::Read};

use hyper::Body;
use wild_doc_client_lib::WildDocClient;

pub(crate) fn make(filename:&str)->Body{
    let mut contents = Vec::new();
    if let Ok(mut f) = File::open(filename){
        let mut sml=String::new();
        if let Ok(_)=f.read_to_string(&mut sml){
            let mut sc=WildDocClient::new("document");
            contents=sc.exec(&sml);
        }
    }
    Body::from(contents)
}