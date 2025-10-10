use anyhow::{Context, Result, bail};
use xshell::{Shell, cmd};

fn convert_sailfish_tags_to_comment(mut template: &str) -> Result<(String, Vec<&str>)> {
    let mut result_text = String::with_capacity(template.len());
    let mut result_dicts = vec![];
    while let Some((before_start, rest)) = template.split_once("<%") {
        result_text.push_str(before_start);
        let comment = format!("/* SAILFISH_TEMPLATE{} */", result_dicts.len());
        let (tag, rest) = rest
            .split_once("%>")
            .context("A <% without %> found in template")?;
        result_dicts.push(tag);
        template = rest;
        result_text.push_str(&comment);
    }
    result_text.push_str(template);
    Ok((result_text, result_dicts))
}

fn convert_comments_to_sailfish_tag(mut template: String, tags: Vec<&str>) -> String {
    for (i, tag) in tags.into_iter().enumerate() {
        let comment = format!("/* SAILFISH_TEMPLATE{i} */");
        let tag_fixed = format!("<%{tag}%>");
        template = template.replace(&comment, &tag_fixed);
    }
    template
}

pub fn main(fix: bool) -> Result<()> {
    let sh = Shell::new()?;

    let temp_dir = sh.create_temp_dir()?;

    for cpp_template in [
        "./zngur-generator/templates/cpp_header.sptl",
        "./zngur-generator/templates/cpp_source.sptl",
    ] {
        let text = std::fs::read_to_string(cpp_template).context("failed to open template file")?;

        let (commented_text, tags) = convert_sailfish_tags_to_comment(&text)?;

        let temp_file_path = temp_dir.path().join("template.cpp");
        std::fs::write(&temp_file_path, commented_text)?;

        cmd!(sh, "clang-format --style=webkit -i {temp_file_path}").run()?;

        let fixed_text = std::fs::read_to_string(&temp_file_path)?;
        let fixed_text = convert_comments_to_sailfish_tag(fixed_text, tags);

        if fix {
            std::fs::write(&cpp_template, fixed_text)?;
        } else {
            if fixed_text != text {
                bail!("Diff detected in file {cpp_template}");
            }
        }
    }

    Ok(())
}
