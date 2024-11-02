// pub mod h3server {
//     pub mod httpreq {
//         include!(concat!(env!("OUT_DIR"), "/h3server.httpreq.rs"));
//     }
// }

// use bytes::Bytes;
// use h3server::httpreq;

// pub fn create_http_request_type(
//     url: String,
//     method: String,
//     version: String,
//     headers: String,

//     body: Bytes,
// ) -> httpreq::HttpRequest {
//     h3server::httpreq::HttpRequest {
//         url,
//         method,
//         version,
//         headers,

//         body: body.to_vec(),
//     }
// }