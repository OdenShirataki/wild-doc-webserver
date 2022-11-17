use hyper::{Request, Body, Response, Method, StatusCode, Uri};
use wild_doc_client_lib::WildDocClient;

use crate::response;

pub(super) async fn request(wd_host:String,wd_port:String,req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let mut response = Response::new(Body::empty());
    if let Some(host)=req.headers().get("host"){
        if let Ok(host)=host.to_str(){
            if let Some(host)=host.split(":").collect::<Vec<&str>>().get(0){
                let host=host.to_string();
                let document_root="document/".to_owned();

                let mut wd=WildDocClient::new(&wd_host,&wd_port,&document_root,&host);

                match req.method(){
                    &Method::GET=>{
                        if let Some(filename)=get_filename(&document_root,&host,req.uri()){
                            if let Ok(b)=response::make(&mut wd,&filename,None){
                                *response.body_mut()=b;
                            }else{
                                *response.body_mut()=Body::from("erro");
                            }
                        }else{
                            let filename=document_root.to_owned()+"/route.xml";
                            if let Ok(b)=response::make(&mut wd,&filename,None){
                                *response.body_mut()=b;
                            }else{
                                *response.body_mut()=Body::from("erro");
                            }
                        }
                        let headers=response.headers_mut();
                        headers.append("content-type","text/html; charset=utf-8".parse().unwrap());
                    }
                    ,&Method::POST=>{
                        if let Some(filename)=get_filename(&document_root,&host,req.uri()){
                            if let Ok(b)=response::make(
                                &mut wd
                                ,&filename
                                ,Some(hyper::body::to_bytes(req.into_body()).await?)
                            ){
                                *response.body_mut()=b;
                            }else{
                                *response.body_mut()=Body::from("erro");
                            }
                        }else{
                            if let Ok(b)=response::make(
                                &mut wd
                                ,&(document_root.to_owned()+&host+"/route.xml")
                                ,Some(hyper::body::to_bytes(req.into_body()).await?)
                            ){
                                *response.body_mut()=b;
                            }else{
                                *response.body_mut()=Body::from("erro");
                            }
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

fn get_filename(document_root:&str,hostname:&str,uri:&Uri)->Option<String>{
    let path=uri.path();
    if path.ends_with("/index.html")!=true{
        let filename=document_root.to_owned()+hostname+"/public"+path+&if path.ends_with("/"){
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