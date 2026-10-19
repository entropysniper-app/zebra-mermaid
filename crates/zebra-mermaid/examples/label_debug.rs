use zebra_mermaid::text::{layout_label, Icons, Measure, MetricsMeasurer};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let icons = Icons::none();
    let m = Measure::new(&MetricsMeasurer, &icons);
    let md = args.get(2).is_some_and(|a| a == "md");
    let l = layout_label(Some(&args[1]), md, &m, 16.0, 4000.0, 0);
    for line in &l.lines {
        for s in line {
            println!("{:?} style={} w={:?}", s.text, s.style, s.width);
        }
        println!("--");
    }
    println!("{:?}", l.widths);
}
