use std::fs::File;
use std::io::{stdin, stdout, Stdin, Write};
use css::STYLE;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::*;

mod simple_xml_builder;
mod css;
use simple_xml_builder::XMLElement;

#[derive(Deserialize,Serialize,Debug,Clone)]
pub struct Creator {
    pub id: String,
    pub name: String,
    pub service: String,
    pub indexed: String, //Todo: Yo lets make the dates work ye?
    pub updated: String,
    pub public_id: String,
    pub relation_id: Option<String>,
    pub posts: Option<Vec<Post>>,
}

#[derive(Deserialize,Serialize,Debug,Clone)]
pub struct Post {
    pub id: String,
    pub user: String,
    pub service: String,
    pub title: String,
    pub content: String,
    pub embed: Value,
    pub shared_file: bool,
    pub added: String,
    pub published: String,
    pub edited: Option<String>,
    pub file: Value,//Option<File>,
    pub attachments: Value,//Option<Vec<File>>,
    pub poll: Value,
    pub captions: Option<bool>,
    pub tags: Option<String>,
    pub opf_extras: Option<Vec<String>>
}

pub struct EpubRequest {
    pub creator: Creator,
    //pub page_count: u32,
    pub title: Title,
    //pub image: Image,
}

pub enum Title {
    Custom((String, bool)),
    StartToEnd,
    NameNumber,
    NameFirstPostName,
    NameLastPostName,
    NameStartToEnd,
    CreatorName
}

//pub enum Image {
//    Cusom(String),
//    Generic
//}

impl Creator {
    pub async fn fetch_posts(&mut self, max: u32) {
        let mut i: u32 = 0;
        loop {
            let posts = reqwest::get(format!("https://kemono.su/api/v1/{}/user/{}?o={}", self.service, self.id, i))
            .await.unwrap().text().await.unwrap().replace("\"", r#"""#);
            
            let posts: Vec<Post> = serde_json::from_str(&posts[..]).unwrap();
            if posts.is_empty(){
                return;
            }
            match &mut self.posts {
                Some(x) => {
                    x.extend(posts);
                },
                None => {
                    self.posts = Some(posts);
                },
            }
            i = i+50;
            //Forced to increment by 50 because that is the forced offset by kemono
            if i > max*50 {
                println!("Reached desired page");
                return;
            }
            println!("Page fetched: {}, Total number of posts: {}", i/50, i);
            
        }
    }
}

impl EpubRequest {
    async fn create_creator(address: String) -> Creator{
        let (service, id) = {
            //https://kemono.su/patreon/user/31891971?o=50
            //^Example^
            //First we get where the domain starts  >kemono.su
            //Second we get the service             >patreon
            //Third we get the id of a creator,     >31891971
            //and discard anything extra after it.

            let kemono = address.find("kemono").expect("Not a kemono url")+7;

            let service_start = address[kemono..].find("/").expect("Cant find service")+kemono+1;
            let service_end = address[service_start..].find("/").expect("Cant find service")+service_start;

            let user_start = address[service_end..].find("/").expect("Cant find id")+service_end+1;
            let user_end = address[user_start..].find("/").expect("Cant find id")+user_start;

            let creator_id_start = address[user_end..].find("/").expect("Cant find creator id")+user_end+1;
            let result = match address[creator_id_start..].find("?") {
                Some(x) => {(
                    &address[service_start..service_end],           //Service
                    &address[creator_id_start..x+creator_id_start]  //ID
                )},
                None => {(
                    &address[service_start..service_end],           //Service
                    &address[creator_id_start..]                    //ID
                )},                 
            };
            
            //  The code below doesnt work because it doesnt discard ?o=50, 
            //  apparently that causes it to return posts.
            //  the code that could cause this MUST be a 
            //  5 star Heaven Golden Chaos Body Immortal Gread Dao treasure. :/

            //  ok never mind, it just probably reads the offset and ignores everything else...
            //  Im leaving the comment above anyway

            //let result = if address[creator_id_start..].find("/").is_some() {
            //    let creator_id_end = address[creator_id_start..].find("/").unwrap();
            //    //https://kemono.su/patreon/user/31891971 / post/124265700
            //
            //    let post_start = address[creator_id_end..].find("/").expect("Cant find post");
            //    let post_end = address[creator_id_end..].find("/").expect("Cant find post");
            //
            //    let post_id = address[post_end..];
            //
            //    //TODO: The above is to epub only posts
            //
            //    (&address[service_start..service_end],                          //Service
            //    &address[creator_id_start..creator_id_end+creator_id_start])    //Creator ID
            //}   //TODO: Add a way for post ids to be returned and then if there is a post id
            //    //  only epub that one singular post, because if there is a post id it's a post url
            //else {
            //    (&address[service_start..service_end],                          //Service
            //    &address[creator_id_start..])                                   //Creator ID
            //};
            result
        };
        let request = reqwest::get(
            format!("https://kemono.su/api/v1/{service}/user/{id}/profile"))
            .await.unwrap().text().await.unwrap().replace("\"", r#"""#);
        //println!("{:#?}", serde_json::from_str::<Value>(&request[..]).unwrap());
        //println!("{:#?}", (service, id));
        serde_json::from_str(&request[..]).unwrap()
        //Makes a creator
        //Will error with some services
        //TODO: Maybe fix this :3
    }
}

pub async fn create_epubrequest (address: String) -> EpubRequest {
    //Filling up EpubRequest
    let page_count = 'page: loop {
        let handle = stdin();
        let mut buf = String::new();
        print!("|| Do you want to fetch all pages? [Y/n] ");
        stdout().flush().unwrap();  //have to flush otherwise STDIN goes before text
        handle.read_line(&mut buf).expect("failed to read line");
        buf = buf.trim().to_lowercase();
        if buf != "y" && buf != "n" {
            continue;
        }
        if buf != "n" {
            break 'page 65535;
        }
        //TODO: Let the user pick for eg. pages from 5-10
        loop {
            print!("|| How many pages would you like to fetch? ");
            stdout().flush().unwrap();  //have to flush otherwise STDIN goes before Text
            buf.clear();                //clear the buffer
            handle.read_line(&mut buf).expect("failed to read line");
            let buf = buf.trim().parse::<u32>();
            if buf.is_err() {
                println!("Error: Not a number or too large, please try again");
                continue;
            }
            break 'page buf.unwrap();
        }
    };

