use std::{
    ops::{Deref, RangeInclusive},
    path::Path,
};

#[derive(Debug, Clone)]
pub struct Source<'src> {
    pub script: &'src str,
    pub location: Option<&'src Path>,
}

impl<'src> Source<'src> {
    pub fn line(&self, n: usize) -> &'src str {
        self.script.lines().nth(n).unwrap_or_default()
    }

    pub fn span(&self, span: &SourceSpan) -> &'src str {
        self.script
            .get(span.bytes_range.clone())
            .unwrap_or_default()
    }
}

impl Deref for Source<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.script
    }
}

pub trait IntoSource<'src> {
    fn into_source(self) -> Source<'src>;
}

impl<'src> IntoSource<'src> for Source<'src> {
    fn into_source(self) -> Source<'src> {
        self.clone()
    }
}

impl<'a: 'src, 'src> IntoSource<'src> for &'a Source<'src> {
    fn into_source(self) -> Source<'src> {
        self.clone()
    }
}

impl<'src> IntoSource<'src> for &'src str {
    fn into_source(self) -> Source<'src> {
        Source {
            script: self.as_ref(),
            location: None,
        }
    }
}

impl<'src> IntoSource<'src> for (&'src str, &'src Path) {
    fn into_source(self) -> Source<'src> {
        let (script, location) = self;
        Source {
            script,
            location: Some(location),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSpan {
    pub line: usize,
    pub char_range: RangeInclusive<usize>,
    pub bytes_range: RangeInclusive<usize>,
}

impl SourceSpan {
    pub fn is_char(&self) -> bool {
        self.char_len() == 0
    }

    pub fn char_len(&self) -> usize {
        self.char_start().abs_diff(self.char_end())
    }

    pub fn char_start(&self) -> usize {
        *self.char_range.start()
    }

    pub fn char_end(&self) -> usize {
        *self.char_range.end()
    }

    pub fn bytes_start(&self) -> usize {
        *self.bytes_range.start()
    }

    pub fn bytes_end(&self) -> usize {
        *self.bytes_range.end()
    }
}

#[derive(Debug, Default)]
pub struct SourceSpanTracker {
    current_line: usize,
    start_char: usize,
    current_char: usize,
    start_byte: usize,
    current_byte: usize,
}

impl SourceSpanTracker {
    pub fn get(&self) -> SourceSpan {
        SourceSpan {
            line: self.current_line,
            char_range: self.start_char..=(self.current_char.saturating_sub(1)),
            bytes_range: self.start_byte..=(self.current_byte.saturating_sub(1)),
        }
    }

    pub fn eof(&self) -> SourceSpan {
        let char = self.current_char;
        let byte = self.current_byte;
        SourceSpan {
            line: self.current_line,
            char_range: char..=char,
            bytes_range: byte..=byte,
        }
    }

    pub fn current_char(&self) -> usize {
        self.current_char
    }

    pub fn current_byte(&self) -> usize {
        self.current_byte
    }

    pub fn set(&mut self, span: SourceSpan) {
        self.current_line = span.line;
        self.start_char = span.char_start();
        self.current_char = span.char_end().saturating_add(1);
        self.start_byte = span.bytes_start();
        self.current_byte = span.bytes_end().saturating_add(1);
    }

    pub fn advance_line(&mut self, lines: usize) {
        self.current_line += lines;
    }

    pub fn advance_char(&mut self, char: char) {
        self.current_char += 1;
        self.current_byte += char.len_utf8();
    }

    pub fn consume(&mut self) -> SourceSpan {
        let span = self.get();
        self.start_char = self.current_char;
        self.start_byte = self.current_byte;
        span
    }
}

#[derive(Debug)]
pub struct SourceSpanTrackerStack(Vec<SourceSpanTracker>);

impl SourceSpanTrackerStack {
    pub fn get(&self) -> SourceSpan {
        self.0.last().unwrap().get()
    }

    pub fn push(&mut self, start: SourceSpan) {
        let mut tracker = SourceSpanTracker::default();
        tracker.start_char = start.char_start();
        tracker.start_byte = start.bytes_start();
        self.0.push(tracker);
    }

    pub fn pop(&mut self) -> SourceSpan {
        assert!(self.0.len() > 1);
        let last = self.0.pop().unwrap();
        last.get()
    }

    pub fn advance_to(&mut self, span: SourceSpan) {
        for tracker in &mut self.0 {
            tracker.current_line = span.line;
            tracker.current_char = span.char_end().saturating_add(1);
            tracker.current_byte = span.bytes_end().saturating_add(1);
        }
    }
}

impl Default for SourceSpanTrackerStack {
    fn default() -> Self {
        Self(vec![Default::default()])
    }
}
