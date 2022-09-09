use serde::Serialize;

// https://hackage.haskell.org/package/pandoc-types-1.22.2.1/docs/Text-Pandoc-Definition.html

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Pandoc {
    pub pandoc_api_version: [u64; 4],
    pub meta: Meta,
    pub blocks: Vec<Block>,
}

#[derive(Debug, Serialize)]
pub struct Meta {}

#[derive(Debug, Serialize)]
#[serde(tag = "t", content = "c")]
pub enum Block {
    Plain(Vec<Inline>),
    Para(Vec<Inline>),
    // LineBlock(Vec<Vec<Inline>>),
    CodeBlock(Attr, String),
    // RawBlock(Format, Text),
    BlockQuote(Vec<Block>),
    OrderedList(ListAttributes, Vec<Vec<Block>>),
    BulletList(Vec<Vec<Block>>),
    // DefinitionList(Vec<(Vec<Inline>, Vec<Vec<Block>>)>),
    Header(u64, Attr, Vec<Inline>),
    HorizontalRule,
    Table(
        Attr,
        Caption,
        Vec<ColSpec>,
        TableHead,
        Vec<TableBody>,
        TableFoot,
    ),
    // Div(Attr, Vec<Block>),
    Null,
}

#[derive(Debug, Serialize)]
#[serde(tag = "t", content = "c")]
pub enum Inline {
    Str(String),
    Emph(Vec<Inline>),
    Strong(Vec<Inline>),
    Strikeout(Vec<Inline>),
    // Superscript(Vec<Inline>),
    // Subscript(Vec<Inline>),
    // SmallCaps(Vec<Inline>),
    // Quoted(QuoteType, Vec<Inline>),
    // Cite(Vec<Citation>, Vec<Inline>),
    Code(Attr, String),
    Space,
    // SoftBreak,
    // LineBreak,
    Math(MathType, String),
    // RawInline(Format, String),
    Link(Attr, Vec<Inline>, Target),
    Image(Attr, Vec<Inline>, Target),
    // Note(Vec<Block>),
    // Span(Attr, Vec<Inline>),
}

impl Inline {
    pub fn to_link(self, url: String) -> Self {
        Self::Link(Attr::default(), vec![self], Target(url, String::new()))
    }
}

#[derive(Debug, Default, Serialize)]
#[serde(tag = "t", content = "c")]
pub enum Alignment {
    // AlignLeft,
    // AlignRight,
    // AlignCenter,
    #[default]
    AlignDefault,
}

#[derive(Debug, Serialize)]
pub struct ListAttributes(pub u64, pub ListNumberStyle, pub ListNumberDelim);

#[derive(Debug, Default, Serialize)]
#[serde(tag = "t", content = "c")]
pub enum ListNumberStyle {
    #[default]
    DefaultStyle,
    // Example,
    Decimal,
    // LowerRoman,
    // UpperRoman,
    // LowerAlpha,
    // UpperAlpha,
}

#[derive(Debug, Default, Serialize)]
#[serde(tag = "t", content = "c")]
pub enum ListNumberDelim {
    #[default]
    DefaultDelim,
    Period,
    // OneParen,
    // TwoParens,
}

#[derive(Debug, Serialize)]
pub struct Format(pub String);

#[derive(Debug, Serialize, Default)]
pub struct Attr(pub String, pub Vec<String>, pub Vec<(String, String)>);

#[derive(Debug, Serialize)]
pub struct Target(pub String, pub String);

#[derive(Debug, Serialize)]
pub struct TableCell(pub Vec<Block>);

#[derive(Debug, Default, Serialize)]
#[serde(tag = "t", content = "c")]
pub enum MathType {
    #[default]
    DisplayMath,
    InlineMath,
}

// structs for Table

#[derive(Debug, Default, Serialize)]
pub struct Caption(pub Option<ShortCaption>, pub Vec<Block>);

#[derive(Debug, Default, Serialize)]
pub struct ShortCaption(pub Vec<Inline>);

#[derive(Debug, Default, Serialize)]
pub struct Row(pub Attr, pub Vec<Cell>);

#[derive(Debug, Default, Serialize)]
pub struct ColSpec(pub Alignment, pub ColWidth);

#[derive(Debug, Default, Serialize)]
pub struct RowHeadColumns(pub u64);

#[derive(Debug, Default, Serialize)]
pub struct TableHead(pub Attr, pub Vec<Row>);

#[derive(Debug, Default, Serialize)]
pub struct TableBody(pub Attr, pub RowHeadColumns, pub Vec<Row>, pub Vec<Row>);

#[derive(Debug, Default, Serialize)]
pub struct TableFoot(pub Attr, pub Vec<Row>);

#[derive(Debug, Default, Serialize)]
#[serde(tag = "t", content = "c")]
pub enum ColWidth {
    // ColWidth(f64),
    #[default]
    ColWidthDefault,
}

#[derive(Debug, Serialize)]
pub struct Cell(
    pub Attr,
    pub Alignment,
    pub RowSpan,
    pub ColSpan,
    pub Vec<Block>,
);

#[derive(Debug, Serialize)]
pub struct RowSpan(pub u64);

#[derive(Debug, Serialize)]
pub struct ColSpan(pub u64);