    let title = 'title: loop {
        let handle = stdin();
        let mut buf = String::new();
        print!("|| Do you want to set a custom title? [y/N] ");
        stdout().flush().unwrap();  //have to flush otherwise STDIN goes before text
        handle.read_line(&mut buf).expect("failed to read line");
        buf = buf.trim().to_lowercase();
        if buf != "y" && buf != "n" {
            continue;
        }
        if buf != "y"{
            break 'title Title::CreatorName;
        }

        print!("{}",
            ("|| Pick title format:\n".to_string()
            + "    1.{First post} - {Last post}\n"
            + "    2.{Creator name} - {First post} - {Last post}\n"
            + "    3.{Creator name} - {Post count}\n"
            + "    4.{Creator name} - {First post}\n"
            + "    5.{Creator name} - {Last post}\n"
            + "    6.{Creator name}\n"
            + "    7.Custom\n"
            + "Please enter a number (1-7): "));
        stdout().flush().unwrap();      //have to flush otherwise STDIN goes before Text
        loop {
            buf.clear();                //clear the buffer
            handle.read_line(&mut buf).expect("failed to read line");
            let buf = buf.trim().parse::<u8>();
            if buf.is_err() {
                println!("Error: Not a number or too large, please try again");
                continue;
            }

            match buf.unwrap() {
                1 => break 'title Title::StartToEnd,        //StartToEnd,
                2 => break 'title Title::NameStartToEnd,    //NameStartToEnd,
                3 => break 'title Title::NameNumber,        //NameNumber,
                4 => break 'title Title::NameFirstPostName, //NameFirstPostName,
                5 => break 'title Title::NameLastPostName,  //NameLastPostName,
                6 => break 'title Title::CreatorName,       //CreatorName
                7 => break 'title custom_title(&handle),    //Custom((String, bool)),
                _ => {println!("Please pick an option that was defined"); continue;},
            }
        }

    };

