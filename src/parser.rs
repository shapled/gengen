use std::marker::PhantomData;

use crate::tokenizer::Tokenizer;

#[derive(Debug, PartialEq)]
struct Node<K> {
    kind: K,
    children: Vec<Node<K>>,
}

impl<K> Node<K> {
    fn new(kind: K) -> Self {
        Self {
            kind,
            children: vec![],
        }
    }
}

struct Parser<K, T, G>
where
    T: Copy + Eq,
    G: Iterator<Item = T>
{
    tokenizer: Tokenizer<T, G>,
    _phantom: PhantomData<K>,
}

impl<K, T, G> Parser<K, T, G>
where
    T: Copy + Eq,
    G: Iterator<Item = T>
{
    fn new(tokenizer: Tokenizer<T, G>) -> Self {
        Self {
            tokenizer,
            _phantom: PhantomData,
        }
    }

    fn mark(&self) -> usize {
        self.tokenizer.mark()
    }

    fn reset(&mut self, pos: usize) {
        self.tokenizer.reset(pos);
    }

    fn expect(&mut self, token: &T) -> Option<T> {
        let next = self.tokenizer.peek_token();
        match next {
            Some(t) if t.eq(token)  => self.tokenizer.get_token(),
            _ => None,
        }
    }

    fn repeat<F>(
        &mut self, 
        at_least: usize, 
        at_most: Option<usize>, 
        callback: &F
    ) -> Option<Vec<Node<K>>> 
    where
        F: Fn(&mut Self) -> Option<Node<K>>
    {
        let position = self.mark();
        let mut nodes: Vec<Node<K>> = vec![];
        loop {
            match at_most {
                Some(most) if nodes.len() == most => return Some(nodes),
                _phantom => (), 
            }
            match callback(self) {
                None => break,
                Some(n) => nodes.push(n),
            }
        }
        if nodes.len() >= at_least {
            return Some(nodes);
        }
        self.reset(position);
        None
    }

    fn lookahead<F>(
        &mut self, 
        positive: bool, 
        callback: &F
    ) -> bool 
    where
        F: Fn(&mut Self) -> Option<Node<K>>
    {
        let position = self.mark();
        let successful = callback(self).is_some();
        self.reset(position);
        successful == positive
    }

}

mod test {
    use unicode_segmentation::UnicodeSegmentation;

    use super::*;

    struct Input {
        data: Vec<&'static str>,
        pos: usize,
    }

    impl Input {
        fn new(input: &'static str) -> Self {
            let iter = input.graphemes(true);
            Input {
                data: iter.collect::<Vec<&'static str>>(),
                pos: 0,
            }
        }
    }

    impl Iterator for Input {
        type Item = &'static str;

        fn next(&mut self) -> Option<Self::Item> {
            match self.pos < self.data.len() {
                false => None,
                true => {
                    let item = self.data[self.pos];
                    self.pos += 1;
                    Some(item)
                },
            }
        }
    }

    #[test]
    fn test_tokenizer() {
        let tokengen = Input::new("你abc好");
        let tokenizer = Tokenizer::new(tokengen);
        let mut parser: Parser<&'static str, _, _> = Parser::new(tokenizer);

        assert_eq!(0, parser.mark());
        assert_eq!(None, parser.expect(&"a"));
        assert_eq!(Some("你"), parser.expect(&"你"));
        assert_eq!(1, parser.mark());
    
        let next_is_a_or_b_or_c = | p: &mut Parser<&'static str, &'static str, Input> | { 
            if p.expect(&"a").is_some()
               || p.expect(&"b").is_some()
               || p.expect(&"c").is_some() {
                Some(Node::new("abc"))
            } else {
                None
            }
        };
    
        // test lookahead
        assert_eq!(true, parser.lookahead(true, &next_is_a_or_b_or_c));
        assert_eq!(true, parser.lookahead(true, &next_is_a_or_b_or_c));
        parser.reset(0);
        assert_eq!(true, parser.lookahead(false, &next_is_a_or_b_or_c));

        // test repeat
        let abc = Node::new("abc");
        
        parser.reset(1);
        let nodes = parser.repeat(0, None, &next_is_a_or_b_or_c).unwrap();
        assert_eq!(3, nodes.len());
        assert_eq!(true, nodes[0].eq(&abc));
        assert_eq!(true, nodes[1].eq(&abc));
        assert_eq!(true, nodes[2].eq(&abc));
        
        parser.reset(1);
        let nodes = parser.repeat(3, None, &next_is_a_or_b_or_c).unwrap();
        assert_eq!(3, nodes.len());
        assert_eq!(true, nodes[0].eq(&abc));
        assert_eq!(true, nodes[1].eq(&abc));
        assert_eq!(true, nodes[2].eq(&abc));

        parser.reset(1);
        let result = parser.repeat(4, None, &next_is_a_or_b_or_c);
        assert_eq!(false, result.is_some());

        parser.reset(1);
        let nodes = parser.repeat(1, Some(2), &next_is_a_or_b_or_c).unwrap();
        assert_eq!(2, nodes.len());
        assert_eq!(true, nodes[0].eq(&abc));
        assert_eq!(true, nodes[1].eq(&abc));

        parser.reset(1);
        let nodes = parser.repeat(1, Some(3), &next_is_a_or_b_or_c).unwrap();
        assert_eq!(3, nodes.len());
        assert_eq!(true, nodes[0].eq(&abc));
        assert_eq!(true, nodes[1].eq(&abc));
        assert_eq!(true, nodes[2].eq(&abc));

        parser.reset(0);
        assert_eq!(0, parser.mark());
    }
}
