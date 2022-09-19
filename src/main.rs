mod notion;
mod pandoc;

use clap::Parser;
use uuid::Uuid;

/// https://hackage.haskell.org/package/pandoc-types-1.22.2.1/docs/Text-Pandoc-Definition.html
static PANDOC_API_VERSION: [u64; 4] = [1, 22, 2, 1];

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(short = 'i')]
    id: String,
    #[clap(short = 's')]
    secret: String,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    let id = Uuid::parse_str(&args.id).expect("ID should be UUID");
    let page = notion::Page::fetch(id, &args.secret).await;
    let rsl = pandoc::Pandoc {
        pandoc_api_version: PANDOC_API_VERSION,
        meta: pandoc::Meta {},
        blocks: page.blocks.into_iter().map(|b| b.to_pandoc()).collect(),
    };
    println!(
        "{}",
        serde_json::to_string(&rsl).expect("failed to serialize")
    )
}

impl notion::Block {
    fn to_pandoc(self) -> pandoc::Block {
        match self.var {
            notion::Var::Paragraph { inline } => pandoc::Block::Para(inline.to_pandoc()),
            notion::Var::Heading1 { inline } => {
                pandoc::Block::Header(2, pandoc::Attr::default(), inline.to_pandoc())
            }
            notion::Var::Heading2 { inline } => {
                pandoc::Block::Header(3, pandoc::Attr::default(), inline.to_pandoc())
            }
            notion::Var::Heading3 { inline } => {
                pandoc::Block::Header(4, pandoc::Attr::default(), inline.to_pandoc())
            }
            notion::Var::Quote { inline } => {
                pandoc::Block::BlockQuote(inline.to_pandoc_with_children(self.children))
            }

            notion::Var::Callout { callout } => pandoc::Block::Div(
                pandoc::Attr("".to_string(), vec!["callout".to_string()], vec![]),
                vec![pandoc::Block::Plain(
                    callout
                        .rich_text
                        .into_iter()
                        .map(|r| r.to_pandoc())
                        .collect(),
                )],
            ),

            // {Bulleted, Numbered, ToDo, Toggle}ListItem should be
            // in a children of BulletedList/NumberedList node
            notion::Var::BulletedListItem { .. }
            | notion::Var::NumberedListItem { .. }
            | notion::Var::ToDoListItem { .. }
            | notion::Var::ToggleListItem { .. } => panic!("list item in top-level"),

            notion::Var::Code { code } => {
                assert!(code.rich_text.len() == 1);
                let text = match code.rich_text.first() {
                    Some(notion::RichText::Text {
                        annotations: _,
                        text,
                    }) => text.content.clone(),
                    _ => panic!("mention or equation in code"),
                };
                pandoc::Block::CodeBlock(
                    pandoc::Attr("".to_string(), vec![code.language], vec![]),
                    text,
                )
            }
            notion::Var::Equation { equation } => pandoc::Block::Para(vec![pandoc::Inline::Math(
                pandoc::MathType::DisplayMath,
                equation.expression,
            )]),

            notion::Var::Image { file } => {
                let (caption, url) = match file {
                    notion::File::File { caption, file } => (caption, file.url),
                    notion::File::External { caption, external } => (caption, external.url),
                };
                let caption = caption.into_iter().map(|r| r.to_pandoc()).collect();
                pandoc::Block::Para(vec![pandoc::Inline::Image(
                    pandoc::Attr::default(),
                    caption,
                    pandoc::Target(url, "".to_string()),
                )])
            }
            notion::Var::Video { file }
            | notion::Var::File { file }
            | notion::Var::PDF { file } => {
                let (caption, url) = match file {
                    notion::File::File { caption, file } => (caption, file.url),
                    notion::File::External { caption, external } => (caption, external.url),
                };
                let caption = if caption.is_empty() {
                    vec![pandoc::Inline::Str(url.clone())]
                } else {
                    caption.into_iter().map(|r| r.to_pandoc()).collect()
                };
                pandoc::Block::Para(vec![pandoc::Inline::Link(
                    pandoc::Attr("".to_string(), vec!["file".to_string()], vec![]),
                    caption,
                    pandoc::Target(url, "".to_string()),
                )])
            }

            notion::Var::Embed { embed } | notion::Var::Bookmark { embed } => {
                let caption = if embed.caption.is_empty() {
                    vec![pandoc::Inline::Str(embed.url.clone())]
                } else {
                    embed.caption.into_iter().map(|r| r.to_pandoc()).collect()
                };
                pandoc::Block::Para(vec![pandoc::Inline::Link(
                    pandoc::Attr("".to_string(), vec!["embed".to_string()], vec![]),
                    caption,
                    pandoc::Target(embed.url, "".to_string()),
                )])
            }
            notion::Var::LinkPreview { link_preview } => pandoc::Block::Para(vec![
                pandoc::Inline::Str(link_preview.url.clone()).to_link(link_preview.url),
            ]),
            notion::Var::LinkToPage { link_to_page } => match link_to_page {
                notion::LinkToPage::PageId { page_id } => pandoc::Block::Div(
                    pandoc::Attr(
                        "".to_string(),
                        vec!["link_to_page".to_string()],
                        vec![("id".to_string(), page_id.to_string())],
                    ),
                    vec![],
                ),
                notion::LinkToPage::DatabaseId { .. } => pandoc::Block::Null,
            },

            notion::Var::Table { table } => {
                if let Some(children) = self.children {
                    let header_start = if table.has_column_header { 1 } else { 0 };
                    let mut header = children;
                    let body = header.split_off(header_start);
                    let header: Vec<pandoc::Row> =
                        header.into_iter().map(Self::convert_table_row).collect();
                    let body: Vec<pandoc::Row> =
                        body.into_iter().map(Self::convert_table_row).collect();
                    let col_specs = (0..table.table_width)
                        .map(|_| pandoc::ColSpec::default())
                        .collect();
                    pandoc::Block::Table(
                        pandoc::Attr::default(),
                        pandoc::Caption::default(),
                        col_specs,
                        pandoc::TableHead(pandoc::Attr::default(), header),
                        vec![pandoc::TableBody(
                            pandoc::Attr::default(),
                            pandoc::RowHeadColumns(0),
                            vec![],
                            body,
                        )],
                        pandoc::TableFoot::default(),
                    )
                } else {
                    pandoc::Block::Null
                }
            }
            notion::Var::TableRow { .. } => panic!("table row in top level"),

            notion::Var::Divider => pandoc::Block::HorizontalRule,
            notion::Var::TableOfContents => pandoc::Block::Null,

            notion::Var::BulletedList => pandoc::Block::BulletList(
                self.children
                    .expect("bulleted list should have children")
                    .into_iter()
                    .map(Self::convert_list_item)
                    .collect(),
            ),
            notion::Var::NumberedList => pandoc::Block::OrderedList(
                pandoc::ListAttributes(
                    1,
                    pandoc::ListNumberStyle::Decimal,
                    pandoc::ListNumberDelim::Period,
                ),
                self.children
                    .expect("numbered list should have children")
                    .into_iter()
                    .map(Self::convert_list_item)
                    .collect(),
            ),

            _ => pandoc::Block::Null,
        }
    }

    fn convert_table_row(x: notion::Block) -> pandoc::Row {
        match x.var {
            notion::Var::TableRow { table_row } => Self::convert_table_cells(table_row.cells),
            _ => panic!("child of table should be a table row"),
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
        match x.var {
            notion::Var::BulletedListItem { inline }
            | notion::Var::NumberedListItem { inline }
            | notion::Var::ToggleListItem { inline } => inline.to_pandoc_with_children(x.children),
            notion::Var::ToDoListItem { to_do } => {
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
            _ => panic!("child of list should be a list item"),
        }
    }
}

impl notion::Inline {
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
                        text: notion::Text { link: None, ..text },
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
                ..
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
            result = pandoc::Inline::Emph(vec![result]);
        }
        if annotations.strikethrough {
            result = pandoc::Inline::Strikeout(vec![result]);
        }
        result
    }
}
