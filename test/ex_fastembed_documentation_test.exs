defmodule ExFastembedDocumentationTest do
  use ExUnit.Case, async: true

  test "every public API function has documentation and a specification" do
    {:docs_v1, _, _, _, module_doc, _, docs} = Code.fetch_docs(ExFastembed)
    {:ok, specs} = Code.Typespec.fetch_specs(ExFastembed)

    assert %{"en" => text} = module_doc
    assert String.trim(text) != ""

    for {name, arity} <- ExFastembed.__info__(:functions) do
      assert {{:function, ^name, ^arity}, _, _, %{"en" => text}, _} =
               Enum.find(docs, &(elem(&1, 0) == {:function, name, arity}))

      assert String.trim(text) != ""
      assert Enum.any?(specs, fn {key, _spec} -> key == {name, arity} end)
    end
  end

  test "public types are documented" do
    {:docs_v1, _, _, _, _, _, docs} = Code.fetch_docs(ExFastembed)

    for {{:type, name, arity}, _, _, doc, _} <- docs do
      assert %{"en" => text} = doc, "Missing @typedoc for #{name}/#{arity}"
      assert String.trim(text) != ""
    end
  end

  test "native entry points stay internal and retain their specifications" do
    {:docs_v1, _, _, _, :hidden, _, docs} = Code.fetch_docs(ExFastembed.Native)
    {:ok, specs} = Code.Typespec.fetch_specs(ExFastembed.Native)

    for {name, _public_arity} <- ExFastembed.__info__(:functions) do
      assert {{:function, ^name, arity}, _, _, :hidden, _} =
               Enum.find(docs, fn {key, _, _, _, _} ->
                 match?({:function, ^name, _arity}, key)
               end)

      assert Enum.any?(specs, fn {key, _spec} -> key == {name, arity} end)
    end
  end

  test "model tasks have help text and callback specifications" do
    for task <- [Mix.Tasks.Fastembed.Models, Mix.Tasks.Fastembed.Download] do
      assert {:docs_v1, _, _, _, %{"en" => text}, _, _} = Code.fetch_docs(task)
      assert text =~ "mix fastembed."
      assert {:ok, specs} = Code.Typespec.fetch_specs(task)
      assert Enum.any?(specs, fn {key, _} -> key == {:run, 1} end)
    end
  end
end
