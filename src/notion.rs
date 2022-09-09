use chrono::{DateTime, Local};
use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

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
    pub variant: BlockVariant,
    #[serde(deserialize_with = "deserialize_children", rename = "has_children")]
    pub children: Option<Vec<Block>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BlockVariant {
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InlineContent {
    pub rich_text: Vec<RichText>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CalloutContent {
    pub rich_text: Vec<RichText>,
    pub icon: Icon,
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
pub struct ToDoContent {
    pub rich_text: Vec<RichText>,
    pub checked: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmbedContent {
    pub caption: Vec<RichText>,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CodeContent {
    pub rich_text: Vec<RichText>,
    pub caption: Vec<RichText>,
    pub language: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum FileContent {
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
pub struct ExternalFileLink {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileLink {
    pub url: String,
    pub expiry_time: DateTime<Local>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LinkToPageContent {
    PageId { page_id: Uuid },
    DatabaseId { database_id: Uuid },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TableContent {
    pub table_width: u64,
    pub has_column_header: bool,
    pub has_row_header: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TableRowContent {
    pub cells: Vec<Vec<RichText>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum RichText {
    Text {
        annotations: Annotations,
        text: TextContent,
    },
    Mention {
        plain_text: String,
        annotations: Annotations,
        mention: MentionContent,
    },
    Equation {
        annotations: Annotations,
        equation: EquationContent,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TextContent {
    pub content: String,
    pub link: Option<Link>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Link {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum MentionContent {
    Page { page: PageId },
    Date,
    User,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PageId {
    pub id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EquationContent {
    pub expression: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Annotations {
    pub bold: bool,
    pub italic: bool,
    pub strikethrough: bool,
    pub underline: bool,
    pub code: bool,
}
