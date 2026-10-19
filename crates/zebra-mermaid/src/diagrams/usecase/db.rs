//! Source-ordered symbol resolution, declaration conflicts and metadata application.
use super::{
    lexer::{line_column, location_text, Source},
    types::*,
};
use crate::{util::chars::normalize_accessibility_description, DiagramType, Error, ErrorCode};
use indexmap::{IndexMap, IndexSet};
#[derive(Clone)]
struct Origin {
    kind: String,
    span: Span,
    generated: bool,
    label: Option<String>,
}
#[derive(Clone)]
struct State {
    named: Named,
    kind: String,
    parent: Option<usize>,
    shape: Option<String>,
    stereotype: Option<String>,
    stereotype_span: Option<Span>,
    classes: Vec<String>,
    actor_type: Option<String>,
    icon: Option<String>,
    business: Option<bool>,
}
#[derive(Clone)]
struct BoundaryState {
    draft: DraftBoundary,
    kind: Option<String>,
    members: Vec<String>,
}
fn error(src: &Source, message: String, span: Span) -> Error {
    let (l, c) = line_column(src, span[0]);
    let mut e = Error::new(message, ErrorCode::Semantic).with_type(DiagramType::Usecase);
    e.line = l;
    e.column = c;
    e
}
fn fail(src: &Source, message: String, span: Span) -> Error {
    error(src, format!("{message} at {}", location_text(src, span[0], span[1])), span)
}
fn conflict(src: &Source, message: String, current: Span, previous: Span, cl: Option<&str>, pl: Option<&str>) -> Error {
    let suffix = |s: Option<&str>| s.map_or_else(String::new, |s| format!(" (label \"{s}\")"));
    error(
        src,
        format!(
            "{message} at {}{}; previous declaration at {}{}",
            location_text(src, current[0], current[1]),
            suffix(cl),
            location_text(src, previous[0], previous[1]),
            suffix(pl)
        ),
        current,
    )
}
fn invalid(src: &Source, id: &str, kind: &str, p: &Property) -> Error {
    fail(src, format!("Metadata property '{}' is invalid for {kind} '{id}'", p.occurrence.key), p.occurrence.key_span)
}
fn unique_push(target: &mut Vec<String>, values: &[String]) {
    let mut seen: IndexSet<_> = target.iter().cloned().collect();
    for s in values {
        if seen.insert(s.clone()) {
            target.push(s.clone());
        }
    }
}
fn claim(src: &Source, symbols: &IndexMap<String, Origin>, named: &Named, kind: &str) -> Result<(), Error> {
    let Some(origin) = symbols.get(&named.id) else {
        return Ok(());
    };
    let cl = if named.generated { Some(named.label.text.as_str()) } else { None };
    let pl = if origin.generated { origin.label.as_deref() } else { None };
    if origin.kind != kind {
        return Err(conflict(
            src,
            format!("ID '{}' is declared as both {} and {kind}", named.id, origin.kind),
            named.span,
            origin.span,
            cl,
            pl,
        ));
    }
    if origin.generated || named.generated {
        return Err(conflict(
            src,
            format!("Generated ID '{}' collides with another declaration", named.id),
            named.span,
            origin.span,
            cl,
            pl,
        ));
    }
    Ok(())
}
fn register(symbols: &mut IndexMap<String, Origin>, n: &Named, kind: &str) {
    symbols.insert(n.id.clone(), Origin { kind: kind.into(), span: n.span, generated: n.generated, label: Some(n.label.text.clone()) });
}
fn unique(src: &Source, symbols: &mut IndexMap<String, Origin>, id: &str, kind: &str, span: Span) -> Result<(), Error> {
    if let Some(previous) = symbols.get(id) {
        return Err(conflict(
            src,
            format!("ID '{id}' is declared more than once ({} and {kind})", previous.kind),
            span,
            previous.span,
            None,
            if previous.generated { previous.label.as_deref() } else { None },
        ));
    }
    symbols.insert(id.into(), Origin { kind: kind.into(), span, generated: false, label: None });
    Ok(())
}
fn metadata(src: &Source, state: &mut State, props: &[Property], replace: bool) -> Result<(), Error> {
    for p in props {
        let key = &p.occurrence.key;
        let value = &p.value;
        let id = &state.named.id;
        let old_span = state.named.span;
        let clashes = |same: bool| {
            if !replace && !same {
                Err(conflict(
                    src,
                    format!("{} '{id}' has conflicting {key} metadata", if state.kind == "actor" { "Actor" } else { "Use case" }),
                    p.occurrence.key_span,
                    old_span,
                    None,
                    None,
                ))
            } else {
                Ok(())
            }
        };
        if state.kind == "actor" {
            if key == "type" && matches!(value.as_str(), Some("normal" | "hollow" | "awesome")) {
                let v = value.as_str().unwrap();
                clashes(state.actor_type.as_deref().is_none_or(|old| old == v))?;
                state.actor_type = Some(v.into());
            } else if key == "icon" && value.as_str().is_some() {
                let v = value.as_str().unwrap();
                clashes(state.icon.as_deref().is_none_or(|old| old == v))?;
                state.icon = Some(v.into());
            } else if key == "business" && value.as_bool().is_some() {
                let v = value.as_bool().unwrap();
                clashes(state.business.is_none_or(|old| old == v))?;
                state.business = Some(v);
            } else {
                return Err(invalid(src, id, "actor", p));
            }
        } else if key == "business" && value.as_bool().is_some() {
            let v = value.as_bool().unwrap();
            clashes(state.business.is_none_or(|old| old == v))?;
            state.business = Some(v);
        } else {
            return Err(invalid(src, id, "usecase", p));
        }
    }
    Ok(())
}
fn infer_kind(props: &[Property]) -> Option<String> {
    let mut possible: IndexSet<&str> = ["actor", "usecase", "boundary", "edge"].into_iter().collect();
    for p in props {
        match p.occurrence.key.as_str() {
            "icon" => {
                possible.clear();
                possible.insert("actor");
            }
            "animate" | "animation" => {
                possible.clear();
                possible.insert("edge");
            }
            "type" => {
                possible.clear();
                match p.value.as_str() {
                    Some("rect" | "package") => {
                        possible.insert("boundary");
                    }
                    Some("normal" | "hollow" | "awesome") => {
                        possible.insert("actor");
                    }
                    _ => {}
                }
            }
            "business" => {
                possible.shift_remove("boundary");
                possible.shift_remove("edge");
            }
            _ => return None,
        }
    }
    if possible.len() == 1 {
        possible.first().map(|s| s.to_string())
    } else {
        None
    }
}
fn publish(model: &mut UsecaseModel, state: &State, boundaries: &[DraftBoundary]) {
    let id = &state.named.id;
    let actor = state.kind == "actor";
    let styles = if actor {
        model.use_cases.shift_remove(id);
        model.actors.get(id)
    } else {
        model.actors.shift_remove(id);
        model.use_cases.get(id)
    }
    .map_or_else(Vec::new, |e| e.styles.clone());
    let e = Element {
        id: id.clone(),
        label: state.named.label.text.clone(),
        label_type: state.named.label.kind.clone(),
        kind: if actor {
            if state.icon.as_ref().is_some_and(|s| !s.is_empty()) {
                "icon".into()
            } else {
                state.actor_type.clone().unwrap_or_else(|| "normal".into())
            }
        } else {
            String::new()
        },
        shape: if actor { String::new() } else { state.shape.clone().unwrap_or_else(|| "ellipse".into()) },
        icon: state.icon.clone().filter(|s| !s.is_empty()),
        business: state.business.unwrap_or(false),
        stereotype: state.stereotype.clone().filter(|s| !s.is_empty()),
        parent: state.parent.map(|i| boundaries[i].named.id.clone()).filter(|s| !s.is_empty()),
        classes: state.classes.clone(),
        styles,
    };
    if actor {
        model.actors.insert(id.clone(), e);
    } else {
        model.use_cases.insert(id.clone(), e);
    }
}
fn mention(first: &mut IndexMap<String, usize>, id: &str, offset: usize) {
    let old = first.entry(id.into()).or_insert(offset);
    *old = (*old).min(offset);
}
fn endpoint(
    src: &Source,
    end: &Endpoint,
    symbols: &mut IndexMap<String, Origin>,
    elements: &mut IndexMap<String, State>,
    first: &mut IndexMap<String, usize>,
) -> Result<String, Error> {
    let n = &end.named;
    if end.classed && !end.declares {
        return Err(fail(src, format!("Relationship endpoint '{}' uses ::: without declaring the node", n.id), n.span));
    }
    if let Some(origin) = symbols.get(&n.id) {
        return Ok(origin.kind.clone());
    }
    elements.insert(
        n.id.clone(),
        State {
            named: n.clone(),
            kind: "usecase".into(),
            shape: Some("ellipse".into()),
            parent: None,
            stereotype: None,
            stereotype_span: None,
            classes: vec![],
            actor_type: None,
            icon: None,
            business: None,
        },
    );
    register(symbols, n, "usecase");
    mention(first, &n.id, n.span[0]);
    Ok("usecase".into())
}
fn stylable<'a>(
    src: &Source,
    model: &'a mut UsecaseModel,
    symbols: &IndexMap<String, Origin>,
    edges: &IndexMap<String, usize>,
    id: &str,
    span: Span,
) -> Result<(&'a mut Vec<String>, &'a mut Vec<String>), Error> {
    let target = match symbols.get(id).map(|o| o.kind.as_str()) {
        Some("actor") => model.actors.get_mut(id).map(|n| (&mut n.classes, &mut n.styles)),
        Some("usecase") => model.use_cases.get_mut(id).map(|n| (&mut n.classes, &mut n.styles)),
        Some("boundary") => model.boundaries.get_mut(id).map(|n| (&mut n.classes, &mut n.styles)),
        Some("json") => model.json_nodes.get_mut(id).map(|n| (&mut n.classes, &mut n.styles)),
        _ => edges.get(id).copied().map(|i| {
            let n = &mut model.relationships[i];
            (&mut n.classes, &mut n.styles)
        }),
    };
    target.ok_or_else(|| fail(src, format!("Class/style target '{id}' is unresolved or anonymous"), span))
}
pub fn resolve(
    source: &str,
    src: &Source,
    drafts: Drafts,
    header: Span,
    mut statements: Vec<Statement>,
    roots: Vec<usize>,
    title: Option<&str>,
) -> Result<UsecaseModel, Error> {
    let mut model = UsecaseModel::default();
    let mut symbols = IndexMap::new();
    let mut elements = IndexMap::<String, State>::new();
    let mut boundaries = IndexMap::<String, BoundaryState>::new();
    let mut edges = IndexMap::new();
    let mut first = IndexMap::new();
    let mut declarations = vec![];
    for r in &drafts.relationships {
        mention(&mut first, &r.source.named.id, r.source.named.span[0]);
        mention(&mut first, &r.target.named.id, r.target.named.span[0]);
    }
    for (i, d) in drafts.elements.iter().enumerate() {
        mention(&mut first, &d.named.id, d.named.span[0]);
        declarations.push((d.named.span[0], 0, i));
    }
    for (i, d) in drafts.boundaries.iter().enumerate() {
        mention(&mut first, &d.named.id, d.named.span[0]);
        declarations.push((d.named.span[0], 1, i));
    }
    for (i, d) in drafts.jsons.iter().enumerate() {
        mention(&mut first, &d.id, d.span[0]);
        declarations.push((d.span[0], 2, i));
    }
    for (i, d) in drafts.relationships.iter().enumerate() {
        if d.id.as_ref().is_some_and(|s| !s.is_empty()) {
            if let Some(s) = d.id_span {
                declarations.push((s[0], 3, i));
            }
        }
    }
    declarations.sort_by_key(|d| d.0);
    for (_, kind, i) in declarations {
        match kind {
            0 => {
                let d = &drafts.elements[i];
                let n = &d.named;
                claim(src, &symbols, n, &d.kind)?;
                let parent = d.parent.filter(|&i| !drafts.boundaries[i].named.id.is_empty());
                if let Some(existing) = elements.get_mut(&n.id) {
                    if existing.named.label.text != n.label.text || existing.named.label.kind != n.label.kind {
                        return Err(conflict(
                            src,
                            format!("ID '{}' has conflicting labels", n.id),
                            n.span,
                            existing.named.span,
                            None,
                            None,
                        ));
                    }
                    if d.shape.is_some() && existing.shape.is_some() && d.shape != existing.shape {
                        return Err(conflict(
                            src,
                            format!("Use case '{}' has conflicting shapes", n.id),
                            n.span,
                            existing.named.span,
                            None,
                            None,
                        ));
                    }
                    if let (Some(p), Some(old)) = (parent, existing.parent) {
                        let now = &drafts.boundaries[p].named;
                        let old = &drafts.boundaries[old].named;
                        if !old.id.is_empty() && now.id != old.id {
                            return Err(conflict(
                                src,
                                format!("Element '{}' belongs to more than one system boundary", n.id),
                                now.span,
                                old.span,
                                None,
                                None,
                            ));
                        }
                    }
                    if d.stereotype.as_ref().is_some_and(|s| !s.is_empty())
                        && existing.stereotype.as_ref().is_some_and(|s| !s.is_empty())
                        && d.stereotype != existing.stereotype
                    {
                        return Err(conflict(
                            src,
                            format!("Element '{}' has conflicting stereotypes", n.id),
                            n.span,
                            existing.stereotype_span.unwrap_or(existing.named.span),
                            None,
                            None,
                        ));
                    }
                    if existing.shape.is_none() {
                        existing.shape = d.shape.clone();
                    }
                    if existing.parent.is_none() {
                        existing.parent = d.parent;
                    }
                    if existing.stereotype.is_none() {
                        existing.stereotype = d.stereotype.clone();
                    }
                    if existing.stereotype_span.is_none() && d.stereotype.as_ref().is_some_and(|s| !s.is_empty()) {
                        existing.stereotype_span = Some(n.span);
                    }
                    unique_push(&mut existing.classes, &d.classes);
                    metadata(src, existing, d.metadata.as_deref().unwrap_or(&[]), false)?;
                } else {
                    let mut state = State {
                        named: n.clone(),
                        kind: d.kind.clone(),
                        parent,
                        shape: d.shape.clone(),
                        stereotype: d.stereotype.clone(),
                        stereotype_span: if d.stereotype.as_ref().is_some_and(|s| !s.is_empty()) { Some(n.span) } else { None },
                        classes: d.classes.clone(),
                        actor_type: None,
                        icon: None,
                        business: None,
                    };
                    metadata(src, &mut state, d.metadata.as_deref().unwrap_or(&[]), false)?;
                    elements.insert(n.id.clone(), state);
                    register(&mut symbols, n, &d.kind);
                }
            }
            1 => {
                let d = &drafts.boundaries[i];
                let n = &d.named;
                claim(src, &symbols, n, "boundary")?;
                if let Some(existing) = boundaries.get_mut(&n.id) {
                    if existing.draft.named.label.text != n.label.text || existing.draft.named.label.kind != n.label.kind {
                        return Err(conflict(
                            src,
                            format!("Boundary '{}' has conflicting titles", n.id),
                            n.span,
                            existing.draft.named.span,
                            None,
                            None,
                        ));
                    }
                    unique_push(&mut existing.draft.classes, &d.classes);
                } else {
                    boundaries.insert(n.id.clone(), BoundaryState { draft: d.clone(), kind: None, members: vec![] });
                    register(&mut symbols, n, "boundary");
                }
            }
            2 => {
                let d = &drafts.jsons[i];
                unique(src, &mut symbols, &d.id, "json", d.span)?;
            }
            _ => {
                let d = &drafts.relationships[i];
                unique(src, &mut symbols, d.id.as_ref().unwrap(), "edge", d.id_span.unwrap())?;
            }
        }
    }
    let mut ordered: Vec<_> = elements.values().collect();
    ordered.sort_by_key(|e| first.get(&e.named.id).copied().unwrap_or(0));
    for state in ordered {
        publish(&mut model, state, &drafts.boundaries);
    }
    let mut ordered: Vec<_> = boundaries.values().collect();
    ordered.sort_by_key(|b| first.get(&b.draft.named.id).copied().unwrap_or(0));
    for state in ordered {
        let n = &state.draft.named;
        model.boundaries.insert(
            n.id.clone(),
            Boundary {
                id: n.id.clone(),
                label: n.label.text.clone(),
                label_type: n.label.kind.clone(),
                kind: "rect".into(),
                ..Default::default()
            },
        );
    }
    let mut ordered: Vec<_> = drafts.jsons.iter().collect();
    ordered.sort_by_key(|n| first.get(&n.id).copied().unwrap_or(0));
    for d in ordered {
        model
            .json_nodes
            .insert(d.id.clone(), JsonNode { id: d.id.clone(), value: d.value.clone(), classes: d.classes.clone(), styles: vec![] });
    }
    let mut anonymous = 0;
    for d in &drafts.relationships {
        let sk = endpoint(src, &d.source, &mut symbols, &mut elements, &mut first)?;
        let tk = endpoint(src, &d.target, &mut symbols, &mut elements, &mut first)?;
        let sn = &d.source.named;
        let tn = &d.target.named;
        for (n, kind, which) in [(sn, sk.as_str(), "source"), (tn, tk.as_str(), "target")] {
            if !matches!(kind, "actor" | "usecase" | "json") {
                return Err(conflict(
                    src,
                    format!("Relationship {which} '{}' cannot be {kind}", n.id),
                    n.span,
                    symbols[&n.id].span,
                    None,
                    None,
                ));
            }
        }
        if matches!(d.kind.as_str(), "include" | "extend") && (sk != "usecase" || tk != "usecase") {
            return Err(fail(src, format!("{} relationship requires use-case endpoints", d.kind), d.span));
        }
        if d.kind == "generalization" && (sk == "json" || sk != tk) {
            return Err(fail(src, "Generalization requires actor-to-actor or use-case-to-use-case endpoints".into(), d.span));
        }
        if d.kind == "association" && (sk == "json" || tk == "json") && d.arrow_type > 2 {
            return Err(fail(
                src,
                format!("JSON relationship '{}' to '{}' permits only point, reversed-point, or markerless solid association", sn.id, tn.id),
                d.span,
            ));
        }
        let id = d.id.clone().unwrap_or_else(|| {
            let s = format!("edge-{anonymous}");
            anonymous += 1;
            s
        });
        edges.insert(id.clone(), model.relationships.len());
        model.relationships.push(Relationship {
            id,
            explicit_id: d.id.as_ref().is_some_and(|s| !s.is_empty()),
            source: sn.id.clone(),
            target: tn.id.clone(),
            kind: d.kind.clone(),
            arrow_type: d.arrow_type,
            label: d.label.as_ref().map(|l| l.text.clone()),
            label_type: d.label.as_ref().map(|l| l.kind.clone()),
            minlen: d.minlen,
            classes: vec![],
            styles: vec![],
            animate: false,
            animation: None,
        });
    }
    let old_actors = std::mem::take(&mut model.actors);
    let old_cases = std::mem::take(&mut model.use_cases);
    let mut ordered: Vec<_> = elements.values().collect();
    ordered.sort_by_key(|e| first.get(&e.named.id).copied().unwrap_or(0));
    for state in ordered {
        if let Some(actor) = old_actors.get(&state.named.id) {
            model.actors.insert(state.named.id.clone(), actor.clone());
        } else if let Some(case) = old_cases.get(&state.named.id) {
            model.use_cases.insert(state.named.id.clone(), case.clone());
        } else {
            publish(&mut model, state, &drafts.boundaries);
        }
    }
    for d in &drafts.metadata {
        let Some(origin) = symbols.get(&d.target) else {
            return Err(fail(
                src,
                format!(
                    "Metadata target '{}' is unresolved{}",
                    d.target,
                    infer_kind(&d.properties).map_or_else(String::new, |k| format!(" (metadata implies {k})"))
                ),
                d.span,
            ));
        };
        let kind = origin.kind.as_str();
        if matches!(kind, "actor" | "usecase") {
            metadata(src, elements.get_mut(&d.target).unwrap(), &d.properties, true)?;
            continue;
        }
        if kind == "edge" {
            let stmt = &mut statements[d.statement];
            stmt.kind = "edgeMetadata".into();
            stmt.edges = Some(vec![EdgeOccurrence {
                id: d.target.clone(),
                span: stmt.span,
                id_span: Some(d.span),
                metadata: stmt.metadata.clone(),
                ..Default::default()
            }]);
            stmt.nodes = None;
        }
        for p in &d.properties {
            let key = &p.occurrence.key;
            let value = &p.value;
            if kind == "boundary"
                && boundaries.contains_key(&d.target)
                && key == "type"
                && matches!(value.as_str(), Some("rect" | "package"))
            {
                boundaries.get_mut(&d.target).unwrap().kind = Some(value.as_str().unwrap().into());
            } else if kind == "edge" && edges.contains_key(&d.target) && key == "animate" && value.as_bool().is_some() {
                model.relationships[edges[&d.target]].animate = value.as_bool().unwrap();
            } else if kind == "edge"
                && edges.contains_key(&d.target)
                && key == "animation"
                && matches!(value.as_str(), Some("fast" | "slow"))
            {
                let e = &mut model.relationships[edges[&d.target]];
                e.animation = Some(value.as_str().unwrap().into());
                e.animate = true;
            } else {
                return Err(invalid(src, &d.target, kind, p));
            }
        }
    }
    for r in &mut model.relationships {
        if r.animation.as_ref().is_some_and(|s| !s.is_empty()) {
            r.animate = true;
        }
    }
    for state in elements.values() {
        let id = &state.named.id;
        let icon = state.icon.as_ref().is_some_and(|s| !s.is_empty());
        if state.kind == "actor" {
            if icon && state.actor_type.as_ref().is_some_and(|s| !s.is_empty() && s != "normal") {
                return Err(fail(
                    src,
                    format!("Actor '{id}' cannot combine icon with type '{}'", state.actor_type.as_ref().unwrap()),
                    state.named.span,
                ));
            }
            if state.business == Some(true) && (icon || state.actor_type.as_deref() == Some("awesome")) {
                return Err(fail(src, format!("Business actor '{id}' must use normal or hollow geometry"), state.named.span));
            }
        } else if state.shape.as_deref() == Some("rect") && state.business == Some(true) {
            return Err(fail(src, format!("Rectangular use case '{id}' cannot be a business use case"), state.named.span));
        }
        publish(&mut model, state, &drafts.boundaries);
    }
    let mut ordered: Vec<_> = drafts.elements.iter().collect();
    ordered.sort_by_key(|d| d.named.span[0]);
    for d in ordered {
        if let Some(parent) = d.parent {
            let id = &drafts.boundaries[parent].named.id;
            if !id.is_empty() {
                let members = &mut boundaries.get_mut(id).unwrap().members;
                if !members.contains(&d.named.id) {
                    members.push(d.named.id.clone());
                }
            }
        }
    }
    for state in boundaries.values() {
        let b = model.boundaries.get_mut(&state.draft.named.id).unwrap();
        b.kind = state.kind.clone().unwrap_or_else(|| "rect".into());
        b.members = state.members.clone();
        b.classes = state.draft.classes.clone();
    }
    for (draft, i) in drafts.notes.iter().zip(0..) {
        let Some(origin) = symbols.get(&draft.target) else {
            return Err(fail(src, format!("Note target '{}' is unresolved", draft.target), draft.span));
        };
        if !matches!(origin.kind.as_str(), "actor" | "usecase") {
            return Err(conflict(
                src,
                format!("Note target '{}' must be an actor or use case, not {}", draft.target, origin.kind),
                draft.span,
                origin.span,
                None,
                None,
            ));
        }
        let id = format!("note-{i}");
        model.notes.insert(
            id.clone(),
            Note { id, target: draft.target.clone(), label: draft.label.text.clone(), label_type: draft.label.kind.clone() },
        );
    }
    for (ids, styles) in &drafts.class_defs {
        for id in ids {
            model.class_defs.insert(id.clone(), styles.clone());
        }
    }
    for assignment in &drafts.classes {
        for (id, span) in &assignment.targets {
            let (classes, _) = stylable(src, &mut model, &symbols, &edges, id, *span)?;
            unique_push(classes, &assignment.classes);
        }
    }
    for a in &drafts.styles {
        let (_, styles) = stylable(src, &mut model, &symbols, &edges, &a.target, a.span)?;
        styles.extend(a.styles.iter().cloned());
    }
    model.direction = drafts.direction.unwrap_or(model.direction);
    model.title = title.map(str::to_owned);
    model.acc_title = drafts.acc_title;
    model.acc_description = drafts.acc_description;
    model.acc_descr = normalize_accessibility_description(&model.acc_description);
    model.source = source.into();
    model.header_span = header;
    model.statements = statements;
    model.roots = roots;
    Ok(model)
}
