use clap::{Parser, ValueEnum};
use rand::Rng;
use tl::ParserOptions;

static BASE_URL: &str = "https://en.wikipedia.org";

#[derive(Parser, Debug)]
#[command(author = "lockness-Ko,woobi3", version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = String::from("Philosophy"))]
    page: String,

    #[arg(short, long, default_value_t = ChoiceMethod::First)]
    method: ChoiceMethod,

    // #[arg(short, long, default_value_t = String::from("Philosophy"))]
    // end_page: String,
}

fn main() {
    let args = Args::parse();

    let url = args.page;

    println!("[START] {}", url);

    get_links(format!("/wiki/{}", url), &mut vec![], args.method);
}

#[derive(ValueEnum, Clone, Copy, Debug)]
enum ChoiceMethod {
    First,
    Last,
    Random,
}

impl std::fmt::Display for ChoiceMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", format!("{:#?}", self).to_lowercase())
    }
}

fn get_links(page: String, visited: &mut Vec<String>, method: ChoiceMethod) -> Vec<String> {
    let mut rng = rand::thread_rng();

    let body = reqwest::blocking::get(format!("{}{}", BASE_URL, page))
        .unwrap()
        .text()
        .unwrap();

    let dom = tl::parse(body.as_str(), ParserOptions::default()).unwrap();
    let parser = dom.parser();

    let main_content = dom
        .get_element_by_id("mw-content-text")
        .unwrap()
        .get(parser)
        .unwrap()
        .as_tag()
        .unwrap()
        .children()
        .all(parser)
        .iter()
        .next()
        .unwrap()
        .as_tag()
        .unwrap();

    let anchors = main_content.query_selector(parser, "p").unwrap();

    let urls: Vec<String> = anchors
        .map(|tag| {
            let tag_href = tag.get(parser).unwrap().as_tag().unwrap();
            let anchors = tag_href.query_selector(parser, "a").unwrap();

            anchors
                .filter_map(|anchor| {
                    let anchor_href = anchor
                        .get(parser)
                        .unwrap()
                        .as_tag()
                        .unwrap()
                        .attributes()
                        .iter()
                        .next()
                        .unwrap()
                        .1
                        .unwrap();
                    if anchor_href.starts_with("/wiki")
                        && !anchor_href.starts_with("/wiki/File:")
                        && !anchor_href.starts_with("/wiki/Help:")
                        && !anchor_href.starts_with("/wiki/Template:")
                        && !anchor_href.starts_with("/wiki/Wikipedia:")
                        && !anchor_href.starts_with("/wiki/Geographic_coordinate_system")
                    {
                        Some(anchor_href.into_owned())
                    } else {
                        None
                    }
                })
                .collect::<Vec<String>>()
        })
        .flatten()
        .collect();

    let next = match method {
        ChoiceMethod::First => urls.first().to_owned().unwrap(),
        ChoiceMethod::Last => urls.last().to_owned().
        unwrap(),
        ChoiceMethod::Random => &urls[rng.gen_range(0..urls.len())],
    }
    .to_owned();
    let owned_next = next.to_owned();

    if visited.contains(&next) {
        println!("[DONE] {}", &owned_next);
        vec![next.into()]

    } else {
        println!("[CRAWL] {}", &owned_next);
        visited.push(owned_next);

        get_links(next, visited, method)
    }
}
