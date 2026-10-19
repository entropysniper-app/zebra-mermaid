//! Requirements and elements with ordered styles and relations.
mod lexer;
mod parser;
mod render;
mod tokens;
use super::ParseInput;
use crate::{DiagramType, Error};
use indexmap::IndexMap;
pub(crate) use render::render;
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct RequirementNode {
    pub name: String,
    pub kind: String,
    pub id: String,
    pub text: String,
    pub risk: String,
    pub verify: String,
    pub doc_ref: String,
    pub styles: Vec<String>,
    pub classes: Vec<String>,
}
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Relation {
    pub kind: String,
    pub src: String,
    pub dst: String,
}
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct RequirementModel {
    pub direction: String,
    pub requirements: IndexMap<String, RequirementNode>,
    pub elements: IndexMap<String, RequirementNode>,
    pub relations: Vec<Relation>,
    pub classes: IndexMap<String, Vec<String>>,
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
    style_copies: usize,
}
impl Default for RequirementModel {
    fn default() -> Self {
        Self {
            direction: "TB".into(),
            requirements: IndexMap::new(),
            elements: IndexMap::new(),
            relations: vec![],
            classes: IndexMap::new(),
            title: None,
            acc_title: None,
            acc_descr: None,
            style_copies: 0,
        }
    }
}
impl RequirementModel {
    fn copy(counter: &mut usize, target: &mut Vec<String>, styles: &[String]) -> Result<(), Error> {
        *counter += styles.len();
        if *counter > 1_000_000 {
            return Err(Error::limit("Requirement diagram applies too many style declarations.").with_type(DiagramType::Requirement));
        }
        target.extend(styles.iter().cloned());
        Ok(())
    }
    fn set_style(&mut self, ids: &[String], styles: &[String]) -> Result<(), Error> {
        for id in ids {
            let node = self.requirements.get_mut(id).or_else(|| self.elements.get_mut(id));
            let Some(node) = node else {
                return Ok(());
            };
            for s in styles {
                Self::copy(&mut self.style_copies, &mut node.styles, &s.split(',').map(str::to_owned).collect::<Vec<_>>())?;
            }
        }
        Ok(())
    }
    fn set_class(&mut self, ids: &[String], classes: &[String]) -> Result<(), Error> {
        for id in ids {
            let node = self.requirements.get_mut(id).or_else(|| self.elements.get_mut(id));
            let Some(node) = node else {
                continue;
            };
            for class in classes {
                node.classes.push(class.clone());
                if let Some(styles) = self.classes.get(class) {
                    Self::copy(&mut self.style_copies, &mut node.styles, styles)?;
                }
            }
        }
        Ok(())
    }
    fn define_class(&mut self, ids: &[String], styles: &[String]) -> Result<(), Error> {
        for id in ids {
            self.classes.entry(id.clone()).or_default().extend(styles.iter().cloned());
            let split: Vec<_> = styles.iter().flat_map(|s| s.split(',').map(str::to_owned)).collect();
            for node in self.requirements.values_mut().chain(self.elements.values_mut()) {
                if node.classes.contains(id) {
                    Self::copy(&mut self.style_copies, &mut node.styles, &split)?;
                }
            }
        }
        Ok(())
    }
}
pub(crate) fn parse(input: &ParseInput) -> Result<RequirementModel, Error> {
    let mut m = RequirementModel { title: input.title.map(str::to_owned), ..Default::default() };
    parser::parse(input.source, &mut m)?;
    Ok(m)
}
