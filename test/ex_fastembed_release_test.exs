defmodule ExFastembedReleaseTest do
  use ExUnit.Case, async: true

  @root Path.expand("..", __DIR__)
  @checksum "checksum-Elixir.ExFastembed.Native.exs"

  setup do
    root = Path.join(System.tmp_dir!(), "fastembed-release-#{System.unique_integer([:positive])}")
    File.mkdir_p!(Path.join(root, "scripts"))
    File.cp!(Path.join(@root, "mix.exs"), Path.join(root, "mix.exs"))
    File.cp!(Path.join(@root, "scripts/checksums.exs"), Path.join(root, "scripts/checksums.exs"))
    on_exit(fn -> File.rm_rf!(root) end)
    %{root: root}
  end

  test "publishing rejects missing checksums before invoking Hex", %{root: root} do
    {output, status} = publish(root)
    assert status != 0
    assert output =~ "Missing NIF checksums"
    refute File.exists?(Path.join(root, "published-args"))
  end

  test "publishing rejects checksums from another release", %{root: root} do
    write_checksums(root, "0.0.0", "sha256:" <> String.duplicate("0", 64))
    {output, status} = publish(root)
    assert status != 0
    assert output =~ "Incomplete or stale NIF checksums"
    refute File.exists?(Path.join(root, "published-args"))
  end

  test "publishing rejects invalid digests", %{root: root} do
    write_checksums(root, Mix.Project.config()[:version], "sha256:invalid")
    {output, status} = publish(root)
    assert status != 0
    assert output =~ "Invalid SHA-256 checksum"
    refute File.exists?(Path.join(root, "published-args"))
  end

  test "valid checksums allow the real task and retain its arguments", %{root: root} do
    write_checksums(root, Mix.Project.config()[:version], "sha256:" <> String.duplicate("0", 64))
    {output, status} = publish(root)
    assert status == 0, output
    assert output =~ "Verified 8 NIF checksums"
    assert File.read!(Path.join(root, "published-args")) == "--dry-run\n--yes"
  end

  defp write_checksums(root, version, digest) do
    entries =
      for nif <- ["2.15", "2.16"],
          target <- [
            "aarch64-apple-darwin",
            "aarch64-unknown-linux-gnu",
            "x86_64-pc-windows-msvc",
            "x86_64-unknown-linux-gnu"
          ],
          into: %{} do
        {prefix, extension} =
          if String.contains?(target, "windows"), do: {"", "dll"}, else: {"lib", "so"}

        {"#{prefix}ex_fastembed-v#{version}-nif-#{nif}-#{target}.#{extension}.tar.gz", digest}
      end

    File.write!(Path.join(root, @checksum), inspect(entries))
  end

  defp publish(root) do
    # A separate VM loads the copied project and a stub Hex task. No publishing,
    # authentication, or network access occurs, even in the successful case.
    script = """
    Mix.start()
    Code.compile_file("mix.exs")
    defmodule Mix.Tasks.Hex.Publish do
      use Mix.Task
      def run(args), do: File.write!("published-args", Enum.join(args, "\\n"))
    end
    Mix.Task.run("hex.publish", ["--dry-run", "--yes"])
    """

    System.cmd("elixir", ["-e", script],
      cd: root,
      env: [{"ERL_FLAGS", "+S 2:2"}],
      stderr_to_stdout: true
    )
  end
end
