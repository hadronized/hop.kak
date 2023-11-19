//! A small program that reads pairs of (line, column) on the standard input and writes triples of (line, column, hint)
//! on the standard output.

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
struct Cli {
  /// Keys to use as base for hints.
  #[clap(short, long)]
  keys: String,

  /// Operation mode. This dictates what is expected on the standard input and what is produced on the standard output.
  #[clap(subcommand)]
  operation: Operation,
}

#[derive(Clone, Debug, Subcommand)]
enum Operation {
  /// Create hints for the content of the buffer (read from the standard input), and display them on the standard
  /// output.
  Hints,

  /// Reduce a set of tagged hints (read from the standard input) with the provided key, and output the reduction on the
  /// standard output.
  Reduce { key: char },
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
struct Pos {
  line: usize,
  col: usize,
}

/// Buffer hint, which is the start and end of a token and the label (keys).
struct Hint {
  start: Pos,
  end: Pos,
  label: String,
}

/// Generate hints.
fn op_hints(keyset: &[char], words: &[Pos]) -> Vec<Hint> {
  let mut trie = Trie::default();
  trie.grow_repeatedly(words.len(), keyset);

  todo!()
}

fn main() {
  let cli = Cli::parse();
  let mut trie = Trie::default();
  let keyset = cli.keys.chars().collect::<Vec<_>>();

  trie.grow_repeatedly(10, &keyset);

  let paths = trie.hints();
  println!("{}\n", paths.join("\n"));
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
