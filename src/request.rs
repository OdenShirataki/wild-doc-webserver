use hyper::{Request, Body, Response, Method, StatusCode};

use crate::response;

pub(super) async fn request(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let mut response = Response::new(Body::empty());
    if let Some(host)=req.headers().get("host"){
        if let Ok(host)=host.to_str(){
            if let Some(host)=host.split(":").collect::<Vec<&str>>().get(0){
                match req.method(){
                    &Method::GET=>{
                        if let Some(filename)=get_filename(host,&req){
                            *response.body_mut() = response::make(&filename);
                            let headers=response.headers_mut();
                            headers.append("content-type","text/html; charset=utf-8".parse().unwrap());
                        }else{
                            *response.status_mut() = StatusCode::NOT_FOUND;
                        }
                    }
                    ,&Method::POST=>{
                        *response.body_mut() = req.into_body();
                    }
                    ,_ => {
                        *response.status_mut() = StatusCode::NOT_FOUND;
                    }
                };
            }
        }
    }
    
    Ok(response)
}

fn get_filename(host:&str,req: &Request<Body>)->Option<String>{
    let path=req.uri().path();
    if path.ends_with("/index.html")!=true{
        let filename="document/".to_owned()+&if path.ends_with("/"){
            host.to_owned()+path+"index.sml"
        }else{
            host.to_owned()+path
        };
        if std::path::Path::new(&filename).exists(){
            Some(filename)
        }else{
            None
        }
    }else{
        None
    }
}