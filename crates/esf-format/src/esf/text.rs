//! Splitting a text document into fits and lines, without the SDE.

use super::model::{Error, Location, State};

pub(super) const SIGILS: [char; 7] = [':', '+', '{', '!', '@', '/', '"'];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LineKind {
    Item,
    Empty,
    Reference,
}

#[derive(Debug)]
pub(super) struct Line {
    pub number: usize,
    pub kind: LineKind,
    pub count: Option<u32>,
    pub name: String,
    pub fit_name: Option<String>,
    pub charge: Option<String>,
    pub charge_count: Option<u32>,
    pub mutaplasmid: Option<String>,
    pub overrides: Vec<(String, String)>,
    pub state: Option<State>,
    pub location: Option<Location>,
    pub index: Option<u32>,
}

impl Line {
    fn new(number: usize, kind: LineKind, count: Option<u32>) -> Line {
        Line {
            number,
            kind,
            count,
            name: String::new(),
            fit_name: None,
            charge: None,
            charge_count: None,
            mutaplasmid: None,
            overrides: Vec::new(),
            state: None,
            location: None,
            index: None,
        }
    }
}

#[derive(Debug)]
pub(super) struct Block {
    pub number: usize,
    pub hull: Option<String>,
    pub hull_line: usize,
    pub fit_name: Option<String>,
    pub mode: Option<String>,
    pub lines: Vec<Line>,
}

pub(super) fn is_count(token: &str) -> bool {
    token
        .strip_suffix('x')
        .is_some_and(|digits| !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
}

struct Cursor<'a> {
    number: usize,
    text: &'a str,
    at: usize,
}

impl<'a> Cursor<'a> {
    fn error<T>(&self, message: impl Into<String>) -> Result<T, Error> {
        Err(Error::at(self.number, message))
    }

