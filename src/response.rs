use std::{fs::File, io::Read, collections::HashMap};

use hyper::{Body, body::Bytes};
use url::form_urlencoded;
use wild_doc_client_lib::WildDocClient;

pub(crate) fn make(document_root:&str,hostname:&str,filename:&str,post:Option<Bytes>)->std::io::Result<Body>{
    
    if let Some(post)=post{
        let params=form_urlencoded::parse(post.as_ref())
            .into_owned()
            .collect::<HashMap<String, String>>()
        ;
        dbg!(params);
    }
    let mut f=File::open(filename)?;
    let mut xml=String::new();
    f.read_to_string(&mut xml)?;
    Ok(Body::from(
        WildDocClient::new(document_root,hostname).exec(&xml)?
    ))
}