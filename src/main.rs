use reqwest;
use serde_json::{self, Value};
use serde::{Serialize,Deserialize};

mod epub;
use epub::*;

use clap::Parser;
use clap_derive::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Full kemono url to a post
    //#[arg(short, long)]
    //post_id: Option<String>,
    /// Full kemono url to creators profile
    #[arg(short, long)]
    creator_id: Option<String>, //dev shit (^\0~0/^)

    // Creator service
    //#[arg(short, long)]
    //service: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    //let id = "31891971".to_string();
    let id = args.creator_id.unwrap();
       //let id = "https://kemono.su/patreon/user/31891971?o=50".to_string();
    let epub_request = create_epubrequest(id).await;
    //println!("Profile: {:#?}", epub_request.creator.posts.as_ref().unwrap().len());
    create_epub(epub_request).await.unwrap();
}