    fn rest(&self) -> &'a str {
        &self.text[self.at..]
    }

    fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }

    fn token(&self) -> &'a str {
        let rest = self.rest();
        &rest[..rest.find(' ').unwrap_or(rest.len())]
    }

    fn skip_spaces(&mut self) -> bool {
        let start = self.at;
        while self.peek() == Some(' ') {
            self.at += 1;
        }
        self.at > start
    }

    /// Step over the space before the next element; false at the end of the line.
    fn next_element(&mut self) -> Result<bool, Error> {
        let spaced = self.skip_spaces();
        if self.rest().is_empty() {
            return Ok(false);
        }
        if self.rest().starts_with("//") {
            if !spaced {
                return self.error("a comment needs a space before it");
            }
            self.at = self.text.len();
            return Ok(false);
        }
        if !spaced {
            return self.error(format!("expected a space before {:?}", self.token()));
        }
        Ok(true)
    }

    fn finish(&mut self) -> Result<(), Error> {
        match self.next_element()? {
            true => self.error(format!("unexpected {:?}", self.token())),
            false => Ok(()),
        }
    }

    fn count(&mut self) -> Result<Option<u32>, Error> {
        let token = self.token();
        if !is_count(token) {
            return Ok(None);
        }
        self.at += token.len();
        if !self.next_element()? {
            return self.error("a count must be followed by a type name");
        }
        match token[..token.len() - 1].parse::<u32>() {
            Ok(0) => self.error("a count must be at least 1"),
            Ok(count) => Ok(Some(count)),
            Err(_) => self.error("a count is too large"),
        }
    }

    fn quoted(&mut self) -> Result<String, Error> {
        self.at += 1;
        let mut out = String::new();
        loop {
            let Some(c) = self.peek() else {
                return self.error("unterminated quoted string");
            };
            self.at += c.len_utf8();
            if c == '"' {
                if self.peek() != Some('"') {
                    break;
                }
                self.at += 1;
            }
            out.push(c);
        }
        if out.is_empty() {
            return self.error("a quoted string cannot be empty");
        }
        Ok(out)
    }

    fn type_name(&mut self) -> Result<String, Error> {
        match self.peek() {
            Some('"') => return self.quoted(),
            None | Some(' ') => return self.error("expected a type name"),
            Some(c) if SIGILS.contains(&c) => return self.error("expected a type name"),
            Some(c @ ('-' | '%')) => {
                return self.error(format!("a name starting with {c:?} must be quoted"));
            }
            Some(_) => {}
        }

        let mut words = vec![self.token()];
        self.at += self.token().len();
        loop {
            let start = self.at;
            self.skip_spaces();
            match self.peek() {
                Some(c) if self.at > start && !SIGILS.contains(&c) => {
                    words.push(self.token());
                    self.at += self.token().len();
                }
                _ => {
                    self.at = start;
                    return Ok(words.join(" "));
                }
            }
        }
    }

    fn location(&mut self) -> Result<(Location, Option<u32>), Error> {
        let rest = &self.rest()[1..];
        let name_end = rest
            .find(|c: char| !c.is_ascii_lowercase())
            .unwrap_or(rest.len());
        let digits_end = rest[name_end..]
            .find(|c: char| !c.is_ascii_digit())
            .map_or(rest.len(), |end| name_end + end);
        let (name, digits) = (&rest[..name_end], &rest[name_end..digits_end]);
        if name.is_empty() {
            return self.error(format!("unknown location {:?}", self.token()));
        }
        self.at += 1 + digits_end;

        let location = Location::from_name(name);
        if digits.is_empty() {
            return match location {
                Some(location) => Ok((location, None)),
                None => self.error(format!("unknown location {name:?}")),
            };
        }
        let Some(rack) = location.filter(|location| location.is_rack()) else {
            return self.error(format!("unknown rack {name:?}"));
        };
        match digits.parse::<u32>() {
            Ok(0) => self.error("slots are numbered from 1"),
            Ok(index) => Ok((rack, Some(index))),
            Err(_) => self.error("a slot number is too large"),
        }
    }

    fn state(&mut self) -> Result<State, Error> {
        let token = self.token();
        let name = token[1..]
            .split(|c: char| !c.is_ascii_lowercase())
            .next()
            .unwrap_or_default();
        let Some(state) = State::from_name(name) else {
            return self.error(format!("unknown state {token:?}"));
        };
        self.at += 1 + name.len();
        Ok(state)
    }

    fn attribute_name(&mut self) -> Option<&'a str> {
        let rest = self.rest();
        if !rest.starts_with(|c: char| c.is_ascii_alphabetic()) {
            return None;
        }
        let end = rest
            .find(|c: char| !c.is_ascii_alphanumeric())
            .unwrap_or(rest.len());
        self.at += end;
        Some(&rest[..end])
    }

    fn value(&mut self) -> Option<&'a str> {
        let rest = self.rest();
        let digits = |from: usize| {
            rest[from..]
                .find(|c: char| !c.is_ascii_digit())
                .map_or(rest.len(), |end| from + end)
        };
        let start = usize::from(rest.starts_with('-'));
        let mut end = digits(start);
        if end == start {
            return None;
        }
        if rest[end..].starts_with('.') {
            let fraction = digits(end + 1);
            if fraction > end + 1 {
                end = fraction;
            }
        }
        self.at += end;
        Some(&rest[..end])
    }

    fn overrides(&mut self) -> Result<Vec<(String, String)>, Error> {
        self.at += 1;
        self.skip_spaces();
        let mut pairs = Vec::new();
        loop {
            let start = self.at;
            let pair = self.attribute_name().and_then(|name| {
                let spaced = self.skip_spaces();
                let value = self.value()?;
                spaced.then(|| (name.to_string(), value.to_string()))
            });
            let Some(pair) = pair else {
                self.at = start;
                return self.error("malformed override; expected 'attribute value'");
            };
            pairs.push(pair);
            self.skip_spaces();
            match self.peek() {
                Some(',') => {
                    self.at += 1;
                    self.skip_spaces();
                }
                Some('}') => {
                    self.at += 1;
                    return Ok(pairs);
                }
                _ => return self.error("malformed overrides; expected ',' or '}'"),
            }
        }
    }
}

fn hull(block: &mut Block, cursor: &mut Cursor) -> Result<(), Error> {
    block.hull_line = cursor.number;
    cursor.skip_spaces();
    if cursor.token() == "-" {
        cursor.at += 1;
        if cursor.next_element()? {
            if cursor.peek() != Some('"') {
                return cursor.error("a hull line of '-' takes only a fit name");
            }
            block.fit_name = Some(cursor.quoted()?);
        }
        return cursor.finish();
    }

    if is_count(cursor.token()) {
        return cursor.error("the hull line cannot have a count");
    }
    block.hull = Some(cursor.type_name()?);
    let mut more = cursor.next_element()?;
    if more && cursor.peek() == Some('"') {
        block.fit_name = Some(cursor.quoted()?);
        more = cursor.next_element()?;
    }
    if more {
        if cursor.peek() != Some('/') {
            return cursor.error(format!("unexpected {:?} on the hull line", cursor.token()));
        }
        cursor.at += 1;
        block.mode = Some(cursor.type_name()?);
    }
    cursor.finish()
}

