# Ollama Cloud Models Design

## Goal

Add `glm-5.2:cloud` and `minimax-m3:cloud` to Commandblock's selectable model list.

## Design

- Add both models to the existing `config.json` `models` array.
- Use `http://localhost:11434/v1` for each model and leave each model-specific API key empty.
- Preserve all existing model entries, top-level settings, and secrets exactly as they are.
- Rely on the user’s existing Ollama Cloud authentication; no provider key is stored in Commandblock.

## Verification

- Confirm each model appears exactly once with the Ollama endpoint.
- Confirm existing configured model count is increased by two and no keys are displayed or modified.
