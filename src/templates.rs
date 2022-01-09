pub const SIMPLE: &str = r#"{{error}}
{{#if (gt (len sources) 0)}}
Caused by:
{{#if (gt (len sources) 1)}}
{{#each sources}}
{{indent @index this}}
{{/each}}
{{else}}
{{#each sources}}
{{indent this}}
{{/each}}
{{/if}}
{{/if}}"#;

pub const COLORED_SIMPLE: &str = r#"{{style "red" error}}
{{#if (gt (len sources) 0)}}
{{style "black.bright" "Caused by:"}}
{{#if (gt (len sources) 1)}}
{{#each sources}}
{{indent @index (style "yellow" this)}}
{{/each}}
{{else}}
{{#each sources}}
{{indent (style "yellow" this)}}
{{/each}}
{{/if}}
{{/if}}"#;