fn entry(cursor: &mut Cursor) -> Result<Line, Error> {
    cursor.skip_spaces();
    let count = cursor.count()?;

    if cursor.token() == "-" {
        cursor.at += 1;
        if !cursor.next_element()? || cursor.peek() != Some('@') {
            return cursor.error("an empty slot must name its rack");
        }
        let (rack, index) = cursor.location()?;
        if !rack.is_rack() {
            return cursor.error("an empty slot must name a rack");
        }
        if index.is_some() && count.is_some() {
            return cursor.error("a count and a pinned slot are mutually exclusive");
        }
        cursor.finish()?;
        let mut line = Line::new(cursor.number, LineKind::Empty, count);
        line.location = Some(rack);
        line.index = index;
        return Ok(line);
    }

    let mut line = Line::new(cursor.number, LineKind::Item, count);
    line.name = cursor.type_name()?;
    let mut more = cursor.next_element()?;

    if more && cursor.peek() == Some('"') {
        line.kind = LineKind::Reference;
        line.fit_name = Some(cursor.quoted()?);
        if cursor.next_element()? {
            if cursor.peek() != Some('@') {
                return cursor.error("a reference takes only a count and a location");
            }
            let (location, index) = cursor.location()?;
            if index.is_some() || location.is_rack() {
                return cursor.error("a reference can only be stored");
            }
            line.location = Some(location);
            cursor.finish()?;
        }
        return Ok(line);
    }

    let mut stage = 0;
    while more {
        match cursor.peek() {
            Some(':') if stage < 1 => {
                cursor.at += 1;
                line.charge_count = cursor.count()?;
                line.charge = Some(cursor.type_name()?);
                stage = 1;
            }
            Some('+') if stage < 2 => {
                cursor.at += 1;
                line.mutaplasmid = Some(cursor.type_name()?);
                stage = 2;
            }
            Some('{') if stage < 3 => {
                line.overrides = cursor.overrides()?;
                stage = 3;
            }
            Some('!') if stage < 4 => {
                line.state = Some(cursor.state()?);
                stage = 4;
            }
            Some('@') if stage < 5 => {
                let (location, index) = cursor.location()?;
                if index.is_none() && location.is_rack() {
                    return cursor
                        .error("an item names a rack only to pin a slot, which needs an index");
                }
                if index.is_some() && count.is_some() {
                    return cursor.error("a count and a pinned slot are mutually exclusive");
                }
                line.location = Some(location);
                line.index = index;
                stage = 5;
            }
            _ => return cursor.error(format!("unexpected {:?}", cursor.token())),
        }
        more = cursor.next_element()?;
    }
    Ok(line)
}

fn is_blank(text: &str) -> bool {
    let text = text.trim_start_matches(' ');
    text.is_empty() || text.starts_with("//")
}

/// `%name/version`, with optional spaces around it.
fn header(text: &str) -> Option<(&str, &str)> {
    let (name, version) = text.trim_matches(' ').strip_prefix('%')?.split_once('/')?;
    let name_ok = name.starts_with(|c: char| c.is_ascii_lowercase())
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
    let version_ok = !version.is_empty() && version.bytes().all(|b| b.is_ascii_digit());
    (name_ok && version_ok).then_some((name, version))
}

/// Split a text document into its fits; other blocks are skipped.
pub(super) fn parse(data: &[u8]) -> Result<Vec<Block>, Error> {
    let text = std::str::from_utf8(data)
        .map_err(|error| Error::new(format!("not valid UTF-8 at byte {}", error.valid_up_to())))?;

    let mut lines: Vec<&str> = text.split('\n').collect();
    if lines.last() == Some(&"") {
        lines.pop();
    }
    if lines.is_empty() {
        return Err(Error::new("the document is empty"));
    }

    let mut fits: Vec<Block> = Vec::new();
    let mut in_fit = false;
    let mut in_block = false;
    for (number, raw) in (1..).zip(lines) {
        let raw = raw.strip_suffix('\r').unwrap_or(raw);
        if raw.contains('\r') {
            return Err(Error::at(number, "a CR must be followed by an LF"));
        }

        if raw.trim_start_matches(' ').starts_with('%') {
            let Some((name, version)) = header(raw) else {
                return Err(Error::at(number, "malformed header"));
            };
            if in_fit && fits.last().is_some_and(|fit| fit.hull_line == 0) {
                return Err(Error::at(
                    fits.last().unwrap().number,
                    "a fit needs a hull line",
                ));
            }
            in_block = true;
            in_fit = name == "esf";
            if in_fit {
                if version != "1" {
                    return Err(Error::at(
                        number,
                        format!("unsupported version esf/{version}"),
                    ));
                }
                fits.push(Block {
                    number,
                    hull: None,
                    hull_line: 0,
                    fit_name: None,
                    mode: None,
                    lines: Vec::new(),
                });
            }
            continue;
        }

        if !in_block {
            return Err(Error::at(number, "a document starts with a header"));
        }
        if !in_fit || is_blank(raw) {
            continue;
        }

        let block = fits.last_mut().unwrap();
        let mut cursor = Cursor {
            number,
            text: raw,
            at: 0,
        };
        if block.hull_line == 0 {
            hull(block, &mut cursor)?;
        } else {
            block.lines.push(entry(&mut cursor)?);
        }
    }

    if in_fit && let Some(fit) = fits.last().filter(|fit| fit.hull_line == 0) {
        return Err(Error::at(fit.number, "a fit needs a hull line"));
    }
    if fits.is_empty() {
        return Err(Error::new("a document holds at least one fit"));
    }
    Ok(fits)
}
