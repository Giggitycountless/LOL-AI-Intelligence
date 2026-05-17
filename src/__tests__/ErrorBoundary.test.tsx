// @vitest-environment jsdom
import { fireEvent, render, screen } from "@testing-library/react";
import React, { useState } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { ErrorBoundary } from "../components/ErrorBoundary";

function BrokenCounter(): React.ReactElement {
  const [count, setCount] = useState(0);

  if (count >= 2) {
    throw new Error("Boom! Counter reached 2");
  }

  return React.createElement(
    "div",
    null,
    React.createElement("span", null, `Count: ${count}`),
    React.createElement(
      "button",
      { onClick: () => setCount((c) => c + 1) },
      "Increment",
    ),
  );
}

beforeEach(() => {
  vi.clearAllMocks();
  // suppress console.error noise from intentional render-throws in tests
  vi.spyOn(console, "error").mockImplementation(() => {});
});

describe("ErrorBoundary", () => {
  it("renders children when no error occurs", () => {
    render(
      React.createElement(
        ErrorBoundary,
        null,
        React.createElement("div", null, "Safe Content"),
      ),
    );

    expect(screen.getByText("Safe Content")).toBeDefined();
  });

  it("shows fallback UI when a child throws", () => {
    render(
      React.createElement(ErrorBoundary, null, React.createElement(BrokenCounter)),
    );

    // trigger the error by clicking twice
    fireEvent.click(screen.getByRole("button", { name: "Increment" }));
    fireEvent.click(screen.getByRole("button", { name: "Increment" }));

    // fallback should now be visible
    expect(screen.getByText(/something went wrong/i)).toBeDefined();
    expect(screen.getByRole("button", { name: /try again/i })).toBeDefined();
  });

  it("hides fallback and re-renders children after clicking retry", () => {
    render(
      React.createElement(ErrorBoundary, null, React.createElement(BrokenCounter)),
    );

    // trigger error
    fireEvent.click(screen.getByRole("button", { name: "Increment" }));
    fireEvent.click(screen.getByRole("button", { name: "Increment" }));
    expect(screen.getByText(/something went wrong/i)).toBeDefined();

    // click retry — should go back to initial state (count=0, no error)
    fireEvent.click(screen.getByRole("button", { name: /try again/i }));
    expect(screen.getByText("Count: 0")).toBeDefined();
    expect(screen.queryByText(/something went wrong/i)).toBeNull();
  });

  it("accepts a custom fallback prop", () => {
    const customFallback = React.createElement(
      "div",
      null,
      "Custom error panel",
    );

    render(
      React.createElement(
        ErrorBoundary,
        { fallback: customFallback },
        React.createElement(BrokenCounter),
      ),
    );

    fireEvent.click(screen.getByRole("button", { name: "Increment" }));
    fireEvent.click(screen.getByRole("button", { name: "Increment" }));

    expect(screen.getByText("Custom error panel")).toBeDefined();
  });
});
