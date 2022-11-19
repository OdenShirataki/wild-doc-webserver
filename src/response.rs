use std::{fs::File, io::Read};

use hyper::{Body};
use wild_doc_client_lib::WildDocClient;

pub(crate) fn make(wdc:&mut WildDocClient,filename:&str,json:&str)->std::io::Result<Body>{
    let mut f=File::open(filename)?;
    let mut xml=String::new();
    f.read_to_string(&mut xml)?;
    Ok(Body::from(
        wdc.exec(&xml,json)?
    ))
}