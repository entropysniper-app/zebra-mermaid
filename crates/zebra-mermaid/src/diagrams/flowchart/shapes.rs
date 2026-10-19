//! Shape aliases ported from Pele flowchart/shapes.ts.
pub fn valid(name: &str) -> bool {
    matches!(
        name,
        "rect"
            | "proc"
            | "process"
            | "rectangle"
            | "squareRect"
            | "rounded"
            | "event"
            | "roundedRect"
            | "stadium"
            | "terminal"
            | "pill"
            | "fr-rect"
            | "subprocess"
            | "subproc"
            | "framed-rectangle"
            | "subroutine"
            | "cyl"
            | "db"
            | "database"
            | "cylinder"
            | "datastore"
            | "data-store"
            | "folder"
            | "directory"
            | "bucket"
            | "console"
            | "browser"
            | "person"
            | "circle"
            | "circ"
            | "bang"
            | "cloud"
            | "diam"
            | "decision"
            | "diamond"
            | "question"
            | "hex"
            | "hexagon"
            | "prepare"
            | "lean-r"
            | "lean-right"
            | "in-out"
            | "lean_right"
            | "lean-l"
            | "lean-left"
            | "out-in"
            | "lean_left"
            | "trap-b"
            | "priority"
            | "trapezoid-bottom"
            | "trapezoid"
            | "trap-t"
            | "manual"
            | "trapezoid-top"
            | "inv-trapezoid"
            | "inv_trapezoid"
            | "dbl-circ"
            | "double-circle"
            | "doublecircle"
            | "text"
            | "notch-rect"
            | "card"
            | "notched-rectangle"
            | "lin-rect"
            | "lined-rectangle"
            | "lined-process"
            | "lin-proc"
            | "shaded-process"
            | "sm-circ"
            | "start"
            | "small-circle"
            | "stateStart"
            | "fr-circ"
            | "stop"
            | "framed-circle"
            | "stateEnd"
            | "fork"
            | "join"
            | "forkJoin"
            | "hourglass"
            | "collate"
            | "brace"
            | "comment"
            | "brace-l"
            | "brace-r"
            | "braces"
            | "bolt"
            | "com-link"
            | "lightning-bolt"
            | "doc"
            | "document"
            | "delay"
            | "half-rounded-rectangle"
            | "h-cyl"
            | "das"
            | "horizontal-cylinder"
            | "lin-cyl"
            | "disk"
            | "lined-cylinder"
            | "curv-trap"
            | "curved-trapezoid"
            | "display"
            | "div-rect"
            | "div-proc"
            | "divided-rectangle"
            | "divided-process"
            | "tri"
            | "extract"
            | "triangle"
            | "win-pane"
            | "internal-storage"
            | "window-pane"
            | "f-circ"
            | "junction"
            | "filled-circle"
            | "notch-pent"
            | "loop-limit"
            | "notched-pentagon"
            | "flip-tri"
            | "manual-file"
            | "flipped-triangle"
            | "sl-rect"
            | "manual-input"
            | "sloped-rectangle"
            | "docs"
            | "documents"
            | "st-doc"
            | "stacked-document"
            | "st-rect"
            | "procs"
            | "processes"
            | "stacked-rectangle"
            | "bow-rect"
            | "stored-data"
            | "bow-tie-rectangle"
            | "cross-circ"
            | "summary"
            | "crossed-circle"
            | "tag-doc"
            | "tagged-document"
            | "tag-rect"
            | "tagged-rectangle"
            | "tag-proc"
            | "tagged-process"
            | "flag"
            | "paper-tape"
            | "odd"
            | "rect_left_inv_arrow"
            | "lin-doc"
            | "lined-document"
            | "state"
            | "choice"
            | "note"
            | "composite"
            | "rectWithTitle"
            | "labelRect"
            | "block_arrow"
            | "collapsedGroup"
            | "iconSquare"
            | "iconCircle"
            | "icon"
            | "iconRounded"
            | "imageSquare"
            | "anchor"
            | "kanbanItem"
            | "mindmapCircle"
            | "defaultMindmapNode"
            | "classBox"
            | "erBox"
            | "requirementBox"
            | "usecaseActor"
            | "usecaseActorHollow"
            | "usecaseActorAwesome"
            | "usecaseActorIcon"
            | "usecaseBusiness"
            | "usecaseEllipse"
            | "usecaseJsonTable"
    )
}
pub fn canonical(name: &str) -> &str {
    match name {
        "square" => "rect",
        "round" => "rounded",
        "ellipse" => "ellipse",
        "rect" | "proc" | "process" | "rectangle" | "squareRect" => "rect",
        "rounded" | "event" | "roundedRect" => "rounded",
        "stadium" | "terminal" | "pill" => "stadium",
        "fr-rect" | "subprocess" | "subproc" | "framed-rectangle" | "subroutine" => "fr-rect",
        "cyl" | "db" | "database" | "cylinder" => "cyl",
        "datastore" | "data-store" => "datastore",
        "folder" | "directory" => "folder",
        "bucket" => "bucket",
        "console" => "console",
        "browser" => "browser",
        "person" => "person",
        "circle" | "circ" => "circle",
        "bang" => "bang",
        "cloud" => "cloud",
        "diam" | "decision" | "diamond" | "question" => "diam",
        "hex" | "hexagon" | "prepare" => "hex",
        "lean-r" | "lean-right" | "in-out" | "lean_right" => "lean-r",
        "lean-l" | "lean-left" | "out-in" | "lean_left" => "lean-l",
        "trap-b" | "priority" | "trapezoid-bottom" | "trapezoid" => "trap-b",
        "trap-t" | "manual" | "trapezoid-top" | "inv-trapezoid" | "inv_trapezoid" => "trap-t",
        "dbl-circ" | "double-circle" | "doublecircle" => "dbl-circ",
        "text" => "text",
        "notch-rect" | "card" | "notched-rectangle" => "notch-rect",
        "lin-rect" | "lined-rectangle" | "lined-process" | "lin-proc" | "shaded-process" => "lin-rect",
        "sm-circ" | "start" | "small-circle" | "stateStart" => "sm-circ",
        "fr-circ" | "stop" | "framed-circle" | "stateEnd" => "fr-circ",
        "fork" | "join" | "forkJoin" => "fork",
        "hourglass" | "collate" => "hourglass",
        "brace" | "comment" | "brace-l" => "brace",
        "brace-r" => "brace-r",
        "braces" => "braces",
        "bolt" | "com-link" | "lightning-bolt" => "bolt",
        "doc" | "document" => "doc",
        "delay" | "half-rounded-rectangle" => "delay",
        "h-cyl" | "das" | "horizontal-cylinder" => "h-cyl",
        "lin-cyl" | "disk" | "lined-cylinder" => "lin-cyl",
        "curv-trap" | "curved-trapezoid" | "display" => "curv-trap",
        "div-rect" | "div-proc" | "divided-rectangle" | "divided-process" => "div-rect",
        "tri" | "extract" | "triangle" => "tri",
        "win-pane" | "internal-storage" | "window-pane" => "win-pane",
        "f-circ" | "junction" | "filled-circle" => "f-circ",
        "notch-pent" | "loop-limit" | "notched-pentagon" => "notch-pent",
        "flip-tri" | "manual-file" | "flipped-triangle" => "flip-tri",
        "sl-rect" | "manual-input" | "sloped-rectangle" => "sl-rect",
        "docs" | "documents" | "st-doc" | "stacked-document" => "docs",
        "st-rect" | "procs" | "processes" | "stacked-rectangle" => "st-rect",
        "bow-rect" | "stored-data" | "bow-tie-rectangle" => "bow-rect",
        "cross-circ" | "summary" | "crossed-circle" => "cross-circ",
        "tag-doc" | "tagged-document" => "tag-doc",
        "tag-rect" | "tagged-rectangle" | "tag-proc" | "tagged-process" => "tag-rect",
        "flag" | "paper-tape" => "flag",
        "odd" | "rect_left_inv_arrow" => "odd",
        "lin-doc" | "lined-document" => "lin-doc",
        "state"
        | "choice"
        | "note"
        | "composite"
        | "rectWithTitle"
        | "labelRect"
        | "block_arrow"
        | "collapsedGroup"
        | "iconSquare"
        | "iconCircle"
        | "icon"
        | "iconRounded"
        | "imageSquare"
        | "anchor"
        | "kanbanItem"
        | "mindmapCircle"
        | "defaultMindmapNode"
        | "classBox"
        | "erBox"
        | "requirementBox"
        | "usecaseActor"
        | "usecaseActorHollow"
        | "usecaseActorAwesome"
        | "usecaseActorIcon"
        | "usecaseBusiness"
        | "usecaseEllipse"
        | "usecaseJsonTable" => name,
        _ => "rect",
    }
}
