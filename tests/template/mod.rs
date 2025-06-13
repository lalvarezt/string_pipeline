use string_pipeline::Template;

pub fn process(input: &str, template: &str) -> Result<String, String> {
    let tmpl = Template::parse(template)?;
    tmpl.format(input)
}

pub mod complex_pipeline;
pub mod map_operations;
pub mod simple_pipeline;
