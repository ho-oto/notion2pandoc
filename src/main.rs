mod notion;
mod pandoc;

use std::collections::HashMap;

use clap::Parser;
use itertools::join;
use uuid::Uuid;
extern crate openssl_probe;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short = 'i')]
    id: String,
    #[clap(short = 's')]
    secret: String,
}

#[tokio::main]
async fn main() {
    openssl_probe::init_ssl_cert_env_vars();
    let args = Args::parse();
    let id = Uuid::parse_str(&args.id).expect("ID should be UUID");
    let page = notion::Page::fetch(id, &args.secret).await;
    let (title, date, lastmod) = notion::fetch_meta(id, &args.secret).await;
    let rsl = pandoc::Pandoc {
        pandoc_api_version: pandoc::PANDOC_API_VERSION,
        meta: pandoc::Meta(HashMap::from_iter([
            (
                "date".to_string(),
                pandoc::MetaValue::MetaString(date.date_naive().to_string()),
            ),
            (
                "lastmod".to_string(),
                pandoc::MetaValue::MetaString(lastmod.date_naive().to_string()),
            ),
            ("title".to_string(), pandoc::MetaValue::MetaString(title)),
            (
                "toc".to_string(),
                pandoc::MetaValue::MetaBool(page.has_toc()),
            ),
        ])),
        blocks: page
            .blocks
            .into_iter()
            .filter_map(|b| b.to_pandoc())
            .collect(),
    };
    println!(
        "{}",
        serde_json::to_string(&rsl).expect("failed to serialize")
    )
}

