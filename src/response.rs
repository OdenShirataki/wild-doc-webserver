use std::{fs::File, io::Read};

use hyper::Body;
use wild_doc_client_lib::WildDocClient;

pub(crate) fn make(document_root:&str,filename:&str)->Body{
    let mut contents=Vec::new();
    if let Ok(mut f)=File::open(filename){
        let mut xml=String::new();
        if let Ok(_)=f.read_to_string(&mut xml){
            dbg!(&xml);
            let mut wd=WildDocClient::new(document_root);
            contents=wd.exec(&xml);
            dbg!(&contents);
        }
    }
    Body::from(contents)
}