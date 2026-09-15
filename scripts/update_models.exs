readme_path = Path.expand("../README.md", __DIR__)
lock_path = Path.expand("../native/ex_fastembed/Cargo.lock", __DIR__)

[_, version] = Regex.run(~r/name = "fastembed"\nversion = "([^"]+)"/, File.read!(lock_path))

[before_models, models_and_after] =
  String.split(File.read!(readme_path), "## Supported Models\n", parts: 2)

[_old_models, after_models] = String.split(models_and_after, "## Development\n", parts: 2)

model_sections =
  Enum.map_join(
    [
      {"Embedding Models", ExFastembed.embed_models()},
      {"Reranker Models", ExFastembed.reranker_models()}
    ],
    "\n",
    fn {heading, names} ->
      "### #{heading}\n\n" <> Enum.map_join(names, "", &"- `\"#{&1}\"`\n")
    end
  )

models = """
## Supported Models

The runtime source is `ExFastembed.embed_models/0` and `ExFastembed.reranker_models/0`.
These lists are generated from the bundled `fastembed-rs` #{version} metadata, with legacy ExFastembed aliases retained.
Repository names and explicit FastEmbed variant names are accepted case-insensitively.
When a repository contains several variants, its name selects the non-quantized model when available.
Use an explicit variant such as `EmbeddingGemma300MQ4` to select a specific quantization.

Regenerate these lists after dependency updates with `mix run scripts/update_models.exs`.

#{String.trim_trailing(model_sections)}

"""

File.write!(readme_path, before_models <> models <> "## Development\n" <> after_models)
Mix.shell().info("Updated README model lists for fastembed #{version}")
