#[derive(PartialEq, Debug)]
pub struct Rule<'a> {
    name: &'a str,
    alts: Vec<Alt<'a>>,
}

#[derive(PartialEq, Debug)]
pub struct Alt<'a> {
    items: Vec<Item<'a>>,
    action: &'a str,
}

#[derive(PartialEq, Debug)]
pub struct NamedItem<'a>{
    name: &'a str, 
    item: &'a str,
}

#[derive(PartialEq, Debug)]
pub struct Repeat<'a>{
    at_least: usize, 
    at_most: Option<usize>, 
    item: &'a str,
}

#[derive(PartialEq, Debug)]
pub struct Lookahead<'a>{
    positive: bool, 
    item: &'a str,
}

#[derive(PartialEq, Debug)]
pub enum Item<'a> {
    NamedItem(NamedItem<'a>),
    Repeat(Repeat<'a>),
    Lookahead(Lookahead<'a>),
    Cut,
}

#[derive(PartialEq, Debug)]
pub struct Action<'a> {
    action: &'a str
}

#[derive(PartialEq, Debug)]
pub struct Grammar<'a> {
    rules: Vec<Rule<'a>>
}
