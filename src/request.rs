use std::io::Read;
use std::{collections::HashMap, fs::File};
use std::convert::Infallible;
use bytes::Bytes;

use hyper::header::HeaderName;
use hyper::http::HeaderValue;
use hyper::{Request, Body, Response, Method, StatusCode, HeaderMap};
use url::form_urlencoded;
use multer::Multipart;
use futures::stream::once;
use serde::{Serialize,Serializer, ser::SerializeMap};

use wild_doc_client_lib::WildDocClient;

enum Param{
    Array(HashMap<String,String>)
    ,Scalar(String) 
}
impl Serialize for Param {
    fn serialize<S>(&self,serializer: S)->Result<S::Ok,S::Error> where S: Serializer
    {
        match self{
            Self::Array(array)=>{
                let mut map = serializer.serialize_map(Some(array.len()))?;
                for (k, v) in array {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
            ,Self::Scalar(string)=>{
                serializer.serialize_str(string)
            }
        }
    }
}

fn get(response:&mut Response<Body>,wdc:&mut WildDocClient,filename:&str,json:&str){
    let mut f=File::open(filename).unwrap();
    let mut xml=String::new();
    f.read_to_string(&mut xml).unwrap();
    if let Ok(r)=wdc.exec(&xml,json){
        if let Ok(result_options)=serde_json::from_str::<HashMap<String,serde_json::Value>>(r.options_json()){
            if let Some(headers)=result_options.get("headers"){
                if let serde_json::Value::Object(headers)=headers{
                    let mut response_headers=HeaderMap::new();
                    for (k,v) in headers{
                        if let(
                            Ok(k)
                            ,Ok(v)
                        )=(
                            HeaderName::from_bytes(k.as_bytes())
                            ,HeaderValue::from_str(&v.to_string())
                        ){
                            response_headers.insert(k,v);
                        }
                    }
                    *response.headers_mut()=response_headers;
                }
            }
        }
        *response.body_mut()=Body::from(
            r.body().to_vec()
        );
    }else{
        *response.body_mut()=Body::from("error");
    }
}
pub(super) async fn request(wd_host:String,wd_port:String,req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let mut response = Response::new(Body::empty());

    let mut headers:HashMap<String,String>=HashMap::new();
    for (key, value) in req.headers(){
        headers.insert(key.to_string(),value.to_str().unwrap().to_owned());
    }
    if let Some(host)=headers.get("host"){
        if let Some(host)=host.split(":").collect::<Vec<&str>>().get(0){
            let host=host.to_string();
            let uri=req.uri().path().to_owned();
            println!("request: {} {}",host,uri);
            let document_root="document/".to_owned();

            let mut wdc=WildDocClient::new(&wd_host,&wd_port,&document_root,&host);

            let mut params_all:HashMap<String,Param>=HashMap::new();
            params_all.insert("uri".to_owned(),Param::Scalar(uri.to_owned()));

            match req.method(){
                &Method::GET=>{
                    params_all.insert("headers".to_owned(),Param::Array(headers));
                    let json=serde_json::to_string(&params_all).unwrap();

                    if let Some(filename)=get_filename(&document_root,&host,&uri){
                        get(&mut response,&mut wdc,&filename,&json);
                    }else{
                        let filename=document_root.to_owned()+&host+"/route.xml";
                        get(&mut response,&mut wdc,&filename,&json);
                    }
                    let headers=response.headers_mut();
                    headers.append("content-type","text/html; charset=utf-8".parse().unwrap());
                }
                ,&Method::POST=>{
                    let content_type=headers.get("content-type").unwrap();
                    let body = hyper::body::to_bytes(req.into_body()).await?;
                    let params={
                        if content_type=="application/x-www-form-urlencoded"{
                            form_urlencoded::parse(body.as_ref())
                                .into_owned()
                                .collect::<HashMap<String, String>>()
                        }else if content_type.starts_with("multipart/form-data;"){
                            let mut params:HashMap<String, String>=HashMap::new();
                            let boundary:Vec<&str>=content_type.split("boundary=").collect();
                            let boundary=boundary[1];
                            let mut multipart = Multipart::new(once(async move { Result::<Bytes, Infallible>::Ok(Bytes::from(body)) }),boundary);
                            while let Some(mut field) = multipart.next_field().await.unwrap() {
                                //let file_name = field.file_name();
                                while let Some(chunk)=field.chunk().await.unwrap() {
                                    if let Some(name)=field.name(){
                                        params.insert(name.to_owned(),std::str::from_utf8(&chunk).unwrap().to_owned());
                                    }
                                }
                            }
                            params
                        }else{
                            HashMap::new()
                        }
                    };
                    
                    params_all.insert("post".to_owned(),Param::Array(params));
                    params_all.insert("headers".to_owned(),Param::Array(headers));
                    let json=serde_json::to_string(&params_all).unwrap();

                    let mut f=File::open(&(document_root.to_owned()+&host+"/post.xml")).unwrap();
                    let mut xml=String::new();
                    f.read_to_string(&mut xml).unwrap();

                    let _=wdc.exec(&xml,&json);

                    if let Some(filename)=get_filename(&document_root,&host,&uri){
                        get(&mut response,&mut wdc,&filename,&json);
                    }else{
                        let filename=document_root.to_owned()+"route.xml";
                        get(&mut response,&mut wdc,&filename,&json);
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
    
    Ok(response)
}

fn get_filename(document_root:&str,hostname:&str,uri:&str)->Option<String>{
    if uri.ends_with("/index.html")!=true{
        let filename=document_root.to_owned()+hostname+"/public"+uri+&if uri.ends_with("/"){
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