project_root = Path.expand("..", __DIR__)
checksum_path = Path.join(project_root, "checksum-Elixir.ExFastembed.Native.exs")
[_, version] = Regex.run(~r/@version "([^"]+)"/, File.read!(Path.join(project_root, "mix.exs")))

targets = [
  "aarch64-apple-darwin",
  "aarch64-unknown-linux-gnu",
  "x86_64-pc-windows-msvc",
  "x86_64-unknown-linux-gnu"
]

expected =
  for nif <- ["2.15", "2.16"], target <- targets do
    {prefix, extension} =
      if String.contains?(target, "windows"), do: {"", "dll"}, else: {"lib", "so"}

    "#{prefix}ex_fastembed-v#{version}-nif-#{nif}-#{target}.#{extension}.tar.gz"
  end
  |> Enum.sort()

case System.argv() do
  ["--check"] ->
    {checksums, _} = Code.eval_file(checksum_path)

    unless Enum.sort(Map.keys(checksums)) == expected,
      do: raise("Incomplete or stale NIF checksums")

    unless Enum.all?(Map.values(checksums), &Regex.match?(~r/^sha256:[0-9a-f]{64}$/, &1)),
      do: raise("Invalid SHA-256 checksum")

    IO.puts("Verified #{map_size(checksums)} NIF checksums for #{version}")

  [artifacts_dir] ->
    files = Path.wildcard(Path.join(artifacts_dir, "*.tar.gz")) |> Enum.sort()

    unless Enum.map(files, &Path.basename/1) == expected,
      do:
        raise(
          "Expected all #{length(expected)} NIF archives for #{version}; found #{length(files)}"
        )

    entries =
      Enum.map_join(files, "", fn file ->
        archive_name = Path.basename(file)
        nif_name = String.replace_suffix(archive_name, ".tar.gz", "")
        {:ok, contents} = :erl_tar.table(String.to_charlist(file), [:compressed])

        unless contents == [String.to_charlist(nif_name)],
          do: raise("Invalid NIF archive: #{file}")

        digest = :crypto.hash(:sha256, File.read!(file)) |> Base.encode16(case: :lower)
        "  #{inspect(archive_name)} => #{inspect("sha256:" <> digest)},\n"
      end)

    File.write!(checksum_path, "%{\n" <> entries <> "}\n")
    IO.puts("Generated checksums from all #{length(expected)} NIF archives for #{version}")

  _ ->
    raise "Usage: elixir scripts/checksums.exs ARTIFACT_DIRECTORY | --check"
end