    EpubRequest {
        creator: {
            let mut creator = EpubRequest::create_creator(address).await;
            creator.fetch_posts(page_count).await;
            creator
        },
        title,
        //image:,
    }
}

fn custom_title(handle: &Stdin) -> Title {
    let mut buf = String::new();
    loop {
        print!("Custom Tile: ");
        stdout().flush().unwrap();  //have to flush otherwise STDIN goes before Text
        buf.clear();                //clear the buffer
        if handle.read_line(&mut buf).is_err() {
            println!("Failed to read buffer try again");
            continue;
        };
        buf = buf.trim().to_string();
        if buf.contains(".epub") {
            break Title::Custom((buf, true));
        }
        break Title::Custom((buf, false));
    }
}

fn match_title(creator: &Creator, title: &Title, epub: bool) -> String{
    let posts = creator.posts.as_ref().unwrap();
    let output: String;
    match title {
        Title::StartToEnd => {
            output = format!("{} — {}", posts.last().unwrap().title, posts.first().unwrap().title)
        },
        Title::NameStartToEnd => {
            output = format!("{} — {} — {}", creator.name, posts.last().unwrap().title, posts.first().unwrap().title)
        },
        Title::NameNumber => {
            output = format!("{} — {}", creator.name, posts.len())
        },
        Title::NameFirstPostName => {
            output = format!("{} — {}", creator.name, posts.last().unwrap().title)
        },
        Title::NameLastPostName => {
            output = format!("{} — {}", creator.name, posts.first().unwrap().title)
        },
        Title::CreatorName => {
            output = format!("{}", creator.name)
        },
        Title::Custom(x) => {
            output = x.0.to_string();

            if epub && x.1 == false {
                return custom_title_replace(output, creator) + ".epub"
            }
            if !epub && x.1 == true {
                return custom_title_replace(output, creator)
            }
            return custom_title_replace(output, creator)
        },
    }
    if epub {
        return custom_title_replace(output, creator) + ".epub"
    }
    custom_title_replace(output, creator)
}

fn custom_title_replace(input: String, creator: &Creator) -> String {
    let posts = creator.posts.as_ref().unwrap();
    input
        .replace("{Creator.name}", &creator.name)
        .replace("{Posts.first}", &posts.last().unwrap().title)
        .replace("{Posts.last}", &posts.first().unwrap().title)
        .replace("{Posts.count}", &posts.len().to_string())
        .replace(r#"\"#, "")
}

fn create_dirs(zip: &mut ZipWriter<File>, options: SimpleFileOptions) -> Result<(), std::io::Error> {
    zip.add_directory("META-INF", options)?;
    zip.add_directory_from_path("OEBPS/Styles", options)?;
    zip.add_directory_from_path("OEBPS/Text", options)?;
    Ok(())
}

pub async fn create_epub(epubrequest: EpubRequest) -> Result<(), std::io::Error>{
    let mut creator = epubrequest.creator;
    let application = br#"application/epub+zip"#;
    let archive = {
        File::create(match_title(&creator, &epubrequest.title, true)).unwrap()
    };
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored);
    let mut zip = ZipWriter::new(archive);

    zip.start_file_from_path("mimetype", options)?;
    zip.write_all(application)?;

    create_dirs(&mut zip, options)?;

    let mut container = XMLElement::new("container");
    container.add_attribute("version", "1.0");
    container.add_attribute("xmlns","urn:oasis:names:tc:opendocument:xmlns:container");

    let mut rootfile = XMLElement::new("rootfile");
    rootfile.add_attribute("full-path", "OEBPS/content.opf");
    rootfile.add_attribute("media-type","application/oebps-package+xml");

    let mut rootfiles = XMLElement::new("rootfiles");
    rootfiles.add_child(rootfile);

    container.add_child(rootfiles);

    zip.start_file_from_path("META-INF/container.xml", options)?;
    zip.write_all(container.to_string().as_bytes())?;

    zip.start_file_from_path("OEBPS/Styles/stylesheet.css", options)?;
    zip.write_all(&STYLE)?;

    create_chapters(&mut creator, &mut zip, options).await?;
    create_toc(&creator, &mut zip, options)?;
    create_cover(&creator, &mut zip, options).await?;
    create_content(&creator, &mut zip, options, &epubrequest.title)?;
    Ok(())
}

