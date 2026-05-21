export function StatePanel({ title, body }: { title: string; body: string }) {
  return (
    <div className="rounded-md border border-zinc-200 dark:border-zinc-700 bg-zinc-50 dark:bg-zinc-800 p-4">
      <p className="text-sm font-semibold text-zinc-950 dark:text-zinc-50">{title}</p>
      <p className="mt-1 text-sm text-zinc-500 dark:text-zinc-400">{body}</p>
    </div>
  );
}
