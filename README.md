# notion2pandoc

Notion page to Pandoc AST

## Usage

```bash
notion2pandoc -i NOTION_PAGE_ID -s NOTION_API_SECRET | pandoc --from json --to {html,markdown,...}
```

## Reference

[Notion API](https://developers.notion.com/reference/intro)

[Pandoc AST](https://hackage.haskell.org/package/pandoc-types-1.22.2.1/docs/Text-Pandoc-Definition.html#t:Attr)

## License

MIT