async fn create_cover(creator: &Creator, zip: &mut ZipWriter<File>, options: SimpleFileOptions) -> Result<(), std::io::Error>{
    //TODO: 1. Fix the hrefs (Get the images) //Already did the first step
    //      2. Make your own Cover and CSS becouse its absolute Trash
    //      3, ???
    //      4. Profit
    let mut html = XMLElement::new("html");
    html.add_attribute("xmlns", "http://www.w3.org/1999/xhtml");

    let mut head = XMLElement::new("head");
    
    let mut title = XMLElement::new("title");
    title.add_text("Cover");
    
    let mut link = XMLElement::new("link");
    link.add_attribute("href", "../Styles/stylesheet.css");
    link.add_attribute("type", "text/css");
    link.add_attribute("rel", "stylesheet");
    
    head.add_child(link);
    head.add_child(title);
    html.add_child(head);

    let mut body = XMLElement::new("body");
    let mut div = XMLElement::new("div");
    div.add_attribute("class", "svg_outer svg_inner");
    
    let mut svg = XMLElement::new("svg");
    svg.add_attribute("xmlns", "http://www.w3.org/2000/svg");
    svg.add_attribute("xmlns:xlink", "http://www.w3.org/1999/xlink");
    svg.add_attribute("height", "99%");
    svg.add_attribute("width", "100%");
    svg.add_attribute("version", "1.1");
    svg.add_attribute("preserveAspectRatio", "xMidYMid meet");
    svg.add_attribute("viewBox", "0 0 400 600");
    
    let mut image = XMLElement::new("image");
    image.add_attribute("xlink:href", "../../Images/".to_string() + &format!(r#"https://img.kemono.su/icons/{0}/{1}"#,
        creator.service, creator.id).replace("/", "_").replace(":", "-"));
    download_image(format!(r#"https://img.kemono.su/icons/{0}/{1}"#,
        creator.service, creator.id), zip, options).await.unwrap();
    image.add_attribute("width", "400");
    image.add_attribute("height", "600");
 
    let mut desc = XMLElement::new("desc");
    desc.add_text(r#"https://www.royalroadcdn.com/public/covers-large/46868-reincarnated-into-a-time-loop-dungeon-as.jpg?time=1705791928"#);
    //TODO: fix desc
    svg.add_child(image);
    svg.add_child(desc);

    div.add_child(svg);

    body.add_child(div);
    
    html.add_child(body);

    zip.start_file_from_path("OEBPS/Text/Cover.xhtml", options)?;
    zip.write_all(html.to_string().replace(r#"<?xml version = "1.0" encoding = "UTF-8"?>"#, 
r#"<?xml version = "1.0" encoding = "UTF-8"?>
<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.1//EN" "http://www.w3.org/TR/xhtml11/DTD/xhtml11.dtd">"#).as_bytes())?;
    Ok(())
}

fn create_content(creator: &Creator, zip: &mut ZipWriter<File>, options: SimpleFileOptions, custom_title: &Title) -> Result<(), std::io::Error> {
    let mut content = XMLElement::new("package");
    content.add_attribute("xmlns", "http://www.idpf.org/2007/opf");
    content.add_attribute("version", "2.0");
    content.add_attribute("unique-identifier","BookId");

    let mut metadata = XMLElement::new("metadata");
    metadata.add_attribute("xmlns:dc","http://purl.org/dc/elements/1.1/");
    metadata.add_attribute("xmlns:opf", "http://www.idpf.org/2007/opf");

    let mut title = XMLElement::new("dc:title");
    title.add_text(match_title(&creator, &custom_title, false));

    let mut language = XMLElement::new("dc:language");
    language.add_text("en");

    let mut date = XMLElement::new("dc:date");
    date.add_text(&creator.updated);

    let mut creator_dc = XMLElement::new("dc:creator");
    creator_dc.add_text(&creator.name);

    let mut identifier = XMLElement::new("dc:identifier");
    identifier.add_attribute("id","BookId");
    identifier.add_attribute("opf:scheme","URI");
    identifier.add_text(format!("https://kemono.su/{0}/user/{1}", creator.service, creator.id));

    let mut contributor = XMLElement::new("dc:contributor");
    contributor.add_attribute("opf:role","bkp");
    contributor.add_text(r#"[https://github.com/TinyCatBag/Kemono_to_EPUB] (ver. 1.0.1.0)"#);

    let mut meta = XMLElement::new("meta");
    meta.add_attribute("content","cover-image");
    meta.add_attribute("name","cover");

    let mut source = XMLElement::new("dc:source");
    source.add_attribute("id","id.cover-image");
    source.add_text(format!(r#"https://img.kemono.su/icons/{0}/{1}"#,creator.service, creator.id));

    metadata.add_child(title);
    metadata.add_child(language);
    metadata.add_child(date);
    metadata.add_child(creator_dc);
    metadata.add_child(identifier);
    metadata.add_child(contributor);
    metadata.add_child(meta);
    metadata.add_child(source);

    let mut manifest = XMLElement::new("manifest");
    let mut cover_image = XMLElement::new("item");
    
    cover_image.add_attribute("href","../Images/".to_string() + 
        &format!("https://img.kemono.su/icons/{0}/{1}"
        , creator.service, creator.id).replace("/", "_").replace(":", "-"));
    cover_image.add_attribute("id", "cover-image");
    cover_image.add_attribute("media-type", "image/webp");

    manifest.add_child(cover_image);

    let mut style = XMLElement::new("item");
    style.add_attribute("href", "Styles/stylesheet.css");
    style.add_attribute("id", "stylesheet");
    style.add_attribute("media-type", "text/css");

    let mut toc_item = XMLElement::new("item");
    toc_item.add_attribute("href", "toc.ncx");
    toc_item.add_attribute("id", "ncx");
    toc_item.add_attribute("media-type", "application/x-dtbncx+xml");
    
    let mut cover = XMLElement::new("item");
    cover.add_attribute("href", "Text/Cover.xhtml");
    cover.add_attribute("id", "cover");
    cover.add_attribute("media-type", "application/xhtml+xml");

    manifest.add_child(style);
    manifest.add_child(toc_item);
    manifest.add_child(cover);

    let mut spine = XMLElement::new("spine");
    spine.add_attribute("toc", "ncx");

    let mut cover_itemref = XMLElement::new("itemref");
    cover_itemref.add_attribute("idref", "cover");

    spine.add_child(cover_itemref);
    for x in 0..creator.posts.as_ref().unwrap().len() {
        let mut source = XMLElement::new("dc:source");
        source.add_attribute("id",format!("id.xhtml{0}", x+1));
        source.add_text(
            format!(r#"https://kemono.su/{0}/user/{1}/post/{2}"#
            , creator.service, creator.id, creator.posts.as_ref().unwrap()[x].id));
        metadata.add_child(source);

        let mut item = XMLElement::new("item");
        item.add_attribute("href", format!("Text/Chapter_{0}.xhtml", x+1));
        item.add_attribute("id", format!("xhtml{0}", x+1));
        item.add_attribute("media-type","application/xhtml+xml");

        manifest.add_child(item);

        let mut itemref = XMLElement::new("itemref");
        itemref.add_attribute("idref", format!("xhtml{}", x+1));

        spine.add_child(itemref);
        if creator.posts.as_ref().unwrap()[x].opf_extras == None {
            continue;
        }
        for z in 0..creator.posts.as_ref().unwrap()[x].opf_extras.as_ref().unwrap().len() {
            let extra = &creator.posts.as_ref().unwrap()[x].opf_extras.as_ref().unwrap()[z];
            let mut image = XMLElement::new("dc:source");
            image.add_attribute("id", format!("id.image{}", z));
            image.add_text(extra);
            metadata.add_child(image);

            let mut image = XMLElement::new("item");
            image.add_attribute("href", format!("../Images/{}", extra.replace("/", "_").replace(":", "-")));
            image.add_attribute("id", format!("image{}", z));
            image.add_attribute("media-type", "image/jpeg");
            manifest.add_child(image);
            //TODO: Finish this Lmao
        }
    }
    content.add_child(metadata);
    content.add_child(manifest);
    content.add_child(spine);

    let mut guide = XMLElement::new("guide");

    let mut reference = XMLElement::new("reference");
    reference.add_attribute("href", "Text/Cover.xhtml");
    reference.add_attribute("title", "Cover");
    reference.add_attribute("type", "Cover");

    guide.add_child(reference);

    content.add_child(guide);

    zip.start_file_from_path("OEBPS/content.opf", options)?;
    zip.write_all(content.to_string().as_bytes())?;
    Ok(())
}

fn create_toc(creator: &Creator, zip: &mut ZipWriter<File>, options: SimpleFileOptions) -> Result<(), std::io::Error> { 
    let mut ncx = XMLElement::new("ncx");
    ncx.add_attribute("xmlns", "http://www.daisy.org/z3986/2005/ncx/");
    ncx.add_attribute("version", "2005-1");
    ncx.add_attribute("xml:lang", "en");

    let mut head = XMLElement::new("head");

    let mut meta_one = XMLElement::new("meta");
    meta_one.add_attribute("content", format!("https://kemono.su/{}/user/{}", creator.service, creator.id));
    meta_one.add_attribute("name", "dtb:uid");

    let mut meta_two = XMLElement::new("meta");
    meta_two.add_attribute("content", "2");
    meta_two.add_attribute("name", "dtb:depth");

    let mut meta_three = XMLElement::new("meta");
    meta_three.add_attribute("content", "0");
    meta_three.add_attribute("name", "dtb:totalPageCount");

    let mut meta_four = XMLElement::new("meta");
    meta_four.add_attribute("content", "0");
    meta_four.add_attribute("name", "dtb:maxPageNumber");

    let mut doctitle = XMLElement::new("docTitle");
    let mut text = XMLElement::new("text");
    text.add_text(&creator.name);

    let mut navmap = XMLElement::new("navMap");

    head.add_child(meta_one);
    head.add_child(meta_two);
    head.add_child(meta_three);
    head.add_child(meta_four);
    ncx.add_child(head);
    doctitle.add_child(text);
    ncx.add_child(doctitle);

    for x in 0..creator.posts.as_ref().unwrap().len() {
        let reverse = creator.posts.as_ref().unwrap().len()-x-1;

        let mut navpoint = XMLElement::new("navPoint");
        navpoint.add_attribute("id", format!("body{}", x+1));
        navpoint.add_attribute("playOrder", x+1);

        let mut navlabel = XMLElement::new("navLabel");
        
        let mut text = XMLElement::new("text");
        text.add_text(&creator.posts.as_ref().unwrap()[reverse].title);

        let mut content = XMLElement::new("content");
        content.add_attribute("src", format!("Text/Chapter_{}.xhtml", x+1));

        navlabel.add_child(text);
        navpoint.add_child(navlabel);
        navpoint.add_child(content);
        navmap.add_child(navpoint);
    }
    ncx.add_child(navmap);
    zip.start_file_from_path("OEBPS/toc.ncx", options)?;
    zip.write_all(ncx.to_string().as_bytes())?;
    Ok(())
}

async fn create_chapters(creator: &mut Creator, zip: &mut ZipWriter<File>, options: SimpleFileOptions) -> Result<(), std::io::Error> {
    for x in 0..creator.posts.as_ref().unwrap().len() {
        let reverse = &creator.posts.as_ref().unwrap().len()-x-1;
        let post = &mut creator.posts.as_mut().unwrap()[reverse];
        if post.content.contains("<img") {
            post.fix_img(zip, options).await;
        }
        zip.start_file_from_path(format!("OEBPS/Text/Chapter_{}.xhtml", x+1), options)?;
        let mut html = XMLElement::new("html");
        html.add_special(r#"<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.1//EN" "http://www.w3.org/TR/xhtml11/DTD/xhtml11.dtd">"#);
        html.add_attribute("xmlns", "http://www.w3.org/1999/xhtml");
        let mut head = XMLElement::new("head");
        let mut title = XMLElement::new("title");
        title.add_text(&post.title);

        let mut link = XMLElement::new("link");
        link.add_attribute("href", "../Styles/stylesheet.css");
        link.add_attribute("type", "text/css");
        link.add_attribute("rel", "stylesheet");

        head.add_child(title);
        head.add_child(link);
        html.add_child(head);

        let mut body = XMLElement::new("body");
        let mut h1 = XMLElement::new("h1");
        h1.add_text(&post.title);

        let mut div = XMLElement::new("div");
        div.add_attribute("class", "chapter-inner chapter-content");
        div.add_text(post.content
            .replace("<br>", "<br/>").replace("<hr>", "<hr/>")
            .replace("<img>", "<img/>").replace("<link>", "<link/>")
            .replace("</p>", "</p>\n").replace("<br>", "<br>\n"));

        body.add_child(h1);
        body.add_child(div);
        html.add_child(body);
        zip.write_all(html.to_string().as_bytes())?;
    }
    Ok(())
}

impl Post {
    async fn fix_img(self: &mut Self, zip: &mut ZipWriter<File>, options: SimpleFileOptions) {
        let mut offset: usize = 0;
        let matches = self.content.matches("<img").count();
        
        for x in 0..matches{
            if x == 0 {
                let start = self.content.find("<img").unwrap();
                let end = self.content[start..].find(">").unwrap()+start+1;

                let url_start = self.content[start..].find(r#"src=""#).unwrap()+start+5;
                let url_end = self.content[url_start..].find(r#"""#).unwrap()+url_start;
                download_image(self.content[url_start..url_end].to_string(), zip, options).await.unwrap();
                match &mut self.opf_extras {
                    Some(x) => x.push(self.content[url_start..url_end].to_string()),
                    None => self.opf_extras = Some(vec![self.content[url_start..url_end].to_string()]),
                };

                offset = offset + start;
                self.content.replace_range(start..end, &self.content[start..end]
                    .replace("/", "_").replace(":", "-").replace(">", r#" alt="" />"#).replace("http", "/Images/http"));
                    
                    //TODO: V3 Holy shit is this spagget, if the url dont start with http WE ARE COOKED
                
                //println!("loop: {x}, offset: {replacments:#?}");
                continue;
            }
            offset = offset+4;
            let start = self.content[offset..].find("<img").unwrap()+offset;
            let end = self.content[start..].find(">").unwrap()+start+1;

            let url_start = self.content[start..].find(r#"src=""#).unwrap()+start+5;
            let url_end = self.content[url_start..].find(r#"""#).unwrap()+url_start;
            download_image(self.content[url_start..url_end].to_string(), zip, options).await.unwrap();
            match &mut self.opf_extras {
                Some(x) => x.push(self.content[url_start..url_end].to_string()),
                None => self.opf_extras = Some(vec![self.content[url_start..url_end].to_string()]),
            };
            //println!("{:#?}", self.opf_extras);
            //println!("Url: {}", self.content[url_start..url_end].to_string());

            offset = offset + start;
            self.content.replace_range(start..end, &self.content[start..end]
                .replace("/", "_").replace(":", "-").replace(">", r#" alt="" />"#).replace("http", "/Images/http"));
                
                //TODO: V3 Holy shit is this spagget, if the url dont start with http WE ARE COOKED
        }
    }
}

async fn download_image(url: String, zip: &mut ZipWriter<File>, options: SimpleFileOptions) -> Result<(), std::io::Error>{
    let image = reqwest::get(&url).await.unwrap().bytes().await.unwrap();
    zip.start_file_from_path(format!("Images/{}", &url.replace("/", "_").replace(":", "-")), options)?;
    zip.write_all(&image)?;
    Ok(())
}
