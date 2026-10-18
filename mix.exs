defmodule ExFastembed.MixProject do
  use Mix.Project

  @version "0.1.2"
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
      aliases: ["hex.publish": [&publish/1]],
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
      files: package_files(),
      licenses: ["Apache-2.0"],
      links: %{
        "GitHub" => @source_url
      }
    ]
  end

  @spec package_files() :: [String.t()]
  defp package_files do
    ~w(
      lib
      mix.exs
      README.md
      CHANGELOG.md
      LICENSE
      guides
      native/ex_fastembed/Cargo.toml
      native/ex_fastembed/Cargo.lock
      native/ex_fastembed/.cargo
      native/ex_fastembed/src
    ) ++ Path.wildcard("checksum-*.exs")
  end

  @spec publish([String.t()]) :: term()
  defp publish(args) do
    # Only the final alias step receives the CLI arguments. Keep validation and
    # delegation together so docs/revert can be recognized before invoking Hex.
    {opts, commands, invalid} =
      OptionParser.parse(args,
        strict: [
          organization: :string,
          yes: :boolean,
          dry_run: :boolean,
          replace: :boolean,
          revert: :string
        ]
      )

    if invalid == [] and (commands == ["docs"] or is_binary(opts[:revert])),
      do: :ok,
      else: check_release_checksums()

    Mix.Task.run("hex.publish", args)
  end

  @spec check_release_checksums() :: :ok
  defp check_release_checksums do
    script = Path.join(__DIR__, "scripts/checksums.exs")

    unless File.regular?(script),
      do: Mix.raise("Publish packages from a source checkout containing scripts/checksums.exs")

    {output, status} = System.cmd("elixir", [script, "--check"], stderr_to_stdout: true)

    if status != 0, do: Mix.raise("Release checksum validation failed:\n#{output}")
    Mix.shell().info(String.trim(output))
    :ok
  end

  @spec docs() :: keyword()
  defp docs do
    [
      main: "readme",
      source_ref: "v#{@version}",
      extras: [
        "README.md",
        "guides/models.md",
        "guides/model_lifecycle.md",
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
        "Model lifecycle": &(&1[:group] == :lifecycle),
        Embeddings: &(&1[:group] == :embeddings),
        Reranking: &(&1[:group] == :reranking)
      ]
    ]
  end

  @spec deps() :: [tuple()]
  defp deps do
    [
      # This Mix compiler invokes Cargo; the native Rustler version is selected
      # in Cargo.toml. Their compatibility is tested by the source-package smoke test.
      {:rustler, "~> 0.38.0", optional: true, runtime: false},
      {:rustler_precompiled, "~> 0.10.0"},
      {:credo, "~> 1.7.19", only: [:dev, :test], runtime: false},
      {:dialyxir, "~> 1.4", only: [:dev, :test], runtime: false},
      {:ex_doc, "~> 0.40.4", only: :dev, runtime: false}
    ]
  end
end
