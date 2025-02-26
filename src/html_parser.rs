use std::error::Error;

#[derive(Default, Clone)]
pub struct ElementInfo {
    id: String,
    class_name: String,
}

#[derive(Clone)]
pub enum Element {
    Text(String, Box<Element>),
    Div(ElementInfo, Vec<Box<Element>>, Box<Element>),
    Body(ElementInfo, Vec<Box<Element>>, Box<Element>),
    Html(ElementInfo, Vec<Box<Element>>),
    Other(ElementInfo, Vec<Box<Element>>, Box<Element>),
}

pub fn parse_html(raw: String) -> Result<Element, Box<dyn Error>> {
    let html_elem = Element::Html(ElementInfo::default(), Vec::new());

    let text = Element::Text("This is text".to_owned(), Box::new(html_elem.clone()));

    let updated_html = match html_elem {
        Element::Html(info, children) => {
            let updated_children = vec![children.clone(), vec![Box::new(text.clone())]].concat();

            Element::Html(info, updated_children)
        }
        _ => html_elem,
    };

    Ok(text)
}
