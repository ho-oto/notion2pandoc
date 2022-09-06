use async_recursion::async_recursion;
use chrono::{DateTime, Local};
use futures::future::join_all;
use reqwest::Client;
use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

static NOTION_API_VERSION: &str = "2022-06-28";
static SECRET: &str = "secret_zzz";

#[tokio::main]
async fn main() {
    let rsl = Pandoc {
        pandoc_api_version: [1, 22, 2, 1],
        meta: Meta {},
        blocks: join_list_block(flatten(
            fetch_page(
                &"c90565cf4ae64e3dbfdbb9140b1f8b04".to_string(),
                &SECRET.to_string(),
            )
            .await,
        ))
        .into_iter()
        .map(|b| b.to_pandoc())
        .collect::<Vec<Block>>(),
    };
    println!("{}", serde_json::to_string(&rsl).unwrap())
}

async fn fetch_page(id: &String, secret: &String) -> Vec<NotionBlock> {
    let mut blocks = fetch_children(id, secret).await;
    join_all(
        blocks
            .iter_mut()
            .map(|x| async { x.fetch_recursive(secret).await }),
    )
    .await;
    blocks
}

#[derive(Deserialize)]
struct NotionResponse {
    has_more: bool,
    next_cursor: Option<String>,
    results: Vec<NotionBlock>,
}
async fn fetch_children(id: &String, secret: &String) -> Vec<NotionBlock> {
    let mut blocks = Vec::<NotionBlock>::new();
    let mut has_more = true;
    let mut next_cursor = None;
    while has_more {
        let url = format!("https://api.notion.com/v1/blocks/{}/children", id);
        let params = if let Some(n) = next_cursor {
            vec![("page_size", "100".to_string()), ("start_cursor", n)]
        } else {
            vec![("page_size", "100".to_string())]
        };
        let client = Client::new()
            .get(&url)
            .query(&params)
            .header("Authorization", format!("Bearer {}", secret))
            .header("Notion-Version", NOTION_API_VERSION);
        let page = client
            .send()
            .await
            .unwrap_or_else(|_| panic!("failed to fetch children blocks of {}", id))
            .json::<NotionResponse>()
            .await
            .unwrap_or_else(|_| panic!("failed to deserialize children blocks of {}", id));
        next_cursor = page.next_cursor;
        has_more = page.has_more;
        blocks.extend(page.results.into_iter().filter(|x| !x.archived));
    }
    blocks
}

fn deserialize_children<'de, D>(deserializer: D) -> Result<Option<Vec<NotionBlock>>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(if bool::deserialize(deserializer)? {
        Some(Vec::<NotionBlock>::new())
    } else {
        None
    })
}
#[derive(Debug, Serialize, Deserialize, Clone)]
struct NotionBlock {
    id: Uuid,
    archived: bool,
    #[serde(flatten)]
    variant: NotionBlockVariant,
    #[serde(deserialize_with = "deserialize_children", rename = "has_children")]
    children: Option<Vec<NotionBlock>>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
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
    #[serde(rename = "to_do")]
    ToDoListItem {
        to_do: ToDoContent,
    },
    #[serde(rename = "toggle")]
    ToggleListItem {
        toggle: InlineContent,
    },
    #[serde(skip)]
    BulletedList,
    #[serde(skip)]
    NumberedList,
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
    async fn fetch_recursive(&mut self, secret: &String) {
        if let Some(_) = self.children {
            let mut children = fetch_children(&self.id.to_string(), secret).await;
            join_all(
                children
                    .iter_mut()
                    .map(|x| async { x.fetch_recursive(secret).await }),
            )
            .await;
            self.children = Some(children);
        }
    }
}

fn flatten(blocks: Vec<NotionBlock>) -> Vec<NotionBlock> {
    let mut result = Vec::<NotionBlock>::new();
    for block in blocks {
        if let Some(children) = block.children {
            let mut flattened_children = flatten(children);
            match block.variant {
                NotionBlockVariant::Paragraph { paragraph: _ } => {
                    result.push(NotionBlock {
                        children: None,
                        ..block
                    });
                    result.append(&mut flattened_children);
                }
                _ => {
                    result.push(NotionBlock {
                        children: Some(flattened_children),
                        ..block
                    });
                }
            }
        } else {
            result.push(block);
        }
    }
    result
}

