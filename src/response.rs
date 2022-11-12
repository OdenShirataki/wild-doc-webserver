use std::{fs::File, io::Read};

use hyper::Body;
use semilattice_client_lib::SemilatticeClient;

pub(crate) fn make(filename:&str)->Body{
    let mut contents = Vec::new();
    if let Ok(mut f) = File::open(filename){
        let mut sml=String::new();
        if let Ok(_)=f.read_to_string(&mut sml){
            let mut sc=SemilatticeClient::new("document");
            contents=sc.exec(&sml);
        }
    }
    Body::from(contents)
}