//! A small program that reads pairs of (line, column) on the standard input and writes triples of (line, column, hint)
//! on the standard output.

use std::str::FromStr;

use clap::Parser;
use unicode_segmentation::UnicodeSegmentation;

#[cfg(feature = "init")]
const RC: &str = include_str!("../hop.kak");

#[derive(Debug, Parser)]
#[clap(
  author = "Dimitri Sabadie <dimitri.sabadie@gmail.com>",
  name = "hop-kak",
  version = concat!(env!("CARGO_PKG_VERSION"), "-", env!("GIT_HEAD")),
  about = "Hopping around in Kakoune!"
)]
struct Cli {
  /// Initialize Kakoune.
  ///
  /// This should be called only once, when starting a Kakoune session.
  #[cfg(feature = "init")]
  #[clap(long)]
  init: bool,

  /// Keyset to use as base for hints.
  #[clap(short, long)]
  keyset: Option<String>,

  /// Selections to act on.
  ///
  /// The syntax of a single selection is two pairs separated by a comma, each pair being a pair of period separated
  /// number: `line_start.column_start,line_end.column_end`.
  ///
  /// Selections are space separated.
  #[clap(short, long)]
  sels: Option<String>,

  /// Labels hints to reduce.
  ///
  /// This is a list of labels, space separated string, to reduce. Those are zipped with `sels`.
  #[clap(short, long)]
  labels: Option<String>,

  /// Reduction key.
  ///
  /// Key used to reduce the list of `labels`.
  #[clap(short = 'z', long)]
  key: Option<String>,
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

  fn labels(&self) -> Vec<String> {
    let mut paths = Vec::default();

    for below in &self.below {
      below.labels_("", &mut paths);
    }

    paths
  }

  fn labels_(&self, path: &str, paths: &mut Vec<String>) {
    let path = format!("{path}{}", self.key);

    if self.below.is_empty() {
      paths.push(path);
    } else {
      for below in &self.below {
        below.labels_(&path, paths);
      }
    }
  }
}

/// Position in the buffer.
#[derive(Clone, Debug)]
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
#[derive(Clone, Debug)]
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

impl Sel {
  fn to_str(&self) -> String {
    format!(
      "{line_start}.{col_start},{line_end}.{col_end}",
      line_start = self.start.line,
      col_start = self.start.col,
      line_end = self.end.line,
      col_end = self.end.col,
    )
  }
}

#[derive(Debug)]
struct App {
  keyset: Vec<char>,
  sels: Vec<Sel>,
  labels: Vec<String>,
  key: Option<String>,
}

impl App {
  fn new(cli: Cli) -> Self {
    let keyset = cli
      .keyset
      .map(|keyset| keyset.chars().collect::<Vec<_>>())
      .unwrap_or_default();
    let sels: Vec<_> = cli
      .sels
      .unwrap_or_default()
      .split_whitespace()
      .filter_map(|sel| sel.parse::<Sel>().ok())
      .collect();
    let labels = cli
      .labels
      .map(|labels| labels.split_whitespace().map(|s| s.to_owned()).collect())
      .unwrap_or_default();
    let key = cli.key;

    Self {
      keyset,
      sels,
      labels,
      key,
    }
  }

  fn process(self) -> Response {
    // if we don’t have any label / no key is set, then we are tasked to generate the labels first
    match self.key {
      None => Self::generate_labels(self.sels, self.keyset),
      Some(key) => Self::reduce(self.sels, self.labels, key),
    }
  }

  fn generate_labels(sels: Vec<Sel>, keyset: Vec<char>) -> Response {
    let mut trie = Trie::default();
    trie.grow_repeatedly(sels.len(), &keyset);

    let replace_ranges = trie
      .labels()
      .into_iter()
      .zip(sels)
      .map(|(label, sel)| ReplaceRange::new(sel, label))
      .collect();

    Response::LabelsGenerated { replace_ranges }
  }

  fn reduce(sels: Vec<Sel>, labels: Vec<String>, key: String) -> Response {
    if key == "<esc>" {
      return Response::Cleanup;
    }

    let replace_ranges = sels
      .into_iter()
      .zip(labels)
      .filter_map(|(sel, label)| {
        label
          .strip_prefix(&key)
          .map(|label| ReplaceRange::new(sel, label.to_owned()))
      })
      .collect();

    Response::Reduced { replace_ranges }
  }
}

