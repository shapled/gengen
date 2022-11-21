use crate::{parser::{Parser, TokenExpected}, grammar::{Meta, MetaValue, Alt, Item, Rule, Grammar}};

#[derive(Copy, Clone, PartialEq, Debug)]
enum BootstrapTokenKind {
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

#[derive(Copy, Clone, PartialEq, Debug)]
struct BootstrapToken<'a> {
    kind: BootstrapTokenKind,
    value: &'a str,
}

enum BootstrapTokenExpected<'a> {
    Kind(BootstrapTokenKind),
    Value(&'a str),
}

impl<'a> TokenExpected<BootstrapToken<'a>> for BootstrapTokenExpected<'a> {
    fn matched(&self, token: &BootstrapToken<'a>) -> bool {
        match self {
            BootstrapTokenExpected::Kind(k) => token.kind.eq(k),
            BootstrapTokenExpected::Value(v) => token.value.eq(*v),
        }
    }
}

fn _k<'a>(kind: BootstrapTokenKind) -> BootstrapTokenExpected<'a> {
    BootstrapTokenExpected::Kind(kind)
}

fn _v<'a>(str: &'a str) -> BootstrapTokenExpected {
    BootstrapTokenExpected::Value(str)
}

pub struct BootstrapGrammarParser<'a, K, G>
where
    G: Iterator<Item = BootstrapToken<'a>>
{
    parser: Parser<K, BootstrapToken<'a>, G>
}

