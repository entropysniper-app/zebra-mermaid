//! Token kinds of the sequence lexer. Ported from Pele `src/diagrams/sequence/tokens.ts`.

pub(super) const END: u8 = 0;
pub(super) const ERROR: u8 = 1;
pub(super) const INVALID: u8 = 2;
pub(super) const NEWLINE: u8 = 3;
pub(super) const SD: u8 = 4;
pub(super) const CREATE: u8 = 5;
pub(super) const BOX: u8 = 6;
pub(super) const REST_OF_LINE: u8 = 7;
pub(super) const KW_END: u8 = 8;
pub(super) const AUTONUMBER: u8 = 9;
pub(super) const NUM: u8 = 10;
pub(super) const OFF: u8 = 11;
pub(super) const ACTIVATE: u8 = 12;
pub(super) const DEACTIVATE: u8 = 13;
pub(super) const TITLE: u8 = 14;
pub(super) const LEGACY_TITLE: u8 = 15;
pub(super) const ACC_TITLE: u8 = 16;
pub(super) const ACC_TITLE_VALUE: u8 = 17;
pub(super) const ACC_DESCR: u8 = 18;
pub(super) const ACC_DESCR_VALUE: u8 = 19;
pub(super) const ACC_DESCR_MULTILINE_VALUE: u8 = 20;
pub(super) const LOOP: u8 = 21;
pub(super) const RECT: u8 = 22;
pub(super) const OPT: u8 = 23;
pub(super) const ALT: u8 = 24;
pub(super) const PAR: u8 = 25;
pub(super) const PAR_OVER: u8 = 26;
pub(super) const CRITICAL: u8 = 27;
pub(super) const BREAK: u8 = 28;
pub(super) const OPTION: u8 = 29;
pub(super) const AND: u8 = 30;
pub(super) const ELSE: u8 = 31;
pub(super) const PARTICIPANT: u8 = 32;
pub(super) const AS: u8 = 33;
pub(super) const PARTICIPANT_ACTOR: u8 = 34;
pub(super) const DESTROY: u8 = 35;
pub(super) const NOTE: u8 = 36;
pub(super) const OVER: u8 = 37;
pub(super) const LINKS: u8 = 38;
pub(super) const LINK: u8 = 39;
pub(super) const PROPERTIES: u8 = 40;
pub(super) const DETAILS: u8 = 41;
pub(super) const COMMA: u8 = 42;
pub(super) const LEFT_OF: u8 = 43;
pub(super) const RIGHT_OF: u8 = 44;
pub(super) const PLUS: u8 = 45;
pub(super) const MINUS: u8 = 46;
pub(super) const CENTRAL: u8 = 47;
pub(super) const ACTOR: u8 = 48;
pub(super) const CONFIG_START: u8 = 49;
pub(super) const CONFIG_CONTENT: u8 = 50;
pub(super) const CONFIG_END: u8 = 51;
pub(super) const TXT: u8 = 52;
pub(super) const SOLID_OPEN_ARROW: u8 = 53;
pub(super) const DOTTED_OPEN_ARROW: u8 = 54;
pub(super) const SOLID_ARROW: u8 = 55;
pub(super) const BIDIRECTIONAL_SOLID_ARROW: u8 = 56;
pub(super) const DOTTED_ARROW: u8 = 57;
pub(super) const BIDIRECTIONAL_DOTTED_ARROW: u8 = 58;
pub(super) const SOLID_CROSS: u8 = 59;
pub(super) const DOTTED_CROSS: u8 = 60;
pub(super) const SOLID_POINT: u8 = 61;
pub(super) const DOTTED_POINT: u8 = 62;
pub(super) const SOLID_ARROW_TOP: u8 = 63;
pub(super) const SOLID_ARROW_BOTTOM: u8 = 64;
pub(super) const STICK_ARROW_TOP: u8 = 65;
pub(super) const STICK_ARROW_BOTTOM: u8 = 66;
pub(super) const SOLID_ARROW_TOP_DOTTED: u8 = 67;
pub(super) const SOLID_ARROW_BOTTOM_DOTTED: u8 = 68;
pub(super) const STICK_ARROW_TOP_DOTTED: u8 = 69;
pub(super) const STICK_ARROW_BOTTOM_DOTTED: u8 = 70;
pub(super) const SOLID_ARROW_TOP_REVERSE: u8 = 71;
pub(super) const SOLID_ARROW_BOTTOM_REVERSE: u8 = 72;
pub(super) const STICK_ARROW_TOP_REVERSE: u8 = 73;
pub(super) const STICK_ARROW_BOTTOM_REVERSE: u8 = 74;
pub(super) const SOLID_ARROW_TOP_REVERSE_DOTTED: u8 = 75;
pub(super) const SOLID_ARROW_BOTTOM_REVERSE_DOTTED: u8 = 76;
pub(super) const STICK_ARROW_TOP_REVERSE_DOTTED: u8 = 77;
pub(super) const STICK_ARROW_BOTTOM_REVERSE_DOTTED: u8 = 78;

/// Names as Mermaid's grammar spells them, for error messages.
pub(super) const TOKEN_NAMES: [&str; 79] = [
    "$end",
    "INVALID",
    "INVALID",
    "NEWLINE",
    "SD",
    "create",
    "box",
    "restOfLine",
    "end",
    "autonumber",
    "NUM",
    "off",
    "activate",
    "deactivate",
    "title",
    "legacy_title",
    "acc_title",
    "acc_title_value",
    "acc_descr",
    "acc_descr_value",
    "acc_descr_multiline_value",
    "loop",
    "rect",
    "opt",
    "alt",
    "par",
    "par_over",
    "critical",
    "break",
    "option",
    "and",
    "else",
    "participant",
    "AS",
    "participant_actor",
    "destroy",
    "note",
    "over",
    "links",
    "link",
    "properties",
    "details",
    ",",
    "left_of",
    "right_of",
    "+",
    "-",
    "()",
    "ACTOR",
    "CONFIG_START",
    "CONFIG_CONTENT",
    "CONFIG_END",
    "TXT",
    "SOLID_OPEN_ARROW",
    "DOTTED_OPEN_ARROW",
    "SOLID_ARROW",
    "BIDIRECTIONAL_SOLID_ARROW",
    "DOTTED_ARROW",
    "BIDIRECTIONAL_DOTTED_ARROW",
    "SOLID_CROSS",
    "DOTTED_CROSS",
    "SOLID_POINT",
    "DOTTED_POINT",
    "SOLID_ARROW_TOP",
    "SOLID_ARROW_BOTTOM",
    "STICK_ARROW_TOP",
    "STICK_ARROW_BOTTOM",
    "SOLID_ARROW_TOP_DOTTED",
    "SOLID_ARROW_BOTTOM_DOTTED",
    "STICK_ARROW_TOP_DOTTED",
    "STICK_ARROW_BOTTOM_DOTTED",
    "SOLID_ARROW_TOP_REVERSE",
    "SOLID_ARROW_BOTTOM_REVERSE",
    "STICK_ARROW_TOP_REVERSE",
    "STICK_ARROW_BOTTOM_REVERSE",
    "SOLID_ARROW_TOP_REVERSE_DOTTED",
    "SOLID_ARROW_BOTTOM_REVERSE_DOTTED",
    "STICK_ARROW_TOP_REVERSE_DOTTED",
    "STICK_ARROW_BOTTOM_REVERSE_DOTTED",
];
