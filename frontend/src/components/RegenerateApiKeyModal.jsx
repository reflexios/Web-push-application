import { useState } from 'react';
import { Modal } from './Modal';
import { ApiKeyReveal } from './ApiKeyReveal';
import { api } from '../api';

export function RegenerateApiKeyModal({ client, onClose, onRegenerated }) {
  const [result, setResult] = useState(null);
  const [error, setError] = useState('');
  const [submitting, setSubmitting] = useState(false);

  async function handleConfirm() {
    setError('');
    setSubmitting(true);
    try {
      const res = await api.regenerateApiKey(client.client_id);
      setResult(res);
      onRegenerated?.();
    } catch (err) {
      setError(err.message || 'Failed to regenerate the key');
    } finally {
      setSubmitting(false);
    }
  }

  if (result) {
    return (
      <Modal title="New API key" onClose={onClose}>
        <div className="alert alert-success">
          The key for "{result.name}" has been regenerated, the old one no longer works
        </div>
        <ApiKeyReveal result={result} onDone={onClose} />
      </Modal>
    );
  }

  return (
    <Modal title="Regenerate API key?" onClose={onClose}>
      {error && <div className="alert alert-error">{error}</div>}
      <div className="alert alert-info">
        The current key for client "{client.name}" will stop working immediately. Any
        integrations using the current api_key will need to be updated.
      </div>
      <div className="modal-footer">
        <button type="button" className="btn btn-secondary" onClick={onClose} disabled={submitting}>
          Cancel
        </button>
        <button type="button" className="btn btn-primary" onClick={handleConfirm} disabled={submitting}>
          {submitting ? 'Regenerating…' : 'Regenerate'}
        </button>
      </div>
    </Modal>
  );
}