impl<'a, K, G> BootstrapGrammarParser<'a, K, G>
where
    G: Iterator<Item = BootstrapToken<'a>>
{
    fn new(parser: Parser<K, BootstrapToken<'a>, G>) -> Self {
        Self {
            parser,
        }
    }

    fn meta(&mut self) -> Option<Meta<'a>> {
        let position = self.parser.mark();
        if self.parser.expect(_v("@")).is_some() {
            if let Some(name) = self.parser.expect(_k(BootstrapTokenKind::Name)) {
                let pos2 = self.parser.mark();
                if self.parser.expect(_k(BootstrapTokenKind::Newline)).is_some() {
                    return Some(Meta{ key: name.value, value: MetaValue::None })
                }
                // TODO(maybe): support literal_eval
                // self.parser.reset(pos2);
                // if let Some(_) = self.parser.expect(BootstrapTokenExpected::Kind(BootstrapTokenKind::String)) {
                //     if self.parser.expect(BootstrapTokenExpected::Kind(BootstrapTokenKind::Newline)).is_some() {
                //         return Some((name.value, MetaValue::None))
                //     }
                // }
                self.parser.reset(pos2);
                if let Some(token) = self.parser.expect(_k(BootstrapTokenKind::Name)) {
                    if self.parser.expect(_k(BootstrapTokenKind::Newline)).is_some() {
                        return Some(Meta{ key: name.value, value: MetaValue::String(token.value)})
                    }
                }
                self.parser.reset(pos2);
                if let Some(token) = self.parser.expect(_k(BootstrapTokenKind::Number)) {
                    if self.parser.expect(_k(BootstrapTokenKind::Newline)).is_some() {
                        return Some(Meta{ key: name.value, value: MetaValue::Number(token.value.parse().unwrap())})
                    }
                }
            }
        }
        self.parser.reset(position);
        None
    }

    fn item(&mut self) -> Option<&'a str> {
        if let Some(name) = self.parser.expect(_k(BootstrapTokenKind::Name)) {
            Some(name.value)
        } else if let Some(str) = self.parser.expect(_k(BootstrapTokenKind::String)) {
            Some(str.value)
        } else {
            None
        }
    }

    fn alternative(&mut self) -> Option<Alt<'a>> {
        let mut items = vec![];
        let mut action_tokens = vec![];

        while let Some(item) = self.item() {
            items.push(Item::Raw(item))
        }

        if items.is_empty() {
            return None;
        }

        if self.parser.expect(_v("{")).is_some() {
            let mut level = 0;
            loop {
                let token = self.parser.tokenizer.get_token().unwrap().value;
                match token {
                    "{" => level += 1,
                    "}" => {
                        level -= 1;
                        if level < 0 {
                            break;
                        }
                    },
                    _ => (),
                }
                action_tokens.push(token);
            }
        }
        let action = action_tokens.join(" ");

        Some(Alt {
            action,
            items,
        })
    }

    fn bar_alt(&mut self) -> Option<Alt<'a>>{
        let position = self.parser.mark();
        if self.parser.expect(_v("|")).is_some() {
            if let Some(alt) = self.alternative() {
                return Some(alt);
            }
        }
        self.parser.reset(position);
        None
    }

    fn alts(&mut self) -> Option<Vec<Alt<'a>>> {
        let position = self.parser.mark();
        let mut alts = vec![];
        if let Some(alt) = self.alternative() {
            alts.push(alt);
            while let Some(alt) = self.bar_alt() {
                alts.push(alt);
            }
            return Some(alts)
        }
        self.parser.reset(position);
        None
    }

    fn alts_newline(&mut self) -> Option<Vec<Alt<'a>>> {
        let position = self.parser.mark();
        if let Some(alts) = self.alts() {
            if self.parser.expect(_k(BootstrapTokenKind::Newline)).is_some() {
                return Some(alts);
            }
        }
        self.parser.reset(position);
        None
    }

    fn bar_alts_newline(&mut self) -> Option<Vec<Alt<'a>>> {
        let position = self.parser.mark();
        if self.parser.expect(_v("|")).is_some() {
            if let Some(alts) = self.alts_newline() {
                return Some(alts);
            }
        }
        self.parser.reset(position);
        None
    }

    fn indented_alts(&mut self) -> Option<Vec<Alt<'a>>> {
        let position = self.parser.mark();
        if self.parser.expect(_k(BootstrapTokenKind::Indent)).is_some() {
            let mut alts = vec![];
            loop {
                if let Some(mut alts1) = self.bar_alts_newline() {
                    alts.append(&mut alts1);
                } else if self.parser.expect(_k(BootstrapTokenKind::NL)).is_some()
                          || self.parser.expect(_k(BootstrapTokenKind::Comment)).is_some() {
                    // Do nothing ...
                } else {
                    break;
                }
            }
            if self.parser.expect(_k(BootstrapTokenKind::Dedent)).is_some() {
                return Some(alts);
            }
        }
        self.parser.reset(position);
        None
    }

    fn rule(&mut self) -> Option<Rule<'a>> {
        let position = self.parser.mark();
        if let Some(name) = self.parser.expect(_k(BootstrapTokenKind::Name)) {
            if self.parser.expect(_v(":")).is_some() {
                let mut alts = if let Some(alts) = self.alts_newline() {
                    alts
                } else if self.parser.expect(_k(BootstrapTokenKind::Newline)).is_some() {
                    vec![]
                } else {
                    self.parser.reset(position);
                    return None
                };
                if let Some(mut alts1) = self.indented_alts() {
                    alts.append(&mut alts1);
                }
                if !alts.is_empty() {
                    return Some(Rule { name: name.value, alts })
                }
            }
        }
        self.parser.reset(position);
        None
    }

    fn grammar(&mut self) -> Grammar<'a> {
        let mut rules = vec![];
        let mut metas = vec![];
        loop {
            if let Some(rule) = self.rule() {
                rules.push(rule);
            } else if let Some(meta) = self.meta() {
                metas.push(meta);
            } else if self.parser.expect(_k(BootstrapTokenKind::NL)).is_some()
                      || self.parser.expect(_k(BootstrapTokenKind::Comment)).is_some() {
                continue;
            } else {
                return Grammar { rules, metas }
            }
        }
    }

}
