mod notion;
mod pandoc;

use async_recursion::async_recursion;
use clap::Parser;
use futures::future::join_all;
use itertools::join;
use reqwest::Client;
use serde::Deserialize;
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

async fn fetch_children(id: Uuid, secret: &String) -> Vec<notion::Block> {
    #[derive(Deserialize)]
    struct NotionResponse {
        has_more: bool,
        next_cursor: Option<String>,
        results: Vec<notion::Block>,
    }
    let mut blocks = Vec::<notion::Block>::new();
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
    blocks: Vec<notion::Block>,
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
            title: Vec<notion::RichText>,
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
                notion::RichText::Text {
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

    fn flatten(blocks: Vec<notion::Block>) -> Vec<notion::Block> {
        let mut result = Vec::<notion::Block>::new();
        for block in blocks {
            if let Some(children) = block.children {
                let mut flattened_children = Self::flatten(children);
                match block.variant {
                    notion::BlockVariant::Paragraph { inline: _ } => {
                        result.push(notion::Block {
                            children: None,
                            ..block
                        });
                        result.append(&mut flattened_children);
                    }
                    _ => {
                        result.push(notion::Block {
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

    fn join_list_block(blocks: Vec<notion::Block>) -> Vec<notion::Block> {
        use notion::BlockVariant::*;
        let mut result = Vec::<notion::Block>::new();
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
                ) => result.push(notion::Block {
                    id: block.id,
                    archived: false,
                    children: Some(vec![block]),
                    variant: BulletedList,
                }),
                (_, NumberedListItem { inline: _ }) => result.push(notion::Block {
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

impl notion::Block {
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

    fn to_pandoc(self) -> pandoc::Block {
        use notion::BlockVariant::*;
        use pandoc::*;
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
                    Some(notion::RichText::Text {
                        annotations: _,
                        text,
                    }) => text.content.clone(),
                    _ => unreachable!(),
                };
                Block::CodeBlock(Attr(String::new(), vec![code.language], vec![]), text)
            }
            Image { file } => {
                let (caption, url) = match file {
                    notion::FileContent::File { caption, file } => (caption, file.url),
                    notion::FileContent::External { caption, external } => (caption, external.url),
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
                    notion::FileContent::File { caption, file } => (caption, file.url),
                    notion::FileContent::External { caption, external } => (caption, external.url),
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

    fn convert_table_row(x: notion::Block) -> pandoc::Row {
        match x.variant {
            notion::BlockVariant::TableRow { table_row } => {
                Self::convert_table_cells(table_row.cells)
            }
            _ => unreachable!(),
        }
    }

    fn convert_table_cells(x: Vec<Vec<notion::RichText>>) -> pandoc::Row {
        pandoc::Row(
            pandoc::Attr::default(),
            x.into_iter().map(Self::convert_table_cell).collect(),
        )
    }

    fn convert_table_cell(x: Vec<notion::RichText>) -> pandoc::Cell {
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

    fn convert_list_item(x: notion::Block) -> Vec<pandoc::Block> {
        use notion::BlockVariant::*;
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

impl notion::InlineContent {
    fn to_pandoc(self) -> Vec<pandoc::Inline> {
        self.rich_text.into_iter().map(|r| r.to_pandoc()).collect()
    }
    fn to_pandoc_with_children(self, children: Option<Vec<notion::Block>>) -> Vec<pandoc::Block> {
        let mut result = vec![pandoc::Block::Plain(self.to_pandoc())];
        if let Some(children) = children {
            result.extend(children.into_iter().map(|b| b.to_pandoc()));
        }
        result
    }
}

impl notion::RichText {
    fn to_pandoc(self) -> pandoc::Inline {
        match self {
            notion::RichText::Text { annotations, text } => {
                if let Some(link) = text.link {
                    notion::RichText::Text {
                        annotations,
                        text: notion::TextContent { link: None, ..text },
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
            notion::RichText::Mention {
                plain_text,
                annotations,
                mention: _,
            } => Self::annotate(pandoc::Inline::Str(plain_text), annotations),
            notion::RichText::Equation {
                annotations,
                equation,
            } => Self::annotate(
                pandoc::Inline::Math(pandoc::MathType::InlineMath, equation.expression),
                annotations,
            ),
        }
    }

    fn annotate(inline: pandoc::Inline, annotations: notion::Annotations) -> pandoc::Inline {
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
