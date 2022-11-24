use std::{fs::File, io::Read};

use wild_doc_client_lib::{WildDocClient, WildDocResult};

pub(crate) fn make(wdc:&mut WildDocClient,filename:&str,json:&str)->std::io::Result<WildDocResult>{
    let mut f=File::open(filename)?;
    let mut xml=String::new();
    f.read_to_string(&mut xml)?;
    wdc.exec(&xml,json)
}