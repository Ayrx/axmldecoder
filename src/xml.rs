use std::collections::HashMap;
use std::convert::TryFrom;
use std::rc::Rc;

use crate::binaryxml::{XmlCdata, XmlElement, XmlStartElement, XmlStartNameSpace};
use crate::stringpool::StringPool;

///Struct representing a parsed XML document.
#[derive(Debug)]
pub struct XmlDocument {
    root: Option<Node>,
}

impl XmlDocument {
    pub(crate) fn new(
        elements: Vec<XmlElement>,
        string_pool: StringPool,
        _resource_map: Vec<u32>,
    ) -> Self {
        let mut namespaces = HashMap::new();

        let mut element_tracker: Vec<Element> = Vec::new();
        for element in elements {
            match element {
                XmlElement::XmlStartNameSpace(e) => {
                    let (uri, prefix) = Self::process_start_namespace(&e, &string_pool);
                    namespaces.insert(uri.clone(), prefix.clone());
                }
                XmlElement::XmlEndNameSpace(_) => {}
                XmlElement::XmlStartElement(e) => {
                    element_tracker.push(Self::process_start_element(
                        &e,
                        &string_pool,
                        &namespaces,
                    ));
                }
                XmlElement::XmlEndElement(_) => {
                    let e = element_tracker.pop().unwrap();

                    if element_tracker.is_empty() {
                        return XmlDocument {
                            root: Some(Node::Element(e)),
                        };
                    }

                    element_tracker
                        .last_mut()
                        .unwrap()
                        .insert_children(Node::Element(e));
                }
                XmlElement::XmlCdata(e) => {
                    let cdata = Self::process_cdata(&e, &string_pool);
                    element_tracker
                        .last_mut()
                        .unwrap()
                        .insert_children(Node::Cdata(cdata))
                }
            };
        }

        Self { root: None }
    }

    ///Returns the root [Element] of the XML document.
    pub fn get_root(&self) -> &Option<Node> {
        &self.root
    }

    fn process_cdata(e: &XmlCdata, string_pool: &StringPool) -> Cdata {
        Cdata {
            data: string_pool
                .get(usize::try_from(e.data).unwrap())
                .unwrap()
                .to_string(),
        }
    }

    fn process_start_namespace(
        e: &XmlStartNameSpace,
        string_pool: &StringPool,
    ) -> (Rc<String>, Rc<String>) {
        let uri = string_pool.get(usize::try_from(e.uri).unwrap()).unwrap();
        let prefix = string_pool.get(usize::try_from(e.prefix).unwrap()).unwrap();

        (uri, prefix)
    }

    fn process_start_element(
        e: &XmlStartElement,
        string_pool: &StringPool,
        namespaces: &HashMap<Rc<String>, Rc<String>>,
    ) -> Element {
        let ns = string_pool.get(usize::try_from(e.attr_ext.ns).unwrap());
        assert_eq!(ns, None);

        let name = string_pool
            .get(usize::try_from(e.attr_ext.name).unwrap())
            .unwrap();
        let name = (*name).clone();

        let mut attributes: HashMap<String, String> = HashMap::new();
        for attr in &e.attributes {
            let ns = string_pool.get(usize::try_from(attr.ns).unwrap());
            let name = string_pool
                .get(usize::try_from(attr.name).unwrap())
                .unwrap();
            let value = attr.typed_value.get_value(&string_pool);

            let mut final_name = String::new();

            if let Some(n) = ns {
                let ns_prefix = namespaces.get(&n).unwrap();
                final_name.push_str(ns_prefix);
                final_name.push(':');
            }
            final_name.push_str(&name);

            attributes.insert(final_name, value.to_string());
        }

        Element {
            attributes,
            tag: name,
            children: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub enum Node {
    Element(Element),
    Cdata(Cdata),
}

///Struct representing an element within the parsed XML document.
#[derive(Debug)]
pub struct Element {
    attributes: HashMap<String, String>,
    tag: String,
    children: Vec<Node>,
}

impl Element {
    ///Returns a map of attributes associated with the element.
    pub fn get_attributes(&self) -> &HashMap<String, String> {
        &self.attributes
    }

    ///Returns the element tag.
    pub fn get_tag(&self) -> &str {
        &self.tag
    }

    ///Returns a list of child nodes.
    pub fn get_children(&self) -> &Vec<Node> {
        &self.children
    }

    fn insert_children(&mut self, child: Node) {
        self.children.push(child);
    }
}

#[derive(Debug)]
pub struct Cdata {
    data: String,
}

impl Cdata {
    pub fn get_data(&self) -> &str {
        &self.data
    }
}
