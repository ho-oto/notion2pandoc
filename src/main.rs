mod pandoc;

use async_recursion::async_recursion;
use chrono::{DateTime, Local};
use clap::Parser;
use futures::future::join_all;
use itertools::join;
use reqwest::Client;
use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

/// https://developers.notion.com/reference/intro
static NOTION_API_VERSION: &str = "2022-06-28";
/// https://hackage.haskell.org/package/pandoc-types-1.22.2.1/docs/Text-Pandoc-Definition.html
static PANDOC_API_VERSION: [u64; 4] = [1, 22, 2, 1];

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(short = 'i')]
    id: String,
    #[clap(short = 'c')]
    cert: String,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    let id = Uuid::parse_str(&args.id).unwrap_or_else(|_| panic!("ID should be UUID"));
    let page = NotionPage::fetch(id, &args.cert).await;
    let mut blocks = vec![pandoc::Block::Header(
        1,
        pandoc::Attr::default(),
        vec![pandoc::Inline::Str(page.title)],
    )];
    blocks.extend(page.blocks.into_iter().map(|b| b.to_pandoc()));
    let rsl = pandoc::Pandoc {
        pandoc_api_version: PANDOC_API_VERSION,
        meta: pandoc::Meta {},
        blocks,
    };
    println!("{}", serde_json::to_string(&rsl).unwrap())
}

