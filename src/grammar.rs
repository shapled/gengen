#[derive(PartialEq, Debug)]
pub struct Rule<'a> {
    pub name: &'a str,
    pub alts: Vec<Alt<'a>>,
}

#[derive(PartialEq, Debug)]
pub struct Alt<'a> {
    pub items: Vec<Item<'a>>,
    pub action: String,
}

#[derive(PartialEq, Debug)]
pub struct NamedItem<'a>{
    pub name: &'a str, 
    pub item: &'a str,
}

#[derive(PartialEq, Debug)]
pub struct Repeat<'a>{
    pub at_least: usize, 
    pub at_most: Option<usize>, 
    pub item: &'a str,
}

#[derive(PartialEq, Debug)]
pub struct Lookahead<'a>{
    pub positive: bool, 
    pub item: &'a str,
}

#[derive(PartialEq, Debug)]
pub enum Item<'a> {
    NamedItem(NamedItem<'a>),
    Repeat(Repeat<'a>),
    Lookahead(Lookahead<'a>),
    Cut,
    Raw(&'a str),
}

#[derive(PartialEq, Debug)]
pub struct Grammar<'a> {
    pub rules: Vec<Rule<'a>>,
    pub metas: Vec<Meta<'a>>,
}

#[derive(PartialEq, Debug)]
pub enum MetaValue<'a> {
    None,
    Number(f64),
    String(&'a str),
}

#[derive(PartialEq, Debug)]
pub struct Meta<'a> {
    pub key: &'a str, 
    pub value: MetaValue<'a>,
}
