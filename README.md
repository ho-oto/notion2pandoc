# notion2pandoc

notion2pandoc is a simple CLI tool to convert Notion page to
[Pandoc AST](https://hackage.haskell.org/package/pandoc-types-1.23/docs/Text-Pandoc-Definition.html)
by using [Notion API](https://developers.notion.com/reference/intro).

notion2pandoc outputs Pandoc AST serialized in JSON to stdout.
Combining with the Pandoc CLI, You can get Notion pages formatted in your favorite markup format supported by Pandoc, as follows.

```bash
notion2pandoc -i ${NOTION_PAGE_ID} -s ${NOTION_API_SECRET} | pandoc --from json --to {html,markdown,...}
```

notion2pandoc outputs [pandoc-types-1.23](https://hackage.haskell.org/package/pandoc-types-1.23).
Validation is done by using Pandoc 3.0.1.

## Why NOTION_API_SECRET is needed?

Instead of taking an output of `curl` or something like that from stdin, notion2pandoc is implemented to take an API secret and call the Notion API internally.
This is because the current version of Notion API requires multiple API calls to retrieve the entire page.

The API allows us to retrieve a list of the blocks that make up a page.
However, when the number of blocks exceeds 100, the API must be called again with [pagination](https://developers.notion.com/reference/pagination).
When a block has child blocks, another API call is necessary to retrieve their contents.

## Unsupported features

notion2pandoc simply ignore blocks with following [Block Type Object](https://developers.notion.com/reference/block#block-type-object).

- child_page
- child_database
- breadcrumb
- column
- column_list
- template
- synced_block

notion2pandoc dose not generate table of contents.
Instead, you can check if a page contains a block with `table_of_contents` type by looking at `toc` value of meta data in generated AST.

## License

MIT
