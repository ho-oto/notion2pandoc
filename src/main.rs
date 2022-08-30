use chrono::{DateTime, Local};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let page_id = "XXX";
    let secret = "secret_YYY";

    let mut has_more = true;
    let mut next_cursor = None;
    let mut blocks = Vec::<NotionBlock>::new();

    while has_more {
        let url = format!("https://api.notion.com/v1/blocks/{}/children", page_id);

        let parms = if let Some(n) = next_cursor {
            vec![("page_size", "100".to_string()), ("next_cursor", n)]
        } else {
            vec![("page_size", "100".to_string())]
        };

        let client = Client::new()
            .get(&url)
            .query(&parms)
            .header("Authorization", format!("Bearer {}", secret))
            .header("Notion-Version", "2022-06-28");

        let mut page = client.send().await?.json::<NotionPage>().await?;

        next_cursor = page.next_cursor.clone();
        has_more = page.has_more;

        blocks.append(&mut page.results);
    }

    println!("{:#?}", blocks);
    Ok(())
}

// fn notion2pandoc(x: NotionPage) -> Pandoc {
//     //let x = Vec::<pandoc::Block>::new();
//     Pandoc {
//         pandoc_api_version: [1, 22, 2, 1],
//         meta: Meta {},
//         blocks: x
//             .results
//             .iter()
//             .map(|_| Block::HorizontalRule)
//             .collect::<Vec<Block>>(),
//     }
// }

#[derive(Debug, Serialize, Deserialize)]
struct NotionPage {
    has_more: bool,
    next_cursor: Option<String>,
    results: Vec<NotionBlock>,
}

