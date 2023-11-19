//! A small program that reads pairs of (line, column) on the standard input and writes triples of (line, column, hint)
//! on the standard output.

use std::{
  io::{stdin, Read},
  str::FromStr,
};

use clap::Parser;

#[derive(Debug, Parser)]
struct Cli {
  /// Keyset to use as base for hints.
  #[clap(short, long)]
  keyset: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Trie {
  key: char,
  below: Vec<Trie>,
}

impl Default for Trie {
  fn default() -> Self {
    Self {
      key: ' ', // root is ignored
      below: Vec::default(),
    }
  }
}

impl Trie {
  fn new(key: char) -> Self {
    Self {
      key,
      below: Vec::default(),
    }
  }

  /// Grow the trie by one key.
  ///
  /// Return `true` if the key was inserted in this trie.
  fn grow(&mut self, keyset: &[char]) {
    if self.below.len() < keyset.len() {
      // we are not saturated, so stop here
      let hint = keyset[self.below.len()];
      self.below.push(Self::new(hint));
    } else {
      // saturated, so go down to try a better place to insert
      let node = self
        .below
        .iter_mut()
        .rfind(|node| node.below.len() < keyset.len());

      if let Some(node) = node {
        // we grow twice because we transform an old leaf into a node
        if node.below.is_empty() {
          node.grow(keyset);
        }

        node.grow(keyset);
      } else {
        let i = self.below.len() - 1;
        self.below[i].grow(keyset);
      }
    }
  }

  /// Grow the trie repeatedly `n` times.
  fn grow_repeatedly(&mut self, n: usize, keyset: &[char]) {
    for _ in 0..n {
      self.grow(keyset);
    }
  }

  fn hints(&self) -> Vec<String> {
    let mut paths = Vec::default();

    for below in &self.below {
      below.hints_("", &mut paths);
    }

    paths
  }

  fn hints_(&self, path: &str, paths: &mut Vec<String>) {
    let path = format!("{path}{}", self.key);

    if self.below.is_empty() {
      paths.push(path);
    } else {
      for below in &self.below {
        below.hints_(&path, paths);
      }
    }
  }
}

/// Position in the buffer.
#[derive(Debug)]
struct Pos {
  line: usize,
  col: usize,
}

impl FromStr for Pos {
  type Err = ();

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut parts = s.split('.');
    let line = parts.next().ok_or(())?.parse().map_err(|_| ())?;
    let col = parts.next().ok_or(())?.parse().map_err(|_| ())?;

    Ok(Pos { line, col })
  }
}

/// A selection in the buffer.
#[derive(Debug)]
struct Sel {
  start: Pos,
  end: Pos,
}

impl FromStr for Sel {
  type Err = ();

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut parts = s.split(',');
    let start = parts.next().ok_or(())?.parse().map_err(|_| ())?;
    let end = parts.next().ok_or(())?.parse().map_err(|_| ())?;

    Ok(Sel { start, end })
  }
}

fn main() {
  let cli = Cli::parse();
  let keyset = cli.keyset.chars().collect::<Vec<_>>();

  // read the selections from standard input
  let mut contents = String::new();
  stdin().read_to_string(&mut contents).unwrap(); // TODO: unwrap
  let sels: Vec<Sel> = contents
    .split_whitespace()
    .flat_map(|s| s.parse())
    .collect::<Vec<_>>();

  let mut trie = Trie::default();
  trie.grow_repeatedly(sels.len(), &keyset);

  print!("set-option buffer hop_ranges %val{{timestamp}} ");
  for (hint, sel) in trie.hints().into_iter().zip(sels) {
    print!(
      "{start_line}.{start_col},{end_line}.{end_col}|{{green}}{hint} ",
      start_line = sel.start.line,
      start_col = sel.start.col,
      end_line = sel.end.line,
      end_col = sel.end.col,
    );
  }
  println!();
}

#[cfg(test)]
mod tests {
  use crate::Trie;

  #[test]
  fn iter() {
    let keyset = "abcd".chars().collect::<Vec<_>>();

    let mut trie = Trie::default();
    trie.grow_repeatedly(4, &keyset);
    let hints = trie.hints();
    assert_eq!(hints, vec!["a", "b", "c", "d"]);

    let mut trie = Trie::default();
    trie.grow_repeatedly(10, &keyset);
    let hints = trie.hints();
    assert_eq!(
      hints,
      vec!["a", "b", "ca", "cb", "cc", "cd", "da", "db", "dc", "dd"]
    );
  }
}
