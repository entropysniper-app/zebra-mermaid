use crate::svg::{
    builder::num,
    edges::{marker, marker_trim},
};
pub fn trim(kind: &str) -> f64 {
    match kind {
        "extension" => 12.0,
        "composition" | "aggregation" => 16.0,
        "lollipop" => 10.0,
        "dependency" => marker_trim("arrow_point"),
        _ => 0.0,
    }
}
pub fn draw(kind: &str, x: f64, y: f64, dx: f64, dy: f64, color: &str) -> String {
    let nx = -dy;
    let ny = dx;
    match kind {
        "extension" => {
            let bx = x - dx * 12.0;
            let by = y - dy * 12.0;
            format!(
                "<path class=\"pele-marker\" d=\"M{},{}L{},{}L{},{}Z\" stroke-linejoin=\"round\"/>",
                num(x),
                num(y),
                num(bx + nx * 6.0),
                num(by + ny * 6.0),
                num(bx - nx * 6.0),
                num(by - ny * 6.0)
            )
        }
        "composition" | "aggregation" => {
            let mx = x - dx * 16.0 / 2.0;
            let my = y - dy * 16.0 / 2.0;
            format!(
                "<path class=\"pele-marker\" d=\"M{},{}L{},{}L{},{}L{},{}Z\" stroke-linejoin=\"round\"{}/>",
                num(x),
                num(y),
                num(mx + nx * 5.0),
                num(my + ny * 5.0),
                num(x - dx * 16.0),
                num(y - dy * 16.0),
                num(mx - nx * 5.0),
                num(my - ny * 5.0),
                if kind == "composition" { format!(" fill=\"{color}\"") } else { String::new() }
            )
        }
        "lollipop" => format!("<circle class=\"pele-marker\" cx=\"{}\" cy=\"{}\" r=\"5\"/>", num(x - dx * 5.0), num(y - dy * 5.0)),
        "dependency" => marker("arrow_point", x, y, dx, dy, color),
        _ => String::new(),
    }
}
