import { useState } from 'react';
import { Modal } from './Modal';

export function ConfirmModal({ title, description, confirmLabel = 'Delete', onConfirm, onClose }) {
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState('');

  async function handleConfirm() {
    setError('');
    setSubmitting(true);
    try {
      await onConfirm();
    } catch (err) {
      setError(err.message || 'Failed to perform the action');
      setSubmitting(false);
    }
  }

  return (
    <Modal title={title} onClose={onClose}>
      {error && <div className="alert alert-error">{error}</div>}
      <p>{description}</p>
      <div className="modal-footer">
        <button type="button" className="btn btn-secondary" onClick={onClose} disabled={submitting}>
          Cancel
        </button>
        <button type="button" className="btn btn-danger" onClick={handleConfirm} disabled={submitting}>
          {submitting ? 'Deleting…' : confirmLabel}
        </button>
      </div>
    </Modal>
  );
}
