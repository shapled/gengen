pub struct Tokenizer<T, G> 
where
    T: Copy + PartialEq,
    G: Iterator<Item = T>
{
    tokengen: G,
    tokens: Vec<T>,
    pos: usize,
}

impl<T, G> Tokenizer<T, G> 
where
    T: Copy + PartialEq,
    G: Iterator<Item = T>
{
    pub fn new(tokengen: G) -> Self {
        Self { 
            tokengen,
            tokens: vec![],
            pos: 0,
        }
    }

    pub fn mark(self: &Self) -> usize {
        self.pos
    }

    pub fn reset(self: &mut Self, pos: usize) {
        self.pos = pos;
    }

    pub fn peek_token(self: &mut Self) -> Option<T> {
        while self.pos >= self.tokens.len() {
            match self.tokengen.next() {
                None => return None,
                Some(token) => self.tokens.push(token), 
            }
        }
        Some(self.tokens[self.pos])
    }

    pub fn get_token(self: &mut Self) -> Option<T> {
        match self.peek_token() {
            None => None,
            Some(token) => {
                self.pos += 1;
                Some(token)
            },
        }
    }
    
}

mod test {
    use unicode_segmentation::UnicodeSegmentation;

    use super::*;

    struct Input<'a> {
        data: Vec<&'a str>,
        pos: usize,
    }

    impl<'a> Input<'a> {
        fn new(input: &'a str) -> Self {
            let iter = input.graphemes(true);
            Input {
                data: iter.collect::<Vec<&'a str>>(),
                pos: 0,
            }
        }
    }

    impl<'a> Iterator for Input<'a> {
        type Item = &'a str;

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
        let tokengen = Input::new("你a好");
        let mut tokenizer = Tokenizer::new(tokengen);
        assert_eq!(0, tokenizer.mark());
        assert_eq!("你", tokenizer.peek_token().unwrap());
        assert_eq!(0, tokenizer.mark());
        tokenizer.reset(1);
        assert_eq!(1, tokenizer.mark());
        assert_eq!("a", tokenizer.get_token().unwrap());
        assert_eq!(2, tokenizer.mark());
        assert_eq!("好", tokenizer.get_token().unwrap());
        assert_eq!(None, tokenizer.get_token());
        assert_eq!(3, tokenizer.mark());
        tokenizer.reset(0);
        assert_eq!(0, tokenizer.mark());
    }
}
