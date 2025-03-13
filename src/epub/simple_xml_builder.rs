pub struct XMLElement {
    special: String,
    children: Vec<XMLElement>,
    text: String,
    title: String,
    attributes: Vec<(String, String)>,
}

impl XMLElement{
    pub fn new(title: impl ToString) -> Self{
        XMLElement{
            special: String::new(),
            children: Vec::new(),
            text: String::new(),
            title: title.to_string(),
            attributes: Vec::new()
        }
    }
    pub fn add_attribute(self: &mut Self, name: impl ToString, value: impl ToString) {
        self.attributes.push(
            (name.to_string(), value.to_string())
        )
    }
    pub fn add_text(self: &mut Self, text: impl ToString) {
        self.text = text.to_string();
    }
    pub fn add_special(self: &mut Self, special: impl ToString) {
        self.special = special.to_string();
    }
    pub fn add_child(self: &mut Self, child: XMLElement){
        self.children.push(child);
    }
    pub fn to_string(self: &Self) -> String {
        r#"<?xml version = "1.0" encoding = "UTF-8"?>"#.to_owned() 
        + &self.special + &self.check_children_text(&self.format_attributes())
                                                                    //^This makes the body of the xml Element
    }
/*========================================Private========================================*/
    fn format_attributes(self: &Self) -> String {
        let mut binding = String::new();
        for x in &self.attributes {
            binding.push_str(&format!(r#" {}="{}""#, x.0, x.1))
        };
        binding
    }   //Formats the attributes for XML, if none exist returns an empty string
    fn check_children_text(self: &Self, attributes: &String) -> String {
        if self.text.is_empty() && self.children.is_empty() {
            format!("<{}{}/>", &self.title, attributes)
        }
        else {
            format!("<{0}{1}>{3}{2}</{0}>", self.title, &attributes, &self.make_children(), &self.text)
        }
    }   //TLDR: If child or text exist but the other one is an empty string it wont do anything
    fn make_children(self: &Self) -> String {
        let mut children = String::new();
        for x in &self.children {
            children.push_str(&x.check_children_text(&x.format_attributes()));
                                    //^This make the body of the xml Element
        }
        children
    }   //Makes the children of the XML element, if none exist returns an empty string
}
