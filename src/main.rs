extern crate iron;
extern crate mount;
extern crate staticfile;
extern crate url;

use iron::prelude::*;
use iron::headers::ContentType;
use iron::modifiers::Header;
use iron::status;

use mount::Mount;
use staticfile::Static;

use std::env;
use std::fs::read_dir;
use std::path::Path;
use std::path::PathBuf;

use url::percent_encoding::percent_decode;

fn main() {

    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage is: {} PATH", args[0]);
    }
    let ref path_arg = args[1];
    let path = PathBuf::from(path_arg);

    fn list_dir(req: &mut Request) -> IronResult<Response> {
        // is there a way not to have to read args every time?
        let args: Vec<_> = env::args().collect();
        if args.len() != 2 {
            println!("Usage is: {} PATH", args[0]);
        }
        let ref path_arg = args[1];
        let mut basepath = PathBuf::from(path_arg);
        basepath.push(""); // normalize trailing slash
        let basepath_len = {
            let basepath_str = basepath.to_str();
            basepath_str.unwrap().len()
        };
        for component in req.url.path() {
            basepath.push(percent_decode(component.as_bytes()).decode_utf8_lossy().into_owned());
        }
        // ok lets build this
        let mut html = String::new();
        // for sorted output: http://stackoverflow.com/questions/40021882/how-to-sort-readdir-iterator
        for entry in read_dir(basepath.clone()).expect("unable to read directory") {
            let file = entry.unwrap();
            let path = file.path();
            let path_str = path.to_str().expect("path is invalid utf8");
            let filename = file.file_name();
            let filename_str = filename.to_str().expect("filename is invalid utf8");
            // remove the runtime argument from the front
            let web_path = &path_str[basepath_len..];
            let link = match path.is_dir() {
                true => format!("<a href=\"/ls/{}/\">{}</a>", web_path, filename_str),
                false => format!("<a href=\"/dl/{0}\">{1}</a> -- (<a href=\"/st/{0}\">stream</a>)", web_path, filename_str)
            };

            html = format!("{}<li>{}</li>", html, link);
        }
        html = format!("<ul><li><a href=\"..\">Parent Directory</a></li></ul><ul>{}</ul>", html);
        let mut resp = Response::with((status::Ok, html));
        resp.set_mut(Header(ContentType::html()));
        Ok(resp)
    }

    fn stream(req: &mut Request) -> IronResult<Response> {
        let web_path = req.url.path().join("/");
        let mut resp = Response::with((status::Ok, format!("<!DOCTYPE html><html><head></head><body><video src=\"/dl/{}\" autoplay controls></video></body></html>", web_path)));
        resp.set_mut(Header(ContentType::html()));
        Ok(resp)
    }

    let mut mount = Mount::new();
    mount.mount("/dl/", Static::new(Path::new(&path)))
         .mount("/ls/", list_dir)
         .mount("/st/", stream);

    Iron::new(mount).http("localhost:3000").unwrap();
}