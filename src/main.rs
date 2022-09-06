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
        .map(convert_block)
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

fn convert_block(block: NotionBlock) -> Block {
    match block.variant {
        NotionBlockVariant::Paragraph { paragraph } => {
            Block::Para(paragraph.rich_text.into_iter().map(convert_rich).collect())
        }
        NotionBlockVariant::Heading1 { heading_1 } => Block::Header(
            1,
            Attr::default(),
            heading_1.rich_text.into_iter().map(convert_rich).collect(),
        ),
        NotionBlockVariant::Heading2 { heading_2 } => Block::Header(
            2,
            Attr::default(),
            heading_2.rich_text.into_iter().map(convert_rich).collect(),
        ),
        NotionBlockVariant::Heading3 { heading_3 } => Block::Header(
            3,
            Attr::default(),
            heading_3.rich_text.into_iter().map(convert_rich).collect(),
        ),
        NotionBlockVariant::Callout { callout: _ } => Block::Null, //TODO
        NotionBlockVariant::Quote { quote } => {
            let mut elms = vec![Block::Plain(
                quote.rich_text.into_iter().map(convert_rich).collect(),
            )];
            if let Some(children) = block.children {
                elms.extend(children.into_iter().map(convert_block));
            }
            Block::BlockQuote(elms)
        }
        NotionBlockVariant::BulletedListItem {
            bulleted_list_item: _,
        } => unreachable!(),
        NotionBlockVariant::NumberedListItem {
            numbered_list_item: _,
        } => unreachable!(),
        NotionBlockVariant::ToDoListItem { to_do: _ } => unreachable!(),
        NotionBlockVariant::ToggleListItem { toggle: _ } => unreachable!(),
        NotionBlockVariant::BulletedList => Block::BulletList(
            block
                .children
                .unwrap()
                .into_iter()
                .map(convert_list_item)
                .collect(),
        ),
        NotionBlockVariant::NumberedList => Block::OrderedList(
            ListAttributes(1, ListNumberStyle::Decimal, ListNumberDelim::Period),
            block
                .children
                .unwrap()
                .into_iter()
                .map(convert_list_item)
                .collect(),
        ),
        NotionBlockVariant::Code { code } => {
            assert!(code.rich_text.len() == 1);
            let text = match code.rich_text.first() {
                Some(NotionRichText::Text {
                    annotations: _,
                    text,
                }) => text.content.clone(),
                _ => unreachable!(),
            };
            Block::CodeBlock(
                Attr(
                    "".to_string(),
                    vec![code.language],
                    Vec::<(String, String)>::new(),
                ),
                text,
            )
        }
        NotionBlockVariant::Image { image: file } => {
            let (caption, url) = match file {
                FileContent::File { caption, file } => (caption, file.url),
                FileContent::External { caption, external } => (caption, external.url),
            };
            let caption = caption.into_iter().map(convert_rich).collect();
            Block::Para(vec![Inline::Image(caption, Target(url, "".to_string()))])
        }
        NotionBlockVariant::Embed { embed } => Block::Para(vec![Inline::Link(
            embed.caption.into_iter().map(convert_rich).collect(),
            Target(embed.url, "".to_string()),
        )]),
        NotionBlockVariant::File { file }
        | NotionBlockVariant::Video { video: file }
        | NotionBlockVariant::PDF { pdf: file } => {
            let (caption, url) = match file {
                FileContent::File { caption, file } => (caption, file.url),
                FileContent::External { caption, external } => (caption, external.url),
            };
            let caption = caption.into_iter().map(convert_rich).collect();
            Block::Para(vec![Inline::Link(caption, Target(url, "".to_string()))])
        }
        NotionBlockVariant::Equation { equation } => Block::Para(vec![Inline::Math(
            MathType::DisplayMath,
            equation.expression,
        )]),
        NotionBlockVariant::Divider => Block::HorizontalRule,
        NotionBlockVariant::TableOfContents => Block::Null,
        NotionBlockVariant::LinkPreview { link_preview } => Block::Para(vec![Inline::Link(
            vec![Inline::Str(link_preview.url.clone())],
            Target(link_preview.url, "".to_string()),
        )]),
        NotionBlockVariant::LinkToPage { link_to_page: _ } => Block::Null, //TODO
        NotionBlockVariant::Table { table } => Block::Null,                // TODO
        NotionBlockVariant::TableRow { table_row: _ } => unreachable!(),
        _ => Block::Null,
    }
}

fn convert_list_item(x: NotionBlock) -> Vec<Block> {
    match x.variant {
        NotionBlockVariant::BulletedListItem {
            bulleted_list_item: variant,
        }
        | NotionBlockVariant::NumberedListItem {
            numbered_list_item: variant,
        }
        | NotionBlockVariant::ToggleListItem { toggle: variant } => {
            let mut result = vec![Block::Plain(
                variant
                    .rich_text
                    .into_iter()
                    .map(|r| convert_rich(r))
                    .collect(),
            )];
            if let Some(children) = x.children {
                result.extend(children.into_iter().map(|b| convert_block(b)));
            }
            result
        }
        NotionBlockVariant::ToDoListItem { to_do } => {
            let check_mark = if to_do.checked {
                "☑".to_string()
            } else {
                "☐".to_string()
            };
            let mut text_with_box = vec![Inline::Str(check_mark), Inline::Space];
            text_with_box.extend(to_do.rich_text.into_iter().map(|r| convert_rich(r)));
            let mut result = vec![Block::Plain(text_with_box)];
            if let Some(children) = x.children {
                result.extend(children.into_iter().map(|b| convert_block(b)));
            }
            result
        }
        _ => unreachable!(),
    }
}

fn convert_rich(x: NotionRichText) -> Inline {
    match x {
        NotionRichText::Text { annotations, text } => {
            if let Some(link) = text.link {
                Inline::Link(
                    vec![convert_rich(NotionRichText::Text {
                        annotations,
                        text: TextContent { link: None, ..text },
                    })],
                    Target(link.url, "".to_string()),
                )
            } else {
                if annotations.bold {
                    Inline::Strong(vec![convert_rich(NotionRichText::Text {
                        annotations: NotionAnnotations {
                            bold: false,
                            ..annotations
                        },
                        text,
                    })])
                } else if annotations.italic {
                    Inline::Strong(vec![convert_rich(NotionRichText::Text {
                        annotations: NotionAnnotations {
                            italic: false,
                            ..annotations
                        },
                        text,
                    })]) //TODO: fix
                } else if annotations.strikethrough {
                    Inline::Strikeout(vec![convert_rich(NotionRichText::Text {
                        annotations: NotionAnnotations {
                            strikethrough: false,
                            ..annotations
                        },
                        text,
                    })])
                } else if annotations.underline {
                    Inline::Emph(vec![convert_rich(NotionRichText::Text {
                        annotations: NotionAnnotations {
                            underline: false,
                            ..annotations
                        },
                        text,
                    })])
                } else if annotations.code {
                    Inline::Code(
                        Attr::default(),
                        vec![convert_rich(NotionRichText::Text {
                            annotations: NotionAnnotations {
                                code: false,
                                ..annotations
                            },
                            text,
                        })],
                    )
                } else {
                    Inline::Str(text.content)
                }
            }
        }
        NotionRichText::Mention {
            plain_text,
            annotations,
            mention: _,
        } => convert_rich(NotionRichText::Text {
            annotations,
            text: TextContent {
                content: plain_text,
                link: None,
            },
        }),
        NotionRichText::Equation {
            annotations, // TODO: support annotation
            equation,
        } => Inline::Math(MathType::InlineMath, equation.expression),
    }
}
