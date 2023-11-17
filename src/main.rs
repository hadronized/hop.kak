//! A small program that reads pairs of (line, column) on the standard input and writes triples of (line, column, hint)
//! on the standard output.

use clap::Parser;

#[derive(Debug, Parser)]
struct Cli {
  /// Keys to use as base for hints.
  #[clap(short, long)]
  keys: String,
}

#[derive(Debug)]
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
  fn grow(&mut self, keys: &[char]) {
    if self.below.len() < keys.len() {
      // we are not saturated, so stop here
      let hint = keys[self.below.len()];
      self.below.push(Self::new(hint));
    } else {
      // saturated, so go down to try a better place to insert
      let node = self
        .below
        .iter_mut()
        .find(|node| node.below.len() < keys.len());

      if let Some(node) = node {
        // we grow twice because we transform an old leaf into a node
        if node.below.is_empty() {
          node.grow(keys);
        }

        node.grow(keys);
      } else {
        self.below[0].grow(keys);
      }
    }
  }

  /// Grow the trie repeatedly `n` times.
  fn grow_repeatedly(&mut self, n: usize, keys: &[char]) {
    for _ in 0..n {
      self.grow(keys);
    }
  }

  /// Traverse the tree paths.
  fn paths(&self) -> Vec<String> {
    let mut paths = Vec::default();

    for below in &self.below {
      below.paths_("", &mut paths);
    }

    paths
  }

  fn paths_(&self, path: &str, paths: &mut Vec<String>) {
    let path = format!("{path}{}", self.key);

    if self.below.is_empty() {
      paths.push(path);
    } else {
      for below in &self.below {
        below.paths_(&path, paths);
      }
    }
  }
}

fn main() {
  let cli = Cli::parse();
  let mut trie = Trie::default();
  let keyset = cli.keys.chars().collect::<Vec<_>>();

  trie.grow_repeatedly(20, &keyset);

  let paths = trie.paths();
  println!("{}\n", paths.join("\n"));
}

#[cfg(test)]
mod tests {
  use crate::Trie;

  #[test]
  fn grow() {
    let mut trie = Trie::default();
    let keyset = "abcd".chars().collect::<Vec<_>>();

    trie.grow(&keyset);
  }
}
