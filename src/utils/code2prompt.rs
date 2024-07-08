use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use arboard::Clipboard;
use git2::{DiffOptions, Repository};
use handlebars::{no_escape, Handlebars};
use ignore::WalkBuilder;
use regex::Regex;
use serde_json::json;
use termtree::Tree;

pub struct Code2Prompt {
    /// Path to the codebase directory
    path: PathBuf,

    /// Optional comma-separated list of file extensions to filter
    filter: Option<String>,

    /// Optional comma-separated list of file extensions to exclude
    exclude: Option<String>,

    /// Optional comma-separated list of file names to exclude
    exclude_files: Option<String>,

    /// Optional comma-separated list of folder paths to exclude
    exclude_folders: Option<String>,

    /// Display the token count of the generated prompt
    tokens: bool,

    /// Optional tokenizer to use for token count
    ///
    /// Supported tokenizers: cl100k (default), p50k, p50k_edit, r50k, gpt2
    encoding: Option<String>,

    diff: bool,

    line_number: bool,
}

impl Default for Code2Prompt {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            filter: Default::default(),
            exclude: Default::default(),
            exclude_files: Default::default(),
            exclude_folders: Default::default(),
            tokens: Default::default(),
            encoding: Default::default(),
            diff: Default::default(),
            line_number: Default::default(),
        }
    }
}

impl Code2Prompt {
    pub fn new(path: &PathBuf) -> Self {
        Self {
            path: path.clone(),
            ..Default::default()
        }
    }
}

const DEFAULT_TEMPLATE: &str = r#"
Project Path: {{ absolute_code_path }}

Source Tree:

```
{{ source_tree }}
```