async fn fetch_children(id: Uuid, secret: &String) -> Vec<NotionBlock> {
    #[derive(Deserialize)]
    struct NotionResponse {
        has_more: bool,
        next_cursor: Option<String>,
        results: Vec<NotionBlock>,
    }
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

struct NotionPage {
    title: String,
    blocks: Vec<NotionBlock>,
}
impl NotionPage {
    async fn fetch(id: Uuid, secret: &String) -> Self {
        #[derive(Deserialize)]
        struct NotionResponse {
            archived: bool,
            properties: TitleContent,
        }
        #[derive(Deserialize)]
        struct TitleContent {
            title: Title,
        }
        #[derive(Deserialize)]
        struct Title {
            title: Vec<NotionRichText>,
        }
        let url = format!("https://api.notion.com/v1/pages/{}", id);
        let meta = Client::new()
            .get(&url)
            .header("Authorization", format!("Bearer {}", secret))
            .header("Notion-Version", NOTION_API_VERSION)
            .send()
            .await
            .unwrap_or_else(|_| panic!("failed to fetch page {}", id))
            .json::<NotionResponse>()
            .await
            .unwrap_or_else(|_| panic!("failed to deserialize page {}", id));
        if meta.archived {
            panic!("archived page")
        }
        let title = join(
            meta.properties.title.title.into_iter().map(|r| match r {
                NotionRichText::Text {
                    annotations: _,
                    text,
                } => text.content.clone(),
                _ => panic!(),
            }),
            "",
        );
        let mut blocks = fetch_children(id, secret).await;
        join_all(
            blocks
                .iter_mut()
                .map(|x| async { x.fetch_recursive(secret).await }),
        )
        .await;
        blocks = Self::join_list_block(Self::flatten(blocks));
        Self { title, blocks }
    }

    fn flatten(blocks: Vec<NotionBlock>) -> Vec<NotionBlock> {
        let mut result = Vec::<NotionBlock>::new();
        for block in blocks {
            if let Some(children) = block.children {
                let mut flattened_children = Self::flatten(children);
                match block.variant {
                    NotionBlockVariant::Paragraph { inline: _ } => {
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
            block.children = block.children.map(Self::join_list_block);
            match (result.last().map(|x| &x.variant), &block.variant) {
                (
                    Some(BulletedList),
                    BulletedListItem { inline: _ }
                    | ToggleListItem { inline: _ }
                    | ToDoListItem { to_do: _ },
                )
                | (Some(NumberedList), NumberedListItem { inline: _ }) => {
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
                    BulletedListItem { inline: _ }
                    | ToggleListItem { inline: _ }
                    | ToDoListItem { to_do: _ },
                ) => result.push(NotionBlock {
                    id: block.id,
                    archived: false,
                    children: Some(vec![block]),
                    variant: BulletedList,
                }),
                (_, NumberedListItem { inline: _ }) => result.push(NotionBlock {
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
        #[serde(rename = "paragraph")]
        inline: InlineContent,
    },
    #[serde(rename = "heading_1")]
    Heading1 {
        #[serde(rename = "heading_1")]
        inline: InlineContent,
    },
    #[serde(rename = "heading_2")]
    Heading2 {
        #[serde(rename = "heading_2")]
        inline: InlineContent,
    },
    #[serde(rename = "heading_3")]
    Heading3 {
        #[serde(rename = "heading_3")]
        inline: InlineContent,
    },
    Quote {
        #[serde(rename = "quote")]
        inline: InlineContent,
    },

    Callout {
        callout: CalloutContent,
    },

    BulletedListItem {
        #[serde(rename = "bulleted_list_item")]
        inline: InlineContent,
    },
    NumberedListItem {
        #[serde(rename = "numbered_list_item")]
        inline: InlineContent,
    },
    #[serde(rename = "to_do")]
    ToDoListItem {
        to_do: ToDoContent,
    },
    #[serde(rename = "toggle")]
    ToggleListItem {
        #[serde(rename = "toggle")]
        inline: InlineContent,
    },

    Code {
        code: CodeContent,
    },
    Equation {
        equation: EquationContent,
    },

    Image {
        #[serde(rename = "image")]
        file: FileContent,
    },
    Video {
        #[serde(rename = "video")]
        file: FileContent,
    },
    File {
        #[serde(rename = "file")]
        file: FileContent,
    },
    #[serde(rename = "pdf")]
    PDF {
        #[serde(rename = "pdf")]
        file: FileContent,
    },

    Embed {
        embed: EmbedContent,
    },
    Bookmark {
        #[serde(rename = "bookmark")]
        embed: EmbedContent,
    },
    LinkPreview {
        link_preview: Link,
    },
    LinkToPage {
        link_to_page: LinkToPageContent,
    },

    Table {
        table: TableContent,
    },
    TableRow {
        table_row: TableRowContent,
    },

    Divider,
    TableOfContents,

    #[serde(skip)]
    BulletedList,
    #[serde(skip)]
    NumberedList,

    // ChildPage,
    // ChildDatabase,
    // Breadcrumb,
    // Column,
    // ColumnList,
    // Template,
    // SyncedBlock,
    #[serde(other)]
    Unsupported,
}
impl NotionBlock {
    #[async_recursion]
    async fn fetch_recursive(&mut self, secret: &String) {
        if let Some(_) = self.children {
            let mut children = fetch_children(self.id, secret).await;
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
    url: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
struct FileLink {
    url: String,
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

impl InlineContent {
    fn to_pandoc(self) -> Vec<pandoc::Inline> {
        self.rich_text.into_iter().map(|r| r.to_pandoc()).collect()
    }
    fn to_pandoc_with_children(self, children: Option<Vec<NotionBlock>>) -> Vec<pandoc::Block> {
        let mut result = vec![pandoc::Block::Plain(self.to_pandoc())];
        if let Some(children) = children {
            result.extend(children.into_iter().map(|b| b.to_pandoc()));
        }
        result
    }
}

impl NotionBlock {
    fn to_pandoc(self) -> pandoc::Block {
        use pandoc::*;
        use NotionBlockVariant::*;
        match self.variant {
            Paragraph { inline } => Block::Para(inline.to_pandoc()),
            Heading1 { inline } => Block::Header(2, Attr::default(), inline.to_pandoc()),
            Heading2 { inline } => Block::Header(3, Attr::default(), inline.to_pandoc()),
            Heading3 { inline } => Block::Header(4, Attr::default(), inline.to_pandoc()),
            Callout { callout: _ } => Block::Null, //TODO
            Quote { inline } => Block::BlockQuote(inline.to_pandoc_with_children(self.children)),
            // {Bulleted,Numbered,ToDo,Toggle}ListItem should be
            // in a children of BulletedList/NumberedList node
            BulletedListItem { inline: _ }
            | NumberedListItem { inline: _ }
            | ToDoListItem { to_do: _ }
            | ToggleListItem { inline: _ } => unreachable!(),
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
            Image { file } => {
                let (caption, url) = match file {
                    FileContent::File { caption, file } => (caption, file.url),
                    FileContent::External { caption, external } => (caption, external.url),
                };
                let caption = caption.into_iter().map(|r| r.to_pandoc()).collect();
                Block::Para(vec![Inline::Image(
                    Attr::default(),
                    caption,
                    Target(url, String::new()),
                )])
            }
            File { file } | Video { file } | PDF { file } => {
                let (caption, url) = match file {
                    FileContent::File { caption, file } => (caption, file.url),
                    FileContent::External { caption, external } => (caption, external.url),
                };
                let caption = caption.into_iter().map(|r| r.to_pandoc()).collect();
                Block::Para(vec![Inline::Link(
                    Attr::default(),
                    caption,
                    Target(url, String::new()),
                )])
            }
            Embed { embed } | Bookmark { embed } => {
                let caption = if embed.caption.is_empty() {
                    vec![Inline::Str(embed.url.clone())]
                } else {
                    embed.caption.into_iter().map(|r| r.to_pandoc()).collect()
                };
                Block::Para(vec![Inline::Link(
                    Attr::default(),
                    caption,
                    Target(embed.url, String::new()),
                )])
            }
            Equation { equation } => Block::Para(vec![Inline::Math(
                MathType::DisplayMath,
                equation.expression,
            )]),
            Divider => Block::HorizontalRule,
            TableOfContents => Block::Null, // TODO: support TOC
            LinkPreview { link_preview } => {
                Block::Para(vec![
                    Inline::Str(link_preview.url.clone()).to_link(link_preview.url)
                ])
            }
            LinkToPage { link_to_page: _ } => Block::Null, // TODO: support LinkToPage
            Table { table } => {
                if let Some(mut children) = self.children {
                    let header_start = if table.has_column_header { 1 } else { 0 };
                    let (header, body) = children.split_at_mut(header_start);
                    let header: Vec<Row> = header
                        .to_owned()
                        .into_iter()
                        .map(Self::convert_table_row)
                        .collect();
                    let body: Vec<Row> = body
                        .to_owned()
                        .into_iter()
                        .map(Self::convert_table_row)
                        .collect();
                    let col_specs = (0..table.table_width).map(|_| ColSpec::default()).collect();
                    Block::Table(
                        Attr::default(),
                        Caption::default(),
                        col_specs,
                        TableHead(Attr::default(), header),
                        vec![TableBody(Attr::default(), RowHeadColumns(0), vec![], body)],
                        TableFoot::default(),
                    )
                } else {
                    Block::Null
                }
            }
            TableRow { table_row: _ } => unreachable!(),
            _ => Block::Null,
        }
    }

    fn convert_table_row(x: NotionBlock) -> pandoc::Row {
        match x.variant {
            NotionBlockVariant::TableRow { table_row } => {
                Self::convert_table_cells(table_row.cells)
            }
            _ => unreachable!(),
        }
    }

    fn convert_table_cells(x: Vec<Vec<NotionRichText>>) -> pandoc::Row {
        pandoc::Row(
            pandoc::Attr::default(),
            x.into_iter().map(Self::convert_table_cell).collect(),
        )
    }

    fn convert_table_cell(x: Vec<NotionRichText>) -> pandoc::Cell {
        pandoc::Cell(
            pandoc::Attr::default(),
            pandoc::Alignment::default(),
            pandoc::RowSpan(1),
            pandoc::ColSpan(1),
            vec![pandoc::Block::Plain(
                x.into_iter().map(|r| r.to_pandoc()).collect(),
            )],
        )
    }

    fn convert_list_item(x: NotionBlock) -> Vec<pandoc::Block> {
        use NotionBlockVariant::*;
        match x.variant {
            BulletedListItem { inline }
            | NumberedListItem { inline }
            | ToggleListItem { inline } => inline.to_pandoc_with_children(x.children),
            ToDoListItem { to_do } => {
                let check_mark = format!("{}", if to_do.checked { "☒" } else { "☐" });
                let mut text_with_box =
                    vec![pandoc::Inline::Str(check_mark), pandoc::Inline::Space];
                text_with_box.extend(to_do.rich_text.into_iter().map(|r| r.to_pandoc()));
                let mut result = vec![pandoc::Block::Plain(text_with_box)];
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
    fn to_pandoc(self) -> pandoc::Inline {
        match self {
            NotionRichText::Text { annotations, text } => {
                if let Some(link) = text.link {
                    NotionRichText::Text {
                        annotations,
                        text: TextContent { link: None, ..text },
                    }
                    .to_pandoc()
                    .to_link(link.url)
                } else {
                    let inline = if annotations.code {
                        pandoc::Inline::Code(pandoc::Attr::default(), text.content)
                    } else {
                        pandoc::Inline::Str(text.content)
                    };
                    Self::annotate(inline, annotations)
                }
            }
            NotionRichText::Mention {
                plain_text,
                annotations,
                mention: _,
            } => Self::annotate(pandoc::Inline::Str(plain_text), annotations),
            NotionRichText::Equation {
                annotations,
                equation,
            } => Self::annotate(
                pandoc::Inline::Math(pandoc::MathType::InlineMath, equation.expression),
                annotations,
            ),
        }
    }

    fn annotate(inline: pandoc::Inline, annotations: NotionAnnotations) -> pandoc::Inline {
        let mut result = inline;
        if annotations.bold {
            result = pandoc::Inline::Strong(vec![result]);
        }
        if annotations.italic || annotations.underline {
            result = pandoc::Inline::Strong(vec![result]);
        }
        if annotations.strikethrough {
            result = pandoc::Inline::Strikeout(vec![result]);
        }
        result
    }
}