#[derive(Debug)]
enum Response {
  Cleanup,
  LabelsGenerated { replace_ranges: Vec<ReplaceRange> },
  Reduced { replace_ranges: Vec<ReplaceRange> },
}

impl Response {
  fn display_replace_ranges(replace_ranges: &[ReplaceRange]) {
    print!("set-option window hop_ranges %val{{timestamp}} ");

    for range in replace_ranges {
      let sel = &range.sel;
      let label = &range.label;
      let label_len = label
        .graphemes(true)
        .count()
        .min(sel.end.col - sel.start.col + 1);
      let mut graphemes = label.graphemes(true).take(label_len);

      // always display the first grapheme differently
      if let Some(head) = graphemes.next() {
        print!(
          "{start_line}.{start_col}+1|{{hop_label_head}}{head} ",
          start_line = sel.start.line,
          start_col = sel.end.col - label_len + 1,
        );

        let tail: String = graphemes.collect();

        if !tail.is_empty() {
          print!(
            "{start_line}.{start_col}+{label_len}|{{hop_label_tail}}{tail} ",
            start_line = sel.start.line,
            start_col = sel.end.col - label_len + 2,
            label_len = label_len - 1
          );
        }
      }
    }

    println!();
  }

  fn display_cleanup() {
    println!("try %{{ remove-highlighter window/hop-ranges }}");
  }

  fn display_reduce_callback(replace_ranges: &[ReplaceRange]) {
    if replace_ranges.len() == 1 {
      Self::display_cleanup();
      return;
    }

    let sels: Vec<_> = replace_ranges.iter().map(|r| r.sel.to_str()).collect();
    let sels = sels.join(" ");
    let labels: Vec<_> = replace_ranges.iter().map(|r| r.label.as_str()).collect();
    let labels = labels.join(" ");

    println!(
      r#"on-key 'evaluate-commands -save-regs ^ -no-hooks -- %sh{{ {bin} --sels "{sels}" --labels "{labels}" --key $kak_key }}'"#,
      bin = std::env::current_exe().unwrap().display()
    );
  }

  fn display_new_sels(replace_ranges: &[ReplaceRange]) {
    print!(r#"set-register ^ "%val{{buffile}}@%val{{timestamp}}@0" "#);
    for range in replace_ranges {
      print!("{} ", range.sel.to_str());
    }
    println!();

    println!("execute-keys z");
  }

  fn into_stdout(self) {
    match self {
      Self::Cleanup => Self::display_cleanup(),

      Self::LabelsGenerated { replace_ranges } => {
        Self::display_cleanup();

        println!("add-highlighter window/hop-ranges replace-ranges hop_ranges");

        Self::display_replace_ranges(&replace_ranges);
        Self::display_reduce_callback(&replace_ranges);
      }

      Self::Reduced { replace_ranges } => {
        Self::display_replace_ranges(&replace_ranges);
        Self::display_new_sels(&replace_ranges);
        Self::display_reduce_callback(&replace_ranges);
      }
    }
  }
}

#[derive(Debug)]
struct ReplaceRange {
  sel: Sel,
  label: String,
}

impl ReplaceRange {
  fn new(sel: Sel, label: impl Into<String>) -> Self {
    Self {
      sel,
      label: label.into(),
    }
  }
}

fn main() {
  let cli = Cli::parse();

  #[cfg(feature = "init")]
  if cli.init {
    print!("{}", RC);
    return;
  }

  let app = App::new(cli);

  let resp = app.process();
  resp.into_stdout();
}

#[cfg(test)]
mod tests {
  use crate::Trie;

  #[test]
  fn iter() {
    let keyset = "abcd".chars().collect::<Vec<_>>();

    let mut trie = Trie::default();
    trie.grow_repeatedly(4, &keyset);
    let hints = trie.labels();
    assert_eq!(hints, vec!["a", "b", "c", "d"]);

    let mut trie = Trie::default();
    trie.grow_repeatedly(10, &keyset);
    let hints = trie.labels();
    assert_eq!(
      hints,
      vec!["a", "b", "ca", "cb", "cc", "cd", "da", "db", "dc", "dd"]
    );
  }
}
