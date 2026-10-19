//! Git graphs. Ported from Pele `src/diagrams/git/` (`db.ts` is this file).

mod parser;
mod render;

use std::cmp::Ordering;

use indexmap::IndexMap;

use crate::config::{Config, ConfigValue};
use crate::detect::DiagramType;
use crate::diagrams::ParseInput;
use crate::error::Error;
use crate::util::{js, math};

pub use parser::{parse_git, CommitTypeName, GitDirection, GitGraphAst, StatementAst};
pub(crate) use render::render;

/// Commit kinds, as Mermaid numbers them.
pub mod commit_type {
    pub const NORMAL: u8 = 0;
    pub const REVERSE: u8 = 1;
    pub const HIGHLIGHT: u8 = 2;
    pub const MERGE: u8 = 3;
    pub const CHERRY_PICK: u8 = 4;
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct GitCommit {
    pub id: String,
    pub message: String,
    pub seq: usize,
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub kind: u8,
    pub tags: Vec<String>,
    pub parents: Vec<String>,
    pub branch: String,
    /// The type a merge was given with `type:`, drawn in place of the merge symbol.
    pub custom_type: Option<u8>,
    /// Whether a merge was given its id with `id:`.
    pub custom_id: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct GitModel {
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
    pub direction: GitDirection,
    /// Keyed by commit id, in the order the ids first appeared.
    pub commits: IndexMap<String, GitCommit>,
    /// The id of each branch's latest commit, or `None` while the branch has none.
    pub branches: IndexMap<String, Option<String>>,
    /// Branch names in drawing order: by `order`, then by creation.
    pub lanes: Vec<String>,
    pub current_branch: String,
    pub head: Option<GitCommit>,
    pub warnings: Vec<String>,
}

fn fail(message: impl Into<String>) -> Error {
    Error::semantic(message).with_type(DiagramType::GitGraph)
}

fn type_code(name: CommitTypeName) -> u8 {
    match name {
        CommitTypeName::Reverse => commit_type::REVERSE,
        CommitTypeName::Highlight => commit_type::HIGHLIGHT,
        CommitTypeName::Normal => commit_type::NORMAL,
    }
}

/// The state Pele's database keeps while the statements run (`buildGit`).
struct Db {
    commits: IndexMap<String, GitCommit>,
    branches: IndexMap<String, Option<String>>,
    orders: IndexMap<String, Option<f64>>,
    warnings: Vec<String>,
    /// The id of the head commit. Pele keeps the commit itself; it is always the commit the
    /// map holds under this id, as every commit stored under an id becomes the head.
    head: Option<String>,
    current: String,
    direction: GitDirection,
    seq: usize,
}

const GOLDEN: f64 = 2654435761.0; // 0x9e3779b1

impl Db {
    /// Mermaid appends seven random hex digits to the sequence number. These digits are
    /// derived from the sequence number instead, so the same text always gives the same ids.
    fn next_id(&self) -> String {
        let mut salt = (self.seq + 1) as f64;
        loop {
            let hash = math::imul(js::to_int32(salt), js::to_int32(GOLDEN)) as u32;
            let hex = format!("{:08x}", hash);
            let id = format!("{}-{}", self.seq, &hex[..7]);
            if !self.commits.contains_key(&id) {
                return id;
            }
            salt += GOLDEN;
        }
    }

    fn take_seq(&mut self) -> usize {
        self.seq += 1;
        self.seq - 1
    }

    fn add(&mut self, commit: GitCommit) {
        self.head = Some(commit.id.clone());
        self.branches.insert(self.current.clone(), Some(commit.id.clone()));
        self.commits.insert(commit.id.clone(), commit);
    }

    fn head_parents(&self) -> Vec<String> {
        self.head.iter().cloned().collect()
    }

    fn checkout(&mut self, branch: &str) -> Result<(), Error> {
        let Some(id) = self.branches.get(branch) else {
            return Err(fail(format!("Trying to checkout branch which is not yet created. (Help try using \"branch {}\")", branch)));
        };
        let head = id.as_ref().filter(|id| !id.is_empty() && self.commits.contains_key(id.as_str())).cloned();
        self.current = branch.to_string();
        self.head = head;
        Ok(())
    }

    fn commit(&mut self, msg: String, id: Option<String>, kind: u8, tags: Vec<String>) {
        let id = match id {
            Some(id) if !id.is_empty() => id,
            _ => self.next_id(),
        };
        let commit = GitCommit {
            id,
            message: msg,
            seq: self.take_seq(),
            kind,
            tags,
            parents: self.head_parents(),
            branch: self.current.clone(),
            custom_type: None,
            custom_id: None,
        };
        if self.commits.contains_key(&commit.id) {
            self.warnings.push(format!("Commit ID {} already exists", commit.id));
        }
        self.add(commit);
    }

    fn branch(&mut self, name: String, order: Option<f64>) -> Result<(), Error> {
        if self.branches.contains_key(&name) {
            return Err(fail(format!(
                "Trying to create an existing branch. (Help: Either use a new name if you want create a new branch or try using \"checkout {}\")",
                name
            )));
        }
        self.branches.insert(name.clone(), self.head.clone());
        self.orders.insert(name.clone(), order);
        self.checkout(&name)
    }

    fn merge(&mut self, other: String, custom_id: String, kind: Option<u8>, tags: Vec<String>) -> Result<(), Error> {
        let current = self.current.clone();
        let current_head = self.branches.get(&current).cloned().flatten().filter(|id| !id.is_empty());
        let other_head = self.branches.get(&other).cloned().flatten().filter(|id| !id.is_empty());
        let current_commit = current_head.as_ref().and_then(|id| self.commits.get(id));
        let other_commit = other_head.as_ref().and_then(|id| self.commits.get(id));
        if let (Some(c), Some(_)) = (current_commit, other_commit) {
            if c.branch == other {
                return Err(fail(format!("Cannot merge branch '{}' into itself.", other)));
            }
        }
        if current == other {
            return Err(fail("Incorrect usage of \"merge\". Cannot merge a branch to itself"));
        }
        let Some(current_commit) = current_commit else {
            return Err(fail(format!("Incorrect usage of \"merge\". Current branch ({})has no commits", current)));
        };
        if !self.branches.contains_key(&other) {
            return Err(fail(format!("Incorrect usage of \"merge\". Branch to be merged ({}) does not exist", other)));
        }
        let Some(other_commit) = other_commit else {
            return Err(fail(format!("Incorrect usage of \"merge\". Branch to be merged ({}) has no commits", other)));
        };
        // Both come from the map, so they are the same commit when their ids are the same.
        if current_commit.id == other_commit.id {
            return Err(fail("Incorrect usage of \"merge\". Both branches have same head"));
        }
        if !custom_id.is_empty() && self.commits.contains_key(&custom_id) {
            return Err(fail(format!(
                "Incorrect usage of \"merge\". Commit with id:{} already exists, use different custom id",
                custom_id
            )));
        }
        let id = if custom_id.is_empty() { self.next_id() } else { custom_id.clone() };
        let parents = match &self.head {
            None => Vec::new(),
            Some(head) => vec![head.clone(), other_head.clone().unwrap_or_default()],
        };
        let commit = GitCommit {
            id,
            message: format!("merged branch {} into {}", other, current),
            seq: self.take_seq(),
            parents,
            branch: current,
            kind: commit_type::MERGE,
            custom_type: kind,
            custom_id: Some(!custom_id.is_empty()),
            tags,
        };
        self.add(commit);
        Ok(())
    }

    fn cherry_pick(&mut self, source_id: Option<String>, tags: Option<Vec<String>>, parent: Option<String>) -> Result<(), Error> {
        let source = source_id.filter(|id| !id.is_empty()).and_then(|id| self.commits.get(&id)).cloned();
        let Some(source) = source else {
            return Err(fail("Incorrect usage of \"cherryPick\". Source commit id should exist and provided"));
        };
        let parent = parent.filter(|p| !p.is_empty());
        if let Some(parent) = &parent {
            if !source.parents.contains(parent) {
                return Err(fail("Invalid operation: The specified parent commit is not an immediate parent of the cherry-picked commit."));
            }
        }
        if source.kind == commit_type::MERGE && parent.is_none() {
            return Err(fail(
                "Incorrect usage of cherry-pick: If the source commit is a merge commit, an immediate parent commit must be specified.",
            ));
        }
        // Pele checks a target id here, which its parser always leaves empty.
        if source.branch == self.current {
            return Err(fail("Incorrect usage of \"cherryPick\". Source commit is already on current branch"));
        }
        let current_head = self.branches.get(&self.current).cloned().flatten().filter(|id| !id.is_empty());
        if !current_head.is_some_and(|id| self.commits.contains_key(&id)) {
            return Err(fail(format!("Incorrect usage of \"cherry-pick\". Current branch ({})has no commits", self.current)));
        }
        let tags = match tags {
            Some(tags) => tags.into_iter().filter(|t| !t.is_empty()).collect(),
            None => vec![format!(
                "cherry-pick:{}{}",
                source.id,
                if source.kind == commit_type::MERGE { format!("|parent:{}", parent.as_deref().unwrap_or("")) } else { String::new() }
            )],
        };
        let id = self.next_id();
        let parents = match &self.head {
            None => Vec::new(),
            Some(head) => vec![head.clone(), source.id.clone()],
        };
        let commit = GitCommit {
            id,
            message: format!("cherry-picked {} into {}", source.message, self.current),
            seq: self.take_seq(),
            parents,
            branch: self.current.clone(),
            kind: commit_type::CHERRY_PICK,
            tags,
            custom_type: None,
            custom_id: None,
        };
        self.add(commit);
        Ok(())
    }
}

fn config_string<'a>(config: &'a Config, key: &str) -> Option<&'a str> {
    config.lookup("gitGraph", key).and_then(ConfigValue::as_str)
}

