use rust_stemmers::{Algorithm, Stemmer};
#[derive(Debug)]
pub struct Lexer<'a> {
    content: &'a [char],
}

impl<'a> Lexer<'a> {
    pub fn new(content: &'a [char]) -> Self {
        Self { content }
    }
    fn trim_left(&mut self) {
        while self.content.len() > 0 && self.content[0].is_whitespace() {
            self.content = &self.content[1..]
        }
    }

    fn chop(&mut self, n: usize) -> &'a [char] {
        let token = &self.content[0..n];
        self.content = &self.content[n..];
        token
    }

    fn chop_while<P>(&mut self, mut predicate: P) -> &'a [char]
    where
        Self: Sized,
        P: FnMut(&char) -> bool,
    {
        let mut n = 0;
        while n < self.content.len() && predicate(&self.content[n]) {
            n += 1;
        }
        self.chop(n)
    }

    fn next_token(&mut self) -> Option<&'a [char]> {
        self.trim_left();

        if self.content.len() == 0 {
            return None;
        }

        if self.content[0].is_numeric() {
            let token = self.chop_while(|x| x.is_numeric());

            return Some(token);
        }

        if self.content[0].is_alphabetic() {
            let token = self.chop_while(|x| x.is_alphanumeric());

            return Some(token);
        }

        let token = self.chop(1);
        Some(token)
    }
}

impl<'a> Iterator for Lexer<'a> {
    // type Item = &'a [char];
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        let token = self.next_token();
        match token {
            Some(token) => {
                Some(String::from_iter(token))
                // let stemmer = Stemmer::create(Algorithm::English);
                // let stemmed_token = stemmer.stem(String::from_iter(token).as_str()).to_string();
                // Some(stemmed_token)
            }
            None => None,
        }
    }
}
