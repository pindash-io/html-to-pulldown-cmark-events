use pulldown_cmark::{CodeBlockKind, CowStr, Event, LinkType, Tag};
use scraper::{node::Text, ElementRef, Html, Node, Selector};

const CRTL: &str = "\n";

pub fn parser(raw: impl AsRef<str>, events: &mut Vec<Event<'_>>) {
    let html = Html::parse_fragment(raw.as_ref());

    for node in html.root_element().children() {
        // blocks
        match node.value() {
            Node::Element(elem) => {
                let name = elem.name();
                match name {
                    "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                        let level =
                            atoi::ascii_to_digit::<usize>(name.bytes().nth(1).unwrap()).unwrap();
                        let tag = Tag::Heading(level.try_into().unwrap(), None, Vec::new());
                        events.push(Event::Start(tag.clone()));

                        parse_inline(events, node);

                        events.push(Event::End(tag));
                    }
                    "p" => {
                        let tag = Tag::Paragraph;
                        events.push(Event::Start(tag.clone()));

                        parse_inline(events, node);

                        events.push(Event::End(tag));
                    }
                    "img" => {
                        let mut attrs = elem
                            .attrs()
                            .into_iter()
                            .filter(|a| a.0 == "src" || a.0 == "alt")
                            .collect::<Vec<_>>();

                        attrs.sort_by_key(|attr| attr.0);

                        if attrs.is_empty() {
                            continue;
                        }

                        let (src, alt) = (
                            attrs[0].1.to_string(),
                            if attrs.len() == 1 {
                                String::new()
                            } else {
                                attrs[1].1.to_string()
                            },
                        );

                        let tag = Tag::Image(LinkType::Inline, src.into(), alt.into());
                        events.push(Event::Start(tag.clone()));
                        events.push(Event::End(tag));
                    }
                    "blockquote" => {
                        let tag = Tag::BlockQuote;
                        events.push(Event::Start(tag.clone()));

                        parse_inline(events, node);

                        events.push(Event::End(tag));
                    }
                    "ol" | "ul" => {
                        parse_list(
                            events,
                            node,
                            (name.chars().next() == Some('o')).then_some(0),
                        );
                    }
                    "br" => {
                        events.push(Event::HardBreak);
                    }
                    "hr" => {
                        events.push(Event::Rule);
                    }
                    "pre" => {
                        let mut kind = CodeBlockKind::Indented;
                        let elem_ref = ElementRef::wrap(node.into()).unwrap();
                        let mut text = String::new();
                        elem_ref.text().collect::<Vec<_>>().iter().for_each(|s| {
                            text.push_str(s);
                        });

                        if let Some(k) = elem
                            .classes()
                            .find_map(|name| name.split_once("language-"))
                            .map(|(_, lang)| {
                                CodeBlockKind::Fenced(CowStr::Boxed(lang.to_string().into()))
                            })
                        {
                            // prism
                            kind = k;
                        } else if elem.classes().find(|name| *name == "highlight").is_some() {
                            // highlight

                            let selector = Selector::parse("code").unwrap();

                            if let Some(code) = elem_ref.select(&selector).next() {
                                if let Some(k) =
                                    code.value().attrs().find(|attr| attr.0 == "data-lang").map(
                                        |(_, lang)| {
                                            CodeBlockKind::Fenced(CowStr::Boxed(
                                                lang.to_string().into(),
                                            ))
                                        },
                                    )
                                {
                                    kind = k;
                                }
                            }
                        }

                        // TODO: https://shiki.matsu.io/

                        let tag = Tag::CodeBlock(kind);
                        events.push(Event::Start(tag.clone()));

                        events.push(Event::Text(CowStr::Boxed(text.to_string().into())));

                        events.push(Event::End(tag));
                    }
                    // "code" => {}
                    // foot
                    _ => {}
                }
            }
            Node::Text(Text { text }) if text.trim_end_matches(' ') == CRTL => {
                // events.push(Event::SoftBreak)
            }
            _ => {}
        }
    }
}

fn parse_list<'a>(
    events: &mut Vec<Event<'_>>,
    parent: ego_tree::NodeRef<'a, Node>,
    kind: Option<u64>,
) {
    let tag = Tag::List(kind);
    events.push(Event::Start(tag.clone()));

    for node in parent.children() {
        if matches!(node.value(), Node::Element(elem) if elem.name() == "li") {
            let tag = Tag::Item;
            events.push(Event::Start(tag.clone()));

            for sub_node in node.children() {
                // nested list
                if let Some(k) = sub_node
                    .value()
                    .as_element()
                    .map(|elem| elem.name())
                    .filter(|name| *name == "ol" || *name == "ul")
                    .and_then(|name| name.chars().next())
                {
                    parse_list(events, sub_node, (k == 'o').then_some(0));
                } else {
                    parse_inline(events, node);
                }
            }

            events.push(Event::End(tag));
        }
    }

    events.push(Event::End(tag));
}

fn parse_inline<'a>(events: &mut Vec<Event<'_>>, parent: ego_tree::NodeRef<'a, Node>) {
    for node in parent.children() {
        match node.value() {
            Node::Element(elem) => {
                let (start, end) = match elem.name() {
                    // Link
                    "a" => {
                        let mut attrs = elem
                            .attrs()
                            .into_iter()
                            .filter(|a| a.0 == "href" || a.0 == "title")
                            .collect::<Vec<_>>();

                        attrs.sort_by_key(|attr| attr.0.clone());

                        if attrs.is_empty() {
                            continue;
                        }

                        let (href, title) = (
                            attrs[0].1.to_string(),
                            if attrs.len() == 1 {
                                String::new()
                            } else {
                                attrs[1].1.to_string()
                            },
                        );

                        let tag = Tag::Link(LinkType::Inline, href.into(), title.into());
                        (Some(Event::Start(tag.clone())), Some(Event::End(tag)))
                    }
                    // Blod
                    "strong" => {
                        let tag = Tag::Strong;
                        (Some(Event::Start(tag.clone())), Some(Event::End(tag)))
                    }
                    // Italic
                    "em" => {
                        let tag = Tag::Emphasis;
                        (Some(Event::Start(tag.clone())), Some(Event::End(tag)))
                    }
                    // Strikethrough
                    "del" => {
                        let tag = Tag::Strikethrough;
                        (Some(Event::Start(tag.clone())), Some(Event::End(tag)))
                    }
                    // Inline Code
                    "code" => (
                        node.first_child()
                            .and_then(|node| node.value().as_text())
                            .map(|text| Event::Code(CowStr::Boxed(text.to_string().into()))),
                        None,
                    ),
                    // Subscript
                    // "sub" => {},
                    // Superscript
                    // "sup" => {},
                    _ => (None, None),
                };

                if let Some(e) = start {
                    events.push(e);
                }

                if let Some(e) = end {
                    parse_inline(events, node);

                    events.push(e);
                }
            }
            Node::Text(Text { text }) if text.trim_end_matches(' ') != CRTL => {
                events.push(Event::Text(CowStr::Boxed(text.to_string().into())));
            }
            _ => {}
        }
    }
}