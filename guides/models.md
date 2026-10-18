# Supported models

Generated from the bundled `fastembed-rs` 7.1.1 metadata, with legacy aliases retained.
The runtime source is `ExFastembed.embed_models/0` and `ExFastembed.reranker_models/0`.
Names are accepted case-insensitively. Multiple names may select the same model.

A shared repository selects the non-quantized model when available. Use an explicit
variant such as `EmbeddingGemma300MQ4` to select a specific quantization.

## Variants and dimensions

There are **46 embedding variants** and **4 rerankers**.
Dimensions are vector lengths; rerankers return relevance scores instead of vectors.
This catalog covers the dense text embedding and reranking APIs exposed by ExFastembed.

| Variant | Type | Dimensions | Repository |
| --- | --- | ---: | --- |
| `AllMiniLML12V2` | embedding | 384 | `Xenova/all-MiniLM-L12-v2` |
| `AllMiniLML12V2Q` | embedding | 384 | `Xenova/all-MiniLM-L12-v2` |
| `AllMiniLML6V2` | embedding | 384 | `Qdrant/all-MiniLM-L6-v2-onnx` |
| `AllMiniLML6V2Q` | embedding | 384 | `Xenova/all-MiniLM-L6-v2` |
| `AllMpnetBaseV2` | embedding | 768 | `Xenova/all-mpnet-base-v2` |
| `BGEBaseENV15` | embedding | 768 | `Xenova/bge-base-en-v1.5` |
| `BGEBaseENV15Q` | embedding | 768 | `Qdrant/bge-base-en-v1.5-onnx-Q` |
| `BGELargeENV15` | embedding | 1024 | `Xenova/bge-large-en-v1.5` |
| `BGELargeENV15Q` | embedding | 1024 | `Qdrant/bge-large-en-v1.5-onnx-Q` |
| `BGELargeZHV15` | embedding | 1024 | `Xenova/bge-large-zh-v1.5` |
| `BGEM3` | embedding | 1024 | `BAAI/bge-m3` |
| `BGESmallENV15` | embedding | 384 | `Xenova/bge-small-en-v1.5` |
| `BGESmallENV15Q` | embedding | 384 | `Qdrant/bge-small-en-v1.5-onnx-Q` |
| `BGESmallZHV15` | embedding | 512 | `Xenova/bge-small-zh-v1.5` |
| `ClipVitB32` | embedding | 512 | `Qdrant/clip-ViT-B-32-text` |
| `EmbeddingGemma300M` | embedding | 768 | `onnx-community/embeddinggemma-300m-ONNX` |
| `EmbeddingGemma300MQ` | embedding | 768 | `onnx-community/embeddinggemma-300m-ONNX` |
| `EmbeddingGemma300MQ4` | embedding | 768 | `onnx-community/embeddinggemma-300m-ONNX` |
| `GTEBaseENV15` | embedding | 768 | `Alibaba-NLP/gte-base-en-v1.5` |
| `GTEBaseENV15Q` | embedding | 768 | `Alibaba-NLP/gte-base-en-v1.5` |
| `GTELargeENV15` | embedding | 1024 | `Alibaba-NLP/gte-large-en-v1.5` |
| `GTELargeENV15Q` | embedding | 1024 | `Alibaba-NLP/gte-large-en-v1.5` |
| `JinaEmbeddingsV2BaseCode` | embedding | 768 | `jinaai/jina-embeddings-v2-base-code` |
| `JinaEmbeddingsV2BaseEN` | embedding | 768 | `jinaai/jina-embeddings-v2-base-en` |
| `ModernBertEmbedLarge` | embedding | 1024 | `lightonai/modernbert-embed-large` |
| `MultilingualE5Base` | embedding | 768 | `intfloat/multilingual-e5-base` |
| `MultilingualE5Large` | embedding | 1024 | `Qdrant/multilingual-e5-large-onnx` |
| `MultilingualE5Small` | embedding | 384 | `intfloat/multilingual-e5-small` |
| `MxbaiEmbedLargeV1` | embedding | 1024 | `mixedbread-ai/mxbai-embed-large-v1` |
| `MxbaiEmbedLargeV1Q` | embedding | 1024 | `mixedbread-ai/mxbai-embed-large-v1` |
| `NomicEmbedTextV1` | embedding | 768 | `nomic-ai/nomic-embed-text-v1` |
| `NomicEmbedTextV15` | embedding | 768 | `nomic-ai/nomic-embed-text-v1.5` |
| `NomicEmbedTextV15Q` | embedding | 768 | `nomic-ai/nomic-embed-text-v1.5` |
| `ParaphraseMLMiniLML12V2` | embedding | 384 | `Xenova/paraphrase-multilingual-MiniLM-L12-v2` |
| `ParaphraseMLMiniLML12V2Q` | embedding | 384 | `Qdrant/paraphrase-multilingual-MiniLM-L12-v2-onnx-Q` |
| `ParaphraseMLMpnetBaseV2` | embedding | 768 | `Xenova/paraphrase-multilingual-mpnet-base-v2` |
| `SnowflakeArcticEmbedL` | embedding | 1024 | `snowflake/snowflake-arctic-embed-l` |
| `SnowflakeArcticEmbedLQ` | embedding | 1024 | `snowflake/snowflake-arctic-embed-l` |
| `SnowflakeArcticEmbedM` | embedding | 768 | `Snowflake/snowflake-arctic-embed-m` |
| `SnowflakeArcticEmbedMLong` | embedding | 768 | `snowflake/snowflake-arctic-embed-m-long` |
| `SnowflakeArcticEmbedMLongQ` | embedding | 768 | `snowflake/snowflake-arctic-embed-m-long` |
| `SnowflakeArcticEmbedMQ` | embedding | 768 | `Snowflake/snowflake-arctic-embed-m` |
| `SnowflakeArcticEmbedS` | embedding | 384 | `snowflake/snowflake-arctic-embed-s` |
| `SnowflakeArcticEmbedSQ` | embedding | 384 | `snowflake/snowflake-arctic-embed-s` |
| `SnowflakeArcticEmbedXS` | embedding | 384 | `snowflake/snowflake-arctic-embed-xs` |
| `SnowflakeArcticEmbedXSQ` | embedding | 384 | `snowflake/snowflake-arctic-embed-xs` |
| `BGERerankerBase` | reranker | — | `BAAI/bge-reranker-base` |
| `BGERerankerV2M3` | reranker | — | `rozgo/bge-reranker-v2-m3` |
| `JINARerankerV1TurboEn` | reranker | — | `jinaai/jina-reranker-v1-turbo-en` |
| `JINARerankerV2BaseMultilingual` | reranker | — | `jinaai/jina-reranker-v2-base-multilingual` |

