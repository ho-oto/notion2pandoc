use chrono::{DateTime, Local};
use serde::{Deserialize, Deserializer};
use uuid::Uuid;

/// https://developers.notion.com/reference/intro
static NOTION_API_VERSION: &str = "2022-06-28";

// struct of Notion page

pub struct Page {
    pub blocks: Vec<Block>,
}

// struct of Notion blocks

fn deserialize_children<'de, D>(deserializer: D) -> Result<Option<Vec<Block>>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(if bool::deserialize(deserializer)? {
        Some(vec![])
    } else {
        None
    })
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct Block {
    pub id: Uuid,
    pub archived: bool,
    #[serde(flatten)]
    pub var: Var,
    #[serde(deserialize_with = "deserialize_children", rename = "has_children")]
    pub children: Option<Vec<Block>>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Var {
    Paragraph {
        #[serde(rename = "paragraph")]
        inline: Inline,
    },
    #[serde(rename = "heading_1")]
    Heading1 {
        #[serde(rename = "heading_1")]
        inline: Inline,
    },
    #[serde(rename = "heading_2")]
    Heading2 {
        #[serde(rename = "heading_2")]
        inline: Inline,
    },
    #[serde(rename = "heading_3")]
    Heading3 {
        #[serde(rename = "heading_3")]
        inline: Inline,
    },
    Quote {
        #[serde(rename = "quote")]
        inline: Inline,
    },

    Callout {
        callout: Callout,
    },

    BulletedListItem {
        #[serde(rename = "bulleted_list_item")]
        inline: Inline,
    },
    NumberedListItem {
        #[serde(rename = "numbered_list_item")]
        inline: Inline,
    },
    #[serde(rename = "to_do")]
    ToDoListItem {
        to_do: ToDo,
    },
    #[serde(rename = "toggle")]
    ToggleListItem {
        #[serde(rename = "toggle")]
        inline: Inline,
    },

    Code {
        code: Code,
    },
    Equation {
        equation: Equation,
    },

    Image {
        #[serde(rename = "image")]
        file: File,
    },
    Video {
        #[serde(rename = "video")]
        file: File,
    },
    File {
        #[serde(rename = "file")]
        file: File,
    },
    #[serde(rename = "pdf")]
    PDF {
        #[serde(rename = "pdf")]
        file: File,
    },

    Embed {
        embed: Embed,
    },
    Bookmark {
        #[serde(rename = "bookmark")]
        embed: Embed,
    },
    LinkPreview {
        link_preview: Link,
    },
    LinkToPage {
        link_to_page: LinkToPage,
    },

    Table {
        table: Table,
    },
    TableRow {
        table_row: TableRow,
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

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct Inline {
    pub rich_text: Vec<RichText>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct Callout {
    pub rich_text: Vec<RichText>,
    pub icon: Icon,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct ToDo {
    pub rich_text: Vec<RichText>,
    pub checked: bool,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct Code {
    pub rich_text: Vec<RichText>,
    pub caption: Vec<RichText>,
    pub language: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct Equation {
    pub expression: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum File {
    External {
        caption: Vec<RichText>,
        external: ExternalFileLink,
    },
    File {
        caption: Vec<RichText>,
        file: FileLink,
    },
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct Embed {
    pub caption: Vec<RichText>,
    pub url: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct Link {
    pub url: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LinkToPage {
    PageId { page_id: Uuid },
    DatabaseId { database_id: Uuid },
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Icon {
    Emoji { emoji: String },
    External { external: Link },
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct Emoji {
    pub emoji: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct ExternalFileLink {
    pub url: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct FileLink {
    pub url: String,
    pub expiry_time: DateTime<Local>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct Table {
    pub table_width: u64,
    pub has_column_header: bool,
    pub has_row_header: bool,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct TableRow {
    pub cells: Vec<Vec<RichText>>,
}

// common structs

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum RichText {
    Text {
        annotations: Annotations,
        text: Text,
    },
    Mention {
        plain_text: String,
        annotations: Annotations,
        mention: Mention,
    },
    Equation {
        annotations: Annotations,
        equation: Equation,
    },
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct Text {
    pub content: String,
    pub link: Option<Link>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Mention {
    Page { page: PageId },
    Date,
    User,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct PageId {
    pub id: Uuid,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct Annotations {
    pub bold: bool,
    pub italic: bool,
    pub strikethrough: bool,
    pub underline: bool,
    pub code: bool,
}

// API reqwest

use async_recursion::async_recursion;
use futures::future::join_all;
use itertools::join;
use reqwest::Client;

async fn fetch_blocks(id: Uuid, secret: &String) -> Vec<Block> {
    #[derive(Deserialize)]
    struct Response {
        has_more: bool,
        next_cursor: Option<String>,
        results: Vec<Block>,
    }

    let mut blocks = vec![];
    let mut has_more = true;
    let mut next_cursor = None;
    while has_more {
        let url = format!("https://api.notion.com/v1/blocks/{}/children", id);
        let params = next_cursor.map(|n| vec![("start_cursor", n)]);
        let page = Client::new()
            .get(&url)
            .query(&params)
            .header("Authorization", format!("Bearer {}", secret))
            .header("Notion-Version", NOTION_API_VERSION)
            .send()
            .await
            .unwrap_or_else(|_| panic!("failed to fetch children blocks of {}", id))
            .json::<Response>()
            .await
            .unwrap_or_else(|_| panic!("failed to deserialize children blocks of {}", id));
        next_cursor = page.next_cursor;
        has_more = page.has_more;
        blocks.extend(page.results.into_iter().filter(|x| !x.archived));
    }
    blocks
}

pub async fn fetch_meta(id: Uuid, secret: &String) -> (String, DateTime<Local>, DateTime<Local>) {
    #[derive(Deserialize)]
    struct Response {
        archived: bool,
        created_time: DateTime<Local>,
        last_edited_time: DateTime<Local>,
        properties: Properties,
    }
    #[derive(Deserialize)]
    struct Properties {
        title: Title,
    }
    #[derive(Deserialize)]
    struct Title {
        title: Vec<RichText>,
    }

    let url = format!("https://api.notion.com/v1/pages/{}", id);
    let meta = Client::new()
        .get(&url)
        .header("Authorization", format!("Bearer {}", secret))
        .header("Notion-Version", NOTION_API_VERSION)
        .send()
        .await
        .unwrap_or_else(|_| panic!("failed to fetch page {}", id))
        .json::<Response>()
        .await
        .unwrap_or_else(|_| panic!("failed to deserialize page {}", id));
    if meta.archived {
        panic!("archived page")
    }
    let title = join(
        meta.properties.title.title.into_iter().map(|r| match r {
            RichText::Text {
                annotations: _,
                text,
            } => text.content,
            _ => panic!("mention or equation in title"),
        }),
        "",
    );
    (title, meta.created_time, meta.last_edited_time)
}

fn flatten_paragraph_block(blocks: Vec<Block>) -> Vec<Block> {
    let mut result = vec![];
    for block in blocks {
        if let Some(children) = block.children {
            let mut flattened_children = flatten_paragraph_block(children);
            match block.var {
                Var::Paragraph { .. } => {
                    result.push(Block {
                        children: None,
                        ..block
                    });
                    result.append(&mut flattened_children);
                }
                _ => {
                    result.push(Block {
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

fn join_list_block(blocks: Vec<Block>) -> Vec<Block> {
    let mut result = vec![];
    for mut block in blocks {
        block.children = block.children.map(join_list_block);
        match (result.last_mut(), &block.var) {
            (
                Some(Block {
                    var: Var::BulletedList,
                    children: Some(children),
                    ..
                }),
                Var::BulletedListItem { .. }
                | Var::ToggleListItem { .. }
                | Var::ToDoListItem { .. },
            )
            | (
                Some(Block {
                    var: Var::NumberedList,
                    children: Some(children),
                    ..
                }),
                Var::NumberedListItem { .. },
            ) => {
                children.push(block);
            }
            (
                _,
                Var::BulletedListItem { .. }
                | Var::ToggleListItem { .. }
                | Var::ToDoListItem { .. },
            ) => result.push(Block {
                id: block.id,
                archived: false,
                children: Some(vec![block]),
                var: Var::BulletedList,
            }),
            (_, Var::NumberedListItem { .. }) => result.push(Block {
                id: block.id,
                archived: false,
                children: Some(vec![block]),
                var: Var::NumberedList,
            }),
            _ => {
                result.push(block);
            }
        };
    }
    result
}

impl Page {
    pub async fn fetch(id: Uuid, secret: &String) -> Self {
        let mut blocks = fetch_blocks(id, secret).await;
        join_all(
            blocks
                .iter_mut()
                .map(|x| async { x.fetch_recursive(secret).await }),
        )
        .await;
        blocks = join_list_block(flatten_paragraph_block(blocks));
        Self { blocks }
    }

    pub fn has_toc(&self) -> bool {
        self.blocks.iter().any(|x| x.has_toc())
    }
}

impl Block {
    #[async_recursion]
    pub async fn fetch_recursive(&mut self, secret: &String) {
        if let Some(_) = self.children {
            let mut children = fetch_blocks(self.id, secret).await;
            join_all(
                children
                    .iter_mut()
                    .map(|x| async { x.fetch_recursive(secret).await }),
            )
            .await;
            self.children = Some(children);
        }
    }

    pub fn has_toc(&self) -> bool {
        let mut result = match &self.var {
            Var::TableOfContents => true,
            _ => false,
        };
        if let Some(children) = &self.children {
            result |= children.iter().any(|b| b.has_toc());
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_flatten_paragraph_block() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();
        let id4 = Uuid::new_v4();
        let id5 = Uuid::new_v4();
        let id6 = Uuid::new_v4();
        let id7 = Uuid::new_v4();
        let id8 = Uuid::new_v4();
        let id9 = Uuid::new_v4();

        let var1 = Var::Paragraph {
            inline: Inline { rich_text: vec![] },
        };
        let var2 = Var::Paragraph {
            inline: Inline { rich_text: vec![] },
        };
        let var3 = Var::Quote {
            inline: Inline { rich_text: vec![] },
        };
        let var4 = Var::Paragraph {
            inline: Inline { rich_text: vec![] },
        };
        let var5 = Var::Quote {
            inline: Inline { rich_text: vec![] },
        };
        let var6 = Var::Paragraph {
            inline: Inline { rich_text: vec![] },
        };
        let var7 = Var::Divider;
        let var8 = Var::Paragraph {
            inline: Inline { rich_text: vec![] },
        };
        let var9 = Var::Paragraph {
            inline: Inline { rich_text: vec![] },
        };

        let input = vec![Block {
            archived: false,
            id: id1,
            var: var1.clone(),
            children: Some(vec![
                Block {
                    archived: false,
                    id: id2,
                    var: var2.clone(),
                    children: None,
                },
                Block {
                    archived: false,
                    id: id3,
                    var: var3.clone(), // Quote
                    children: Some(vec![Block {
                        archived: false,
                        id: id4,
                        var: var4.clone(),
                        children: Some(vec![
                            Block {
                                archived: false,
                                id: id5,
                                var: var5.clone(), // Quote
                                children: Some(vec![Block {
                                    archived: false,
                                    id: id6,
                                    var: var6.clone(),
                                    children: None,
                                }]),
                            },
                            Block {
                                archived: false,
                                id: id7,
                                var: var7.clone(), // Divider
                                children: None,
                            },
                            Block {
                                archived: false,
                                id: id8,
                                var: var8.clone(),
                                children: Some(vec![Block {
                                    archived: false,
                                    id: id9,
                                    var: var9.clone(),
                                    children: None,
                                }]),
                            },
                        ]),
                    }]),
                },
            ]),
        }];

        let expect = vec![
            Block {
                archived: false,
                id: id1,
                var: var1,
                children: None,
            },
            Block {
                archived: false,
                id: id2,
                var: var2,
                children: None,
            },
            Block {
                archived: false,
                id: id3,
                var: var3, // Quote
                children: Some(vec![
                    Block {
                        archived: false,
                        id: id4,
                        var: var4,
                        children: None,
                    },
                    Block {
                        archived: false,
                        id: id5,
                        var: var5, // Quote
                        children: Some(vec![Block {
                            archived: false,
                            id: id6,
                            var: var6,
                            children: None,
                        }]),
                    },
                    Block {
                        archived: false,
                        id: id7,
                        var: var7, // Divider
                        children: None,
                    },
                    Block {
                        archived: false,
                        id: id8,
                        var: var8,
                        children: None,
                    },
                    Block {
                        archived: false,
                        id: id9,
                        var: var9,
                        children: None,
                    },
                ]),
            },
        ];

        let result = flatten_paragraph_block(input);
        assert_eq!(result, expect);
    }

    #[test]
    fn test_join_list_block() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();
        let id4 = Uuid::new_v4();
        let id5 = Uuid::new_v4();
        let id6 = Uuid::new_v4();
        let id7 = Uuid::new_v4();
        let id8 = Uuid::new_v4();
        let id9 = Uuid::new_v4();

        let var1 = Var::BulletedListItem {
            inline: Inline { rich_text: vec![] },
        };
        let var2 = Var::BulletedListItem {
            inline: Inline { rich_text: vec![] },
        };
        let var3 = Var::ToggleListItem {
            inline: Inline { rich_text: vec![] },
        };
        let var4 = Var::NumberedListItem {
            inline: Inline { rich_text: vec![] },
        };
        let var5 = Var::NumberedListItem {
            inline: Inline { rich_text: vec![] },
        };
        let var6 = Var::ToggleListItem {
            inline: Inline { rich_text: vec![] },
        };
        let var7 = Var::NumberedListItem {
            inline: Inline { rich_text: vec![] },
        };
        let var8 = Var::NumberedListItem {
            inline: Inline { rich_text: vec![] },
        };
        let var9 = Var::BulletedListItem {
            inline: Inline { rich_text: vec![] },
        };

        let input = vec![
            Block {
                archived: false,
                id: id1,
                var: var1.clone(),
                children: None,
            },
            Block {
                archived: false,
                id: id2,
                var: var2.clone(),
                children: None,
            },
            Block {
                archived: false,
                id: id3,
                var: var3.clone(),
                children: None,
            },
            Block {
                archived: false,
                id: id4,
                var: var4.clone(), // NumberedListItem
                children: None,
            },
            Block {
                archived: false,
                id: id5,
                var: var5.clone(), // NumberedListItem
                children: None,
            },
            Block {
                archived: false,
                id: id6,
                var: var6.clone(),
                children: Some(vec![
                    Block {
                        archived: false,
                        id: id7,
                        var: var7.clone(), // NumberedListItem
                        children: None,
                    },
                    Block {
                        archived: false,
                        id: id8,
                        var: var8.clone(), // NumberedListItem
                        children: None,
                    },
                    Block {
                        archived: false,
                        id: id9,
                        var: var9.clone(),
                        children: None,
                    },
                ]),
            },
        ];

        let expect = vec![
            Block {
                archived: false,
                id: id1,
                var: Var::BulletedList,
                children: Some(vec![
                    Block {
                        archived: false,
                        id: id1,
                        var: var1,
                        children: None,
                    },
                    Block {
                        archived: false,
                        id: id2,
                        var: var2,
                        children: None,
                    },
                    Block {
                        archived: false,
                        id: id3,
                        var: var3,
                        children: None,
                    },
                ]),
            },
            Block {
                archived: false,
                id: id4,
                var: Var::NumberedList,
                children: Some(vec![
                    Block {
                        archived: false,
                        id: id4,
                        var: var4,
                        children: None,
                    },
                    Block {
                        archived: false,
                        id: id5,
                        var: var5,
                        children: None,
                    },
                ]),
            },
            Block {
                archived: false,
                id: id6,
                var: Var::BulletedList,
                children: Some(vec![Block {
                    archived: false,
                    id: id6,
                    var: var6,
                    children: Some(vec![
                        Block {
                            archived: false,
                            id: id7,
                            var: Var::NumberedList,
                            children: Some(vec![
                                Block {
                                    archived: false,
                                    id: id7,
                                    var: var7, // NumberedListItem
                                    children: None,
                                },
                                Block {
                                    archived: false,
                                    id: id8,
                                    var: var8, // NumberedListItem
                                    children: None,
                                },
                            ]),
                        },
                        Block {
                            archived: false,
                            id: id9,
                            var: Var::BulletedList,
                            children: Some(vec![Block {
                                archived: false,
                                id: id9,
                                var: var9,
                                children: None,
                            }]),
                        },
                    ]),
                }]),
            },
        ];

        let result = join_list_block(input);
        assert_eq!(result, expect);
    }

    #[test]
    fn test_has_toc() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        let var1 = Var::Paragraph {
            inline: Inline { rich_text: vec![] },
        };
        let var2 = Var::TableOfContents;

        let input = Page {
            blocks: vec![Block {
                archived: false,
                id: id1,
                var: var1.clone(),
                children: Some(vec![Block {
                    archived: false,
                    id: id2,
                    var: var2,
                    children: None,
                }]),
            }],
        };

        assert!(input.has_toc());

        let input = Page {
            blocks: vec![Block {
                archived: false,
                id: id1,
                var: var1.clone(),
                children: Some(vec![Block {
                    archived: false,
                    id: id2,
                    var: var1,
                    children: None,
                }]),
            }],
        };

        assert!(!input.has_toc());
    }
}
