// use reqwest::Client;
use serde::{Deserialize, Serialize};

// #[tokio::main]
// async
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let url = url_notion_blocks("XXX".to_string());
    // let client = Client::new()
    //     .get(&url)
    //     .header("Authorization", format!("Bearer {}", "secret_XXX"))
    //     .header("Notion-Version", "2022-06-28");

    let sample_json = r###"
    {"object":"list","results":[
        {"object":"block","id":"3ed23ff7-8cfb-4466-96a2-d8ad7afb6419","parent":{"type":"page_id","page_id":"c90565cf-4ae6-4e3d-bfdb-b9140b1f8b04"},"created_time":"2022-08-28T08:20:00.000Z","last_edited_time":"2022-08-28T08:22:00.000Z","created_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"last_edited_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"has_children":false,"archived":false,
        "type":"paragraph",
        "paragraph":
            {"rich_text":[{"type":"text","text":{"content":"aaa","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"aaa","href":null},{"type":"text","text":{"content":"bbb","link":null},"annotations":{"bold":true,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"bbb","href":null},{"type":"text","text":{"content":"ccc","link":null},"annotations":{"bold":true,"italic":false,"strikethrough":false,"underline":true,"code":false,"color":"default"},"plain_text":"ccc","href":null},{"type":"text","text":{"content":"ddd","link":null},"annotations":{"bold":true,"italic":false,"strikethrough":true,"underline":true,"code":false,"color":"default"},"plain_text":"ddd","href":null},{"type":"text","text":{"content":"eee","link":null},"annotations":{"bold":true,"italic":false,"strikethrough":true,"underline":true,"code":true,"color":"default"},"plain_text":"eee","href":null},{"type":"text","text":{"content":"fff","link":null},"annotations":{"bold":true,"italic":true,"strikethrough":true,"underline":true,"code":true,"color":"default"},"plain_text":"fff","href":null},{"type":"equation","equation":{"expression":"E=mc^2"},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"E=mc^2","href":null},{"type":"text","text":{"content":"selflink","link":{"url":"/c90565cf4ae64e3dbfdbb9140b1f8b04"}},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"selflink","href":"/c90565cf4ae64e3dbfdbb9140b1f8b04"},{"type":"text","text":{"content":" ","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":" ","href":null},{"type":"mention","mention":{"type":"date","date":{"start":"2022-08-28","end":null,"time_zone":null}},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"2022-08-28 → ","href":null},{"type":"text","text":{"content":" ","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":" ","href":null},{"type":"mention","mention":{"type":"user","user":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"}},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"@Anonymous","href":null},{"type":"text","text":{"content":" ","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":" ","href":null},{"type":"mention","mention":{"type":"page","page":{"id":"c90565cf-4ae6-4e3d-bfdb-b9140b1f8b04"}},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"sample","href":"https://www.notion.so/c90565cf4ae64e3dbfdbb9140b1f8b04"},{"type":"text","text":{"content":" ","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":" ","href":null},{"type":"text","text":{"content":"foo","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"foo","href":null}],
            "color":"default"}},
        {"object":"block","id":"6f4dd1bf-e41c-4412-b6cd-3368b6bd7508","parent":{"type":"page_id","page_id":"c90565cf-4ae6-4e3d-bfdb-b9140b1f8b04"},"created_time":"2022-08-28T08:26:00.000Z","last_edited_time":"2022-08-28T08:26:00.000Z","created_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"last_edited_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"has_children":false,"archived":false,
        "type":"child_page",
        "child_page":
            {"title":""}},
        {"object":"block","id":"95eeded3-4e6f-4271-9751-1e9aed6594a6","parent":{"type":"page_id","page_id":"c90565cf-4ae6-4e3d-bfdb-b9140b1f8b04"},"created_time":"2022-08-28T08:27:00.000Z","last_edited_time":"2022-08-28T08:27:00.000Z","created_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"last_edited_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"has_children":false,"archived":false,
        "type":"to_do",
        "to_do":
            {"rich_text":[{"type":"text","text":{"content":"foo","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"foo","href":null}],
            "checked":false,
            "color":"default"}},
        {"object":"block","id":"1a990466-2a4f-4e24-a220-a6e77249528a","parent":{"type":"page_id","page_id":"c90565cf-4ae6-4e3d-bfdb-b9140b1f8b04"},"created_time":"2022-08-28T08:27:00.000Z","last_edited_time":"2022-08-28T08:28:00.000Z","created_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"last_edited_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"has_children":true,"archived":false,
        "type":"to_do",
        "to_do":
            {"rich_text":[{"type":"text","text":{"content":"bar","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"bar","href":null}],
            "checked":false,
            "color":"default"}},
        {"object":"block","id":"cdb44fc2-8c80-4f5d-aef0-42e11dfad18c","parent":{"type":"page_id","page_id":"c90565cf-4ae6-4e3d-bfdb-b9140b1f8b04"},"created_time":"2022-08-28T08:28:00.000Z","last_edited_time":"2022-08-28T08:28:00.000Z","created_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"last_edited_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"has_children":false,"archived":false,
        "type":"heading_1",
        "heading_1":
            {"rich_text":[{"type":"text","text":{"content":"h1","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"h1","href":null},{"type":"text","text":{"content":"h1","link":null},"annotations":{"bold":true,"italic":true,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"h1","href":null}],
            "is_toggleable":false,
            "color":"default"}},
        {"object":"block","id":"5a6f9cf2-922e-4b60-9ab1-8b75daded097","parent":{"type":"page_id","page_id":"c90565cf-4ae6-4e3d-bfdb-b9140b1f8b04"},"created_time":"2022-08-28T08:28:00.000Z","last_edited_time":"2022-08-28T08:29:00.000Z","created_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"last_edited_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"has_children":false,"archived":false,
        "type":"heading_2",
        "heading_2":
            {"rich_text":[{"type":"text","text":{"content":"h2","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"h2","href":null},{"type":"text","text":{"content":"h2","link":null},"annotations":{"bold":false,"italic":true,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"h2","href":null}],
            "is_toggleable":false,
            "color":"default"}},
        {"object":"block","id":"deda4803-4844-4582-a8c0-4f7ea0bd7001","parent":{"type":"page_id","page_id":"c90565cf-4ae6-4e3d-bfdb-b9140b1f8b04"},"created_time":"2022-08-28T08:29:00.000Z","last_edited_time":"2022-08-28T08:29:00.000Z","created_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"last_edited_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"has_children":false,"archived":false,
        "type":"heading_3",
        "heading_3":
            {"rich_text":[{"type":"text","text":{"content":"h3","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"h3","href":null},{"type":"text","text":{"content":"h3","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":true,"code":false,"color":"default"},"plain_text":"h3","href":null}],
            "is_toggleable":false,
            "color":"default"}},
        {"object":"block","id":"4d1ef2eb-a377-4b0e-95fe-67b184ceb400","parent":{"type":"page_id","page_id":"c90565cf-4ae6-4e3d-bfdb-b9140b1f8b04"},"created_time":"2022-08-28T08:29:00.000Z","last_edited_time":"2022-08-28T08:29:00.000Z","created_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"last_edited_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"has_children":true,"archived":false,
        "type":"table",
        "table":
            {"table_width":2,
            "has_column_header":true,
            "has_row_header":true}},
        {"object":"block","id":"ae525198-2b3c-4037-b11a-ad869476f59f","parent":{"type":"page_id","page_id":"c90565cf-4ae6-4e3d-bfdb-b9140b1f8b04"},"created_time":"2022-08-28T08:29:00.000Z","last_edited_time":"2022-08-28T08:29:00.000Z","created_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"last_edited_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"has_children":false,"archived":false,
        "type":"bulleted_list_item",
        "bulleted_list_item":
            {"rich_text":[{"type":"text","text":{"content":"foo","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"foo","href":null}],
            "color":"default"}},
        {"object":"block","id":"9edf03d7-161b-48e2-92ac-7c84e3941fa6","parent":{"type":"page_id","page_id":"c90565cf-4ae6-4e3d-bfdb-b9140b1f8b04"},"created_time":"2022-08-28T08:29:00.000Z","last_edited_time":"2022-08-28T08:29:00.000Z","created_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"last_edited_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"has_children":true,"archived":false,
        "type":"bulleted_list_item",
        "bulleted_list_item":
            {"rich_text":[{"type":"text","text":{"content":"bar","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"bar","href":null}],
            "color":"default"}},
        {"object":"block","id":"81c0bc59-3db2-4805-8be2-4867e1733240","parent":{"type":"page_id","page_id":"c90565cf-4ae6-4e3d-bfdb-b9140b1f8b04"},"created_time":"2022-08-28T08:30:00.000Z","last_edited_time":"2022-08-28T08:30:00.000Z","created_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"last_edited_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"has_children":false,"archived":false,
        "type":"numbered_list_item",
        "numbered_list_item":
            {"rich_text":[{"type":"text","text":{"content":"Foo","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"Foo","href":null}],
            "color":"default"}},
        {"object":"block","id":"a463e75e-6ac6-482e-81a6-dc0a04a53b6c","parent":{"type":"page_id","page_id":"c90565cf-4ae6-4e3d-bfdb-b9140b1f8b04"},"created_time":"2022-08-28T08:30:00.000Z","last_edited_time":"2022-08-28T08:30:00.000Z","created_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"last_edited_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"has_children":true,"archived":false,
        "type":"numbered_list_item",
        "numbered_list_item":
            {"rich_text":[{"type":"text","text":{"content":"Bar","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"Bar","href":null}],
            "color":"default"}},
        {"object":"block","id":"a11bd916-0f24-408b-b8f6-c0f9689eef87","parent":{"type":"page_id","page_id":"c90565cf-4ae6-4e3d-bfdb-b9140b1f8b04"},"created_time":"2022-08-28T08:30:00.000Z","last_edited_time":"2022-08-28T08:30:00.000Z","created_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"last_edited_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"has_children":false,"archived":false,
        "type":"toggle",
        "toggle":
            {"rich_text":[{"type":"text","text":{"content":"hoge","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"hoge","href":null}],
            "color":"default"}},
        {"object":"block","id":"4dfff265-c02a-493e-bacb-923dc0cb56bd","parent":{"type":"page_id","page_id":"c90565cf-4ae6-4e3d-bfdb-b9140b1f8b04"},"created_time":"2022-08-28T08:30:00.000Z","last_edited_time":"2022-08-28T08:31:00.000Z","created_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"last_edited_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"has_children":true,"archived":false,
        "type":"toggle",
        "toggle":
            {"rich_text":[{"type":"text","text":{"content":"hoge","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"hoge","href":null}],
            "color":"default"}},
        {"object":"block","id":"9ce65fa0-52b0-4ee0-bae0-928407ece597","parent":{"type":"page_id","page_id":"c90565cf-4ae6-4e3d-bfdb-b9140b1f8b04"},"created_time":"2022-08-28T08:31:00.000Z","last_edited_time":"2022-08-28T08:32:00.000Z","created_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"last_edited_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"has_children":true,"archived":false,
        "type":"quote",
        "quote":
            {"rich_text":[{"type":"text","text":{"content":"quote \nquote","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"quote \nquote","href":null}],
            "color":"default"}},
        {"object":"block","id":"c5e16d7c-0d77-4a7d-952c-6ba35edcf2d5","parent":{"type":"page_id","page_id":"c90565cf-4ae6-4e3d-bfdb-b9140b1f8b04"},"created_time":"2022-08-28T08:31:00.000Z","last_edited_time":"2022-08-28T08:32:00.000Z","created_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"last_edited_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"has_children":false,"archived":false,
        "type":"quote",
        "quote":
            {"rich_text":[{"type":"text","text":{"content":"qqqq","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"qqqq","href":null}],
            "color":"default"}},
        {"object":"block","id":"5794b26c-82c5-4753-9bd0-e37d7ae7c814","parent":{"type":"page_id","page_id":"c90565cf-4ae6-4e3d-bfdb-b9140b1f8b04"},"created_time":"2022-08-28T08:32:00.000Z","last_edited_time":"2022-08-28T08:32:00.000Z","created_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"last_edited_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"has_children":false,"archived":false,
        "type":"divider",
        "divider":
            {}},
        {"object":"block","id":"774e6523-dcf1-4518-acda-8c24f86f0dbb","parent":{"type":"page_id","page_id":"c90565cf-4ae6-4e3d-bfdb-b9140b1f8b04"},"created_time":"2022-08-28T08:32:00.000Z","last_edited_time":"2022-08-28T08:32:00.000Z","created_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"last_edited_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"has_children":false,"archived":false,
        "type":"link_to_page",
        "link_to_page":
            {"type":"page_id",
            "page_id":"c90565cf-4ae6-4e3d-bfdb-b9140b1f8b04"}},
        {"object":"block","id":"d1ae585f-70b6-4170-ade3-30c51d82ad01","parent":{"type":"page_id","page_id":"c90565cf-4ae6-4e3d-bfdb-b9140b1f8b04"},"created_time":"2022-08-28T08:32:00.000Z","last_edited_time":"2022-08-28T08:33:00.000Z","created_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"last_edited_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"has_children":false,"archived":false,
        "type":"callout",
        "callout":
            {"rich_text":[{"type":"text","text":{"content":"callout ","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"callout ","href":null},{"type":"equation","equation":{"expression":"E=mac^2"},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"E=mac^2","href":null}],
            "icon":{"type":"emoji","emoji":"📚"},
            "color":"gray_background"}},
        {"object":"block","id":"b701e19e-3e6e-42ae-bccc-28da68080a91","parent":{"type":"page_id","page_id":"c90565cf-4ae6-4e3d-bfdb-b9140b1f8b04"},"created_time":"2022-08-28T08:33:00.000Z","last_edited_time":"2022-08-28T08:34:00.000Z","created_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"last_edited_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"has_children":false,"archived":false,
        "type":"image",
        "image":
            {"caption":[{"type":"text","text":{"content":"note","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"note","href":null}],
            "type":"file",
            "file":{"url":"https://s3.us-west-2.amazonaws.com/secure.notion-static.com/21482588-4d92-4275-8029-abf6f9a973cc/bunbougu_note.png?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Content-Sha256=UNSIGNED-PAYLOAD&X-Amz-Credential=AKIAT73L2G45EIPT3X45%2F20220828%2Fus-west-2%2Fs3%2Faws4_request&X-Amz-Date=20220828T084656Z&X-Amz-Expires=3600&X-Amz-Signature=d7c7b1e55db0d2029b81999f65a1611f5aa038d82b37f3982450dfd40dd3d08b&X-Amz-SignedHeaders=host&x-id=GetObject","expiry_time":"2022-08-28T09:46:56.342Z"}}},
        {"object":"block","id":"572c7a79-64bb-4370-825e-3e067f4052bf","parent":{"type":"page_id","page_id":"c90565cf-4ae6-4e3d-bfdb-b9140b1f8b04"},"created_time":"2022-08-28T08:34:00.000Z","last_edited_time":"2022-08-28T08:35:00.000Z","created_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"last_edited_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"has_children":false,"archived":false,
        "type":"image",
        "image":
            {"caption":[],
            "type":"external",
            "external":{"url":"https://2.bp.blogspot.com/-FFQ5vJihLlQ/WCP3o-ejZ9I/AAAAAAAA_cc/d_2bxXJpqi4buPScUk6gmVdDBhPBFc8BwCLcB/s450/bunbougu_note.png"}}},
        {"object":"block","id":"14d42fa7-aeaf-46ef-aeb9-c451b1b09d09","parent":{"type":"page_id","page_id":"c90565cf-4ae6-4e3d-bfdb-b9140b1f8b04"},"created_time":"2022-08-28T08:35:00.000Z","last_edited_time":"2022-08-28T08:35:00.000Z","created_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"last_edited_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"has_children":false,"archived":false,
        "type":"bookmark",
        "bookmark":
            {"caption":[],
            "url":"https://ja.wikipedia.org/wiki/%E3%83%A1%E3%82%A4%E3%83%B3%E3%83%9A%E3%83%BC%E3%82%B8"}},
        {"object":"block","id":"6bf8d68d-a5c2-4b39-8679-a40ddb082054","parent":{"type":"page_id","page_id":"c90565cf-4ae6-4e3d-bfdb-b9140b1f8b04"},"created_time":"2022-08-28T08:35:00.000Z","last_edited_time":"2022-08-28T08:36:00.000Z","created_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"last_edited_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"has_children":false,"archived":false,
        "type":"paragraph",
        "paragraph":
            {"rich_text":[{"type":"text","text":{"content":"https://ja.wikipedia.org/wiki/メインページ","link":{"url":"https://ja.wikipedia.org/wiki/%E3%83%A1%E3%82%A4%E3%83%B3%E3%83%9A%E3%83%BC%E3%82%B8"}},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"https://ja.wikipedia.org/wiki/メインページ","href":"https://ja.wikipedia.org/wiki/%E3%83%A1%E3%82%A4%E3%83%B3%E3%83%9A%E3%83%BC%E3%82%B8"}],
            "color":"default"}},
        {"object":"block","id":"e8947f24-d4c2-4276-b255-d24bcdc06b1c","parent":{"type":"page_id","page_id":"c90565cf-4ae6-4e3d-bfdb-b9140b1f8b04"},"created_time":"2022-08-28T08:36:00.000Z","last_edited_time":"2022-08-28T08:36:00.000Z","created_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"last_edited_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"has_children":false,"archived":false,
        "type":"bookmark",
        "bookmark":
            {"caption":[],
            "url":"https://ja.wikipedia.org/wiki/%E3%83%A1%E3%82%A4%E3%83%B3%E3%83%9A%E3%83%BC%E3%82%B8"}},
        {"object":"block","id":"2017380e-05da-4ed0-8b19-7395a1d9e3b6","parent":{"type":"page_id","page_id":"c90565cf-4ae6-4e3d-bfdb-b9140b1f8b04"},"created_time":"2022-08-28T08:36:00.000Z","last_edited_time":"2022-08-28T08:36:00.000Z","created_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"last_edited_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"has_children":false,"archived":false,
        "type":"code",
        "code":
            {"caption":[{"type":"text","text":{"content":"aaa","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"aaa","href":null}],
            "rich_text":[{"type":"text","text":{"content":"def a(a):\n\tretrun a","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"def a(a):\n\tretrun a","href":null}],
            "language":"python"}},
        {"object":"block","id":"a11f785a-dbf4-459d-8692-9797eebd92eb","parent":{"type":"page_id","page_id":"c90565cf-4ae6-4e3d-bfdb-b9140b1f8b04"},"created_time":"2022-08-28T08:37:00.000Z","last_edited_time":"2022-08-28T08:37:00.000Z","created_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"last_edited_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"has_children":false,"archived":false,
        "type":"video",
        "video":
            {"caption":[],
            "type":"external",
            "external":{"url":"https://www.youtube.com/watch?v=jNQXAC9IVRw"}}},
        {"object":"block","id":"d4b7e370-7614-4c80-bdc4-8dc7856da424","parent":{"type":"page_id","page_id":"c90565cf-4ae6-4e3d-bfdb-b9140b1f8b04"},"created_time":"2022-08-28T08:37:00.000Z","last_edited_time":"2022-08-28T08:38:00.000Z","created_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"last_edited_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"has_children":false,"archived":false,
        "type":"embed",
        "embed":
            {"caption":[],
            "url":"https://open.spotify.com/track/783j0wyaEcJAKIlTwRwTZb?si=ef02f4812ee34754"}},
        {"object":"block","id":"ebcdd9c3-8b30-4fb6-a707-154351832b52","parent":{"type":"page_id","page_id":"c90565cf-4ae6-4e3d-bfdb-b9140b1f8b04"},"created_time":"2022-08-28T08:38:00.000Z","last_edited_time":"2022-08-28T08:38:00.000Z","created_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"last_edited_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"has_children":false,"archived":false,
        "type":"file",
        "file":
            {"caption":[],
            "type":"external",
            "external":{"url":""}}},
        {"object":"block","id":"73f704a5-a059-4410-ab80-a20d9c8e91af","parent":{"type":"page_id","page_id":"c90565cf-4ae6-4e3d-bfdb-b9140b1f8b04"},"created_time":"2022-08-28T08:38:00.000Z","last_edited_time":"2022-08-28T08:38:00.000Z","created_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"last_edited_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"has_children":false,"archived":false,
        "type":"table_of_contents",
        "table_of_contents":
            {"color":"gray"}},
        {"object":"block","id":"218e5317-eebf-4472-9876-29de9a38ceff","parent":{"type":"page_id","page_id":"c90565cf-4ae6-4e3d-bfdb-b9140b1f8b04"},"created_time":"2022-08-28T08:38:00.000Z","last_edited_time":"2022-08-28T08:39:00.000Z","created_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"last_edited_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"has_children":false,"archived":false,
        "type":"equation",
        "equation":
            {"expression":"\\int_{-\\infty}^{\\infty}dx e^{-x^2}"}},
        {"object":"block","id":"c6663395-1f29-43d7-98dd-d58364ea299e","parent":{"type":"page_id","page_id":"c90565cf-4ae6-4e3d-bfdb-b9140b1f8b04"},"created_time":"2022-08-28T08:40:00.000Z","last_edited_time":"2022-08-28T08:40:00.000Z","created_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"last_edited_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"has_children":false,"archived":false,
        "type":"breadcrumb",
        "breadcrumb":
            {}},
        {"object":"block","id":"ec642647-5122-431a-8f46-b4481546f964","parent":{"type":"page_id","page_id":"c90565cf-4ae6-4e3d-bfdb-b9140b1f8b04"},"created_time":"2022-08-28T08:40:00.000Z","last_edited_time":"2022-08-28T08:40:00.000Z","created_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"last_edited_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"has_children":true,"archived":false,
        "type":"heading_1",
        "heading_1":
            {"rich_text":[{"type":"text","text":{"content":"hoge","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"hoge","href":null}],
            "is_toggleable":true,
            "color":"default"}},
        {"object":"block","id":"1c56f798-08ef-412c-b6f2-395730d6a10a","parent":{"type":"page_id","page_id":"c90565cf-4ae6-4e3d-bfdb-b9140b1f8b04"},"created_time":"2022-08-28T08:41:00.000Z","last_edited_time":"2022-08-28T08:41:00.000Z","created_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"last_edited_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"has_children":false,"archived":false,
        "type":"code",
        "code":
            {"caption":[],
            "rich_text":[{"type":"text","text":{"content":"graph TD\n  Mermaid --> Diagram","link":null},"annotations":{"bold":false,"italic":false,"strikethrough":false,"underline":false,"code":false,"color":"default"},"plain_text":"graph TD\n  Mermaid --> Diagram","href":null}],
            "language":"mermaid"}},
        {"object":"block","id":"dd653792-a74c-4ea4-8d73-edc8795858b0","parent":{"type":"page_id","page_id":"c90565cf-4ae6-4e3d-bfdb-b9140b1f8b04"},"created_time":"2022-08-28T08:41:00.000Z","last_edited_time":"2022-08-28T08:41:00.000Z","created_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"last_edited_by":{"object":"user","id":"66afb78c-8a2d-4ed4-ae45-02cb6b2d2063"},"has_children":false,"archived":false,
        "type":"paragraph",
        "paragraph":
            {"rich_text":[],
            "color":"default"}}
    ],"next_cursor":null,"has_more":false,"type":"block","block":{}}
    "###;

    let r: PageContents = serde_json::from_str(sample_json)?;
    println!("{:#?}", r);
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct PageContents {
    results: Vec<Block>,
}

/// Rich Text Object
/// https://developers.notion.com/reference/rich-text
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum RichText {
    Text {
        #[serde(flatten)]
        common: RichTextCommon,
        text: TextContent,
    },
    Mention {
        #[serde(flatten)]
        common: RichTextCommon,
        mention: MentionContent,
    },
    Equation {
        #[serde(flatten)]
        common: RichTextCommon,
        equation: EquationContent,
    },
}
#[derive(Debug, Serialize, Deserialize)]
struct RichTextCommon {
    /// The plain text without annotations.
    plain_text: String,
    /// The URL of any link or internal Notion mention in this text, if any.
    href: Option<String>,
    /// All annotations that apply to this rich text.
    annotations: Annotations,
}
#[derive(Debug, Serialize, Deserialize)]
struct TextContent {
    /// Text content. This field contains the actual content of your text and
    /// is probably the field you'll use most often.
    content: String,
    /// Any inline link in this text.
    link: Option<Link>,
}
#[derive(Debug, Serialize, Deserialize)]
struct Link {
    url: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct MentionContent {} // ignore mention
#[derive(Debug, Serialize, Deserialize)]
struct EquationContent {
    /// The LaTeX string representing this inline equation.
    expression: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct Annotations {
    bold: bool,
    italic: bool,
    strikethrough: bool,
    underline: bool,
    code: bool,
    color: String,
}

/// Block Object
/// https://developers.notion.com/reference/block
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Block {
    Paragraph {
        #[serde(flatten)]
        common: BlockCommon,
        paragraph: ParagraphContent,
    },
    #[serde(rename = "heading_1")]
    Heading1 {
        #[serde(flatten)]
        common: BlockCommon,
        heading_1: HeadingContent,
    },
    #[serde(rename = "heading_2")]
    Heading2 {
        #[serde(flatten)]
        common: BlockCommon,
        heading_2: HeadingContent,
    },
    #[serde(rename = "heading_3")]
    Heading3 {
        #[serde(flatten)]
        common: BlockCommon,
        heading_3: HeadingContent,
    },
    Callout {
        #[serde(flatten)]
        common: BlockCommon,
        callout: ParagraphContent, // icon is ignored
    },
    Quote {
        #[serde(flatten)]
        common: BlockCommon,
        quote: ParagraphContent,
    },
    BulletedListItem {
        #[serde(flatten)]
        common: BlockCommon,
        bulleted_list_item: ParagraphContent,
    },
    NumberedListItem {
        #[serde(flatten)]
        common: BlockCommon,
        numbered_list_item: ParagraphContent,
    },
    ToDo {
        #[serde(flatten)]
        common: BlockCommon,
        to_do: ToDoContent,
    },
    Toggle {
        #[serde(flatten)]
        common: BlockCommon,
        toggle: ParagraphContent,
    },
    Code {
        #[serde(flatten)]
        common: BlockCommon,
        code: CodeContent,
    },
    ChildPage,
    ChildDatabase,
    Embed {
        #[serde(flatten)]
        common: BlockCommon,
        embed: Link,
    },
    Image {
        #[serde(flatten)]
        common: BlockCommon,
        image: FileContent,
    },
    Video {
        #[serde(flatten)]
        common: BlockCommon,
        video: FileContent,
    },
    File {
        #[serde(flatten)]
        common: BlockCommon,
        file: FileContent,
    },
    #[serde(rename = "pdf")]
    PDF {
        #[serde(flatten)]
        common: BlockCommon,
        pdf: FileContent,
    },
    Bookmark,
    Equation {
        #[serde(flatten)]
        common: BlockCommon,
        equation: EquationContent,
    },
    Divider {
        #[serde(flatten)]
        common: BlockCommon,
    },
    TableOfContents {
        #[serde(flatten)]
        common: BlockCommon,
    },
    Breadcrumb {
        #[serde(flatten)]
        common: BlockCommon,
    },
    Column,
    ColumnList,
    LinkPreview {
        #[serde(flatten)]
        common: BlockCommon,
        link_preview: LinkPreviewContent,
    },
    Template,
    LinkToPage {
        #[serde(flatten)]
        common: BlockCommon,
        link_to_page: LinkToPageContent,
    },
    SyncedBlock,
    Table {
        #[serde(flatten)]
        common: BlockCommon,
        table: TableContent,
    },
    TableRow {
        #[serde(flatten)]
        common: BlockCommon,
        table_row: TableRowContent,
    },
    Unsupported,
}
#[derive(Debug, Serialize, Deserialize)]
struct BlockCommon {
    id: String,
    archived: bool,
    has_children: bool,
}
#[derive(Debug, Serialize, Deserialize)]
struct ParagraphContent {
    rich_text: Vec<RichText>,
    color: String,
    children: Option<Vec<Block>>,
}
#[derive(Debug, Serialize, Deserialize)]
struct HeadingContent {
    rich_text: Vec<RichText>,
    color: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct ToDoContent {
    rich_text: Vec<RichText>,
    #[serde(default)]
    checked: bool,
    color: String,
    children: Option<Vec<Block>>,
}
#[derive(Debug, Serialize, Deserialize)]
struct CodeContent {
    rich_text: Vec<RichText>,
    caption: Vec<RichText>,
    language: String,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum FileContent {
    External {
        caption: Vec<RichText>,
        external: ExternalFileLink,
    },
    File {
        caption: Vec<RichText>,
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
    expiry_time: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct LinkPreviewContent {}
#[derive(Debug, Serialize, Deserialize)]
struct LinkToPageContent {}
#[derive(Debug, Serialize, Deserialize)]
struct TableContent {}
#[derive(Debug, Serialize, Deserialize)]
struct TableRowContent {}
