extern crate hyper_tls;
extern crate html5ever;
extern crate hyper;
extern crate futures;

use hyper_tls::HttpsConnector;
use hyper::Client;
use hyper::rt::{self,Stream,Future};

use html5ever::rcdom::{RcDom,Handle};
use html5ever::tendril::StrTendril;
use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use html5ever::rcdom::NodeData;

use std::collections::HashSet;
use std::default::Default;
use std::sync::Arc;
use std::borrow::Borrow;
use std::clone::Clone;

use futures::future;

#[derive(Clone)]
struct FocusHref {
    ahref:HashSet<String>,
    imghref:HashSet<String>,
}
impl FocusHref {
    fn new(url:String) -> impl Future<Item = FocusHref,Error = ()> {
        let url:hyper::Uri = url.parse().unwrap();
        let https = HttpsConnector::new(4).unwrap();
        let client = Client::builder().build::<_,hyper::Body>(https);
        let mut dummy = Arc::new(
            FocusHref {
                ahref:HashSet::new(),
                imghref:HashSet::new(),
            });
        let done = client.get(url).map(move |res| {
            let before = res.into_body().chunks(100).into_future().and_then(move |resource| {
                if let Some(ref items) = resource.0 {
                    let mut str_tendrils:Vec<StrTendril> = Vec::new();
                    for item in items.into_iter() {
                        let tendril = StrTendril::from(std::str::from_utf8(item.as_ref()).unwrap());
                        str_tendrils.push(tendril);
                    }
                    let dom = parse_document(RcDom::default(),Default::default()).from_iter(str_tendrils);
                    Arc::make_mut(&mut dummy).visit(dom.document);
                    return Ok(Arc::make_mut(&mut dummy).clone());
                } else {
                    panic!("occur error!");
                }
            }).wait().unwrap();
            return before;
        }).map_err(|_| {

        });
        return done;
    }
    fn visit(&mut self,handle:Handle) {
        let nodes = handle;
        match nodes.data {
            NodeData::Element {
                ref name,
                ref attrs,
                ..
            } => {
                let a_span = "a";
                let img_span = "img";
                let local_checking = &name.local.to_string();
                let attr_span = "href";
                let img_attr = "data-src";
                if local_checking == a_span {
                    for attr in attrs.borrow().iter() {
                        let attr_checking = &attr.name.local.to_string();
                        if attr_checking == attr_span {
                            self.ahref.insert(attr.value.to_string());
                        }
                    }
                }
                if local_checking == img_span {
                    for attr in attrs.borrow().iter() {
                        let attr_checking = &attr.name.local.to_string();
                        if attr_checking == img_attr {
                            self.imghref.insert(attr.value.to_string());
                        }
                    }
                }
            },
            _ => ()
        }
        for child in nodes.children.borrow().iter() {
            self.visit(child.clone());
        }
    }
}
fn main() {
    let focus = FocusHref::new("https://alpha.wallhaven.cc/random".to_string());
    let done = focus.and_then(|hrefs| {
        println!("<a> label href list ---------->");
        for elem in hrefs.ahref.iter() {
            println!("{}",elem);
        }
        println!("ending of <a> label href list ---------");
        println!("<img> label href list ---------->");
        for elem in hrefs.imghref.iter() {
            println!("{}",elem);
        }
        println!("ending of <img> label href list --------");
        return Ok(());
    }).map_err(|_| {
    });
    rt::run(done);
}