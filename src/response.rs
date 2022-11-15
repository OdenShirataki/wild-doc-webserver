use std::{fs::File, io::Read, collections::HashMap};

use hyper::{Body, body::Bytes};
use url::form_urlencoded;
use wild_doc_client_lib::WildDocClient;

pub(crate) fn make(document_root:&str,filename:&str,post:Option<Bytes>)->Body{
    let mut contents=Vec::new();

    let mut wdc=WildDocClient::new(document_root);
    if let Some(post)=post{
        let params=form_urlencoded::parse(post.as_ref())
            .into_owned()
            .collect::<HashMap<String, String>>()
        ;
        dbg!(params);
    }
    if let Ok(mut f)=File::open(filename){
        let mut xml=String::new();
        if let Ok(_)=f.read_to_string(&mut xml){
            contents=wdc.exec(&xml);
        }
    }
    Body::from(contents)
}