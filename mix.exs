defmodule ExFastembed.MixProject do
  use Mix.Project

  @version "0.1.0"

  def project do
    [
      app: :ex_fastembed,
      name: "ExFastembed",
      version: @version,
      elixir: "~> 1.18",
      build_embedded: Mix.env() == :prod,
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      description: description(),
      package: package(),
      docs: [
        main: "readme",
        extras: [
          "README.md",
          "LICENSE"
        ]
      ],
      source_url: "https://github.com/elchemista/ex_fastembed",
      homepage_url: "https://github.com/elchemista/ex_fastembed"
    ]
  end

  # Run "mix help compile.app" to learn about applications.
  def application do
    [
      extra_applications: []
    ]
  end

  defp description() do
    "ExFastembed is an Elixir wrapper around the fastembed-rs crate. It provides a simple interface to load and run various text embedding models and reranker models."
  end

  defp package() do
    [
      name: "ex_fastembed",
      maintainers: ["Yuriy Zhar"],
      files: ~w(mix.exs README.md lib native test LICENSE checksum-*.exs .formatter.exs),
      licenses: ["Apache-2.0"],
      links: %{
        "GitHub" => "https://github.com/elchemista/ex_fastembed"
      }
    ]
  end

  # Run "mix help deps" to learn about dependencies.
  defp deps do
    [
      {:rustler, ">= 0.0.0", optional: true},
      {:credo, "~> 1.6", only: [:dev, :test], runtime: false},
      {:dialyxir, "~> 1.4", only: [:dev, :test], runtime: false},
      {:rustler_precompiled, "~> 0.8"},
      # Documentation Provider
      {:ex_doc, "~> 0.28.3", only: [:dev, :test], optional: true, runtime: false}
    ]
  end
end
