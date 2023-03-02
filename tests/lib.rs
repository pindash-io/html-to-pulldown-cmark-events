use anyhow::Result;
use html_to_pulldown_cmark_events::parser;

// https://commonmark.org/help/
// https://docs.github.com/en/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax
// #[test]
// fn parse_entity() -> Result<()> {
//     let file = std::fs::File::open("tests/fixtures/cloudflare.xml")?;
//     let feed_rs::model::Feed { entries, .. } = feed_rs::parser::parse(file)?;
//
//     let entity = &entries[0];
//
//     let content = entity
//         .content
//         .as_ref()
//         .and_then(|c| c.body.clone())
//         .unwrap_or_default();
//
//     let mut events = Vec::new();
//     easymark::parser(content, &mut events);
//
//     dbg!(events);
//     Ok(())
// }

#[test]
fn parse_simple() -> Result<()> {
    let content = include_str!("fixtures/simple.html");

    let mut events = Vec::new();
    parser(content, &mut events);

    dbg!(events);
    Ok(())
}