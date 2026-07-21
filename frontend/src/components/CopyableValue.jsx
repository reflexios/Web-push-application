import { useState } from 'react';

function truncateMiddle(str, keep = 6) {
  if (!str || str.length <= keep * 2 + 1) return str;
  return `${str.slice(0, keep)}…${str.slice(-keep)}`;
}

export function CopyableValue({ value, keep = 6 }) {
  const [copied, setCopied] = useState(false);

  async function handleCopy() {
    try {
      await navigator.clipboard.writeText(value);
      setCopied(true);
      setTimeout(() => setCopied(false), 1200);
    } catch {
    }
  }

  return (
    <button
      type="button"
      className="mono copyable-value"
      title={copied ? 'Copied' : `${value} (click to copy)`}
      onClick={handleCopy}
    >
      {copied ? 'Copied' : truncateMiddle(value, keep)}
    </button>
  );
}