## Accepted names and aliases

`JINARerankerV2BaseMultilingual` is the corrected upstream name. The previous
`JINARerankerV2BaseMultiligual` spelling and misspelled repository alias remain
accepted for compatibility. Metadata returns the corrected variant name.

### Embedding Models

- `"Alibaba-NLP/gte-base-en-v1.5"`
- `"Alibaba-NLP/gte-large-en-v1.5"`
- `"AllMiniLML12V2"`
- `"AllMiniLML12V2Q"`
- `"AllMiniLML6V2"`
- `"AllMiniLML6V2Q"`
- `"AllMpnetBaseV2"`
- `"BAAI/bge-base-en-v1.5"`
- `"BAAI/bge-large-en-v1.5"`
- `"BAAI/bge-large-zh-v1.5"`
- `"BAAI/bge-m3"`
- `"BAAI/bge-small-en-v1.5"`
- `"BAAI/bge-small-zh-v1.5"`
- `"BGEBaseENV15"`
- `"BGEBaseENV15Q"`
- `"BGELargeENV15"`
- `"BGELargeENV15Q"`
- `"BGELargeZHV15"`
- `"BGEM3"`
- `"BGESmallENV15"`
- `"BGESmallENV15Q"`
- `"BGESmallZHV15"`
- `"ClipVitB32"`
- `"EmbeddingGemma300M"`
- `"EmbeddingGemma300MQ"`
- `"EmbeddingGemma300MQ4"`
- `"GTEBaseENV15"`
- `"GTEBaseENV15Q"`
- `"GTELargeENV15"`
- `"GTELargeENV15Q"`
- `"intfloat/multilingual-e5-base"`
- `"intfloat/multilingual-e5-large"`
- `"intfloat/multilingual-e5-small"`
- `"jinaai/jina-embeddings-v2-base-code"`
- `"jinaai/jina-embeddings-v2-base-en"`
- `"JinaEmbeddingsV2BaseCode"`
- `"JinaEmbeddingsV2BaseEN"`
- `"lightonai/ModernBERT-embed-large"`
- `"lightonai/modernbert-embed-large"`
- `"mixedbread-ai/mxbai-embed-large-v1"`
- `"ModernBertEmbedLarge"`
- `"MultilingualE5Base"`
- `"MultilingualE5Large"`
- `"MultilingualE5Small"`
- `"MxbaiEmbedLargeV1"`
- `"MxbaiEmbedLargeV1Q"`
- `"nomic-ai/nomic-embed-text-v1"`
- `"nomic-ai/nomic-embed-text-v1.5"`
- `"NomicEmbedTextV1"`
- `"NomicEmbedTextV15"`
- `"NomicEmbedTextV15Q"`
- `"onnx-community/embeddinggemma-300m-ONNX"`
- `"ParaphraseMLMiniLML12V2"`
- `"ParaphraseMLMiniLML12V2Q"`
- `"ParaphraseMLMpnetBaseV2"`
- `"Qdrant/all-MiniLM-L6-v2-onnx"`
- `"Qdrant/bge-base-en-v1.5-onnx-Q"`
- `"Qdrant/bge-large-en-v1.5-onnx-Q"`
- `"Qdrant/bge-small-en-v1.5-onnx-Q"`
- `"Qdrant/clip-ViT-B-32-text"`
- `"Qdrant/multilingual-e5-large-onnx"`
- `"Qdrant/paraphrase-multilingual-MiniLM-L12-v2-onnx-Q"`
- `"sentence-transformers/all-MiniLM-L12-v2"`
- `"sentence-transformers/all-MiniLM-L6-v2"`
- `"sentence-transformers/all-mpnet-base-v2"`
- `"sentence-transformers/paraphrase-MiniLM-L12-v2"`
- `"sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2"`
- `"sentence-transformers/paraphrase-multilingual-mpnet-base-v2"`
- `"snowflake/snowflake-arctic-embed-l"`
- `"Snowflake/snowflake-arctic-embed-m"`
- `"snowflake/snowflake-arctic-embed-m"`
- `"snowflake/snowflake-arctic-embed-m-long"`
- `"snowflake/snowflake-arctic-embed-s"`
- `"snowflake/snowflake-arctic-embed-xs"`
- `"SnowflakeArcticEmbedL"`
- `"SnowflakeArcticEmbedLQ"`
- `"SnowflakeArcticEmbedM"`
- `"SnowflakeArcticEmbedMLong"`
- `"SnowflakeArcticEmbedMLongQ"`
- `"SnowflakeArcticEmbedMQ"`
- `"SnowflakeArcticEmbedS"`
- `"SnowflakeArcticEmbedSQ"`
- `"SnowflakeArcticEmbedXS"`
- `"SnowflakeArcticEmbedXSQ"`
- `"Xenova/all-MiniLM-L12-v2"`
- `"Xenova/all-MiniLM-L6-v2"`
- `"Xenova/all-mpnet-base-v2"`
- `"Xenova/bge-base-en-v1.5"`
- `"Xenova/bge-large-en-v1.5"`
- `"Xenova/bge-large-zh-v1.5"`
- `"Xenova/bge-small-en-v1.5"`
- `"Xenova/bge-small-zh-v1.5"`
- `"Xenova/paraphrase-multilingual-MiniLM-L12-v2"`
- `"Xenova/paraphrase-multilingual-mpnet-base-v2"`

### Reranker Models

- `"BAAI/bge-reranker-base"`
- `"BAAI/bge-reranker-v2-m3"`
- `"BGERerankerBase"`
- `"BGERerankerV2M3"`
- `"jinaai/jina-reranker-v1-turbo-en"`
- `"jinaai/jina-reranker-v2-base-multiligual"`
- `"jinaai/jina-reranker-v2-base-multilingual"`
- `"JINARerankerV1TurboEn"`
- `"JINARerankerV2BaseMultiligual"`
- `"JINARerankerV2BaseMultilingual"`
- `"rozgo/bge-reranker-v2-m3"`

## Updating the catalog

Run `EX_FASTEMBED_BUILD=1 mix run scripts/update_models.exs` after updating the native lockfile.
The tests and CI verify that these lists match the compiled library.
