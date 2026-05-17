import { useCallback, useRef } from "react";
import { isCommandError } from "../backend/commands";
import type { Feedback } from "../backend/types";

const DEFAULT_TIMEOUT_MS = 30_000;

function errorMessage(error: unknown) {
  if (isCommandError(error)) {
    return error.message;
  }

  return error instanceof Error ? error.message : "Unexpected error";
}

function withTimeout<T>(
  promise: Promise<T>,
  timeoutMs: number,
): Promise<{ ok: true; value: T } | { ok: false }> {
  if (timeoutMs <= 0) {
    return promise.then((value) => ({ ok: true as const, value }));
  }

  const timeoutPromise = new Promise<{ ok: false }>((resolve) =>
    setTimeout(() => resolve({ ok: false }), timeoutMs),
  );

  return Promise.race([
    promise.then((value) => ({ ok: true as const, value })),
    timeoutPromise,
  ]);
}

/**
 * Wraps a common async pattern: set loading true → await action → on success,
 * or catch → set error feedback → return false → finally set loading false.
 *
 * @param setFeedback - state setter for feedback messages (usually from useState)
 * @returns an object with a `run` function
 *
 * Usage inside a useCallback:
 *   const action = useCallback(async (input: X) => {
 *     return run(() => backendCall(input), setLoading, (result) => setState(result));
 *   }, [run]);
 */
export function useAsyncAction(setFeedback: (feedback: Feedback | null) => void) {
  const feedbackRef = useRef(setFeedback);
  feedbackRef.current = setFeedback;

  const run = useCallback(
    async <T>(
      action: () => Promise<T>,
      setLoading?: (value: boolean) => void,
      onSuccess?: (result: T) => void,
      timeoutMs?: number,
      suppressFeedback?: boolean,
    ): Promise<boolean> => {
      setLoading?.(true);

      try {
        const result = await withTimeout(action(), timeoutMs ?? DEFAULT_TIMEOUT_MS);

        if (!result.ok) {
          if (!suppressFeedback) {
            const seconds = (timeoutMs ?? DEFAULT_TIMEOUT_MS) / 1000;
            feedbackRef.current({
              kind: "error",
              message: `Request timed out after ${seconds}s`,
            });
          }
          return false;
        }

        onSuccess?.(result.value);
        return true;
      } catch (caught: unknown) {
        if (!suppressFeedback) {
          feedbackRef.current({ kind: "error", message: errorMessage(caught) });
        }
        return false;
      } finally {
        setLoading?.(false);
      }
    },
    [], // feedbackRef avoids re-creating run on every setFeedback change
  );

  return { run };
}
