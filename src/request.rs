use hyper::{Request, Body, Response, Method, StatusCode, Uri};

use crate::response;

pub(super) async fn request(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let mut response = Response::new(Body::empty());
    if let Some(host)=req.headers().get("host"){
        if let Ok(host)=host.to_str(){
            if let Some(host)=host.split(":").collect::<Vec<&str>>().get(0){
                let document_root="document/".to_owned()+host;
                match req.method(){
                    &Method::GET=>{
                        if let Some(filename)=get_filename(&document_root,req.uri()){
                            *response.body_mut()=response::make(&document_root,&filename,None);
                        }else{
                            let filename=document_root.to_owned()+"/route.xml";
                            *response.body_mut()=response::make(&document_root,&filename,None);
                        }
                        let headers=response.headers_mut();
                        headers.append("content-type","text/html; charset=utf-8".parse().unwrap());
                    }
                    ,&Method::POST=>{
                        if let Some(filename)=get_filename(&document_root,req.uri()){
                            *response.body_mut()=response::make(
                                &document_root
                                ,&filename
                                ,Some(hyper::body::to_bytes(req.into_body()).await?)
                            );
                        }else{
                            *response.body_mut()=response::make(
                                &document_root
                                ,&(document_root.to_owned()+"/route.xml")
                                ,Some(hyper::body::to_bytes(req.into_body()).await?)
                            );
                        }
                        let headers=response.headers_mut();
                        headers.append("content-type","text/html; charset=utf-8".parse().unwrap());
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

fn get_filename(document_root:&str,uri:&Uri)->Option<String>{
    let path=uri.path();
    if path.ends_with("/index.html")!=true{
        let filename=document_root.to_owned()+"/public"+path+&if path.ends_with("/"){
            "index.html"
        }else{
            ""
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