import { useCallback } from "react";
import { isCommandError } from "../backend/commands";
import type { Feedback } from "../backend/types";

function errorMessage(error: unknown) {
  if (isCommandError(error)) {
    return error.message;
  }

  return error instanceof Error ? error.message : "Unexpected error";
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
  const run = useCallback(
    async <T>(
      action: () => Promise<T>,
      setLoading?: (value: boolean) => void,
      onSuccess?: (result: T) => void,
    ): Promise<boolean> => {
      setLoading?.(true);
      try {
        const result = await action();
        onSuccess?.(result);
        return true;
      } catch (caught: unknown) {
        setFeedback({ kind: "error", message: errorMessage(caught) });
        return false;
      } finally {
        setLoading?.(false);
      }
    },
    [setFeedback],
  );

  return { run };
}