fn join_list_block(blocks: Vec<NotionBlock>) -> Vec<NotionBlock> {
    use NotionBlockVariant::*;
    let mut result = Vec::<NotionBlock>::new();
    for mut block in blocks {
        block.children = block.children.map(|x| join_list_block(x));
        match (result.last().map(|x| &x.variant), &block.variant) {
            (
                Some(BulletedList),
                BulletedListItem {
                    bulleted_list_item: _,
                }
                | ToggleListItem { toggle: _ }
                | ToDoListItem { to_do: _ },
            )
            | (
                Some(NumberedList),
                NumberedListItem {
                    numbered_list_item: _,
                },
            ) => {
                result
                    .last_mut()
                    .unwrap()
                    .children
                    .as_mut()
                    .unwrap()
                    .push(block);
            }
            (
                _,
                BulletedListItem {
                    bulleted_list_item: _,
                }
                | ToggleListItem { toggle: _ }
                | ToDoListItem { to_do: _ },
            ) => result.push(NotionBlock {
                id: block.id,
                archived: false,
                children: Some(vec![block]),
                variant: BulletedList,
            }),
            (
                _,
                NumberedListItem {
                    numbered_list_item: _,
                },
            ) => result.push(NotionBlock {
                id: block.id,
                archived: false,
                children: Some(vec![block]),
                variant: NumberedList,
            }),
            _ => {
                result.push(block);
            }
        };
    }
    result
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct InlineContent {
    rich_text: Vec<NotionRichText>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
struct CalloutContent {
    rich_text: Vec<NotionRichText>,
    icon: Icon,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Icon {
    Emoji { emoji: String },
    External { external: Link },
}
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Emoji {
    emoji: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
struct ToDoContent {
    rich_text: Vec<NotionRichText>,
    checked: bool,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
struct EmbedContent {
    caption: Vec<NotionRichText>,
    url: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
struct CodeContent {
    rich_text: Vec<NotionRichText>,
    caption: Vec<NotionRichText>,
    language: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
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
#[derive(Debug, Serialize, Deserialize, Clone)]
struct ExternalFileLink {
    /// Link to the externally hosted content.
    url: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
struct FileLink {
    /// Authenticated S3 URL to the file. The file URL will be valid for 1 hour
    /// but updated links can be requested if required.
    url: String,
    /// Date and time when this will expire. Formatted as an ISO 8601 date time string.
    expiry_time: DateTime<Local>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
enum LinkToPageContent {
    PageId { page_id: Uuid },
    DatabaseId { database_id: Uuid },
}
#[derive(Debug, Serialize, Deserialize, Clone)]
struct TableContent {
    table_width: u64,
    has_column_header: bool,
    has_row_header: bool,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
struct TableRowContent {
    cells: Vec<Vec<NotionRichText>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
#[derive(Debug, Serialize, Deserialize, Clone)]
struct TextContent {
    content: String,
    link: Option<Link>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Link {
    url: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
enum MentionContent {
    Page { page: NotionPageId },
    Date,
    User,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
struct NotionPageId {
    id: Uuid,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
struct EquationContent {
    expression: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
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
    Null,
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
#[derive(Debug, Serialize, Deserialize, Default)]
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

impl InlineContent {
    fn to_pandoc(self) -> Vec<Inline> {
        self.rich_text.into_iter().map(|r| r.to_pandoc()).collect()
    }
    fn to_pandoc_with_children(self, children: Option<Vec<NotionBlock>>) -> Vec<Block> {
        let mut result = vec![Block::Plain(self.to_pandoc())];
        if let Some(children) = children {
            result.extend(children.into_iter().map(|b| b.to_pandoc()));
        }
        result
    }
}

impl NotionBlock {
    fn to_pandoc(self) -> Block {
        use NotionBlockVariant::*;
        match self.variant {
            Paragraph { paragraph } => Block::Para(paragraph.to_pandoc()),
            Heading1 { heading_1 } => Block::Header(1, Attr::default(), heading_1.to_pandoc()),
            Heading2 { heading_2 } => Block::Header(2, Attr::default(), heading_2.to_pandoc()),
            Heading3 { heading_3 } => Block::Header(3, Attr::default(), heading_3.to_pandoc()),
            Callout { callout: _ } => Block::Null, //TODO
            Quote { quote } => Block::BlockQuote(quote.to_pandoc_with_children(self.children)),
            // {Bulleted,Numbered,ToDo,Toggle}ListItem should be
            // in a children of BulletedList/NumberedList node
            BulletedListItem {
                bulleted_list_item: _,
            }
            | NumberedListItem {
                numbered_list_item: _,
            }
            | ToDoListItem { to_do: _ }
            | ToggleListItem { toggle: _ } => unreachable!(),
            BulletedList => Block::BulletList(
                self.children
                    .unwrap()
                    .into_iter()
                    .map(Self::convert_list_item)
                    .collect(),
            ),
            NumberedList => Block::OrderedList(
                ListAttributes(1, ListNumberStyle::Decimal, ListNumberDelim::Period),
                self.children
                    .unwrap()
                    .into_iter()
                    .map(Self::convert_list_item)
                    .collect(),
            ),
            Code { code } => {
                assert!(code.rich_text.len() == 1);
                let text = match code.rich_text.first() {
                    Some(NotionRichText::Text {
                        annotations: _,
                        text,
                    }) => text.content.clone(),
                    _ => unreachable!(),
                };
                Block::CodeBlock(Attr(String::new(), vec![code.language], vec![]), text)
            }
            Image { image: file } => {
                let (caption, url) = match file {
                    FileContent::File { caption, file } => (caption, file.url),
                    FileContent::External { caption, external } => (caption, external.url),
                };
                let caption = caption.into_iter().map(|r| r.to_pandoc()).collect();
                Block::Para(vec![Inline::Image(caption, Target(url, String::new()))])
            }
            File { file } | Video { video: file } | PDF { pdf: file } => {
                let (caption, url) = match file {
                    FileContent::File { caption, file } => (caption, file.url),
                    FileContent::External { caption, external } => (caption, external.url),
                };
                let caption = caption.into_iter().map(|r| r.to_pandoc()).collect();
                Block::Para(vec![Inline::Link(caption, Target(url, String::new()))])
            }
            Embed { embed } => Block::Para(vec![Inline::Link(
                embed.caption.into_iter().map(|r| r.to_pandoc()).collect(),
                Target(embed.url, String::new()),
            )]),
            Equation { equation } => Block::Para(vec![Inline::Math(
                MathType::DisplayMath,
                equation.expression,
            )]),
            Divider => Block::HorizontalRule,
            TableOfContents => Block::Null,
            LinkPreview { link_preview } => Block::Para(vec![Inline::Link(
                vec![Inline::Str(link_preview.url.clone())],
                Target(link_preview.url, String::new()),
            )]),
            LinkToPage { link_to_page: _ } => Block::Null, //TODO
            Table { table } => Block::Null,                // TODO
            TableRow { table_row: _ } => unreachable!(),
            _ => Block::Null,
        }
    }

    fn convert_list_item(x: NotionBlock) -> Vec<Block> {
        use NotionBlockVariant::*;
        match x.variant {
            BulletedListItem {
                bulleted_list_item: variant,
            }
            | NumberedListItem {
                numbered_list_item: variant,
            }
            | ToggleListItem { toggle: variant } => variant.to_pandoc_with_children(x.children),
            ToDoListItem { to_do } => {
                let check_mark = if to_do.checked {
                    "☑".to_string()
                } else {
                    "☐".to_string()
                };
                let mut text_with_box = vec![Inline::Str(check_mark), Inline::Space];
                text_with_box.extend(to_do.rich_text.into_iter().map(|r| r.to_pandoc()));
                let mut result = vec![Block::Plain(text_with_box)];
                if let Some(children) = x.children {
                    result.extend(children.into_iter().map(|b| b.to_pandoc()));
                }
                result
            }
            _ => unreachable!(),
        }
    }
}

impl NotionRichText {
    fn to_pandoc(self) -> Inline {
        match self {
            NotionRichText::Text { annotations, text } => {
                if let Some(link) = text.link {
                    let trg = Target(link.url, String::new());
                    let str = NotionRichText::Text {
                        annotations,
                        text: TextContent { link: None, ..text },
                    };
                    Inline::Link(vec![str.to_pandoc()], trg)
                } else {
                    let mut str = Inline::Str(text.content);
                    if annotations.bold {
                        str = Inline::Strong(vec![str]);
                    }
                    if annotations.italic || annotations.underline {
                        str = Inline::Strong(vec![str]);
                    }
                    if annotations.strikethrough {
                        str = Inline::Strikeout(vec![str]);
                    }
                    if annotations.code {
                        str = Inline::Code(Attr::default(), vec![str]);
                    }
                    str
                }
            }
            NotionRichText::Mention {
                plain_text,
                annotations,
                mention: _,
            } => Self::annotate(Inline::Str(plain_text), annotations),
            NotionRichText::Equation {
                annotations,
                equation,
            } => Self::annotate(
                Inline::Math(MathType::InlineMath, equation.expression),
                annotations,
            ),
        }
    }

    fn annotate(inline: Inline, annotations: NotionAnnotations) -> Inline {
        let mut result = inline;
        if annotations.bold {
            result = Inline::Strong(vec![result]);
        }
        if annotations.italic || annotations.underline {
            result = Inline::Strong(vec![result]);
        }
        if annotations.strikethrough {
            result = Inline::Strikeout(vec![result]);
        }
        if annotations.code {
            result = Inline::Code(Attr::default(), vec![result]);
        }
        result
    }
}