{{#each files}}
{{#if code}}
`{{path}}`:

{{code}}

{{/if}}
{{/each}}
"#;

pub fn code2prompt(path: &PathBuf) -> Result<()> {
    let args = Code2Prompt::new(path);
    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(no_escape);
    handlebars.register_template_string("default", DEFAULT_TEMPLATE)?;

    let create_tree = traverse_directory(
        &args.path,
        &args.filter,
        &args.exclude,
        &args.exclude_files,
        &args.exclude_folders,
        args.line_number,
    );

    let (tree, files) = create_tree?;

    let mut git_diff = String::default();
    if args.diff {
        git_diff = get_git_diff(&args.path)?;
    }

    let data = json!({
        "absolute_code_path": args.path.canonicalize().unwrap().display().to_string(),
        "source_tree": tree,
        "files": files,
        "git_diff": git_diff,
    });

    let rendered = handlebars.render("default", &data)?;

    let rendered = rendered.trim();

    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(rendered)?;

    Ok(())
}

/// Extracts undefined variable names from the template string.
#[inline]
fn extract_undefined_variables(template: &str) -> Vec<String> {
    let registered_identifiers = vec!["path", "code", "git_diff"];
    let re = Regex::new(r"\{\{\s*(?P<var>[a-zA-Z_][a-zA-Z_0-9]*)\s*\}\}").unwrap();
    re.captures_iter(template)
        .map(|cap| cap["var"].to_string())
        .filter(|var| !registered_identifiers.contains(&var.as_str()))
        .collect()
}

/// Wraps the code block with backticks and adds the file extension as a label
#[inline]
fn wrap_code_block(code: &str, extension: &str, line_numbers: bool) -> String {
    let backticks = "`".repeat(7);
    let mut code_with_line_numbers = String::new();

    if line_numbers {
        for (line_number, line) in code.lines().enumerate() {
            code_with_line_numbers.push_str(&format!("{:4} | {}\n", line_number + 1, line));
        }
    } else {
        code_with_line_numbers = code.to_string();
    }

    format!(
        "{}{}\n{}\n{}",
        backticks, extension, code_with_line_numbers, backticks
    )
}

/// Returns the directory name as a label
#[inline]
fn label<P: AsRef<Path>>(p: P) -> String {
    let path = p.as_ref();
    if path.file_name().is_none() {
        // If the path is the current directory or a root directory
        path.to_str().unwrap_or(".").to_owned()
    } else {
        // Otherwise, use the file name as the label
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_owned()
    }
}

/// Traverses the directory, builds the tree, and collects information about each file.
fn traverse_directory(
    root_path: &PathBuf,
    filter: &Option<String>,
    exclude: &Option<String>,
    exclude_files: &Option<String>,
    exclude_folders: &Option<String>,
    line_number: bool,
) -> Result<(String, Vec<serde_json::Value>)> {
    let mut files = Vec::new();

    let canonical_root_path = root_path.canonicalize()?;

    let tree = WalkBuilder::new(&canonical_root_path)
        .git_ignore(true)
        .build()
        .filter_map(|e| e.ok())
        .fold(Tree::new(label(&canonical_root_path)), |mut root, entry| {
            let path = entry.path();
            // Calculate the relative path from the root directory to this entry
            if let Ok(relative_path) = path.strip_prefix(&canonical_root_path) {
                let mut current_tree = &mut root;
                for component in relative_path.components() {
                    let component_str = component.as_os_str().to_string_lossy().to_string();

                    current_tree = if let Some(pos) = current_tree
                        .leaves
                        .iter_mut()
                        .position(|child| child.root == component_str)
                    {
                        &mut current_tree.leaves[pos]
                    } else {
                        let new_tree = Tree::new(component_str.clone());
                        current_tree.leaves.push(new_tree);
                        current_tree.leaves.last_mut().unwrap()
                    };
                }

                if path.is_file() {
                    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

                    if let Some(ref exclude_ext) = exclude {
                        let exclude_extensions: Vec<&str> =
                            exclude_ext.split(',').map(|s| s.trim()).collect();
                        if exclude_extensions.contains(&extension) {
                            return root;
                        }
                    }

                    if let Some(ref filter_ext) = filter {
                        let filter_extensions: Vec<&str> =
                            filter_ext.split(',').map(|s| s.trim()).collect();
                        if !filter_extensions.contains(&extension) {
                            return root;
                        }
                    }

                    if let Some(ref exclude_files_str) = exclude_files {
                        let exclude_files_list: Vec<&str> =
                            exclude_files_str.split(',').map(|s| s.trim()).collect();
                        if exclude_files_list.contains(&path.file_name().unwrap().to_str().unwrap())
                        {
                            return root;
                        }
                    }

                    if let Some(ref exclude_folders_str) = exclude_folders {
                        let exclude_folders_list: Vec<&str> =
                            exclude_folders_str.split(',').map(|s| s.trim()).collect();
                        if let Some(parent_path) = path.parent() {
                            let relative_parent_path =
                                parent_path.strip_prefix(&canonical_root_path).unwrap();
                            if exclude_folders_list
                                .iter()
                                .any(|folder| relative_parent_path.starts_with(folder))
                            {
                                return root;
                            }
                        }
                    }

                    let code_bytes = fs::read(&path).expect("Failed to read file");
                    let code = String::from_utf8_lossy(&code_bytes);

                    let code_block = wrap_code_block(&code, extension, line_number);

                    if !code.trim().is_empty() && !code.contains(char::REPLACEMENT_CHARACTER) {
                        files.push(json!({
                            "path": path.display().to_string(),
                            "extension": extension,
                            "code": code_block,
                        }));
                    }
                }
            }

            root
        });

    Ok((tree.to_string(), files))
}

fn get_git_diff(repo_path: &Path) -> Result<String, git2::Error> {
    let repo = Repository::open(repo_path)?;
    let head = repo.head()?;
    let head_tree = head.peel_to_tree()?;
    let diff = repo.diff_tree_to_index(
        Some(&head_tree),
        None,
        Some(DiffOptions::new().ignore_whitespace(true)),
    )?;
    let mut diff_text = Vec::new();
    diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
        diff_text.extend_from_slice(line.content());
        true
    })?;
    Ok(String::from_utf8_lossy(&diff_text).into_owned())
}
