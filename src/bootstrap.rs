use crate::parser::Parser;

enum BootstrapToken {
    Comment, 
    Dedent, 
    EndMarker, 
    Indent, 
    Name, 
    Newline, 
    NL, 
    Number,
    String,
}

pub struct BootstrapGrammar<K, T, G>
where
    T: Copy + PartialEq,
    G: Iterator<Item = T>
{
    parser: Parser<K, T, G>
}
