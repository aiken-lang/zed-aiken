use zed::lsp::CompletionKind;
use zed::{CodeLabel, CodeLabelSpan, LanguageServerId};
use zed_extension_api::{self as zed, Result};

struct AikenExtension;

impl AikenExtension {
    fn language_server_binary_path(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<String> {
        worktree
            .which("aiken")
            .ok_or("aiken not found; https://aiken-lang.org/installation-instructions".to_string())
    }
}

impl zed::Extension for AikenExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        Ok(zed::Command {
            command: self.language_server_binary_path(language_server_id, worktree)?,
            args: vec!["lsp".to_string()],
            env: Default::default(),
        })
    }

    fn label_for_completion(
        &self,
        _language_server_id: &LanguageServerId,
        completion: zed::lsp::Completion,
    ) -> Option<zed::CodeLabel> {
        let name = &completion.label;
        let ty = strip_newlines_from_detail(&completion.detail?);
        let let_binding = "let a";
        let colon = ": ";
        let assignment = " = ";
        let call = match completion.kind? {
            CompletionKind::Function | CompletionKind::Constructor => "()",
            _ => "",
        };
        let code = format!("{let_binding}{colon}{ty}{assignment}{name}{call}");

        Some(CodeLabel {
            spans: vec![
                CodeLabelSpan::code_range({
                    let start = let_binding.len() + colon.len() + ty.len() + assignment.len();
                    start..start + name.len()
                }),
                CodeLabelSpan::code_range({
                    let start = let_binding.len();
                    start..start + colon.len()
                }),
                CodeLabelSpan::code_range({
                    let start = let_binding.len() + colon.len();
                    start..start + ty.len()
                }),
            ],
            filter_range: (0..name.len()).into(),
            code,
        })
    }
}

zed::register_extension!(AikenExtension);

fn strip_newlines_from_detail(detail: &str) -> String {
    let without_newlines = detail
        .replace("->\n  ", "-> ")
        .replace("\n  ", "")
        .replace(",\n", "");

    let comma_delimited_parts = without_newlines.split(',');

    comma_delimited_parts
        .map(|part| part.trim())
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use crate::strip_newlines_from_detail;

    #[test]
    fn test_strip_newlines_from_detail() {
        let detail = "fn(\n  Selector(a),\n  b,\n  fn(Dynamic, Dynamic, Dynamic, Dynamic, Dynamic, Dynamic, Dynamic) -> a,\n) -> Selector(a)";
        let expected = "fn(Selector(a), b, fn(Dynamic, Dynamic, Dynamic, Dynamic, Dynamic, Dynamic, Dynamic) -> a) -> Selector(a)";
        assert_eq!(strip_newlines_from_detail(detail), expected);

        let detail = "fn(Selector(a), b, fn(Dynamic, Dynamic, Dynamic, Dynamic, Dynamic, Dynamic) -> a) ->\n  Selector(a)";
        let expected = "fn(Selector(a), b, fn(Dynamic, Dynamic, Dynamic, Dynamic, Dynamic, Dynamic) -> a) -> Selector(a)";
        assert_eq!(strip_newlines_from_detail(detail), expected);

        let detail = "fn(\n  Method,\n  List(#(String, String)),\n  a,\n  Scheme,\n  String,\n  Option(Int),\n  String,\n  Option(String),\n) -> Request(a)";
        let expected = "fn(Method, List(#(String, String)), a, Scheme, String, Option(Int), String, Option(String)) -> Request(a)";
        assert_eq!(strip_newlines_from_detail(detail), expected);
    }
}
