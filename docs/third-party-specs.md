# Third Party Specifications

## Well formed "rules" specs for various agents

* Amazon Q: <https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/context-project-rules.html>
* Claude: <https://code.claude.com/docs/en/memory#organize-rules-with-claude/rules/>
* Cline: <https://docs.cline.bot/customization/cline-rules>
* Cursor: <https://cursor.com/docs/rules>
* GitHub Copilot:
  * CLI Documentation: <https://docs.github.com/en/copilot/how-tos/copilot-cli/customize-copilot/add-custom-instructions>
  * IDE Documentation: <https://docs.github.com/en/copilot/how-tos/configure-custom-instructions-in-your-ide/add-repository-instructions-in-your-ide>
  * GitHub calls these "custom instructions", not rules
* JetBrains AI Assistant: <https://www.jetbrains.com/help/ai-assistant/configure-project-rules.html>
* Devin / Windsurf: <https://docs.devin.ai/desktop/cascade/memories#rules>

## Not really rules

* Codex: <https://developers.openai.com/codex/rules>
  * These are sandbox permissions, otherwise `AGENTS.md`
* OpenCode: <https://opencode.ai/docs/rules/>
  * Only has `AGENTS.md`, but calls it "rules"
* Goose: <https://goose-docs.ai/docs/guides/context-engineering/using-goosehints/>
  * Closest thing is the `.goosehints` file, but that's effectively an `AGENTS.md`
