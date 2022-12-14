use bytes::Bytes;
use futures::stream::once;
use hyper::{
    header::HeaderName, http::HeaderValue, Body, HeaderMap, Method, Request, Response, StatusCode,
};
use multer::Multipart;
use std::{
    collections::HashMap,
    convert::Infallible,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
use wild_doc_client_lib::WildDocClient;

fn get_static_filename(document_root: &Path, hostname: &str, uri: &str) -> Option<PathBuf> {
    if uri.ends_with("/index.html") != true {
        let mut filename = document_root.to_path_buf();
        filename.push(hostname);
        filename.push("static");
        let mut uri = uri.to_owned();
        uri.remove(0);
        filename.push(&uri);
        if uri.ends_with("/") {
            filename.push("index.html");
        }
        if filename.exists() {
            Some(filename)
        } else {
            None
        }
    } else {
        None
    }
}

#[derive(Serialize)]
struct UploadFile {
    file_name: String,
    content_type: String,
    len: usize,
    data: String, //(base64)
}
struct UploadFileWrapper {
    key: String,
    file: UploadFile,
}
pub(super) async fn request(
    wd_host: String,
    wd_port: String,
    document_dir: String,
    req: Request<Body>,
) -> Result<Response<Body>, hyper::Error> {
    let document_dir = std::path::PathBuf::from(document_dir);
    let mut response = Response::new(Body::empty());

    let mut headers: HashMap<String, String> = HashMap::new();
    for (key, value) in req.headers() {
        headers.insert(key.to_string(), value.to_str().unwrap().to_owned());
    }
    if let Some(host) = headers.get("host") {
        if let Some(host) = host.split(":").collect::<Vec<&str>>().get(0) {
            let host = host.to_string();
            let uri = req.uri().path().to_owned();
            if let Some(static_file) = get_static_filename(&document_dir, &host, &uri) {
                let mut f = File::open(static_file).unwrap();
                let mut buf = Vec::new();
                f.read_to_end(&mut buf).unwrap();
                *response.body_mut() = Body::from(buf);
            } else {
                let mut wdc = WildDocClient::new(&wd_host, &wd_port, &document_dir, &host);

                let mut params_all: HashMap<String, serde_json::Value> = HashMap::new();

                params_all.insert("uri".to_owned(), serde_json::Value::String(uri));

                if let Some(query) = req.uri().query() {
                    if let Ok(query) = queryst::parse(query) {
                        params_all.insert("get".to_owned(), query);
                    }
                }

                let mut files = vec![];
                let params_all = match req.method() {
                    &Method::GET => {
                        if let Ok(headers) = serde_json::to_string(&headers) {
                            if let Ok(headers) = serde_json::from_str(&headers) {
                                params_all.insert("headers".to_owned(), headers);
                            }
                        }
                        Some(params_all)
                    }
                    &Method::POST => {
                        let content_type = headers.get("content-type").unwrap();
                        let body = hyper::body::to_bytes(req.into_body()).await?;

                        if content_type == "application/x-www-form-urlencoded" {
                            if let Ok(body) = std::str::from_utf8(body.as_ref()) {
                                if let Ok(params) = queryst::parse(body) {
                                    params_all.insert("post".to_owned(), params);
                                }
                            }
                        } else if content_type.starts_with("multipart/form-data;") {
                            let boundary: Vec<&str> = content_type.split("boundary=").collect();
                            let boundary = boundary[1];
                            let mut multipart = Multipart::new(
                                once(async move { Result::<Bytes, Infallible>::Ok(body) }),
                                boundary,
                            );
                            let mut params = "".to_owned();
                            while let Some(mut field) = multipart.next_field().await.unwrap() {
                                while let Some(chunk) = field.chunk().await.unwrap() {
                                    if let Some(name) = field.name() {
                                        if let (Some(file_name), Some(content_type)) =
                                            (field.file_name(), field.content_type())
                                        {
                                            if params.len() > 0 {
                                                params += "&";
                                            }
                                            let key = boundary.to_owned()
                                                + "-"
                                                + &files.len().to_string();
                                            params += &urlencoding::encode(name).into_owned();
                                            params += "=";
                                            params += &key;
                                            files.push(UploadFileWrapper {
                                                key,
                                                file: UploadFile {
                                                    file_name: file_name.to_owned(),
                                                    content_type: content_type.to_string(),
                                                    len: chunk.len(),
                                                    data: base64::encode(chunk),
                                                },
                                            });
                                        } else {
                                            if let Ok(v) = std::str::from_utf8(&chunk) {
                                                if params.len() > 0 {
                                                    params += "&";
                                                }
                                                params += &urlencoding::encode(name).into_owned();
                                                params += "=";
                                                params += &urlencoding::encode(v).into_owned();
                                            }
                                        }
                                    }
                                }
                            }
                            if let Ok(params) = queryst::parse(&params) {
                                params_all.insert("post".to_owned(), params);
                            }
                        }

                        if let Ok(headers) = serde_json::to_string(&headers) {
                            if let Ok(headers) = serde_json::from_str(&headers) {
                                params_all.insert("headers".to_owned(), headers);
                            }
                        }
                        Some(params_all)
                    }
                    _ => None,
                };
                if let Some(params_all) = params_all {
                    if let Ok(mut json) = serde_json::to_string(&params_all) {
                        for f in files {
                            if let Ok(file) = serde_json::to_string(&f.file) {
                                let key = "\"".to_string() + &f.key + "\"";
                                json = json.replace(&key, &file);
                            }
                        }
                        let mut filename = document_dir.clone();
                        filename.push(host);
                        filename.push("request.xml");
                        let mut f = File::open(filename).unwrap();
                        let mut xml = String::new();
                        f.read_to_string(&mut xml).unwrap();
                        if let Ok(r) = wdc.exec(&xml, &json) {
                            let result_options = serde_json::from_str::<
                                HashMap<String, serde_json::Value>,
                            >(r.options_json());

                            let mut need_response_body = true;
                            if let Ok(result_options) = result_options {
                                let mut response_status = StatusCode::FOUND;
                                if let Some(status) = result_options.get("status") {
                                    let status = status.to_string();
                                    if status == "404" {
                                        response_status = StatusCode::NOT_FOUND;
                                    }
                                }
                                let mut response_headers = HeaderMap::new();
                                if let Some(headers) = result_options.get("headers") {
                                    if let serde_json::Value::Object(headers) = headers {
                                        for (k, v) in headers {
                                            if let Some(v) = v.as_str() {
                                                if let (Ok(k), Ok(v)) = (
                                                    HeaderName::from_bytes(k.as_bytes()),
                                                    HeaderValue::from_str(v),
                                                ) {
                                                    response_headers.insert(k, v);
                                                }
                                            }
                                        }
                                    }
                                }
                                if response_headers.contains_key("location") {
                                    response_status = StatusCode::SEE_OTHER;
                                    need_response_body = false;
                                }
                                *response.headers_mut() = response_headers;
                                if response_status != StatusCode::FOUND {
                                    *response.status_mut() = response_status;
                                }
                            }
                            if need_response_body {
                                *response.body_mut() = Body::from(r.body().to_vec());
                            }
                        } else {
                            *response.body_mut() = Body::from("error");
                        }
                    }
                } else {
                    *response.status_mut() = StatusCode::NOT_FOUND;
                }
            }
        }
    }
    Ok(response)
}
