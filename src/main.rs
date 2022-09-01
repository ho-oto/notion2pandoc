use async_recursion::async_recursion;
use chrono::{DateTime, Local};
use futures::future::join_all;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

static NOTION_API_VERSION: &str = "2022-06-28";
static SECRET: &str = "secret_zzz";

#[tokio::main]
async fn main() {
    println!(
        "{:#?}",
        fetch_page(
            "c90565cf4ae64e3dbfdbb9140b1f8b04".to_string(),
            SECRET.to_string()
        )
        .await
    )
}

async fn fetch_page(id: String, secret: String) -> Vec<NotionBlock> {
    let mut x = fetch_children(id, secret).await;
    join_all(x.iter_mut().map(|x| async { x.fetch_recursive().await })).await;
    x
}

#[derive(Deserialize)]
struct NotionResponse {
    has_more: bool,
    next_cursor: Option<String>,
    results: Vec<NotionBlock>,
}
async fn fetch_children(id: String, secret: String) -> Vec<NotionBlock> {
    let mut blocks = Vec::<NotionBlock>::new();
    let mut has_more = true;
    let mut next_cursor = None;
    while has_more {
        let url = format!("https://api.notion.com/v1/blocks/{}/children", id);
        let parms = if let Some(n) = next_cursor {
            vec![("page_size", "100".to_string()), ("start_cursor", n)]
        } else {
            vec![("page_size", "100".to_string())]
        };
        let client = Client::new()
            .get(&url)
            .query(&parms)
            .header("Authorization", format!("Bearer {}", secret))
            .header("Notion-Version", NOTION_API_VERSION);
        let mut page = client
            .send()
            .await
            .unwrap_or_else(|_| panic!("failed to fetch children blocks of {}", id))
            .json::<NotionResponse>()
            .await
            .unwrap_or_else(|_| panic!("failed to deserialize children blocks of {}", id));
        next_cursor = page.next_cursor;
        has_more = page.has_more;
        blocks.append(&mut page.results);
    }
    blocks
}

#[derive(Debug, Serialize, Deserialize)]
struct NotionBlock {
    id: Uuid,
    archived: bool,
    has_children: bool,
    #[serde(skip)]
    children: Option<Vec<NotionBlock>>,
    #[serde(flatten)]
    variant: NotionBlockVariant,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum NotionBlockVariant {
    Paragraph {
        paragraph: InlineContent,
    },
    #[serde(rename = "heading_1")]
    Heading1 {
        heading_1: InlineContent,
    },
    #[serde(rename = "heading_2")]
    Heading2 {
        heading_2: InlineContent,
    },
    #[serde(rename = "heading_3")]
    Heading3 {
        heading_3: InlineContent,
    },
    Callout {
        callout: CalloutContent,
    },
    Quote {
        quote: InlineContent,
    },
    BulletedListItem {
        bulleted_list_item: InlineContent,
    },
    NumberedListItem {
        numbered_list_item: InlineContent,
    },
    ToDo {
        to_do: ToDoContent,
    },
    Toggle {
        toggle: InlineContent,
    },
    Code {
        code: CodeContent,
    },
    // ChildPage,
    // ChildDatabase,
    Embed {
        embed: EmbedContent,
    },
    Image {
        image: FileContent,
    },
    Video {
        video: FileContent,
    },
    File {
        file: FileContent,
    },
    #[serde(rename = "pdf")]
    PDF {
        pdf: FileContent,
    },
    // Bookmark,
    Equation {
        equation: EquationContent,
    },
    Divider,
    TableOfContents,
    // Breadcrumb,
    // Column,
    // ColumnList,
    LinkPreview {
        link_preview: Link,
    },
    // Template,
    LinkToPage {
        link_to_page: LinkToPageContent,
    },
    // SyncedBlock,
    Table {
        table: TableContent,
    },
    TableRow {
        table_row: TableRowContent,
    },
    #[serde(other)]
    Unsupported,
}
impl NotionBlock {
    #[async_recursion]
    async fn fetch_recursive(&mut self) {
        if !self.has_children {
            return;
        }
        let mut children = fetch_children(self.id.to_string(), SECRET.to_string()).await;
        join_all(
            children
                .iter_mut()
                .map(|x| async { x.fetch_recursive().await }),
        )
        .await;
        self.children = Some(children);
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct InlineContent {
    rich_text: Vec<NotionRichText>,
}
#[derive(Debug, Serialize, Deserialize)]
struct CalloutContent {
    rich_text: Vec<NotionRichText>,
    icon: Icon,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Icon {
    Emoji { emoji: String },
    External { external: Link },
}
#[derive(Debug, Serialize, Deserialize)]
struct Emoji {
    emoji: String,
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
