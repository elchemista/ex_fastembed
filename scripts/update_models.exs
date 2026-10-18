guide_path = Path.expand("../guides/models.md", __DIR__)
lock_path = Path.expand("../native/ex_fastembed/Cargo.lock", __DIR__)

[_, version] = Regex.run(~r/name = "fastembed"\nversion = "([^"]+)"/, File.read!(lock_path))

catalog = ExFastembed.models()
embedding_count = Enum.count(catalog, &(&1.kind == :embedding))
reranker_count = Enum.count(catalog, &(&1.kind == :reranker))

variant_rows =
  Enum.map_join(catalog, "\n", fn model ->
    "| `#{model.name}` | #{model.kind} | #{model.dimension || "—"} | `#{model.repository}` |"
  end)

model_sections =
  Enum.map_join(
    [
      {"Embedding Models", ExFastembed.embed_models()},
      {"Reranker Models", ExFastembed.reranker_models()}
    ],
    "\n",
    fn {heading, names} ->
      "### #{heading}\n\n" <> Enum.map_join(names, "", &"- `#{inspect(&1)}`\n")
    end
  )

models = """
# Supported models

Generated from the bundled `fastembed-rs` #{version} metadata, with legacy aliases retained.
The runtime source is `ExFastembed.embed_models/0` and `ExFastembed.reranker_models/0`.
Names are accepted case-insensitively. Multiple names may select the same model.

A shared repository selects the non-quantized model when available. Use an explicit
variant such as `EmbeddingGemma300MQ4` to select a specific quantization.

## Variants and dimensions

There are **#{embedding_count} embedding variants** and **#{reranker_count} rerankers**.
Dimensions are vector lengths; rerankers return relevance scores instead of vectors.
This catalog covers the dense text embedding and reranking APIs exposed by ExFastembed.

| Variant | Type | Dimensions | Repository |
| --- | --- | ---: | --- |
#{variant_rows}

## Accepted names and aliases

`JINARerankerV2BaseMultilingual` is the corrected upstream name. The previous
`JINARerankerV2BaseMultiligual` spelling and misspelled repository alias remain
accepted for compatibility. Metadata returns the corrected variant name.

#{String.trim_trailing(model_sections)}

## Updating the catalog

Run `EX_FASTEMBED_BUILD=1 mix run scripts/update_models.exs` after updating the native lockfile.
The tests and CI verify that these lists match the compiled library.
"""

File.mkdir_p!(Path.dirname(guide_path))
File.write!(guide_path, models)
Mix.shell().info("Updated model guide for fastembed #{version}")
