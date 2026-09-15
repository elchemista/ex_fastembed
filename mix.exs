defmodule ExFastembed.MixProject do
  use Mix.Project

  @version "0.1.0"
  @source_url "https://github.com/elchemista/ex_fastembed"

  @doc false
  @spec project() :: keyword()
  def project do
    [
      app: :ex_fastembed,
      name: "ExFastembed",
      version: @version,
      elixir: "~> 1.18",
      build_embedded: Mix.env() == :prod,
      start_permanent: Mix.env() == :prod,
      dialyzer: [plt_add_apps: [:mix]],
      test_coverage: [
        summary: [threshold: 90],
        # NIF stubs are replaced by Rust, which has its own coverage report.
        ignore_modules: [ExFastembed.Native]
      ],
      deps: deps(),
      description: description(),
      package: package(),
      rustler_precompiled: [
        provider: :github,
        owner: "elchemista",
        repo: "ex_fastembed",
        tag: "v#{@version}"
      ],
      docs: docs(),
      source_url: @source_url,
      homepage_url: @source_url
    ]
  end

  @doc false
  @spec application() :: keyword()
  def application do
    [
      extra_applications: []
    ]
  end

  @spec description() :: String.t()
  defp description do
    "Local text embeddings and document reranking for Elixir, powered by FastEmbed and ONNX Runtime."
  end

  @spec package() :: keyword()
  defp package do
    [
      name: "ex_fastembed",
      maintainers: ["Yuriy Zhar"],
      files: ~w(
        lib
        mix.exs
        README.md
        CHANGELOG.md
        LICENSE
        checksum-*.exs
        guides
        native/ex_fastembed/Cargo.toml
        native/ex_fastembed/Cargo.lock
        native/ex_fastembed/.cargo
        native/ex_fastembed/src
      ),
      licenses: ["Apache-2.0"],
      links: %{
        "GitHub" => @source_url
      }
    ]
  end

  @spec docs() :: keyword()
  defp docs do
    [
      main: "readme",
      source_ref: "v#{@version}",
      extras: [
        "README.md",
        "guides/models.md",
        "guides/development.md",
        "guides/releasing.md",
        "CHANGELOG.md",
        "LICENSE"
      ],
      groups_for_extras: [
        Guides: ~r/guides\//
      ],
      groups_for_docs: [
        "Model discovery": &(&1[:group] == :discovery),
        Embeddings: &(&1[:group] == :embeddings),
        Reranking: &(&1[:group] == :reranking)
      ]
    ]
  end

  @spec deps() :: [tuple()]
  defp deps do
    [
      {:rustler, "~> 0.38.0", optional: true, runtime: false},
      {:rustler_precompiled, "~> 0.9.0"},
      {:credo, "~> 1.7.19", only: [:dev, :test], runtime: false},
      {:dialyxir, "~> 1.4", only: [:dev, :test], runtime: false},
      {:ex_doc, "~> 0.40.4", only: :dev, runtime: false}
    ]
  end
end
