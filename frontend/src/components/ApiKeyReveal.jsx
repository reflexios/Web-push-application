import { useState } from 'react';

export function ApiKeyReveal({ result, onDone }) {
  const [copied, setCopied] = useState(false);

  async function copyApiKey() {
    try {
      await navigator.clipboard.writeText(result.api_key);
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    } catch {
    }
  }

  return (
    <>
      <div className="field">
        <label>API key</label>
        <div className="secret-row">
          <input className="input mono" readOnly value={result.api_key} onFocus={(e) => e.target.select()} />
          <button type="button" className="btn btn-secondary btn-sm" onClick={copyApiKey}>
            {copied ? 'Copied' : 'Copy'}
          </button>
        </div>
        <p className="hint">
          Save this key now - it is shown only once and the client will need it
          to send subscriptions and notifications (the <code>Authorization: Bearer …</code> header).
        </p>
      </div>

      <div className="field">
        <label>VAPID public key</label>
        <div className="secret-row">
          <input
            className="input mono"
            readOnly
            value={result.vapid_public_key}
            onFocus={(e) => e.target.select()}
          />
        </div>
      </div>

      <div className="modal-footer">
        <button className="btn btn-primary" onClick={onDone}>
          Done
        </button>
      </div>
    </>
  );
}
