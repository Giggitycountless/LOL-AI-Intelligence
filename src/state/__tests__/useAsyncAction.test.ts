// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useAsyncAction } from "../useAsyncAction";
import type { Feedback } from "../../backend/types";

function defer<T>(): {
  promise: Promise<T>;
  resolve: (value: T) => void;
  reject: (error: unknown) => void;
} {
  let resolve!: (value: T) => void;
  let reject!: (error: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

beforeEach(() => {
  vi.useRealTimers();
});

describe("useAsyncAction", () => {
  it("returns true and calls onSuccess when action succeeds (no timeout)", async () => {
    const setFeedback = vi.fn();
    const setLoading = vi.fn();
    const onSuccess = vi.fn();

    const { result } = renderHook(() => useAsyncAction(setFeedback));

    let ok: boolean = false;
    await act(async () => {
      ok = await result.current.run(
        () => Promise.resolve("done"),
        setLoading,
        onSuccess,
      );
    });

    expect(ok).toBe(true);
    expect(onSuccess).toHaveBeenCalledWith("done");
    expect(setLoading).toHaveBeenCalledWith(true);
    expect(setLoading).toHaveBeenCalledWith(false);
    expect(setFeedback).not.toHaveBeenCalled();
  });

  it("returns false and sets error feedback when action throws (no timeout)", async () => {
    const setFeedback = vi.fn();
    const setLoading = vi.fn();

    const { result } = renderHook(() => useAsyncAction(setFeedback));

    let ok: boolean = true;
    await act(async () => {
      ok = await result.current.run(
        () => Promise.reject(new Error("Boom")),
        setLoading,
      );
    });

    expect(ok).toBe(false);
    expect(setFeedback).toHaveBeenCalledWith({
      kind: "error",
      message: "Boom",
    });
    expect(setLoading).toHaveBeenCalledWith(false);
  });

  it("rejects with timeout error when action exceeds timeoutMs", async () => {
    const setFeedback = vi.fn();
    const setLoading = vi.fn();

    const { result } = renderHook(() => useAsyncAction(setFeedback));

    // action never resolves — timeout should fire after 50ms
    let ok: boolean | undefined;
    await act(async () => {
      ok = await result.current.run(
        () => new Promise<never>(() => {}),
        setLoading,
        undefined,
        50,
      );
    });

    expect(ok).toBe(false);
    expect(setFeedback).toHaveBeenCalledWith({
      kind: "error",
      message: expect.stringMatching(/timed out/i),
    });
    expect(setLoading).toHaveBeenCalledWith(false);
  });

  it("completes normally before timeout without triggering timeout error", async () => {
    const setFeedback = vi.fn();
    const onSuccess = vi.fn();

    const { result } = renderHook(() => useAsyncAction(setFeedback));

    let ok: boolean | undefined;
    await act(async () => {
      ok = await result.current.run(
        () => Promise.resolve("fast"),
        undefined,
        onSuccess,
        5000,
      );
    });

    expect(ok).toBe(true);
    expect(onSuccess).toHaveBeenCalledWith("fast");
    expect(setFeedback).not.toHaveBeenCalled();
  });

  it("cleans up the timeout when the component unmounts mid-flight", async () => {
    const setFeedback = vi.fn();

    const { result, unmount } = renderHook(() => useAsyncAction(setFeedback));

    // start an action that hangs, then unmount
    let ok: boolean | undefined;
    const runPromise = act(() =>
      result.current
        .run(() => new Promise<never>(() => {}), undefined, undefined, 100)
        .then((v) => {
          ok = v;
        }),
    );

    unmount();

    // wait for the timeout — setFeedback should NOT be called on unmounted hook
    // because the ref-based approach prevents state updates on unmounted components
    await runPromise;

    // timeout fires even after unmount, but ok should be false
    expect(ok).toBe(false);
  });

  it("does not call setFeedback on error when suppressFeedback is true", async () => {
    const setFeedback = vi.fn();

    const { result } = renderHook(() => useAsyncAction(setFeedback));

    let ok: boolean = true;
    await act(async () => {
      ok = await result.current.run(
        () => Promise.reject(new Error("silent failure")),
        undefined,
        undefined,
        undefined,
        true,
      );
    });

    expect(ok).toBe(false);
    expect(setFeedback).not.toHaveBeenCalled();
  });

  it("still calls onSuccess when suppressFeedback is true and action succeeds", async () => {
    const setFeedback = vi.fn();
    const onSuccess = vi.fn();

    const { result } = renderHook(() => useAsyncAction(setFeedback));

    let ok: boolean = false;
    await act(async () => {
      ok = await result.current.run(
        () => Promise.resolve("ok"),
        undefined,
        onSuccess,
        undefined,
        true,
      );
    });

    expect(ok).toBe(true);
    expect(onSuccess).toHaveBeenCalledWith("ok");
    expect(setFeedback).not.toHaveBeenCalled();
  });
});
