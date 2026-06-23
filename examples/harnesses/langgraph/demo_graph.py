#!/usr/bin/env python3
"""Minimal LangGraph demo with Attestack callbacks (no API key required)."""

from __future__ import annotations

from typing import TypedDict

from langchain_core.language_models.fake_chat_models import FakeListChatModel
from langchain_core.tools import tool
from langgraph.graph import END, START, StateGraph

from attestack_callback import AttestackCallbackHandler, attestack_session


class State(TypedDict):
  messages: list


@tool
def get_weather(city: str) -> str:
  """Return weather for a city."""
  return f"Weather in {city}: sunny, 72F"


def build_graph() -> object:
  model = FakeListChatModel(responses=["Austin is sunny at 72F."])

  def tool_node(state: State, config) -> State:
    output = get_weather.invoke({"city": "Austin"}, config=config)
    return {"messages": state["messages"] + [output]}

  def summarize_node(state: State, config) -> State:
    reply = model.invoke("Summarize the weather for the user", config=config)
    return {"messages": state["messages"] + [reply]}

  graph = StateGraph(State)
  graph.add_node("lookup_weather", tool_node)
  graph.add_node("summarize", summarize_node)
  graph.add_edge(START, "lookup_weather")
  graph.add_edge("lookup_weather", "summarize")
  graph.add_edge("summarize", END)
  return graph.compile()


def main() -> None:
  graph = build_graph()
  handler = AttestackCallbackHandler()
  result = graph.invoke(
    {"messages": ["What's the weather in Austin?"]},
    config={"callbacks": [handler]},
  )
  handler.record_decision(
    summary="Answered weather question for Austin",
    rationale="Called get_weather then summarized with the model",
  )
  last = result["messages"][-1]
  print(getattr(last, "content", last))


if __name__ == "__main__":
  with attestack_session("langgraph demo"):
    main()
