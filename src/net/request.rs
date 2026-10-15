use reqwless::request::Method;

pub struct HttpRequest<'a> {
    pub method: Method,
    pub path: &'a str,
    pub headers: &'a [(&'a str, &'a str)],
    pub body: Option<&'a [u8]>,
}
