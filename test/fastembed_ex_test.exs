defmodule FastembedExTest do
  use ExUnit.Case
  doctest FastembedEx

  test "greets the world" do
    assert FastembedEx.hello() == :world
  end
end