impl notion::Block {
    fn to_pandoc(self) -> Option<pandoc::Block> {
        match self.var {
            notion::Var::Paragraph { inline } => Some(pandoc::Block::Para(inline.to_pandoc())),
            notion::Var::Heading1 { inline } => Some(pandoc::Block::Header(
                2,
                pandoc::Attr::default(),
                inline.to_pandoc(),
            )),
            notion::Var::Heading2 { inline } => Some(pandoc::Block::Header(
                3,
                pandoc::Attr::default(),
                inline.to_pandoc(),
            )),
            notion::Var::Heading3 { inline } => Some(pandoc::Block::Header(
                4,
                pandoc::Attr::default(),
                inline.to_pandoc(),
            )),
            notion::Var::Quote { inline } => Some(pandoc::Block::BlockQuote(
                inline.to_pandoc_with_children(self.children),
            )),

            notion::Var::Callout { callout } => Some(pandoc::Block::Div(
                pandoc::Attr("".to_string(), vec!["callout".to_string()], vec![]),
                vec![pandoc::Block::Plain(
                    callout
                        .rich_text
                        .into_iter()
                        .map(|r| r.to_pandoc())
                        .collect(),
                )],
            )),

            // {Bulleted, Numbered, ToDo, Toggle}ListItem should be
            // in a children of BulletedList/NumberedList node
            notion::Var::BulletedListItem { .. }
            | notion::Var::NumberedListItem { .. }
            | notion::Var::ToDoListItem { .. }
            | notion::Var::ToggleListItem { .. } => panic!("list item in top-level"),

            notion::Var::Code { code } => {
                let text = join(
                    code.rich_text.into_iter().map(|r| match r {
                        notion::RichText::Text {
                            annotations: _,
                            text,
                        } => text.content,
                        _ => panic!("mention or equation in title"),
                    }),
                    "",
                );
                Some(pandoc::Block::CodeBlock(
                    pandoc::Attr("".to_string(), vec![code.language], vec![]),
                    text,
                ))
            }
            notion::Var::Equation { equation } => {
                Some(pandoc::Block::Para(vec![pandoc::Inline::Math(
                    pandoc::MathType::DisplayMath,
                    equation.expression,
                )]))
            }

            notion::Var::Image { file } => {
                let (caption, url, loc) = Self::unpack_file(file);
                Some(pandoc::Block::Para(vec![pandoc::Inline::Image(
                    pandoc::Attr("".to_string(), vec![loc], vec![]),
                    caption.into_iter().map(|r| r.to_pandoc()).collect(),
                    pandoc::Target(url, "".to_string()),
                )]))
            }
            notion::Var::Video { file } => {
                let (caption, url, loc) = Self::unpack_file(file);
                Some(Self::link(url, caption, vec!["video".to_string(), loc]))
            }
            notion::Var::File { file } => {
                let (caption, url, loc) = Self::unpack_file(file);
                Some(Self::link(url, caption, vec!["file".to_string(), loc]))
            }
            notion::Var::PDF { file } => {
                let (caption, url, loc) = Self::unpack_file(file);
                Some(Self::link(url, caption, vec!["pdf".to_string(), loc]))
            }

            notion::Var::Embed { embed } | notion::Var::Bookmark { embed } => Some(Self::link(
                embed.url,
                embed.caption,
                vec!["embed".to_string()],
            )),
            notion::Var::LinkPreview { link_preview } => {
                Some(pandoc::Block::Para(vec![pandoc::Inline::Str(
                    link_preview.url.clone(),
                )
                .to_link(link_preview.url)]))
            }
            notion::Var::LinkToPage { link_to_page } => match link_to_page {
                notion::LinkToPage::PageId { page_id } => Some(pandoc::Block::Div(
                    pandoc::Attr(
                        "".to_string(),
                        vec!["link_to_page".to_string()],
                        vec![("id".to_string(), page_id.to_string())],
                    ),
                    vec![],
                )),
                notion::LinkToPage::DatabaseId { .. } => None,
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
                    Some(pandoc::Block::Table(
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
                    ))
                } else {
                    None
                }
            }
            notion::Var::TableRow { .. } => panic!("table row in top level"),

            notion::Var::Divider => Some(pandoc::Block::HorizontalRule),
            notion::Var::TableOfContents => None,

            notion::Var::BulletedList => Some(pandoc::Block::BulletList(
                self.children
                    .expect("bulleted list should have children")
                    .into_iter()
                    .map(Self::convert_list_item)
                    .collect(),
            )),
            notion::Var::NumberedList => Some(pandoc::Block::OrderedList(
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
            )),

            _ => None,
        }
    }

    fn unpack_file(file: notion::File) -> (Vec<notion::RichText>, String, String) {
        match file {
            notion::File::File { caption, file } => (caption, file.url, "internal".to_string()),
            notion::File::External { caption, external } => {
                (caption, external.url, "external".to_string())
            }
        }
    }

    fn link(url: String, cap: Vec<notion::RichText>, attr: Vec<String>) -> pandoc::Block {
        let caption = if cap.is_empty() {
            vec![pandoc::Inline::Str(url.clone())]
        } else {
            cap.into_iter().map(|r| r.to_pandoc()).collect()
        };
        pandoc::Block::Para(vec![pandoc::Inline::Link(
            pandoc::Attr("".to_string(), attr, vec![]),
            caption,
            pandoc::Target(url, "".to_string()),
        )])
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
                    result.extend(children.into_iter().filter_map(|b| b.to_pandoc()));
                }
                result
            }
            _ => panic!("child of list should be a list item"),
        }
    }
}

impl notion::Inline {
    fn to_pandoc(self) -> Vec<pandoc::Inline> {
        let mut result = vec![];
        for inline in self.rich_text.into_iter().map(|r| r.to_pandoc()) {
            if let Some(pandoc::Inline::Link(attr_last, vec_last, trg_last)) = result.last_mut() {
                if let pandoc::Inline::Link(attr, mut vec, trg) = inline {
                    if *attr_last == attr && *trg_last == trg {
                        vec_last.append(&mut vec);
                    }
                }
            } else {
                result.push(inline);
            }
        }
        result
    }

    fn to_pandoc_with_children(self, children: Option<Vec<notion::Block>>) -> Vec<pandoc::Block> {
        let mut result = vec![pandoc::Block::Plain(self.to_pandoc())];
        if let Some(children) = children {
            result.extend(children.into_iter().filter_map(|b| b.to_pandoc()));
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
                annotations,
                mention,
                ..
            } => Self::annotate(
                pandoc::Inline::Span(
                    if let notion::Mention::Page { page } = mention {
                        pandoc::Attr(
                            "".to_string(),
                            vec!["link_to_page".to_string()],
                            vec![("id".to_string(), page.id.to_string())],
                        )
                    } else {
                        pandoc::Attr(
                            "".to_string(),
                            vec!["unsupported".to_string(), "mention".to_string()],
                            vec![],
                        )
                    },
                    vec![],
                ),
                annotations,
            ),
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