/// Block Object
/// https://developers.notion.com/reference/block
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum NotionBlock {
    /// paragraph
    Paragraph {
        id: Uuid,
        archived: bool,
        paragraph: InlineContent,
        has_children: bool,
        #[serde(skip)]
        children: Option<Vec<NotionBlock>>,
    },
    #[serde(rename = "heading_1")]
    Heading1 {
        id: Uuid,
        archived: bool,
        heading_1: InlineContent,
    },
    #[serde(rename = "heading_2")]
    Heading2 {
        id: Uuid,
        archived: bool,
        heading_2: InlineContent,
    },
    #[serde(rename = "heading_3")]
    Heading3 {
        id: Uuid,
        archived: bool,
        heading_3: InlineContent,
    },
    Callout {
        id: Uuid,
        archived: bool,
        callout: InlineContent, // icon is ignored
        has_children: bool,
        #[serde(skip)]
        children: Option<Vec<NotionBlock>>,
    },
    Quote {
        id: Uuid,
        archived: bool,
        quote: InlineContent,
        has_children: bool,
        #[serde(skip)]
        children: Option<Vec<NotionBlock>>,
    },
    BulletedListItem {
        id: Uuid,
        archived: bool,
        bulleted_list_item: InlineContent,
        has_children: bool,
        #[serde(skip)]
        children: Option<Vec<NotionBlock>>,
    },
    NumberedListItem {
        id: Uuid,
        archived: bool,
        numbered_list_item: InlineContent,
        has_children: bool,
        #[serde(skip, default)]
        children: Option<Vec<NotionBlock>>,
    },
    ToDo {
        id: Uuid,
        archived: bool,
        to_do: ToDoContent,
        has_children: bool,
        #[serde(skip, default)]
        children: Option<Vec<NotionBlock>>,
    },
    Toggle {
        id: Uuid,
        archived: bool,
        toggle: InlineContent,
        has_children: bool,
        #[serde(skip, default)]
        children: Option<Vec<NotionBlock>>,
    },
    Code {
        id: Uuid,
        archived: bool,
        code: CodeContent,
    },
    // ChildPage,
    // ChildDatabase,
    Embed {
        id: Uuid,
        archived: bool,
        embed: EmbedContent,
    },
    Image {
        id: Uuid,
        archived: bool,
        image: FileContent,
    },
    Video {
        id: Uuid,
        archived: bool,
        video: FileContent,
    },
    File {
        id: Uuid,
        archived: bool,
        file: FileContent,
    },
    #[serde(rename = "pdf")]
    PDF {
        id: Uuid,
        archived: bool,
        pdf: FileContent,
    },
    // Bookmark,
    Equation {
        id: Uuid,
        archived: bool,
        equation: EquationContent,
    },
    Divider {
        id: Uuid,
        archived: bool,
    },
    TableOfContents {
        id: Uuid,
        archived: bool,
    },
    // Breadcrumb,
    // Column,
    // ColumnList,
    // LinkPreview,
    // Template,
    LinkToPage {
        id: Uuid,
        archived: bool,
        link_to_page: LinkToPageContent,
    },
    // SyncedBlock,
    Table {
        id: Uuid,
        archived: bool,
        table: TableContent,
        #[serde(skip)]
        children: Vec<NotionBlock>,
    },
    TableRow {
        id: Uuid,
        archived: bool,
        table_row: TableRowContent,
    },
    #[serde(other)]
    Unsupported,
}
#[derive(Debug, Serialize, Deserialize)]
struct InlineContent {
    rich_text: Vec<NotionRichText>,
}
#[derive(Debug, Serialize, Deserialize)]
struct ToDoContent {
    rich_text: Vec<NotionRichText>,
    checked: bool,
}
#[derive(Debug, Serialize, Deserialize)]
struct EmbedContent {
    caption: Vec<NotionRichText>,
    url: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct CodeContent {
    rich_text: Vec<NotionRichText>,
    caption: Vec<NotionRichText>,
    language: String,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum FileContent {
    External {
        caption: Vec<NotionRichText>,
        external: ExternalFileLink,
    },
    File {
        caption: Vec<NotionRichText>,
        file: FileLink,
    },
}
#[derive(Debug, Serialize, Deserialize)]
struct ExternalFileLink {
    /// Link to the externally hosted content.
    url: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct FileLink {
    /// Authenticated S3 URL to the file. The file URL will be valid for 1 hour
    /// but updated links can be requested if required.
    url: String,
    /// Date and time when this will expire. Formatted as an ISO 8601 date time string.
    expiry_time: DateTime<Local>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum LinkToPageContent {
    PageId { page_id: Uuid },
    DatabaseId { database_id: Uuid },
}
#[derive(Debug, Serialize, Deserialize)]
struct TableContent {
    table_width: u64,
    has_column_header: bool,
    has_row_header: bool,
}
#[derive(Debug, Serialize, Deserialize)]
struct TableRowContent {
    cells: Vec<Vec<NotionRichText>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum NotionRichText {
    Text {
        annotations: NotionAnnotations,
        text: TextContent,
    },
    Mention {
        plain_text: String,
        annotations: NotionAnnotations,
        mention: MentionContent,
    },
    Equation {
        annotations: NotionAnnotations,
        equation: EquationContent,
    },
}
#[derive(Debug, Serialize, Deserialize)]
struct TextContent {
    content: String,
    link: Option<Link>,
}
#[derive(Debug, Serialize, Deserialize)]
struct Link {
    url: String,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum MentionContent {
    Page { page: NotionPageId },
    Date,
    User,
}
#[derive(Debug, Serialize, Deserialize)]
struct NotionPageId {
    id: Uuid,
}
#[derive(Debug, Serialize, Deserialize)]
struct EquationContent {
    expression: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct NotionAnnotations {
    bold: bool,
    italic: bool,
    strikethrough: bool,
    underline: bool,
    code: bool,
}

// Pandoc
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Pandoc {
    pandoc_api_version: [u64; 4],
    meta: Meta,
    blocks: Vec<Block>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Meta {}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
enum Block {
    Plain(Vec<Inline>),
    Para(Vec<Inline>),
    CodeBlock(Attr, String),
    // RawBlock,
    BlockQuote(Vec<Block>),
    OrderedList(ListAttributes, Vec<Vec<Block>>),
    BulletList(Vec<Vec<Block>>),
    // DefinitionList,
    Header(u64, Attr, Vec<Inline>),
    HorizontalRule,
    Table(
        Vec<Inline>,
        Vec<Alignment>,
        Vec<f64>,
        Vec<TableCell>,
        Vec<Vec<TableCell>>,
    ),
    // Div,
    // Null,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
enum Inline {
    Str(String),
    Emph(Vec<Inline>),
    Strong(Vec<Inline>),
    Strikeout(Vec<Inline>),
    // Superscript,
    // Subscript,
    // SmallCaps,
    // Quoted(QuoteType, Vec<Inline>),
    // Cite(Vec<Citation>, Vec<Inline>),
    Code(Attr, Vec<Inline>),
    Space,
    LineBreak,
    Math(MathType, String),
    // RawInline,
    Link(Vec<Inline>, Target),
    Image(Vec<Inline>, Target),
    Note(Vec<Block>),
    Span(Attr, Vec<Inline>),
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
enum Alignment {
    AlignLeft,
    AlignRight,
    AlignCenter,
    #[default]
    AlignDefault,
}

#[derive(Debug, Serialize, Deserialize)]
struct ListAttributes(u64, ListNumberStyle, ListNumberDelim);

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
enum ListNumberStyle {
    #[default]
    DefaultStyle,
    Example,
    Decimal,
    LowerRoman,
    UpperRoman,
    LowerAlpha,
    UpperAlpha,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
enum ListNumberDelim {
    #[default]
    DefaultDelim,
    Period,
    OneParen,
    TwoParens,
}

#[derive(Debug, Serialize, Deserialize)]
struct Format(String);

#[derive(Debug, Serialize, Deserialize)]
struct Attr(String, Vec<String>, Vec<(String, String)>);

#[derive(Debug, Serialize, Deserialize)]
struct Target(String, String);

#[derive(Debug, Serialize, Deserialize)]
struct TableCell(Vec<Block>);

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
enum MathType {
    #[default]
    DisplayMath,
    InlineMath,
}
