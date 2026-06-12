import React, { Component, type ReactNode } from "react";

interface Props {
  // Optional so createElement(ErrorBoundary, { fallback }, child) typechecks —
  // children supplied as a separate argument aren't part of the props object.
  children?: ReactNode;
  fallback?: ReactNode;
}

interface State {
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error, info: React.ErrorInfo) {
    console.error("[ErrorBoundary]", error, info.componentStack);
  }

  private handleRetry = () => {
    this.setState({ error: null });
  };

  render(): ReactNode {
    if (this.state.error !== null) {
      if (this.props.fallback !== undefined) {
        return this.props.fallback;
      }

      return React.createElement(
        "div",
        {
          role: "alert",
          style: {
            padding: "2rem",
            textAlign: "center",
            color: "#e74c3c",
          } as React.CSSProperties,
        },
        React.createElement("h2", null, "Something went wrong"),
        React.createElement(
          "p",
          { style: { margin: "1rem 0", color: "#666" } },
          "The application encountered an unexpected error.",
        ),
        React.createElement(
          "button",
          {
            onClick: this.handleRetry,
            style: {
              padding: "0.5rem 1.5rem",
              cursor: "pointer",
            },
          },
          "Try Again",
        ),
      );
    }

    return this.props.children;
  }
}