fn config_number(config: &Config, key: &str) -> Option<f64> {
    config.lookup("gitGraph", key).and_then(ConfigValue::as_f64)
}

/// The sign of `a - b`, as a sort comparator returns it. NaN (two equal infinities) is equal.
fn compare(a: f64, b: f64) -> Ordering {
    let d = a - b;
    if d < 0.0 {
        Ordering::Less
    } else if d > 0.0 {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}

pub fn build_git(ast: GitGraphAst, config: &Config, title: Option<&str>) -> Result<GitModel, Error> {
    let main_name = config_string(config, "mainBranchName").unwrap_or("main").to_string();
    let main_order = config_number(config, "mainBranchOrder").unwrap_or(0.0);

    let mut db = Db {
        commits: IndexMap::new(),
        branches: IndexMap::from([(main_name.clone(), None)]),
        orders: IndexMap::from([(main_name.clone(), Some(main_order))]),
        warnings: Vec::new(),
        head: None,
        current: main_name,
        direction: GitDirection::LR,
        seq: 0,
    };

    if let Some(dir) = ast.dir {
        db.direction = dir;
    }
    for statement in ast.statements {
        match statement {
            StatementAst::Commit { id, message, tags, kind } => {
                db.commit(message.unwrap_or_default(), id, kind.map_or(commit_type::NORMAL, type_code), tags);
            }
            StatementAst::Branch { name, order } => db.branch(name, Some(order.unwrap_or(0.0)))?,
            StatementAst::Merge { branch, id, tags, kind } => db.merge(branch, id.unwrap_or_default(), kind.map(type_code), tags)?,
            StatementAst::Checkout { branch } => db.checkout(&branch)?,
            StatementAst::CherryPicking { id, tags, parent } => {
                db.cherry_pick(id, if tags.is_empty() { None } else { Some(tags) }, parent)?
            }
        }
    }

    // A branch made without an order sorts by creation among those of order zero, as in Mermaid.
    let mut lanes: Vec<(String, f64)> = db
        .orders
        .iter()
        .enumerate()
        .map(|(i, (name, order))| (name.clone(), order.unwrap_or_else(|| js::parse_float(&format!("0.{}", i)))))
        .collect();
    lanes.sort_by(|a, b| compare(a.1, b.1));

    let head = db.head.as_ref().and_then(|id| db.commits.get(id)).cloned();
    Ok(GitModel {
        title: ast.title.filter(|t| !t.is_empty()).or_else(|| title.map(str::to_string)),
        acc_title: ast.acc_title.filter(|t| !t.is_empty()).map(|t| js::trim_start(&t).to_string()),
        acc_descr: ast.acc_descr.filter(|t| !t.is_empty()).map(|t| crate::diagrams::squeeze_line_starts(&t)),
        direction: db.direction,
        commits: db.commits,
        branches: db.branches,
        lanes: lanes.into_iter().map(|(name, _)| name).collect(),
        current_branch: db.current,
        head,
        warnings: db.warnings,
    })
}

pub(crate) fn parse(input: &ParseInput) -> Result<GitModel, Error> {
    build_git(parse_git(input.source)?, input.config, input.title)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_ids_are_stable() {
        let model = build_git(parse_git("gitGraph\n commit\n commit\n").unwrap(), &Config::new(), None).unwrap();
        let ids: Vec<&String> = model.commits.keys().collect();
        // 1 * 0x9e3779b1 and 2 * 0x9e3779b1 (mod 2^32), first seven hex digits.
        assert_eq!(ids, ["0-9e3779b", "1-3c6ef36"]);
    }
}
