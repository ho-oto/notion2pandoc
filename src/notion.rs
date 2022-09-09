use chrono::{DateTime, Local};
use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

// https://developers.notion.com/reference/intro

// struct of Notion blocks

fn deserialize_children<'de, D>(deserializer: D) -> Result<Option<Vec<Block>>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(if bool::deserialize(deserializer)? {
        Some(Vec::<Block>::new())
    } else {
        None
    })
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block {
    pub id: Uuid,
    pub archived: bool,
    #[serde(flatten)]
    pub var: Var,
    #[serde(deserialize_with = "deserialize_children", rename = "has_children")]
    pub children: Option<Vec<Block>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Inline {
    pub rich_text: Vec<RichText>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Callout {
    pub rich_text: Vec<RichText>,
    pub icon: Icon,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToDo {
    pub rich_text: Vec<RichText>,
    pub checked: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Code {
    pub rich_text: Vec<RichText>,
    pub caption: Vec<RichText>,
    pub language: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Equation {
    pub expression: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Embed {
    pub caption: Vec<RichText>,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Link {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LinkToPage {
    PageId { page_id: Uuid },
    DatabaseId { database_id: Uuid },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Icon {
    Emoji { emoji: String },
    External { external: Link },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Emoji {
    pub emoji: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExternalFileLink {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileLink {
    pub url: String,
    pub expiry_time: DateTime<Local>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Table {
    pub table_width: u64,
    pub has_column_header: bool,
    pub has_row_header: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TableRow {
    pub cells: Vec<Vec<RichText>>,
}

// common structs

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Text {
    pub content: String,
    pub link: Option<Link>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Mention {
    Page { page: PageId },
    Date,
    User,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PageId {
    pub id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Annotations {
    pub bold: bool,
    pub italic: bool,
    pub strikethrough: bool,
    pub underline: bool,
    pub code: bool,
}
