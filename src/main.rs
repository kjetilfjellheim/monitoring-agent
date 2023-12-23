use std::collections::HashMap;

enum TypeSend<'a> {
    Tcp(VerifyTcpRequest<'a>, Option<ServerCertificateType<'a>>, Option<ClientCertificateType<'a>>, VerifyTcp),      
    Http(VerifyHttpRequest<'a>, Option<HttpsServerCertificateType<'a>>, Option<ServerCertificateType<'a>>, Option<ClientCertificateType<'a>>, VerifyHttp)   
}

enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Option,
    Head
}

struct HttpsServerCertificateType<'a> {
    certificate: &'a str
}

struct ServerCertificateType<'a> {
    certificate: &'a str
}

struct ClientCertificateType<'a> {
    certificate: &'a str,
    password: &'a str
}

struct VerifyTcpRequest<'a> {
    ip: &'a str,
    port: i32
}

struct VerifyHttpRequest<'a> {
    url: &'a str,
    method: HttpMethod,
    body: &'a str,
    headers: HashMap<&'a str, &'a str>
}

struct VerifyTcp {
}

struct VerifyHttp {
    http_code: i32
}

enum TlsType<'a> {
    Tls(&'a str),                       // Server certificate
    Mtls(&'a str, &'a str, &'a str)     // Server certificate, Client key, Client password
}

struct Monitor<'a> {
    test_type: TypeSend<'a>,
    tls: Option<TlsType<'a>>
}


fn main() {

}
